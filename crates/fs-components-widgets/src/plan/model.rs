//! The plain data of the plan screens.
//!
//! Pattern: Value object — what the host fills in; texts are already in the reader's language, no program type
//! appears here.

use super::card::PlanCard;

/// A view tab (Board, List, Calendar ...). A tab that is not built is `locked`: drawn weak, does nothing, says why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tab {
    /// Names the tab for the host.
    pub key: String,
    /// Its text.
    pub title: String,
    /// Not built yet: shown, grey, without a click.
    pub locked: bool,
}

impl Tab {
    /// A tab that works.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            locked: false,
        }
    }

    /// The same tab, not built yet.
    #[must_use]
    pub fn locked(mut self) -> Self {
        self.locked = true;
        self
    }
}

/// One column of the board.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    /// Names the column for the host.
    pub key: String,
    /// The heading.
    pub title: String,
    /// The cards, in order.
    pub cards: Vec<PlanCard>,
}

impl Column {
    /// An empty column.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            cards: Vec::new(),
        }
    }

    /// The heading with the number of cards: `To do  26`, as the board shows it.
    pub fn head(&self) -> (String, String) {
        (self.title.clone(), self.cards.len().to_string())
    }
}

/// One labelled value in the side panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Detail {
    /// What it is.
    pub label: String,
    /// Its value.
    pub value: String,
}

impl Detail {
    /// A value with its label.
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// The side panel of the chosen card: it exists only while a card is chosen.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Panel {
    /// The title of the card.
    pub title: String,
    /// A sentence under the title (the reason a task waits).
    pub note: Option<String>,
    /// The values.
    pub details: Vec<Detail>,
}
