//! A command as three steps, so that the server's time is never the interface thread's.
//!
//! Pattern: part of [`Session`] (State), the same split as `session_retry.rs` (Command object + two halves).
//! [`Session::submit_begin`] checks the form and takes what the call needs; [`SubmitJob::run`] makes the call (and the
//! reading of the records after it) on a worker, with no lock held; [`Session::submit_finish`] puts the answers in, under
//! the lock. While a command is on its way the session is `busy`: a second submit gives `None` (it is refused, not
//! queued, so a write is never carried out twice), and `begin` and `cancel` do nothing, so the screen the answer
//! belongs to stays.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use fs_types::{CommandKind, Edit};

use crate::backend::{Backend, BackendError, Outcome, Record, Values};
use crate::commands::CommandPlan;
use crate::screen::Screen;
use crate::session::Session;
use crate::session_done::target_screen;

/// Where a command is: not sent, on its way (the program may hang), or its answer is going in under the lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Idle,
    Sending,
    Finishing,
}

enum Call {
    Run { name: String, input: Values },
    Change { id: String, edit: Edit },
    Find(Values),
}

/// What a command needs, taken from the session so that the work needs no lock.
pub struct SubmitJob {
    backend: Arc<dyn Backend>,
    plan: CommandPlan,
    record: Option<String>,
    call: Call,
    query: Values,
    /// Each command with whether its description says it is supported.
    commands: Vec<(String, bool)>,
}

/// The readings made after a command, so that the session does not ask again on the interface thread.
struct After {
    reads: Vec<(Values, Result<Vec<Record>, BackendError>)>,
    supported: BTreeMap<String, bool>,
}

enum Answer {
    /// Nothing was sent (the work could not even be started).
    NotSent(BackendError),
    Done(Result<Outcome, BackendError>, Option<After>),
    Found(Values, Result<Vec<Record>, BackendError>),
}

/// What a command found out, to be put into the session by [`Session::submit_finish`].
pub struct SubmitOutcome {
    plan: CommandPlan,
    record: Option<String>,
    answer: Answer,
}

impl SubmitOutcome {
    /// A command that could not even be started (no thread to run it on).
    #[must_use]
    pub fn failed(job: &SubmitJob, error: BackendError) -> Self {
        Self {
            plan: job.plan.clone(),
            record: job.record.clone(),
            answer: match &job.call {
                Call::Find(q) => Answer::Found(q.clone(), Err(error)),
                _ => Answer::NotSent(error),
            },
        }
    }
}

impl SubmitJob {
    /// Make the call; if it worked, read the records again (and ask what the core supports). A backend that panics is
    /// a failed command (`Unavailable`), never one that does not end: the session would stay busy for good.
    #[must_use]
    pub fn run(self) -> SubmitOutcome {
        let guarded = catch_unwind(AssertUnwindSafe(|| self.call()));
        let answer = guarded.unwrap_or_else(|_| {
            let e = BackendError::Unavailable("command panicked".into());
            match &self.call {
                Call::Find(q) => Answer::Found(q.clone(), Err(e)),
                _ => Answer::Done(Err(e), None),
            }
        });
        SubmitOutcome {
            plan: self.plan,
            record: self.record,
            answer,
        }
    }

    fn call(&self) -> Answer {
        let b = &self.backend;
        let result = match &self.call {
            Call::Find(q) => return Answer::Found(q.clone(), b.list(q)),
            Call::Run { name, input } => b.run(name, self.record.as_deref(), input),
            Call::Change { id, edit } => b.change(id, edit),
        };
        let Ok(outcome) = &result else {
            return Answer::Done(result, None);
        };
        let mut reads = vec![(self.query.clone(), b.list(&self.query))];
        // What the session will ask for again under its lock must be read here: the record the command leads to may not
        // be in a narrowed list, and a narrowed search the program refuses now is dropped and everything is read.
        let target = match target_screen(&self.plan, self.record.clone(), outcome) {
            Screen::Detail(id) => Some(id),
            _ => None,
        };
        let all_needed = match reads.first() {
            Some((_, Ok(records))) => target.is_some_and(|id| !records.iter().any(|r| r.id == id)),
            Some((_, Err(BackendError::Rejected(_)))) => true,
            _ => false,
        };
        if all_needed && !self.query.is_empty() {
            let all = Values::new();
            reads.push((all.clone(), b.list(&all)));
        }
        let supported = self
            .commands
            .iter()
            .map(|(name, described)| (name.clone(), *described && b.supported(name)))
            .collect();
        Answer::Done(result, Some(After { reads, supported }))
    }
}

impl Session {
    /// Whether a command is on its way to the program (the window shows "running" and refuses another).
    pub fn is_busy(&self) -> bool {
        self.phase == Phase::Sending
    }

    /// First half of a command: check the form and take what the call needs, to be run with no lock held. `None`
    /// when nothing is to be sent (not a command screen, locked, the form has findings) or when a command is on its way
    /// already (a second press does nothing).
    pub fn submit_begin(&mut self) -> Option<SubmitJob> {
        if self.is_busy() {
            return None;
        }
        let (name, record) = match &self.screen {
            Screen::Command { name, record } => (name.clone(), record.clone()),
            Screen::Confirm { name, record } => (name.clone(), Some(record.clone())),
            _ => return None,
        };
        let plan = self.command(&name).cloned()?;
        // A form that was opened while the command was open may stand on after the core stopped offering it.
        if !self.is_supported(&name) {
            return None;
        }
        let call = if plan.kind == CommandKind::Change {
            let (Some(id), Some(form)) = (record.clone(), self.form.as_mut()) else {
                return None;
            };
            Call::Change {
                id,
                edit: form.submit_changes().ok()?,
            }
        } else {
            let mut input = match self.form.as_mut().map(crate::form::FormModel::submit) {
                Some(Ok(v)) => v,
                Some(Err(_)) => return None,
                None => Values::new(),
            };
            if let (Some(field), Some(id)) = (&plan.bound, &record) {
                input.insert(field.clone(), vec![id.clone()]);
            }
            if plan.kind == CommandKind::Find {
                Call::Find(input)
            } else {
                Call::Run { name, input }
            }
        };
        self.phase = Phase::Sending;
        self.in_flight = self.form.as_ref().map(|f| f.values.clone());
        Some(SubmitJob {
            backend: Arc::clone(&self.backend),
            plan,
            record,
            call,
            query: self.query.clone(),
            commands: self
                .plans
                .iter()
                .map(|p| (p.name.clone(), p.described_supported))
                .collect(),
        })
    }

    /// Second half: put the answers in (the next screen, or the refusal). Returns whether the command ran.
    pub fn submit_finish(&mut self, outcome: SubmitOutcome) -> bool {
        self.phase = Phase::Idle;
        let sent = self.in_flight.take();
        let SubmitOutcome {
            plan,
            record,
            answer,
        } = outcome;
        match answer {
            Answer::Found(query, read) => match read {
                Ok(_) => self.search_apply(query, read),
                Err(e) => {
                    self.refused(&e);
                    false
                }
            },
            Answer::Done(Ok(outcome), after) => {
                if let Some(After { reads, supported }) = after {
                    self.preread = reads;
                    self.preread_support = Some(supported);
                }
                // Text typed after the command was sent is not part of it: the form goes with the answer, so say so
                // (never silent).
                let edited = match (&sent, &self.form) {
                    (Some(sent), Some(form)) => *sent != form.values,
                    _ => false,
                };
                self.phase = Phase::Finishing;
                self.done(&plan, record, outcome);
                self.phase = Phase::Idle;
                self.preread.clear();
                self.preread_support = None;
                if edited {
                    self.messages.solve("edited-after-send");
                    self.messages
                        .raise(crate::MessageKind::Error, "edited-after-send");
                }
                true
            }
            Answer::NotSent(e) => {
                self.refused(&e);
                false
            }
            Answer::Done(Err(BackendError::Unavailable(_)), _) => {
                // A write that got no answer may or may not have been made: say so, and offer no "try again" that
                // would write twice.
                self.messages.solve("unknown-result");
                self.messages
                    .raise(crate::MessageKind::Error, "unknown-result");
                false
            }
            Answer::Done(Err(e), _) => {
                self.refused(&e);
                false
            }
        }
    }
}
