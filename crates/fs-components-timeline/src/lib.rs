//! `fs-components-timeline`: a horizontal time axis with bars, progress, milestones, dependency
//! arrows, zoom levels (day, week, month) and a line for today.
//!
//! Plain data in (`Item`, `Dependency`, days already localised), a trait for the data
//! (`TimelineSource`) and one for what the reader chose (`TimelineEvents`), a pure model and
//! layout maths, and a Blinc view in the look of `fs-components-styles`; the arrows are drawn on a
//! Blinc canvas. Texts are translation keys answered by `Say`. Entry point: [`Timeline::build`].
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]
// Pixel arithmetic over rows and days (counts far below 2^23) and short names for the style and
// palette in the view functions: the casts cannot lose anything that matters here.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names
)]

pub mod handle;
pub mod layout;
pub mod model;
pub mod route;
mod scroll;
pub mod source;
pub mod state;
pub mod view;

pub use handle::Timeline;
pub use model::{Day, Dependency, Item, Kind, Tone, Zoom};
pub use source::{NoEvents, TimelineEvents, TimelineSource, VecSource};
pub use state::{Step, TimelineState};
