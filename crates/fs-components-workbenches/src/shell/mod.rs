//! Putting the reader on screen.
//!
//! Pattern: none (declares modules and re-exports only) — [`ReaderWidget`] is the reader to place in a program's own window,
//! [`launch`] opens a bare app (the reader over the whole screen, ADR 0056), [`snapshot`] draws a picture without one.

mod error;
mod picture;
mod setup;
mod snapshot;
mod widget;
mod windowed;

pub use error::ShellError;
pub use picture::{Picture, ScrollInfo, Snapshot, Step};
pub use setup::{register_fonts, styles};
pub use snapshot::snapshot;
pub use widget::ReaderWidget;
pub use windowed::{Options, launch};
