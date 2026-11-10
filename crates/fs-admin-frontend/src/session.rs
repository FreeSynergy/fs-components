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
            screen: Screen::List,
            form: None,
            result: None,
            messages: Arc::new(Messages::new(clock)),
        };
        s.messages.enter(&s.screen.key());
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
        };
        self.messages.raise(MessageKind::Error, code);
    }

    /// Read the records again with the search that is set. The error of a program that cannot answer stays.
    pub fn reload(&mut self) -> bool {
        match self.backend.list(&self.query) {
            Ok(records) => {
                self.list.set_records(records);
                self.messages.solve("unavailable");
                true
            }
            Err(e) => {
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
        self.form = None;
        self.go(Screen::List);
    }

    /// Show one record. Returns `false` if the list has none with this id.
    pub fn open_detail(&mut self, id: &str) -> bool {
        if !self.list.select(id) {
            return false;
        }
        self.form = None;
        self.go(Screen::Detail(id.to_owned()));
        true
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
    }

    /// Open a command. A command on a record needs the selection; a delete asks first. Returns `false` if the
    /// command is unknown or has no record to work on.
    pub fn begin(&mut self, name: &str) -> bool {
        let Some(plan) = self.command(name).cloned() else {
            return false;
        };
        let record = match plan.placement {
            Placement::OnList => None,
            Placement::OnRecord => match self.list.selected() {
                Some(id) => Some(id.to_owned()),
                None => return false,
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
        match self.backend.run(&name, &input) {
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

    /// The program said no. Findings that point at fields of the open form are marked there; anything else (a
    /// confirm screen has no form, a finding may name a field the form does not have) is an error message: never silent.
    pub(crate) fn refused(&mut self, e: &BackendError) {
        if let (BackendError::Rejected(issues), Some(form)) = (e, self.form.as_mut())
            && issues
                .iter()
                .all(|i| form.fields().iter().any(|f| f.name == i.field))
        {
            form.refuse(issues.clone());
            return;
        }
        self.fail(e);
    }
}
