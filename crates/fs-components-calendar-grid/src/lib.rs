//! `fs-components-calendar-grid`: month grid, week grid and day column of a calendar.
//!
//! Plain data in (`GridItem`, already localised, no time zone, no JSCalendar types), a trait for
//! the data (`GridSource`) and one for what the reader chose (`GridEvents`), a pure model and
//! layout maths, and a Blinc view in the look of `fs-components-styles`. Texts are translation
//! keys answered by `Say`. Entry point: [`CalendarGrid::build`].
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]
// Pixel arithmetic over a few dozen cells and lanes (counts far below 2^23) and short names for the
// style, palette and sizes in the view functions: the casts cannot lose anything that matters here.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::similar_names
)]

pub mod axis;
pub mod day;
pub mod handle;
pub mod layout;
pub mod model;
pub mod source;
pub mod spans;
pub mod state;
pub mod view;

pub use day::{Day, MINUTES_PER_DAY, Moment};
pub use handle::CalendarGrid;
pub use model::{GridItem, Mode, Nav, Slot, Tone};
pub use source::{GridEvents, GridSource, NoEvents, VecSource};
pub use state::{GridState, KeyOutcome};
