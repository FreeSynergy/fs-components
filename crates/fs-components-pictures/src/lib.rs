//! `fs-components-pictures`: the picture comparison of the frontend rebuild (`contract/umbau`, step `same-pictures`).
//!
//! A [`Group`] draws blocks offscreen in the Craft style with its bundled font, on the software Vulkan driver
//! ([`gpu`]), and either writes them as reference PNGs (`UPDATE_UMBAU_PICTURES=1`) or compares them pixel by pixel
//! with the stored ones ([`compare`]). Test support only; nothing here is published.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// Test support: failing loudly (panic) is its job.
#![allow(
    clippy::must_use_candidate, clippy::module_name_repetitions, clippy::missing_panics_doc,
    clippy::many_single_char_names // pixel coordinates and channels
)]

pub mod compare;
pub mod gpu;
mod group;

pub use compare::{Report, Tolerance, compare, diff_image};
pub use group::{Group, Verdict};
