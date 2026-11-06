//! Which item is shown.
//!
//! Pattern: State machine, pure — no Blinc. Every step answers with an [`Outcome`].

/// An item to show. Only pictures are drawn; anything else is announced, not skipped.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Media {
    /// Identity given back in outcomes.
    pub key: String,
    /// Path or address of the picture.
    pub source: String,
    /// A title shown under the picture; may be empty.
    pub caption: String,
    /// False for something that is not a picture (a video, a document): the box says so.
    pub picture: bool,
}

impl Media {
    /// A picture.
    pub fn picture(
        key: impl Into<String>,
        source: impl Into<String>,
        caption: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            source: source.into(),
            caption: caption.into(),
            picture: true,
        }
    }

    /// Something the box cannot draw (it shows the caption and a note).
    pub fn other(key: impl Into<String>, caption: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            source: String::new(),
            caption: caption.into(),
            picture: false,
        }
    }
}

/// Why a step did nothing.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// First or last item and stepping does not wrap.
    AtEdge,
    /// There are no items.
    Empty,
    /// No item with that key.
    NoSuchItem,
}

/// What a step did.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// This item is shown now (key).
    Shown(String),
    /// The person closed the box; the caller removes it.
    Closed,
    /// The step was not possible.
    Refused(Refusal),
    /// The key is not bound to anything.
    Ignored,
}

/// The items and the one shown.
#[derive(Clone, Debug, Default)]
pub struct Viewer {
    items: Vec<Media>,
    index: usize,
    wrap: bool,
}

impl Viewer {
    /// A viewer at the first item, not wrapping.
    pub fn new(items: Vec<Media>) -> Self {
        Self {
            items,
            index: 0,
            wrap: false,
        }
    }

    /// Wrap from the last item to the first and back.
    #[must_use]
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Does stepping wrap around?
    pub fn wraps(&self) -> bool {
        self.wrap
    }

    /// The items.
    pub fn items(&self) -> &[Media] {
        &self.items
    }

    /// The item shown (none for no items).
    pub fn current(&self) -> Option<&Media> {
        self.items.get(self.index)
    }

    /// Position of the item shown and the number of items, counting from 1: `(3, 12)`; `(0, 0)`
    /// for none.
    pub fn position(&self) -> (usize, usize) {
        if self.items.is_empty() {
            (0, 0)
        } else {
            (self.index + 1, self.items.len())
        }
    }

    /// Replace the items; the item shown stays when its key is still there (else the nearest
    /// index).
    pub fn set_items(&mut self, items: Vec<Media>) {
        let key = self.current().map(|m| m.key.clone());
        self.items = items;
        self.index = key
            .and_then(|k| self.items.iter().position(|m| m.key == k))
            .unwrap_or(self.index)
            .min(self.items.len().saturating_sub(1));
    }

    /// Show the item with this key.
    pub fn show(&mut self, key: &str) -> Outcome {
        match self.items.iter().position(|m| m.key == key) {
            Some(i) => self.go(i),
            None => Outcome::Refused(Refusal::NoSuchItem),
        }
    }

    /// The next item.
    pub fn forward(&mut self) -> Outcome {
        self.step(true)
    }

    /// The previous item.
    pub fn back(&mut self) -> Outcome {
        self.step(false)
    }

    /// The first item.
    pub fn first(&mut self) -> Outcome {
        self.go(0)
    }

    /// The last item.
    pub fn last(&mut self) -> Outcome {
        self.go(self.items.len().wrapping_sub(1))
    }

    fn step(&mut self, forward: bool) -> Outcome {
        let len = self.items.len();
        let target = match (forward, self.index) {
            _ if len == 0 => return Outcome::Refused(Refusal::Empty),
            (true, i) if i + 1 < len => i + 1,
            (true, _) if self.wrap => 0,
            (false, i) if i > 0 => i - 1,
            (false, _) if self.wrap => len - 1,
            _ => return Outcome::Refused(Refusal::AtEdge),
        };
        self.go(target)
    }

    fn go(&mut self, index: usize) -> Outcome {
        match self.items.get(index) {
            Some(item) => {
                self.index = index;
                Outcome::Shown(item.key.clone())
            }
            None => Outcome::Refused(Refusal::Empty),
        }
    }
}
