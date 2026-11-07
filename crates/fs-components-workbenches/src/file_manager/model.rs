//! What the file source and the workbench talk about.
//!
//! Pattern: none — plain data, with nothing of a store or a program core in it. A key is a
//! text the source understands; nothing in the workbench looks inside it.

use std::any::Any;
use std::sync::Arc;

/// A file or a folder.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// A file.
    File,
    /// A folder.
    Folder,
}

/// Names something the source knows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryRef {
    /// The key.
    pub key: String,
    /// What it is.
    pub kind: EntryKind,
}

/// One thing in a listing.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileEntry {
    /// The key.
    pub key: String,
    /// The real name.
    pub name: String,
    /// What it is.
    pub kind: EntryKind,
    /// Media type; empty for a folder.
    pub media_type: String,
    /// Bytes; 0 for a folder.
    pub size: u64,
    /// When it last changed, in words, if the source can say.
    pub modified: Option<String>,
    /// Categories of sensitive data (ADR 0029); not empty = marked, wherever it appears.
    pub sensitive: Vec<String>,
    /// The folder it is in; `None` = the top.
    pub folder: Option<String>,
    /// In the trash.
    pub trashed: bool,
}

impl FileEntry {
    /// A file.
    pub fn file(key: impl Into<String>, name: impl Into<String>, size: u64) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            kind: EntryKind::File,
            media_type: String::new(),
            size,
            modified: None,
            sensitive: Vec::new(),
            folder: None,
            trashed: false,
        }
    }

    /// A folder.
    pub fn folder(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            kind: EntryKind::Folder,
            ..Self::file(key, name, 0)
        }
    }

    /// Is it marked sensitive?
    pub fn is_marked(&self) -> bool {
        !self.sensitive.is_empty()
    }

    /// What names it for the source.
    pub fn reference(&self) -> EntryRef {
        EntryRef {
            key: self.key.clone(),
            kind: self.kind,
        }
    }
}

/// A folder, for the tree and the breadcrumb.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderInfo {
    /// The key.
    pub key: String,
    /// The real name.
    pub name: String,
    /// The folder it is in; `None` = the top.
    pub parent: Option<String>,
}

impl FolderInfo {
    /// A folder.
    pub fn new(key: impl Into<String>, name: impl Into<String>, parent: Option<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            parent,
        }
    }
}

/// What to sort by.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortKey {
    /// The name.
    #[default]
    Name,
    /// When it last changed.
    Modified,
    /// The size.
    Size,
    /// The media type.
    Type,
}

/// A sort order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SortOrder {
    /// The key.
    pub key: SortKey,
    /// Largest, newest or last first.
    pub descending: bool,
}

/// Where a listing looks.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Directly in this folder; `None` = the top.
    Folder(Option<String>),
    /// In all folders.
    Everywhere,
    /// In the trash.
    Trash,
}

/// A filter by size and date. Dates are `YYYY-MM-DD`.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListFilter {
    /// At least this many bytes.
    pub min_size: Option<u64>,
    /// At most this many bytes.
    pub max_size: Option<u64>,
    /// Changed on or after this day.
    pub modified_from: Option<String>,
    /// Changed before this day.
    pub modified_before: Option<String>,
}

impl ListFilter {
    /// Does it filter at all?
    pub fn is_active(&self) -> bool {
        *self != Self::default()
    }
}

/// What a listing asks for.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListRequest {
    /// Where to look.
    pub scope: Scope,
    /// Part of the name; empty = any.
    pub term: String,
    /// By size and date.
    pub filter: ListFilter,
    /// The order.
    pub sort: SortOrder,
}

/// Where the next page of a listing starts. Opaque: only a source looks inside.
#[derive(Clone)]
pub struct PageMark(pub Arc<dyn Any + Send + Sync>);

impl std::fmt::Debug for PageMark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PageMark")
    }
}

/// One page of a listing.
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct EntryPage {
    /// What could be read.
    pub entries: Vec<FileEntry>,
    /// What could not be read, named as well as the source can. The rest of the page is kept.
    pub unreadable: Vec<String>,
    /// Where the next page starts; `None` on the last.
    pub next: Option<PageMark>,
}

impl EntryPage {
    /// A page with these entries.
    pub fn new(entries: Vec<FileEntry>, next: Option<PageMark>) -> Self {
        Self {
            entries,
            unreadable: Vec::new(),
            next,
        }
    }
}

/// The rule for a name that is taken. There is no third one: nothing ever replaces a file.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NameRule {
    /// Fail and change nothing.
    #[default]
    Refuse,
    /// Keep both: the incoming thing gets a free name.
    KeepBoth,
}
