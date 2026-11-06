//! The handle of a lightbox.
//!
//! Pattern: Facade + Observer — a clonable handle over the [`Viewer`]; no method returns a lock
//! guard; every step answers with an [`Outcome`], tells the callback and draws again.

use std::sync::{Arc, Mutex, PoisonError};

use super::bindings::{Bindings, Command};
use super::model::{Media, Outcome, Viewer};
use crate::keys::KeyEvent;
use crate::live::Live;

/// Called with what every step did.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

/// The words of the box.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Button: previous.
    pub prev: String,
    /// Button: next.
    pub next: String,
    /// Button: close.
    pub close: String,
    /// Counter; `{}` twice: position, then number of items.
    pub counter: String,
    /// The picture is not there or cannot be opened.
    pub unavailable: String,
    /// The item is not a picture.
    pub not_a_picture: String,
    /// No items.
    pub empty: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            prev: s("Previous"),
            next: s("Next"),
            close: s("Close"),
            counter: s("{} / {}"),
            unavailable: s("This picture cannot be shown."),
            not_a_picture: s("This item cannot be shown here."),
            empty: s("Nothing to show."),
        }
    }
}

struct Inner {
    viewer: Viewer,
    on_outcome: Option<OnOutcome>,
}

/// A lightbox. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct LightBox {
    shared: Arc<Mutex<Inner>>,
    pub(super) live: Live,
    pub(super) id: String,
    pub(super) bindings: Option<Bindings>,
    pub(super) labels: Labels,
}

impl LightBox {
    /// A box over `items`, at the first.
    pub fn new(items: Vec<Media>) -> Self {
        Self::over(Viewer::new(items))
    }

    /// A box over a prepared viewer (wrapping, say).
    pub fn over(viewer: Viewer) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Inner {
                viewer,
                on_outcome: None,
            })),
            live: Live::new(),
            id: "fs-lightbox".into(),
            bindings: None,
            labels: Labels::default(),
        }
    }

    /// The element id (buttons are `<id>-prev`, `<id>-next`, `<id>-close`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Take keys while the box has the keyboard, by these bindings.
    #[must_use]
    pub fn keys(mut self, bindings: Bindings) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Hear of every step.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.lock().on_outcome = Some(handler);
        self
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn act(&self, f: impl FnOnce(&mut Viewer) -> Outcome) -> Outcome {
        let (outcome, handler) = {
            let mut inner = self.lock();
            (f(&mut inner.viewer), inner.on_outcome.clone())
        };
        if let (true, Some(handler)) = (outcome != Outcome::Ignored, handler) {
            handler(&outcome);
        }
        self.live.refresh();
        outcome
    }

    pub(super) fn read<T>(&self, f: impl FnOnce(&Viewer) -> T) -> T {
        f(&self.lock().viewer)
    }

    /// Key of the item shown.
    pub fn current_key(&self) -> Option<String> {
        self.read(|v| v.current().map(|m| m.key.clone()))
    }

    /// Position of the item shown and the number of items (counting from 1).
    pub fn position(&self) -> (usize, usize) {
        self.read(Viewer::position)
    }

    /// Replace the items.
    pub fn set_items(&self, items: Vec<Media>) {
        self.lock().viewer.set_items(items);
        self.live.refresh();
    }

    /// Show the item with this key.
    pub fn show(&self, key: &str) -> Outcome {
        self.act(|v| v.show(key))
    }

    /// Do one step.
    pub fn command(&self, command: Command) -> Outcome {
        self.act(|v| match command {
            Command::Next => v.forward(),
            Command::Prev => v.back(),
            Command::First => v.first(),
            Command::Last => v.last(),
            _ => Outcome::Closed,
        })
    }

    /// A key press: through the bindings if there are any ([`Outcome::Ignored`] otherwise).
    pub fn press(&self, event: &KeyEvent) -> Outcome {
        match self.bindings.as_ref().and_then(|b| b.command(event)) {
            Some(command) => self.command(command),
            None => Outcome::Ignored,
        }
    }
}
