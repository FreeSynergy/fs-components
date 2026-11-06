//! Putting the reader on screen.
//!
//! Pattern: Facade — [`ReaderWidget`] is the reader to place in a program's own window,
//! [`launch`] opens a window with nothing else, [`snapshot`] draws a picture without one.

mod error;
mod picture;
mod setup;
mod snapshot;
mod widget;
mod windowed;

pub use error::ShellError;
pub use picture::{Picture, ScrollInfo, Snapshot, Step};
pub use snapshot::snapshot;
pub use widget::ReaderWidget;
pub use windowed::{Options, launch};
