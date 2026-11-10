//! The editor as a Blinc component.
//!
//! Pattern: none — declares modules and re-exports the public API.

mod area;
mod build;
mod handle;
mod toolbar;
mod trait_impl;

pub use handle::{Change, EditorView, OnChange, Previewer};
