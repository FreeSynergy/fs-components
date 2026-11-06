//! `text-view`: draws a document's blocks and marks.
//!
//! Pattern: Adapter — [`TextView`] turns the neutral model of [`crate::text`] into Blinc elements
//! (Blinc stays behind this API); a Strategy per block kind lives in `blocks.rs`.
//!
//! Colours, fonts and sizes come from the theme (`look.rs` and the CSS variables in `styles.rs`).
//! Links are only *reported*: the view calls the [`LinkHandler`] with the href as written and
//! knows nothing about what it points to. Gestures are not part of a component (ADR 0037).

mod blocks;
mod code;
mod emphasis;
mod flow;
mod inline;
mod list;
mod look;
mod styles;
mod table;

mod view;

pub use styles::STYLES;
pub use view::{LinkHandler, TextView, open_external};
