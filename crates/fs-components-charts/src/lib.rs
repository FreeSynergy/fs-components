//! `fs-components-charts`: charts as a widget, from a validated subset of Vega-Lite.
//!
//! Layers, each testable alone: [`spec`] reads and validates, data and scales and layout are
//! pure and give exact numbers, the view draws on a Blinc canvas.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]
// Chart maths turns counts into pixels and back and compares exact tick values on purpose; the
// values are far below 2^52 and every cast is on such a bounded quantity.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::float_cmp,
    clippy::many_single_char_names,
    clippy::similar_names
)]

pub mod alt;
pub mod chart;
pub mod color;
pub mod data;
pub mod labels;
pub mod layout;
pub mod scale;
pub mod scene;
pub mod spec;
pub mod view;

pub use alt::{Table, TextAlternative};
pub use chart::Chart;
pub use labels::Labels;
pub use view::ChartView;

/// What a widget returns to be placed in a layout: the one place a Blinc type is visible.
pub use blinc_layout::prelude::ElementBuilder as Element;
