//! The list as an element.
//!
//! Pattern: Adapter — the entries become the rows and columns of the `list-view` widget; the words are the caller's
//! (translatable, `coding.md` §2a). The look is the widget's and the style's, not this crate's.

use std::sync::Arc;

use fs_components_widgets::{Column, ColumnWidth, Element, ListPosition, ListView, Row, VecRows};

use crate::{FileEntry, SortBy, human_size, sorted};

/// The words the list shows.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct FileListWords {
    /// Heading of the name column.
    pub name: String,
    /// Heading of the size column.
    pub size: String,
    /// Heading of the type column.
    pub kind: String,
    /// Heading of the date column.
    pub modified: String,
    /// The mark before the name of a folder.
    pub folder: String,
    /// Shown when there is nothing.
    pub empty: String,
}

impl FileListWords {
    /// The words.
    pub fn new(
        (name, size, kind, modified): (&str, &str, &str, &str),
        folder: &str,
        empty: &str,
    ) -> Self {
        Self {
            name: name.into(),
            size: size.into(),
            kind: kind.into(),
            modified: modified.into(),
            folder: folder.into(),
            empty: empty.into(),
        }
    }
}

/// The list of files and folders.
#[must_use]
pub struct FileList {
    entries: Vec<FileEntry>,
    words: FileListWords,
    selected: Option<String>,
    on_select: Option<fs_components_widgets::list_view::OnSelect>,
    position: ListPosition,
}

impl FileList {
    /// A list of `entries`, ordered by `sort`.
    pub fn new(
        entries: Vec<FileEntry>,
        sort: SortBy,
        descending: bool,
        words: FileListWords,
    ) -> Self {
        Self {
            entries: sorted(entries, sort, descending),
            words,
            selected: None,
            on_select: None,
            position: ListPosition::new(),
        }
    }

    /// The entry with this key stands out.
    pub fn selected(mut self, key: Option<&str>) -> Self {
        self.selected = key.map(str::to_owned);
        self
    }

    /// Report the key of the row the person chose.
    pub fn on_select(mut self, handler: fs_components_widgets::list_view::OnSelect) -> Self {
        self.on_select = Some(handler);
        self
    }

    /// The scroll position to keep between builds.
    pub fn position(mut self, position: &ListPosition) -> Self {
        self.position = position.clone();
        self
    }

    /// The keys in the order shown.
    pub fn keys(&self) -> Vec<&str> {
        self.entries.iter().map(|e| e.key.as_str()).collect()
    }

    /// The rows: name, size (empty for a folder), type, date.
    pub fn rows(&self) -> Vec<Row> {
        self.entries
            .iter()
            .map(|e| {
                let size = if e.is_folder {
                    String::new()
                } else {
                    human_size(e.size)
                };
                let row = Row::new(
                    e.key.clone(),
                    vec![
                        e.name.clone(),
                        size,
                        e.media_type.clone(),
                        e.modified.clone(),
                    ],
                );
                if e.is_folder {
                    row.with_mark(self.words.folder.clone())
                } else {
                    row
                }
            })
            .collect()
    }

    /// The element, `width` x `height`.
    pub fn build(self, width: f32, height: f32) -> impl Element + use<> {
        let columns = vec![
            Column::new(self.words.name.clone(), ColumnWidth::Share(3.0)),
            Column::new(self.words.size.clone(), ColumnWidth::Pixels(90.0)),
            Column::new(self.words.kind.clone(), ColumnWidth::Share(1.5)),
            Column::new(self.words.modified.clone(), ColumnWidth::Share(2.0)),
        ];
        let mut view = ListView::new(columns, Arc::new(VecRows::new(self.rows())))
            .selected(self.selected.as_deref())
            .position(&self.position)
            .named("fs-file-list")
            .when_empty(self.words.empty.clone());
        if let Some(handler) = self.on_select {
            view = view.on_select(handler);
        }
        view.build(width, height)
    }
}
