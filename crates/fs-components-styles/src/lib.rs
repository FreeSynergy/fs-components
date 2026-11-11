//! `fs-components-styles`: the style system of FreeSynergy (ADR 0066, items 5 and 6).
//!
//! A [`Style`] is plain data: spacing, heights, type, radii, colours (light and dark), effects (glass,
//! backdrop, shadow, focus ring) and an icon set. Four are built in: Admin (SAP Fiori measures),
//! Simple, Apple, Windows. No Blinc here: views in `fs-components-fields` and others read a style
//! and turn it into elements. [`Style::demands`] lists what a style asks of the GUI engine, so each
//! request can be measured.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
// A style is one long table of numbers (`styles/*.rs`), not logic: it may exceed 100 lines.
// Colours are written as hex `0xRRGGBB`, the notation of every design sheet; the casts in `Rgba::hex` take one byte each on purpose.
#![allow(
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::unreadable_literal,
    clippy::too_many_lines,
    clippy::cast_possible_truncation
)]

pub mod color;
pub mod effects;
mod glyph;
mod icons;
pub mod style;
pub mod styles;
pub mod tokens;

pub use color::Rgba;
pub use effects::{Effects, FocusRing, Shadow, Surface};
pub use glyph::Glyph;
pub use icons::{Cap, IconSet, Join};
pub use style::{Builtin, Demand, Provenance, Style, StyleCatalog, StyleId, style};
pub use tokens::{Heights, Palette, Radii, Spacing, Type};

#[cfg(test)]
mod tests;
