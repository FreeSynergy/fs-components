//! Putting the reader on screen.
//!
//! Pattern: Facade — [`launch`] opens a window, [`snapshot`] draws a picture without one; both
//! hide Blinc completely. Both draw the same view from the same [`Reader`] state.

mod error;
mod setup;
mod snapshot;
mod windowed;

pub use error::ShellError;
pub use snapshot::{Step, snapshot};
pub use windowed::{Options, launch};
