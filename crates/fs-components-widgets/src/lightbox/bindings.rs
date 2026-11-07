//! Keys to commands (optional).
//!
//! Pattern: Value object, as `sortable-list/bindings`: plain data, the keys and the step each one names.

use crate::keys::{Key, KeyEvent};

/// One step of the lightbox.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// The next item.
    Next,
    /// The previous item.
    Prev,
    /// The first item.
    First,
    /// The last item.
    Last,
    /// Close the box.
    Close,
}

/// Which key does what.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings {
    map: Vec<(Key, Command)>,
}

impl Default for Bindings {
    /// Left and Right step, Home and End jump, Escape closes.
    fn default() -> Self {
        Self {
            map: vec![
                (Key::Right, Command::Next),
                (Key::Left, Command::Prev),
                (Key::PageDown, Command::Next),
                (Key::PageUp, Command::Prev),
                (Key::Home, Command::First),
                (Key::End, Command::Last),
                (Key::Escape, Command::Close),
            ],
        }
    }
}

impl Bindings {
    /// No keys; add with [`Bindings::bind`].
    pub fn none() -> Self {
        Self { map: Vec::new() }
    }

    /// `key` does `command` (replacing what it did).
    #[must_use]
    pub fn bind(mut self, key: Key, command: Command) -> Self {
        self.map.retain(|(k, _)| *k != key);
        self.map.push((key, command));
        self
    }

    /// The command for a press (none with Ctrl or Alt held, or for an unbound key).
    pub fn command(&self, event: &KeyEvent) -> Option<Command> {
        if event.ctrl || event.alt {
            return None;
        }
        self.map
            .iter()
            .find(|(k, _)| *k == event.key)
            .map(|(_, c)| *c)
    }
}
