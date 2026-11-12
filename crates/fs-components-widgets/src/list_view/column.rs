//! One column of the list: its heading, its room, and what the heading can do.
//!
//! Pattern: none — plain data the caller gives; the list has no idea what a column means.

/// How much room a column takes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    /// Exactly this many pixels.
    Pixels(f32),
    /// A share of what the fixed columns leave over.
    Share(f32),
}

/// One column: a heading, which comes from the caller, and its width.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Column {
    /// The heading; an empty text shows nothing.
    pub title: String,
    /// The room it takes.
    pub width: ColumnWidth,
    /// What the list reports to `on_head` when the heading is clicked; `None` = not clickable.
    pub sort: Option<String>,
    /// `Some(descending)` while the list is ordered by this column (an arrow shows it).
    pub order: Option<bool>,
    /// Where this column stands among the columns the list is ordered by (1 = first); shown next to the arrow
    /// when there is more than one.
    pub rank: Option<u8>,
    /// What the column is called in a layout (`ColumnLayout`); without one the `sort` key, else the title.
    pub id: Option<String>,
}

impl Column {
    /// A column with this heading and width.
    pub fn new(title: impl Into<String>, width: ColumnWidth) -> Self {
        Self {
            title: title.into(),
            width,
            sort: None,
            order: None,
            rank: None,
            id: None,
        }
    }

    /// Name the column for a layout (what the person hides, moves and widens is kept by this name).
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Where the column stands among several order columns (1 = first).
    #[must_use]
    pub fn ranked(mut self, rank: u8) -> Self {
        self.rank = Some(rank);
        self
    }

    /// The name of the column in a layout.
    pub fn key(&self) -> String {
        self.id
            .clone()
            .or_else(|| self.sort.clone())
            .unwrap_or_else(|| self.title.clone())
    }

    /// Make the heading clickable: a click reports `key` to `ListView::on_head`.
    #[must_use]
    pub fn sortable(mut self, key: impl Into<String>) -> Self {
        self.sort = Some(key.into());
        self
    }

    /// The list is ordered by this column, backwards if `descending`.
    #[must_use]
    pub fn ordered(mut self, descending: bool) -> Self {
        self.order = Some(descending);
        self
    }
}
