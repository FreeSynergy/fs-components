//! What a command leaves behind: a search narrows the list, a finished command leads to the next screen.
//!
//! Pattern: State, part of [`Session`] (the transitions after a command ran, split off to keep the file small).

use fs_types::CommandKind;

use crate::backend::{BackendError, Outcome, Values};
use crate::commands::CommandPlan;
use crate::form::FormModel;
use crate::messages::MessageKind;
use crate::screen::Screen;
use crate::session::Session;

impl Session {
    pub(crate) fn search(&mut self, query: Values) -> bool {
        match self.backend.list(&query) {
            Ok(records) => {
                self.query = query;
                self.list.set_records(records);
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
                    self.query = old;
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
            if !mine.is_empty() {
                form.refuse(mine);
            }
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
}
