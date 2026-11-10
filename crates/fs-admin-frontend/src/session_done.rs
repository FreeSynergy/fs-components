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
                self.fail(&e);
                false
            }
        }
    }

    pub(crate) fn done(&mut self, plan: &CommandPlan, record: Option<String>, outcome: Outcome) {
        self.form = None;
        self.reload();
        let target = match plan.kind {
            CommandKind::Create | CommandKind::Change => outcome
                .id
                .clone()
                .or(record)
                .filter(|id| self.list.record(id).is_some())
                .map_or(Screen::List, Screen::Detail),
            CommandKind::Delete => Screen::List,
            _ if !plan.result.is_empty() => Screen::Result(plan.name.clone()),
            _ => record.map_or(Screen::List, Screen::Detail),
        };
        if let Screen::Detail(id) = &target {
            self.list.select(id);
        }
        self.result = Some(outcome);
        self.go(target);
        self.messages.raise(MessageKind::Success, "done");
    }
}
