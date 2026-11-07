//! What the tasks workbench and its source talk about.
//!
//! Pattern: none — neutral rows, columns, sections of labelled fields and a draft. Nothing
//! here knows what a task is made of.

/// What a source can do; an action it cannot do is not offered.
#[non_exhaustive]
#[allow(clippy::struct_excessive_bools)] // one per action, read by name
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TaskCapabilities {
    /// Change a task.
    pub edit: bool,
    /// Make a new task.
    pub create: bool,
    /// Delete a task.
    pub delete: bool,
    /// Move a task to another column.
    pub move_tasks: bool,
}

impl TaskCapabilities {
    /// Reading only.
    pub fn read_only() -> Self {
        Self::default()
    }

    /// Everything.
    pub fn all() -> Self {
        Self {
            edit: true,
            create: true,
            delete: true,
            move_tasks: true,
        }
    }
}

/// What rows are asked for.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RowQuery {
    /// The search text; empty for all.
    pub term: String,
    /// Only tasks still to be done.
    pub open_only: bool,
}

/// One column of the status view.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskColumn {
    /// Names the column for the source; opaque to the workbench.
    pub key: String,
    /// Its heading.
    pub title: String,
}

impl TaskColumn {
    /// A column.
    pub fn new(key: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
        }
    }
}

/// One task in a list or a column.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskRow {
    /// Names the task for the source; opaque to the workbench.
    pub key: String,
    /// What to call it.
    pub title: String,
    /// The key of its column.
    pub column: String,
    /// One text per list column of [`TaskLabels::columns`](super::TaskLabels::columns).
    pub cells: Vec<String>,
    /// Smaller lines on its card.
    pub lines: Vec<String>,
    /// Tags on its card.
    pub tags: Vec<String>,
    /// Short markings that must be seen wherever it appears, in the source's words.
    pub marks: Vec<String>,
    /// Shown only when the person chooses this row, never opened by the workbench on its own.
    pub only_when_chosen: bool,
    /// It cannot be moved, and why, in the source's words.
    pub move_refusal: Option<String>,
    /// Opaque marker of the state this row was read from; a move from an old one is a conflict.
    pub revision: Option<String>,
}

impl TaskRow {
    /// A row in `column`.
    pub fn new(
        key: impl Into<String>,
        title: impl Into<String>,
        column: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            column: column.into(),
            ..Self::default()
        }
    }

    /// The row of a task to be marked and opened only when chosen (ADR 0029).
    #[must_use]
    pub fn guarded(mut self, mark: impl Into<String>) -> Self {
        self.marks.push(mark.into());
        self.only_when_chosen = true;
        self
    }
}

/// One labelled value of a task.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskField {
    /// What the value is.
    pub label: String,
    /// The value.
    pub value: String,
}

impl TaskField {
    /// A field.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Fields that belong together, under a heading.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSection {
    /// The heading; an empty text shows none.
    pub title: String,
    /// The fields.
    pub fields: Vec<TaskField>,
}

impl TaskSection {
    /// A section.
    pub fn new(title: impl Into<String>, fields: Vec<TaskField>) -> Self {
        Self {
            title: title.into(),
            fields,
        }
    }
}

/// One task, to be read.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskSheet {
    /// What to call it.
    pub title: String,
    /// Short markings that must be seen at once, in the source's words.
    pub marks: Vec<String>,
    /// A sentence to read before changing it.
    pub note: Option<String>,
    /// The details.
    pub sections: Vec<TaskSection>,
    /// The key of its column.
    pub column: String,
    /// It cannot be moved, and why, in the source's words.
    pub move_refusal: Option<String>,
    /// Opaque marker of the state this sheet was read from.
    pub revision: Option<String>,
}

impl TaskSheet {
    /// A sheet.
    pub fn new(title: impl Into<String>, sections: Vec<TaskSection>) -> Self {
        Self {
            title: title.into(),
            sections,
            ..Self::default()
        }
    }
}
