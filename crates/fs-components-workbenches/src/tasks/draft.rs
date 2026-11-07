//! A task as a form.
//!
//! Pattern: none — labelled fields that remember what they started as, so a source writes
//! only what the person changed and whatever a form does not show stays as it is.

/// One thing a choice field offers.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldOption {
    /// The value kept in the draft.
    pub key: String,
    /// What the person reads.
    pub label: String,
}

impl FieldOption {
    /// An option.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// How a field is filled in.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FieldKind {
    /// One line of text.
    #[default]
    Line,
    /// Several lines of text.
    LongText,
    /// One of these.
    Choice(Vec<FieldOption>),
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
    /// How it is filled in.
    pub kind: FieldKind,
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
            kind: FieldKind::Line,
        }
    }

    /// The same field, filled in this way.
    #[must_use]
    pub fn kind(mut self, kind: FieldKind) -> Self {
        self.kind = kind;
        self
    }

    /// Has the person changed it?
    pub fn changed(&self) -> bool {
        self.value != self.original
    }
}

/// A task as a form. A field emptied by the person means: remove this value.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskDraft {
    /// Opaque marker of the state the draft was made from; a save from an old one is a conflict.
    pub revision: Option<String>,
    /// Short markings that must stay in view while editing, in the source's words.
    pub marks: Vec<String>,
    /// A sentence to read while editing.
    pub note: Option<String>,
    /// The fields.
    pub fields: Vec<DraftField>,
}

impl TaskDraft {
    /// A draft.
    pub fn new(revision: Option<String>, fields: Vec<DraftField>) -> Self {
        Self {
            revision,
            marks: Vec::new(),
            note: None,
            fields,
        }
    }

    /// Has the person changed anything?
    pub fn changed(&self) -> bool {
        self.fields.iter().any(DraftField::changed)
    }

    /// The text of a field.
    pub fn value(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.key == key)
            .map(|f| f.value.as_str())
    }
}
