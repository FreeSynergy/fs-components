//! Which commands are offered where, and with which form.
//!
//! Pattern: Strategy by command kind — one `plan_commands` reads the description and decides placement,
//! confirmation and the form fields from the kind and the input; a program says nothing about buttons (ADR 0066
//! point 3: not the look, but what is possible).
//!
//! Provisional rules (reported to the strategy): a command works on one record when its input has a `reference` to
//! the type itself (that field is filled from the selection and not shown) or when its kind is `change` or `delete`;
//! otherwise it works on the list. `delete` asks for confirmation. `find` is the search of the list.

use fs_types::{BasicType, CommandDescription, CommandKind, FieldDescription, TypeDescription};

/// Where the button is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    /// With the list (create, find, other).
    OnList,
    /// With the selected record (change, delete, or anything that takes the record as input).
    OnRecord,
}

/// One command as the frontend offers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandPlan {
    /// The command name of the description.
    pub name: String,
    /// What it does.
    pub kind: CommandKind,
    /// Translation key of the label.
    pub label: String,
    /// Where it is offered.
    pub placement: Placement,
    /// The input field filled from the selected record, if any.
    pub bound: Option<String>,
    /// The fields to type: the input without the bound field.
    pub form: Vec<FieldDescription>,
    /// The result fields to show after it ran.
    pub result: Vec<FieldDescription>,
    /// A person is asked once more before it runs.
    pub confirm: bool,
}

fn plan(type_name: &str, c: &CommandDescription) -> CommandPlan {
    let bound = c
        .input
        .iter()
        .find(|f| f.basic == BasicType::Reference && f.target == type_name)
        .map(|f| f.name.clone());
    let on_record = bound.is_some() || matches!(c.kind, CommandKind::Change | CommandKind::Delete);
    CommandPlan {
        name: c.name.clone(),
        kind: c.kind,
        label: c.label.clone(),
        placement: if on_record {
            Placement::OnRecord
        } else {
            Placement::OnList
        },
        form: c
            .input
            .iter()
            .filter(|f| Some(&f.name) != bound.as_ref())
            .cloned()
            .collect(),
        bound,
        result: c.result.clone(),
        confirm: c.kind == CommandKind::Delete,
    }
}

/// All commands of a type, in the order of the description.
pub fn plan_commands(description: &TypeDescription) -> Vec<CommandPlan> {
    description
        .commands
        .iter()
        .map(|c| plan(&description.name, c))
        .collect()
}
