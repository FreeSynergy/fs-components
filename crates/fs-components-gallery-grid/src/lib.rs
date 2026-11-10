//! `fs-components-gallery-grid`: the gallery grid component (ADR 0066: everything that shows something is a component).
//!
//! A program gives [`Picture`] values and the words; the component orders them newest first, names the group of each
//! (day, month or year, the "timeline" of `requirements.md`) and draws the tiles with the `file-tile` widget. It knows
//! no server and no contract. A thumbnail is not drawn yet: a tile shows the short type word where the picture will go
//! (the widget has no image tile; named in `DESIGN.md`).
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod grid;
mod group;
mod picture;

pub use grid::GalleryGrid;
pub use group::{Grouping, Section, group_label, sections};
pub use picture::Picture;
