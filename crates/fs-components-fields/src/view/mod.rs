//! The views: one function per mode, written once for every basic type.
//!
//! Pattern: none — declares the modules and re-exports.

mod cell;
mod compact;
mod display;
mod input;
mod look;
mod say;
mod suggestions;

pub use cell::cell;
pub use compact::compact;
pub use display::display;
pub use input::FieldInput;
pub use look::Look;
pub use say::{KeysAsText, Say};
