//! What the people workbench and its source talk about.
//!
//! Pattern: Value objects — neutral rows, sections of labelled fields and a draft of labelled
//! fields. Nothing here knows what a person is made of.

/// What a source can do; an action it cannot do is not offered.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PeopleCapabilities {
    /// Change a person.
    pub edit: bool,
    /// Make a new person.
    pub create: bool,
    /// Delete a person.
    pub delete: bool,
}

impl PeopleCapabilities {
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

/// One row of the list.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonRow {
    /// Names the person for the source; opaque to the workbench.
    pub key: String,
    /// One text per column of [`PeopleLabels::columns`](super::PeopleLabels::columns).
    pub cells: Vec<String>,
}

impl PersonRow {
    /// A row.
    pub fn new(key: impl Into<String>, cells: Vec<String>) -> Self {
        Self {
            key: key.into(),
            cells,
        }
    }
}

/// One labelled value of a person.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonField {
    /// What the value is.
    pub label: String,
    /// The value.
    pub value: String,
}

impl PersonField {
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
pub struct PersonSection {
    /// The heading; an empty text shows none.
    pub title: String,
    /// The fields.
    pub fields: Vec<PersonField>,
}

impl PersonSection {
    /// A section.
    pub fn new(title: impl Into<String>, fields: Vec<PersonField>) -> Self {
        Self {
            title: title.into(),
            fields,
        }
    }
}

/// One person, to be read.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonSheet {
    /// What to call the person.
    pub title: String,
    /// Short markings that must be seen at once (an AI, sensitive data), in the source's words.
    pub marks: Vec<String>,
    /// The details.
    pub sections: Vec<PersonSection>,
}

impl PersonSheet {
    /// A sheet.
    pub fn new(title: impl Into<String>, sections: Vec<PersonSection>) -> Self {
        Self {
            title: title.into(),
            marks: Vec::new(),
            sections,
        }
    }
}

/// One field of a draft.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftField {
    /// Names the field for the source; unique within the draft.
    pub key: String,
    /// What it asks for.
    pub label: String,
    /// The text as it is now.
    pub value: String,
    /// The text when the draft was made; the source writes only what differs from it, so whatever
    /// a form does not show stays as it is.
    pub original: String,
    /// Added in this edit: it has no counterpart in the source yet.
    pub added: bool,
}

impl DraftField {
    /// A field holding `value`.
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            key: key.into(),
            label: label.into(),
            original: value.clone(),
            value,
            added: false,
        }
    }

    /// Has the person changed it?
    pub fn changed(&self) -> bool {
        self.value != self.original
    }
}

/// Fields that belong together; a group that may grow offers an Add action.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftGroup {
    /// Names the group for the source.
    pub key: String,
    /// The heading, and the label of a field added to it.
    pub label: String,
    /// May the person add fields?
    pub can_add: bool,
    /// The fields.
    pub fields: Vec<DraftField>,
}

impl DraftGroup {
    /// A group.
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        can_add: bool,
        fields: Vec<DraftField>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            can_add,
            fields,
        }
    }
}

/// A person as a form. A field emptied by the person means: remove this value.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonDraft {
    /// Opaque marker of the state the draft was made from; a save from an old one is a conflict.
    pub revision: Option<String>,
    /// The fields, by group.
    pub groups: Vec<DraftGroup>,
}

impl PersonDraft {
    /// A draft.
    pub fn new(revision: Option<String>, groups: Vec<DraftGroup>) -> Self {
        Self { revision, groups }
    }

    /// Every field, in order.
    pub fn fields(&self) -> impl Iterator<Item = &DraftField> {
        self.groups.iter().flat_map(|g| g.fields.iter())
    }

    /// Has the person changed anything?
    pub fn changed(&self) -> bool {
        self.fields().any(DraftField::changed)
    }
}
