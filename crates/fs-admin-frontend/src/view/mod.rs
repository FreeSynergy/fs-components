//! The views of the generated Admin frontend.
//!
//! Pattern: none — declares the modules and re-exports.

mod chrome;
mod detail;
mod form;
mod inputs;
mod kinds;
mod list;
mod screens;
mod surface;
mod value;

pub use kinds::{WITHOUT_COMPONENT, kind_of};
pub use surface::{AboutInfo, AdminSurface};
