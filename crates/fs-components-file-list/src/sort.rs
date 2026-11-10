//! The order of the rows.
//!
//! Pattern: Strategy — [`SortBy`] picks the comparison; ties are broken by name, then key, so the order is the same on
//! every call (the contract of the files program breaks ties the same way).

use std::cmp::Ordering;

use crate::FileEntry;

/// What the list is ordered by.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortBy {
    /// By name, case-insensitive, folders first.
    Name,
    /// By the time of the last change.
    Modified,
    /// By size; a folder counts as 0.
    Size,
    /// By media type; a folder counts as `inode/directory`.
    Kind,
}

fn kind(e: &FileEntry) -> &str {
    if e.is_folder {
        "inode/directory"
    } else {
        &e.media_type
    }
}

fn by(sort: SortBy, a: &FileEntry, b: &FileEntry) -> Ordering {
    match sort {
        SortBy::Name => b.is_folder.cmp(&a.is_folder),
        SortBy::Modified => a.modified.cmp(&b.modified),
        SortBy::Size => a.size.cmp(&b.size),
        SortBy::Kind => kind(a).cmp(kind(b)),
    }
}

/// The entries in the order asked for.
pub fn sorted(mut entries: Vec<FileEntry>, sort: SortBy, descending: bool) -> Vec<FileEntry> {
    entries.sort_by(|a, b| {
        let first = by(sort, a, b);
        let first = if descending && sort != SortBy::Name {
            first.reverse()
        } else {
            first
        };
        first
            .then_with(|| {
                let (x, y) = (a.name.to_lowercase(), b.name.to_lowercase());
                if descending && sort == SortBy::Name {
                    y.cmp(&x)
                } else {
                    x.cmp(&y)
                }
            })
            .then_with(|| a.key.cmp(&b.key))
    });
    entries
}
