//! Keys to commands.
//!
//! Pattern: Command + Strategy — [`Command`] is one step of the move logic; [`Bindings`] is the
//! default mapping, optional and replaceable: a frontend may build its own or none at all.

use crate::keys::{Key, KeyEvent};

/// One step of the sorter.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Up one item (held: move the item up).
    Up,
    /// Down one item.
    Down,
    /// To the previous lane.
    Left,
    /// To the next lane.
    Right,
    /// To the first item of the lane.
    First,
    /// To the last item of the lane.
    Last,
    /// Up by a page of ten.
    PageUp,
    /// Down by a page of ten.
    PageDown,
    /// Pick up the item under the cursor, or drop the one held.
    Toggle,
    /// Give up the hold.
    Cancel,
    /// Choose the item under the cursor to be opened ([`Outcome::Chosen`](super::Outcome::Chosen),
    /// source keyboard). Not bound by default; bind it to `Key::Activate` to get "Enter opens"
    /// instead of "Enter picks up".
    Open,
}

/// Which key does what.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings {
    map: Vec<(Key, Command)>,
}

impl Default for Bindings {
    /// Arrows, Home, End, Page keys; Space or Enter (arriving as `Activate`) pick up and drop;
    /// Escape gives up.
    fn default() -> Self {
        Self {
            map: vec![
                (Key::Up, Command::Up),
                (Key::Down, Command::Down),
                (Key::Left, Command::Left),
                (Key::Right, Command::Right),
                (Key::Home, Command::First),
                (Key::End, Command::Last),
                (Key::PageUp, Command::PageUp),
                (Key::PageDown, Command::PageDown),
                (Key::Activate, Command::Toggle),
                (Key::Space, Command::Toggle),
                (Key::Enter, Command::Toggle),
                (Key::Escape, Command::Cancel),
            ],
        }
    }
}

impl Bindings {
    /// No keys at all; add with [`Bindings::bind`].
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

    /// The command for a press; none when the key is not bound or a modifier is held (Ctrl, Alt
    /// belong to the program, not to the list).
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
