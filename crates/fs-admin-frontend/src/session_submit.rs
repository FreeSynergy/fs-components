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
                _ => Answer::Done(Err(error), None),
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
        // The record the command leads to may not be in a narrowed list: the session then reads everything.
        let target = outcome.id.as_ref().or(self.record.as_ref());
        if let (Some(id), Some((_, Ok(records)))) = (target, reads.first())
            && !self.query.is_empty()
            && !records.iter().any(|r| &r.id == id)
        {
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
        self.submitting
    }

    /// First half of a command: check the form and take what the call needs, to be run with no lock held. `None`
    /// when nothing is to be sent (not a command screen, locked, the form has findings) or when a command is on its way
    /// already (a second press does nothing).
    pub fn submit_begin(&mut self) -> Option<SubmitJob> {
        if self.submitting {
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
        self.submitting = true;
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
        self.submitting = false;
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
                self.done(&plan, record, outcome);
                self.preread.clear();
                self.preread_support = None;
                true
            }
            Answer::Done(Err(e), _) => {
                self.refused(&e);
                false
            }
        }
    }
}
