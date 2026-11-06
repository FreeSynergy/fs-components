//! The neutral sender and the words for its kind.
//!
//! Pattern: Value objects. The kind is data from the caller; the component never guesses it from
//! a name or a text. `Sender::new` starts as `Unknown`: being a person must be said.

/// What kind of participant wrote something.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SenderKind {
    /// Not known. Drawn as unknown, never as a person.
    #[default]
    Unknown,
    /// A human.
    Person,
    /// A program that is not an AI.
    Software,
    /// An AI.
    Ai,
}

/// The one who sent a message or runs an agent.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sender {
    /// Stable key of the sender (for the caller to find it again).
    pub key: String,
    /// Name to show.
    pub name: String,
    /// What kind of participant it is.
    pub kind: SenderKind,
    /// Who answers for it (for an AI: the operator); may be empty. Shown next to the name.
    pub operator: String,
}

impl Sender {
    /// A sender of unknown kind. Use [`Sender::of_kind`] to say what it is.
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            kind: SenderKind::Unknown,
            operator: String::new(),
        }
    }

    /// The same sender, of this kind.
    #[must_use]
    pub fn of_kind(mut self, kind: SenderKind) -> Self {
        self.kind = kind;
        self
    }

    /// The same sender with the one who answers for it.
    #[must_use]
    pub fn operated_by(mut self, operator: impl Into<String>) -> Self {
        self.operator = operator.into();
        self
    }
}

/// The words of the sender marks. They come from the caller; English by default.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KindLabels {
    /// A person.
    pub person: String,
    /// Software.
    pub software: String,
    /// An AI.
    pub ai: String,
    /// Not known.
    pub unknown: String,
}

impl Default for KindLabels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            person: s("Person"),
            software: s("Software"),
            ai: s("AI"),
            unknown: s("Unknown"),
        }
    }
}

impl KindLabels {
    /// The word for a kind.
    pub fn word(&self, kind: SenderKind) -> &str {
        match kind {
            SenderKind::Person => &self.person,
            SenderKind::Software => &self.software,
            SenderKind::Ai => &self.ai,
            _ => &self.unknown,
        }
    }
}
