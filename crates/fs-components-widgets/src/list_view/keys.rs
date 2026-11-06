//! The keys of the list.
//!
//! Pattern: pure function from a key to the row it leads to; `view.rs` connects it to the framework.

/// A key the list reacts to, when its keys are switched on.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListKey {
    /// One row up.
    Up,
    /// One row down.
    Down,
    /// One view up.
    PageUp,
    /// One view down.
    PageDown,
    /// The first row.
    Home,
    /// The last row.
    End,
}

impl ListKey {
    /// The key with this key code of the framework.
    pub(super) fn from_code(code: u32) -> Option<Self> {
        match code {
            0x26 => Some(Self::Up),
            0x28 => Some(Self::Down),
            0x21 => Some(Self::PageUp),
            0x22 => Some(Self::PageDown),
            0x24 => Some(Self::Home),
            0x23 => Some(Self::End),
            _ => None,
        }
    }
}

/// The row `key` leads to, from the row `cursor` (none chosen yet: the first), in a list of `count`
/// rows with `page` rows in view. `None` if there are no rows.
pub fn key_target(key: ListKey, cursor: Option<usize>, count: usize, page: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    let Some(cursor) = cursor else {
        return Some(if key == ListKey::End { last } else { 0 });
    };
    let cursor = cursor.min(last);
    let page = page.max(1);
    Some(match key {
        ListKey::Up => cursor.saturating_sub(1),
        ListKey::Down => (cursor + 1).min(last),
        ListKey::PageUp => cursor.saturating_sub(page),
        ListKey::PageDown => cursor.saturating_add(page).min(last),
        ListKey::Home => 0,
        ListKey::End => last,
    })
}
