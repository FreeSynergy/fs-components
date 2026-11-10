//! One line of the list.
//!
//! Pattern: Value object — plain data the caller fills; the component has no idea what a file is beyond these fields.

/// A file or a folder as the list shows it.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileEntry {
    /// Handed back when the row is chosen.
    pub key: String,
    /// The name.
    pub name: String,
    /// A folder is listed before the files when the list is sorted by name, and carries the folder mark.
    pub is_folder: bool,
    /// The media type (`image/png`); empty for a folder or when not known.
    pub media_type: String,
    /// Bytes; a folder has none.
    pub size: u64,
    /// The time of the last change, as the caller wants it read (an RFC 3339 text sorts by time).
    pub modified: String,
}

impl FileEntry {
    /// A file.
    pub fn file(key: impl Into<String>, name: impl Into<String>, size: u64) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            size,
            ..Self::default()
        }
    }

    /// A folder.
    pub fn folder(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            is_folder: true,
            ..Self::default()
        }
    }

    /// With this media type.
    #[must_use]
    pub fn with_media_type(mut self, media_type: impl Into<String>) -> Self {
        self.media_type = media_type.into();
        self
    }

    /// With this time of the last change.
    #[must_use]
    pub fn with_modified(mut self, modified: impl Into<String>) -> Self {
        self.modified = modified.into();
        self
    }
}
