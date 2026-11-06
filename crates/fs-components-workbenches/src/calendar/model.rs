//! What the calendar workbench and its source talk about.
//!
//! Pattern: Value objects — capabilities and sections of labelled fields. Nothing here
//! knows what an event is made of.

use fs_components_widgets::{CalendarItem, Day, ViewRange};

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

/// Something that could not be shown, named so that the person can deal with it.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemProblem {
    /// What to open to deal with it, if there is something to open.
    pub key: Option<String>,
    /// What is wrong, in the source's words.
    pub text: String,
}

impl ItemProblem {
    /// A problem with an item that can be opened.
    pub fn of(key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            text: text.into(),
        }
    }
}

/// What is on some days: what could be read, and what could not. One bad item costs itself only.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemsRead {
    /// The items.
    pub items: Vec<CalendarItem>,
    /// What could not be shown.
    pub problems: Vec<ItemProblem>,
}

impl ItemsRead {
    /// All of it was read.
    pub fn all(items: Vec<CalendarItem>) -> Self {
        Self {
            items,
            problems: Vec::new(),
        }
    }

    /// Some of it could not be.
    pub fn with_problems(items: Vec<CalendarItem>, problems: Vec<ItemProblem>) -> Self {
        Self { items, problems }
    }
}

/// What the workbench is showing, for the program around it (ADR 0056). A plain value.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarContext {
    /// The key of the item open, if one is.
    pub item: Option<String>,
    /// What is shown: a month, a week, a day or the agenda.
    pub view: ViewRange,
    /// The day being worked on: the day chosen if it is in view, else the day the view is around.
    pub date: Day,
}
