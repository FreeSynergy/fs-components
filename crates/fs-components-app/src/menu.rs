//! What a program says about its menus: labels and entries, nothing about how they look.
//!
//! Pattern: Value object — `MenuEntry` and `Menu` are values, compared by value; an entry is named by a stable id
//! and one handler of the program receives that id. There is no operation enum, so this is not a Command.

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
    /// Marked as the chosen one of a group of lines (the shell draws the mark; the label stays the plain name).
    pub checked: bool,
    /// The next level: a line with children opens them instead of choosing itself.
    pub children: Vec<MenuEntry>,
}

impl MenuEntry {
    /// An entry that can be chosen.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            disabled: false,
            checked: false,
            children: Vec::new(),
        }
    }

    /// A line that opens a next level of lines. It is no choice itself: its `id` is only a name for the element, and
    /// the handler never receives it.
    pub fn submenu(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<MenuEntry>,
    ) -> Self {
        Self {
            children,
            ..Self::new(id, label)
        }
    }

    /// The same entry, marked (or not) as the chosen one.
    #[must_use]
    pub fn checked(mut self, on: bool) -> Self {
        self.checked = on;
        self
    }

    /// This entry and all below it, depth first.
    pub(crate) fn walk<'a>(&'a self, out: &mut Vec<&'a MenuEntry>) {
        out.push(self);
        for child in &self.children {
            child.walk(out);
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

impl Menu {
    /// All entries of this menu, also those of the next levels, depth first.
    pub(crate) fn all_entries(&self) -> Vec<&MenuEntry> {
        let mut out = Vec::new();
        for entry in &self.entries {
            entry.walk(&mut out);
        }
        out
    }
}
