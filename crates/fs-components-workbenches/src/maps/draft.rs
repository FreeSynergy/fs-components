//! A place as a form.
//!
//! Pattern: Value objects — labelled fields with what they held at the start, so a source can write
//! only what differs and keep everything a form does not show.

/// How a field of a draft is asked for.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldKind {
    /// A line of text.
    #[default]
    Text,
    /// Several lines of text.
    LongText,
    /// Yes or no; the value is `true` or `false`.
    Flag,
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
    /// The text when the draft was made; the source writes only what differs from it, so whatever a
    /// form does not show stays as it is.
    pub original: String,
    /// How it is asked for.
    pub kind: FieldKind,
    /// Shown but not to be changed; the draft's note says why.
    pub read_only: bool,
}

impl DraftField {
    /// A text field holding `value`.
    pub fn new(key: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            key: key.into(),
            label: label.into(),
            original: value.clone(),
            value,
            kind: FieldKind::Text,
            read_only: false,
        }
    }

    /// Asked for this way.
    #[must_use]
    pub fn kind(mut self, kind: FieldKind) -> Self {
        self.kind = kind;
        self
    }

    /// Shown but not to be changed.
    #[must_use]
    pub fn locked(mut self) -> Self {
        self.read_only = true;
        self
    }

    /// Has the person changed it?
    pub fn changed(&self) -> bool {
        self.value != self.original
    }
}

/// A place as a form.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaceDraft {
    /// Opaque marker of the state the draft was made from; a save from an old one is a conflict.
    pub revision: Option<String>,
    /// Short markings that must stay in view while editing, in the source's words.
    pub marks: Vec<String>,
    /// A sentence about what this edit does and does not reach, in the source's words.
    pub note: Option<String>,
    /// The fields.
    pub fields: Vec<DraftField>,
}

impl PlaceDraft {
    /// A draft.
    pub fn new(revision: Option<String>, fields: Vec<DraftField>) -> Self {
        Self {
            revision,
            marks: Vec::new(),
            note: None,
            fields,
        }
    }

    /// The text of a field.
    pub fn value(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|f| f.key == key)
            .map(|f| f.value.as_str())
    }

    /// Has the person changed anything?
    pub fn changed(&self) -> bool {
        self.fields.iter().any(DraftField::changed)
    }
}
