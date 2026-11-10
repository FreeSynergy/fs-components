//! `fs-components-file-list`: the file list component (ADR 0066: everything that shows something is a component).
//!
//! A program gives [`FileEntry`] values and the words; the component sorts them, writes the size for people and draws
//! the rows with the `list-view` widget. It knows no server and no contract.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod entry;
mod list;
mod size;
mod sort;

pub use entry::FileEntry;
pub use list::{FileList, FileListWords};
pub use size::human_size;
pub use sort::{SortBy, sorted};
