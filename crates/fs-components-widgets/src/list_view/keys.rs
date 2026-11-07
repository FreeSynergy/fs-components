//! Which row a key leads to.
//!
//! Pattern: none — a pure function, the list's answer to a [`Key`]; `view.rs` connects it through
//! `bind_keys`, which already keeps a press from reaching two widgets.

use crate::keys::Key;

/// The row `key` leads to, from the row `cursor` (none chosen yet: the first), in a list of `count`
/// rows with `page` rows in view. `None` if there are no rows or the list does not use this key.
pub fn key_target(key: Key, cursor: Option<usize>, count: usize, page: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    let page = page.max(1);
    match (key, cursor.map(|c| c.min(last))) {
        (Key::Home, _) | (Key::Up | Key::Down | Key::PageUp | Key::PageDown, None) => Some(0),
        (Key::End, _) => Some(last),
        (Key::Up, Some(c)) => Some(c.saturating_sub(1)),
        (Key::Down, Some(c)) => Some((c + 1).min(last)),
        (Key::PageUp, Some(c)) => Some(c.saturating_sub(page)),
        (Key::PageDown, Some(c)) => Some(c.saturating_add(page).min(last)),
        _ => None,
    }
}
