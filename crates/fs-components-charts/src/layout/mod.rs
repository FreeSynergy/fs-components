//! Layout: validated spec and clean data to a [`Scene`](crate::scene::Scene), without a GUI.
//!
//! Pattern: Strategy per mark (`bar`, `line`, `point`, `arc`) behind one Builder (`build`); the
//! axes, legend and notes are shared. Every number in the scene can be tested exactly.

mod arc;
mod axis;
mod axis_items;
mod bar;
mod build;
mod cartesian;
mod cats;
mod ctx;
pub mod dash;
pub mod describe;
mod legend;
mod line;
mod measure;
mod notes;
mod point;
mod sanitize;
mod stack;

pub use axis::{Axis, Scale, Source, Tick, Use};
pub use build::{MAX_SERIES, build, series};
pub use cats::{Cats, categories, key};
pub use measure::{Fixed, Measure, Metrics};
pub use notes::report_lines;
pub use stack::{Entry, stack};
