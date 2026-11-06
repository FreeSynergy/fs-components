//! What a program says about its menus: labels and entries, nothing about how they look.
//!
//! Pattern: Value object for the menus + Command — an entry is named by a stable id, and one handler
//! of the program receives that id.

use std::sync::Arc;

/// The one function of a program that receives the id of a chosen menu entry.
pub type MenuHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// One line of a menu.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuEntry {
    /// Stable name, unique among all entries of the program; the handler receives it.
    pub id: String,
    /// What the menu shows; comes from the program.
    pub label: String,
    /// Shown, but cannot be chosen.
    pub disabled: bool,
}

impl MenuEntry {
    /// An entry that can be chosen.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
        }
    }

    /// The same entry, shown but not to be chosen.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// One menu of the title bar. A menu without entries is not shown.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Menu {
    /// What the title bar shows; comes from the program.
    pub label: String,
    /// The lines of the menu, in this order.
    pub entries: Vec<MenuEntry>,
}

impl Menu {
    /// A menu with this label and these entries.
    pub fn new(label: impl Into<String>, entries: Vec<MenuEntry>) -> Self {
        Self {
            label: label.into(),
            entries,
        }
    }
}
