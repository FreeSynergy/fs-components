//! What a source can do, and what it says after a larger act.
//!
//! Pattern: Value objects — capabilities decide what is offered; the reports are plain data to show.

/// What a source can do; what it cannot is not offered.
#[allow(clippy::struct_excessive_bools)]
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileCapabilities {
    /// Create, rename and move folders and files, and copy files.
    pub change: bool,
    /// The keep-both rule for a taken name.
    pub keep_both: bool,
    /// Put in the trash, restore, purge, empty.
    pub trash: bool,
    /// Restore to another place than the original.
    pub restore_to: bool,
    /// Take bytes in from a path, and list unfinished uploads.
    pub upload: bool,
    /// Write bytes out to a path.
    pub download: bool,
    /// Bring in changes made from outside.
    pub rescan: bool,
    /// Sort by something else than the name.
    pub sort: bool,
    /// Sorting by type can also go backwards.
    pub type_descending: bool,
    /// Search by name, and filter by size and date.
    pub search: bool,
    /// Entries say when they last changed.
    pub dates: bool,
}

impl FileCapabilities {
    /// Everything.
    pub fn all() -> Self {
        Self {
            change: true,
            keep_both: true,
            trash: true,
            restore_to: true,
            upload: true,
            download: true,
            rescan: true,
            sort: true,
            type_descending: true,
            search: true,
            dates: true,
        }
    }

    /// Looking only.
    #[must_use]
    pub fn read_only(self) -> Self {
        Self {
            search: self.search,
            sort: self.sort,
            type_descending: self.type_descending,
            dates: self.dates,
            ..Self::default()
        }
    }
}

/// How much is in the trash.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TrashCount {
    /// Files.
    pub files: usize,
    /// Of these, marked sensitive.
    pub marked: usize,
}

impl TrashCount {
    /// So many files, of which so many marked.
    pub fn new(files: usize, marked: usize) -> Self {
        Self { files, marked }
    }
}

/// What emptying the trash did.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EmptyReport {
    /// Files gone for good.
    pub purged: usize,
    /// Names of what could not be purged and stays in the trash.
    pub failed: Vec<String>,
}

impl EmptyReport {
    /// So many gone, these not.
    pub fn new(purged: usize, failed: Vec<String>) -> Self {
        Self { purged, failed }
    }
}

/// What a rescan found.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RescanSummary {
    /// Names taken over.
    pub added: Vec<String>,
    /// Names whose content changed.
    pub updated: Vec<String>,
    /// Names without bytes; their records are kept.
    pub missing: Vec<String>,
    /// What was not taken over, and why.
    pub skipped: Vec<String>,
}

/// An unfinished upload.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingUpload {
    /// The id to go on with.
    pub id: String,
    /// The name it will have.
    pub name: String,
    /// Bytes held so far.
    pub received: u64,
}

impl PendingUpload {
    /// An unfinished upload.
    pub fn new(id: impl Into<String>, name: impl Into<String>, received: u64) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            received,
        }
    }
}

/// The start of a file, read for a preview.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewBytes {
    /// As many bytes as were asked for, or fewer.
    pub bytes: Vec<u8>,
    /// How big the file is.
    pub total: u64,
}

impl PreviewBytes {
    /// These bytes of so many.
    pub fn new(bytes: Vec<u8>, total: u64) -> Self {
        Self { bytes, total }
    }
}
