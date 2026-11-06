//! Keys to commands (optional).
//!
//! Pattern: Command + Strategy, as in `sortable-list`. A key typed into the filter field never
//! reaches these (the key guard ignores keys while a text input has the keyboard).

use crate::keys::{Key, KeyEvent};

/// One step of the dialog.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    /// Cursor up.
    Up,
    /// Cursor down.
    Down,
    /// Choose or unchoose the row under the cursor.
    Toggle,
    /// The previous permission of the row.
    PermissionBack,
    /// The next permission of the row.
    PermissionForward,
    /// Confirm.
    Confirm,
    /// Cancel.
    Cancel,
}

/// Which key does what.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bindings {
    map: Vec<(Key, Command)>,
}

impl Default for Bindings {
    /// Up/Down, Space or Enter (arriving as `Activate`) choose or unchoose, Left/Right change the
    /// permission, Escape cancels. Confirm has no key by default: a window takes Enter for
    /// itself, so a person confirms by Tab to the Done button and Enter or Space there (that is
    /// Blinc's own button activation). A frontend may bind `Command::Confirm` to another key.
    fn default() -> Self {
        Self {
            map: vec![
                (Key::Up, Command::Up),
                (Key::Down, Command::Down),
                (Key::Activate, Command::Toggle),
                (Key::Space, Command::Toggle),
                (Key::Left, Command::PermissionBack),
                (Key::Right, Command::PermissionForward),
                (Key::Escape, Command::Cancel),
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
