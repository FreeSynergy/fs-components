//! What the calendar workbench and its source talk about.
//!
//! Pattern: Value objects — capabilities and sections of labelled fields. Nothing here
//! knows what an event is made of.

use fs_components_widgets::Day;

/// What a source can do; an action it cannot do is not offered.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CalendarCapabilities {
    /// Change an item.
    pub edit: bool,
    /// Make a new item.
    pub create: bool,
    /// Delete an item.
    pub delete: bool,
}

impl CalendarCapabilities {
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
        }
    }
}

/// One labelled value of an item.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SheetField {
    /// What the value is.
    pub label: String,
    /// The value.
    pub value: String,
}

impl SheetField {
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
pub struct SheetSection {
    /// The heading; an empty text shows none.
    pub title: String,
    /// The fields.
    pub fields: Vec<SheetField>,
}

impl SheetSection {
    /// A section.
    pub fn new(title: impl Into<String>, fields: Vec<SheetField>) -> Self {
        Self {
            title: title.into(),
            fields,
        }
    }
}

/// One item, to be read.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemSheet {
    /// What to call it.
    pub title: String,
    /// Short markings that must be seen at once, in the source's words.
    pub marks: Vec<String>,
    /// A sentence to read before acting on it, in the source's words.
    pub note: Option<String>,
    /// The details.
    pub sections: Vec<SheetSection>,
    /// The day to show when the workbench goes to this item.
    pub day: Day,
    /// Can this one be changed here?
    pub can_edit: bool,
    /// Can this one be deleted here?
    pub can_delete: bool,
    /// What a delete would take with it, in the source's words; added to the question.
    pub delete_warning: Option<String>,
}

impl ItemSheet {
    /// A sheet of an item that can be changed and deleted.
    pub fn new(title: impl Into<String>, day: Day, sections: Vec<SheetSection>) -> Self {
        Self {
            title: title.into(),
            marks: Vec::new(),
            note: None,
            sections,
            day,
            can_edit: true,
            can_delete: true,
            delete_warning: None,
        }
    }
}
