//! What the caller gives the list: columns, rows and what to do with a chosen row.
//!
//! Pattern: Strategy + Value objects — [`RowSource`] is all the widget asks of its data; [`Column`]
//! and [`Row`] are plain data, so the file manager and the people program use the same list.

use std::sync::Arc;

use super::position::ListPosition;
use super::window::DEFAULT_ROW_HEIGHT;

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
}

impl Column {
    /// A column with this heading and width.
    pub fn new(title: impl Into<String>, width: ColumnWidth) -> Self {
        Self {
            title: title.into(),
            width,
        }
    }
}

/// One row: a key that the selection reports, and one text per column.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// Names the row for the caller; the widget only hands it back.
    pub key: String,
    /// The texts, in the order of the columns. Missing ones stay empty, extra ones are not shown.
    pub cells: Vec<String>,
}

impl Row {
    /// A row with this key and these texts.
    pub fn new(key: impl Into<String>, cells: Vec<String>) -> Self {
        Self {
            key: key.into(),
            cells,
        }
    }
}

/// Where the rows come from. Called on the interface thread, a page at a time, while scrolling.
pub trait RowSource: Send + Sync {
    /// How many rows there are.
    fn count(&self) -> usize;
    /// The rows from `first` on, at most `limit`. A source may answer with fewer.
    ///
    /// # Errors
    /// A reason a person can read; the list shows it in place of the rows.
    fn rows(&self, first: usize, limit: usize) -> Result<Vec<Row>, String>;
}

/// Rows that are all in memory.
#[derive(Clone, Debug, Default)]
pub struct VecRows(pub Arc<Vec<Row>>);

impl VecRows {
    /// These rows.
    pub fn new(rows: Vec<Row>) -> Self {
        Self(Arc::new(rows))
    }
}

impl RowSource for VecRows {
    fn count(&self) -> usize {
        self.0.len()
    }

    fn rows(&self, first: usize, limit: usize) -> Result<Vec<Row>, String> {
        Ok(self.0.iter().skip(first).take(limit).cloned().collect())
    }
}

/// Called with the key of the row the person chose.
pub type OnSelect = Arc<dyn Fn(&str) + Send + Sync>;

/// The list, ready to be built into a layout.
#[must_use]
pub struct ListView {
    pub(super) columns: Vec<Column>,
    pub(super) source: Arc<dyn RowSource>,
    pub(super) selected: Option<String>,
    pub(super) on_select: Option<OnSelect>,
    pub(super) row_height: f32,
    pub(super) position: Option<ListPosition>,
    pub(super) keys: bool,
    pub(super) empty: Option<String>,
    pub(super) name: Option<String>,
}

impl ListView {
    /// A list of `source` with these columns; nothing selected, keys off.
    pub fn new(columns: Vec<Column>, source: Arc<dyn RowSource>) -> Self {
        Self {
            columns,
            source,
            selected: None,
            on_select: None,
            row_height: DEFAULT_ROW_HEIGHT,
            position: None,
            keys: false,
            empty: None,
            name: None,
        }
    }

    /// The row with this key stands out. The list does not remember a choice by itself: the
    /// caller answers `on_select` by building the list again with the new key.
    pub fn selected(mut self, key: Option<&str>) -> Self {
        self.selected = key.map(str::to_owned);
        self
    }

    /// Report the key of the row the person chose.
    pub fn on_select(mut self, handler: OnSelect) -> Self {
        self.on_select = Some(handler);
        self
    }

    /// The height of every row in pixels. All rows have one height: that is what lets a very long
    /// list know where a row is without measuring the ones before it.
    pub fn row_height(mut self, pixels: f32) -> Self {
        self.row_height = pixels.max(1.0);
        self
    }

    /// Keep the scroll position here, across rebuilds. Without one a rebuild starts at the top.
    pub fn position(mut self, position: &ListPosition) -> Self {
        self.position = Some(position.clone());
        self
    }

    /// Switch the arrow keys, Page Up/Down, Home and End on (default off): the list becomes a tab
    /// stop and acts on a key only while it has the keyboard, so several lists and a program's own
    /// shortcuts (ADR 0037) do not collide.
    pub fn keys(mut self, on: bool) -> Self {
        self.keys = on;
        self
    }

    /// Give the list the element id `<name>` and every row `<name>-<key>`, so a program or a test
    /// can find them. Names must differ between lists in one window. Default: no ids.
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// The text to show when there are no rows; it comes from the caller.
    pub fn when_empty(mut self, text: impl Into<String>) -> Self {
        self.empty = Some(text.into());
        self
    }
}
