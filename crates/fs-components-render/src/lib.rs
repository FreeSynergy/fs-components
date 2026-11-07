//! `fs-components-render`: what a window and a picture have in common, and the picture itself.
//!
//! [`bundle`] and [`register_fonts`] set up theme and fonts; [`draw_to_png`] draws an element
//! without a window. It knows no workbench and no app, so both may depend on it (ADR 0043 §4).
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod error;
mod offscreen;
mod setup;

pub use error::RenderError;
pub use offscreen::{Canvas, draw_to_png};
pub use setup::{bundle, register_fonts};
