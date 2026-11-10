//! The four ways a basic type is shown (ADR 0066, item 4).
//!
//! Pattern: none (plain data).

/// Which of the four components of a basic type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    /// To type into, split into its parts, with the question under it.
    Input,
    /// To read: every part with its label.
    Display,
    /// In one line of running text, with the icon of the type.
    Compact,
    /// In a list: one line, no icon, the height of a row.
    Cell,
}

impl Mode {
    /// All four, in a fixed order.
    pub const ALL: [Mode; 4] = [Mode::Input, Mode::Display, Mode::Compact, Mode::Cell];
}
