//! The view: the scene drawn on a Blinc canvas with the theme's colours.
//!
//! Pattern: Adapter — Blinc stays behind this module; the only public type is [`ChartView`],
//! which returns an `impl Element`.

mod draw;
mod look;
mod measure;
mod rotated;
mod shapes;
mod widget;

pub use look::Look;
pub use measure::BlincMeasure;
pub use widget::{ALT_ROWS, ChartView, notice};
