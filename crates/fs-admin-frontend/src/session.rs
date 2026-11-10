//! The state of the generated frontend: which screen, which form, what the commands did.
//!
//! Pattern: State (`coding.md` §2) — [`Screen`] is the state, the methods are the transitions; every transition tells
//! the messages which screen is shown, so a message never outlives its screen (ADR 0065 point 4). Commands go to the
//! [`Backend`] only after the form has checked them.

use std::sync::Arc;

use fs_types::{CommandKind, TypeDescription};

use crate::backend::{Backend, BackendError, Outcome, Record, Values};
use crate::clock::Clock;
use crate::commands::{CommandPlan, Placement, plan_commands};
use crate::detail::{Entry, entries};
use crate::form::FormModel;
use crate::list::ListModel;
use crate::messages::{MessageKind, Messages};
use crate::screen::Screen;

/// The frontend of one data type.
pub struct Session {
    pub(crate) description: TypeDescription,
    pub(crate) backend: Arc<dyn Backend>,
    pub(crate) plans: Vec<CommandPlan>,
    pub(crate) list: ListModel,
    pub(crate) query: Values,
    /// The last reading of the records failed: what the list holds is old and no record of it is shown as current.
    pub(crate) stale: bool,
    /// What the core said about each command when it was last asked (see [`Backend::supported`]).
    pub(crate) supported: std::collections::BTreeMap<String, bool>,
    pub(crate) screen: Screen,
    pub(crate) form: Option<FormModel>,
    pub(crate) result: Option<Outcome>,
    pub(crate) messages: Arc<Messages>,
}

impl Session {
    /// Open on the list. A program that cannot answer gives an empty list and an error message that stays.
    pub fn new(
        description: TypeDescription,
        backend: Arc<dyn Backend>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let mut s = Self {
            plans: plan_commands(&description),
            list: ListModel::new(&description, Vec::new()),
            description,
            backend,
            query: Values::new(),
            stale: false,
            supported: std::collections::BTreeMap::new(),
            screen: Screen::List,
            form: None,
            result: None,
            messages: Arc::new(Messages::new(clock)),
        };
        s.messages.enter(&s.screen.key());
        s.refresh_support();
        s.reload();
        s
    }

    /// The description the screens are made from.
    pub fn description(&self) -> &TypeDescription {
        &self.description
    }

    /// The screen now.
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// The list.
    pub fn list(&self) -> &ListModel {
        &self.list
    }

    /// The list, to sort, filter and select.
    pub fn list_mut(&mut self) -> &mut ListModel {
        &mut self.list
    }

    /// The form of the open command.
    pub fn form(&self) -> Option<&FormModel> {
        self.form.as_ref()
    }

    /// The form, to type into.
    pub fn form_mut(&mut self) -> Option<&mut FormModel> {
        self.form.as_mut()
    }

    /// What the last command answered (on the result screen).
    pub fn result(&self) -> Option<&Outcome> {
        self.result.as_ref()
    }

    /// The messages.
    pub fn messages(&self) -> &Arc<Messages> {
        &self.messages
    }

    /// The commands for the list or for a record.
    pub fn commands(&self, placement: Placement) -> Vec<&CommandPlan> {
        self.plans
            .iter()
            .filter(|p| p.placement == placement)
            .collect()
    }

    /// One command.
    pub fn command(&self, name: &str) -> Option<&CommandPlan> {
        self.plans.iter().find(|p| p.name == name)
    }

    /// The record shown on the detail screen.
    pub fn record(&self) -> Option<&Record> {
        if self.stale {
            return None; // old values are not shown as current
        }
        match &self.screen {
            Screen::Detail(id)
            | Screen::Confirm { record: id, .. }
            | Screen::Command {
                record: Some(id), ..
            } => self.list.record(id),
            _ => None,
        }
    }

    /// The fields of the shown record with their values.
    pub fn detail(&self) -> Vec<Entry> {
        self.record()
            .map_or_else(Vec::new, |r| entries(&self.description, r))
    }

    pub(crate) fn go(&mut self, screen: Screen) {
        self.messages.enter(&screen.key());
        self.screen = screen;
    }

    pub(crate) fn fail(&self, e: &BackendError) {
        let code = match e {
            BackendError::Rejected(_) => "rejected",
            BackendError::NotFound => "not-found",
            BackendError::Unavailable(_) => "unavailable",
            BackendError::Handshake(_) => "handshake",
        };
        // The same error is told once, however often it happens.
        self.messages.solve(code);
        match e {
            // The words of fs-interfaces, as they are, and the way to try again.
            BackendError::Handshake(text) => {
                self.messages
                    .raise_with(MessageKind::Error, code, Some(text.clone()), true);
            }
            _ => {
                self.messages.raise(MessageKind::Error, code);
            }
        }
    }

    /// "Try again" after a failed handshake: connect again, then read the records again. A failure is told again
    /// (once), with its own words; success closes the message.
    pub fn retry(&mut self) -> bool {
        match self.backend.reconnect() {
            Ok(()) => {
                self.messages.solve("handshake");
                self.refresh_support();
                self.reload()
            }
            Err(e) => {
                self.stale = matches!(e, BackendError::Unavailable(_) | BackendError::Handshake(_));
                self.fail(&e);
                false
            }
        }
    }

    /// Read the records again with the search that is set. The error of a program that cannot answer stays.
    pub fn reload(&mut self) -> bool {
        match self.backend.list(&self.query) {
            Ok(records) => {
                self.refresh_support();
                self.list.set_records(records);
                self.stale = false;
                self.messages.solve("unavailable");
                self.messages.solve("handshake");
                true
            }
            Err(BackendError::Rejected(_)) if !self.query.is_empty() => {
                // The stored search is refused (it worked when it was set): drop it, say so, read everything. Keeping
                // it would repeat the refusal at every screen.
                self.query.clear();
                self.fail(&BackendError::Rejected(Vec::new()));
                self.reload()
            }
            Err(e) => {
                // Only a program that cannot be reached makes the records old; a refusal or a missing record does not.
                self.stale = matches!(e, BackendError::Unavailable(_) | BackendError::Handshake(_));
                self.fail(&e);
                false
            }
        }
    }

    /// Go to the list, with all records.
    pub fn open_list(&mut self) {
        self.query.clear();
        self.form = None;
        self.go(Screen::List);
        self.reload();
    }

    /// Back to the list as it is, with the search that is set (from a record or a result).
    pub fn back_to_list(&mut self) {
        // From a record that was not loaded (or with old records) the list is read again, so it is not shown stale
        // without a word: a failure is told on the list screen.
        let unread = self.stale
            || matches!(&self.screen, Screen::Detail(id) if self.list.record(id).is_none());
        self.form = None;
        self.go(Screen::List);
        if unread {
            self.reload();
        }
    }

    /// Show one record. Returns `false` if the list has none with this id, or if reading again (the records were old)
    /// shows that it is gone: then the list is shown with a `not-found` message.
    pub fn open_detail(&mut self, id: &str) -> bool {
        if !self.list.select(id) {
            return false;
        }
        self.form = None;
        self.go(Screen::Detail(id.to_owned()));
        self.read_again_if_stale();
        !self.leave_if_gone()
    }

    /// Back from a form, a question or an answer: to the record if there was one, else to the list.
    pub fn cancel(&mut self) {
        let back = match &self.screen {
            Screen::Command {
                record: Some(id), ..
            }
            | Screen::Confirm { record: id, .. }
                if self.list.record(id).is_some() =>
            {
                Screen::Detail(id.clone())
            }
            _ => Screen::List,
        };
        self.form = None;
        self.go(back);
        self.read_again_if_stale();
        self.leave_if_gone();
    }

    /// Send the open form, or confirm the question. Returns whether the command ran.
    pub fn submit(&mut self) -> bool {
        let (name, record) = match &self.screen {
            Screen::Command { name, record } => (name.clone(), record.clone()),
            Screen::Confirm { name, record } => (name.clone(), Some(record.clone())),
            _ => return false,
        };
        let Some(plan) = self.command(&name).cloned() else {
            return false;
        };
        // A form that was opened while the command was open may stand on after the core stopped offering it.
        if !self.is_supported(&name) {
            return false;
        }
        if plan.kind == CommandKind::Change {
            return self.submit_change(&plan, record);
        }
        let mut input = match self.form.as_mut().map(FormModel::submit) {
            Some(Ok(v)) => v,
            Some(Err(_)) => return false,
            None => Values::new(),
        };
        if let (Some(field), Some(id)) = (&plan.bound, &record) {
            input.insert(field.clone(), vec![id.clone()]);
        }
        if plan.kind == CommandKind::Find {
            return self.search(input);
        }
        match self.backend.run(&name, record.as_deref(), &input) {
            Ok(outcome) => {
                self.done(&plan, record, outcome);
                true
            }
            Err(e) => {
                self.refused(&e);
                false
            }
        }
    }
}
