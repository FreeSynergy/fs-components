//! The plain data of the board: columns, cards, the operations a host may support.
//!
//! Pattern: Value object — small types the host fills; no program type appears here, texts are
//! already in the reader's language (they are the host's data, not component words).

/// Colour slot of a label (the style decides the colour).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Tone {
    /// The accent colour.
    #[default]
    Accent,
    /// Success colour.
    Success,
    /// Error colour.
    Error,
    /// Neutral, weak.
    Neutral,
}

/// A label on a card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    /// Text.
    pub text: String,
    /// Colour slot.
    pub tone: Tone,
}

/// How urgent a due date is; the host decides, the card only marks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DueState {
    /// Some time away.
    Open,
    /// Close.
    Soon,
    /// Past.
    Overdue,
    /// The card is done.
    Done,
}

/// The due mark of a card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Due {
    /// The date as the reader should see it.
    pub text: String,
    /// Urgency.
    pub state: DueState,
}

/// One card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Card {
    /// Identity, handed back by every event.
    pub key: String,
    /// Title.
    pub title: String,
    /// Second line.
    pub subtitle: Option<String>,
    /// Labels.
    pub labels: Vec<Label>,
    /// Due mark.
    pub due: Option<Due>,
    /// Name of the assignee; the card shows initials.
    pub assignee: Option<String>,
}

impl Card {
    /// A card with a title and nothing else.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            subtitle: None,
            labels: Vec::new(),
            due: None,
            assignee: None,
        }
    }
}

/// One column.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    /// Identity.
    pub key: String,
    /// Title.
    pub title: String,
    /// Work-in-progress limit, if any.
    pub limit: Option<usize>,
    /// Cards, in order.
    pub cards: Vec<Card>,
}

impl Column {
    /// An empty column without a limit.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            limit: None,
            cards: Vec::new(),
        }
    }

    /// More cards than the limit allows.
    pub fn over_limit(&self) -> bool {
        self.limit.is_some_and(|l| self.cards.len() > l)
    }
}

/// What a board can be asked to do; the host says which it supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    /// Add a column.
    AddColumn,
    /// Rename a column.
    Rename,
    /// Set a column's limit.
    Limit,
    /// Move a card to another column.
    Move,
}

impl Operation {
    /// All operations.
    pub const ALL: [Operation; 4] = [
        Operation::AddColumn,
        Operation::Rename,
        Operation::Limit,
        Operation::Move,
    ];

    /// Name for translation keys and element ids.
    pub fn name(self) -> &'static str {
        match self {
            Operation::AddColumn => "add_column",
            Operation::Rename => "rename",
            Operation::Limit => "limit",
            Operation::Move => "move",
        }
    }
}
