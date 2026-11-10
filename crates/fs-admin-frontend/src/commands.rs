//! Which commands are offered where, and with which form.
//!
//! Pattern: none — a small set of pure functions for one concept (nothing is chosen from a table or a trait).
//! `plan_commands` reads the description and decides placement, confirmation and the form fields from the kind and
//! the input; a program says nothing about buttons (ADR 0066 point 3: not the look, but what is possible).
//!
//! Rules (decided 2026-11-10, written in `types.capnp`, `TypeDescription::record_commands`): a command works on one
//! record when it is `change` or `delete` or its input holds a reference to the type itself; the id of the record
//! travels in the program's call, so `change` and `delete` have no input for it. `change` is an `Edit` of the fields of
//! the type; its form is the type's own fields. `delete` asks once more. `find` is the search (and the list).

use fs_types::{BasicType, CommandDescription, CommandKind, FieldDescription, TypeDescription};

/// Where the button is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placement {
    /// With the list (create, find, other).
    OnList,
    /// With the shown record (change, delete, or anything that takes the record as input).
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
    /// The input field filled from the shown record, if the command has a reference to the type itself.
    pub bound: Option<String>,
    /// The fields to type: the input without the bound field; for `change` the fields of the type.
    pub form: Vec<FieldDescription>,
    /// The result fields to show after it ran.
    pub result: Vec<FieldDescription>,
    /// A person is asked once more before it runs.
    pub confirm: bool,
    /// What the description says (`CommandDescription::supported`). The control is free only if the program says so
    /// too (`Backend::supported`).
    pub described_supported: bool,
}

fn plan(description: &TypeDescription, c: &CommandDescription, on_record: bool) -> CommandPlan {
    let bound = c
        .input
        .iter()
        .find(|f| {
            f.basic == BasicType::Reference && (f.target.is_empty() || f.target == description.name)
        })
        .map(|f| f.name.clone());
    CommandPlan {
        name: c.name.clone(),
        kind: c.kind,
        label: c.label.clone(),
        placement: if on_record {
            Placement::OnRecord
        } else {
            Placement::OnList
        },
        form: if c.kind == CommandKind::Change {
            description.fields.clone()
        } else {
            c.input
                .iter()
                .filter(|f| Some(&f.name) != bound.as_ref())
                .cloned()
                .collect()
        },
        bound,
        result: c.result.clone(),
        confirm: c.kind == CommandKind::Delete,
        described_supported: c.supported,
    }
}

/// All commands of a type, in the order of the description.
pub fn plan_commands(description: &TypeDescription) -> Vec<CommandPlan> {
    let on_record: Vec<&str> = description
        .record_commands()
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    description
        .commands
        .iter()
        .map(|c| plan(description, c, on_record.contains(&c.name.as_str())))
        .collect()
}
