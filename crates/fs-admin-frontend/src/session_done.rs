//! What a command leaves behind: a search narrows the list, a finished command leads to the next screen.
//!
//! Pattern: State, part of [`Session`] (the transitions after a command ran, split off to keep the file small).

use fs_types::CommandKind;

use crate::backend::{Outcome, Values};
use crate::commands::CommandPlan;
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
                self.query.clear();
                read = self.reload();
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
}
