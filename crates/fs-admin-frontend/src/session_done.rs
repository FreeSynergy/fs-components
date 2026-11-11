//! The transitions around a command: opening one, a search narrowing the list, a finished command leading to the next
//! screen, refusals, old records, the commands the core supports.
//!
//! Pattern: State, part of [`Session`] (the transitions around commands, split off to keep the file small).

use fs_types::CommandKind;

use crate::backend::{BackendError, Outcome, Record, Values};
use crate::commands::CommandPlan;
use crate::commands::Placement;
use crate::form::FormModel;
use crate::messages::MessageKind;
use crate::screen::Screen;
use crate::session::Session;

impl Session {
    /// Take what a search answered (the reading was done by the caller, on a worker or in place).
    pub(crate) fn search_apply(
        &mut self,
        query: Values,
        read: Result<Vec<Record>, BackendError>,
    ) -> bool {
        match read {
            Ok(records) => {
                self.query = query;
                self.list.set_records(records);
                self.stale = false;
                self.messages.solve("unavailable");
                self.form = None;
                self.go(Screen::List);
                true
            }
            Err(e) => {
                self.refused(&e);
                false
            }
        }
    }

    /// The command ran. Go to the screen it leads to first, then read the records again there, so a failed reload is
    /// raised on that screen and stays (the command itself did run, so "done" is true as well).
    pub(crate) fn done(&mut self, plan: &CommandPlan, record: Option<String>, outcome: Outcome) {
        self.form = None;
        let target = match plan.kind {
            CommandKind::Create | CommandKind::Change => outcome
                .id
                .clone()
                .or(record)
                .map_or(Screen::List, Screen::Detail),
            CommandKind::Delete => Screen::List,
            _ if !plan.result.is_empty() => Screen::Result(plan.name.clone()),
            _ => record.map_or(Screen::List, Screen::Detail),
        };
        self.result = Some(outcome);
        self.go(target.clone());
        let mut read = self.reload();
        if let Screen::Detail(id) = target {
            if read && self.list.record(&id).is_none() && !self.query.is_empty() {
                // the record is not in the narrowed list: show everything, the record is what the person asked for
                // the search is dropped only if the wider reading works; else the old search stays, matching the list
                let old = std::mem::take(&mut self.query);
                read = self.reload();
                if !read {
                    // the list still holds the first, narrowed reading: it is fresh, the error stays as a message
                    self.query = old;
                    self.stale = false;
                }
            }
            if self.list.select(&id) {
                // loaded: the screen shows it
            } else {
                // not loaded: no selection of another record stays, and the screen says so
                self.list.clear_selection();
                if read {
                    self.go(Screen::List);
                }
            }
        }
        self.messages.raise(MessageKind::Success, "done");
    }

    /// The program said no. Findings that point at fields of the open form are marked there; anything else (a confirm
    /// screen has no form, a finding on a field the form does not have, a refusal with no findings at all) is an error
    /// message: never silent.
    pub(crate) fn refused(&mut self, e: &BackendError) {
        if let (BackendError::Rejected(issues), Some(form)) = (e, self.form.as_mut()) {
            let (mine, others): (Vec<_>, Vec<_>) = issues
                .iter()
                .cloned()
                .partition(|i| form.fields().iter().any(|f| f.name == i.field));
            form.refuse(mine); // also when empty: marks from an earlier try go
            if others.is_empty() && !issues.is_empty() {
                return;
            }
        }
        self.fail(e);
    }

    /// Whether an open form holds text the person has not sent. A search is no work to lose.
    pub fn has_unsaved_form(&self) -> bool {
        match &self.screen {
            Screen::Command { name, .. } => {
                self.command(name)
                    .is_some_and(|p| p.kind != CommandKind::Find)
                    && self.form.as_ref().is_some_and(FormModel::is_dirty)
            }
            _ => false,
        }
    }

    /// A record screen whose record is gone after a successful reading: the selection goes, the list is shown, and
    /// `not-found` is told there. (An unreadable record is not this case: that screen says "not loaded".)
    pub(crate) fn leave_if_gone(&mut self) -> bool {
        let gone = matches!(&self.screen, Screen::Detail(id) if !self.stale && self.list.record(id).is_none());
        if gone {
            self.list.clear_selection();
            self.go(Screen::List);
            self.fail(&BackendError::NotFound);
        }
        gone
    }

    /// Old records are not shown without a word: a screen that shows records (the list, a record), entered while the
    /// last reading failed, reads again, and a failure is told on that screen.
    pub(crate) fn read_again_if_stale(&mut self) {
        if self.stale {
            self.reload();
        }
    }

    /// Whether a search narrows the list.
    pub fn search_active(&self) -> bool {
        !self.query.is_empty()
    }

    /// Whether the records shown are old because the last reading failed.
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Ask the core which commands it can carry out now.
    pub(crate) fn refresh_support(&mut self) {
        // free only if the description and the program both say so
        self.supported = self
            .plans
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    p.described_supported && self.backend.supported(&p.name),
                )
            })
            .collect();
    }

    /// Whether the core can carry out the command (as last asked when the records were read). A command that is
    /// unknown here is not supported.
    pub fn is_supported(&self, name: &str) -> bool {
        self.supported.get(name).copied().unwrap_or(false)
    }

    /// Open a command. A command on a record needs the selection; a delete asks first. Returns `false` if the
    /// command is unknown or has no record to work on.
    pub fn begin(&mut self, name: &str) -> bool {
        if self.submitting {
            return false; // a command is on its way
        }
        let Some(plan) = self.command(name).cloned() else {
            return false;
        };
        if !self.is_supported(name) {
            return false; // locked: "coming soon"
        }
        let record = match plan.placement {
            Placement::OnList => None,
            // The record is the one of the screen that is shown, and only if it is loaded: the selection of the list
            // may point elsewhere (or nowhere) after a failed reload.
            Placement::OnRecord => match &self.screen {
                Screen::Detail(id) if !self.stale && self.list.record(id).is_some() => {
                    Some(id.clone())
                }
                _ => return false,
            },
        };
        if plan.confirm {
            let Some(record) = record else { return false };
            self.form = None;
            self.go(Screen::Confirm {
                name: plan.name,
                record,
            });
            return true;
        }
        let initial = match (&plan.kind, &record) {
            (CommandKind::Change, Some(id)) => self
                .list
                .record(id)
                .map(|r| r.values.clone())
                .unwrap_or_default(),
            _ => Values::new(),
        };
        self.form = Some(FormModel::new(&plan.form, &initial));
        self.go(Screen::Command {
            name: plan.name,
            record,
        });
        true
    }
}
