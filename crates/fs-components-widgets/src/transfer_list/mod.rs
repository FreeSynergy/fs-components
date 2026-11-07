//! `transfer-list`: running uploads and downloads, given as data, with progress, failure and
//! retry reported as outcomes.
//!
//! Pattern: Adapter over `blinc_cn` (progress bar, buttons) plus Observer (the caller's
//! [`Outcome`] callback). The list moves nothing and remembers nothing: the caller hands over the
//! transfers as they are now and answers the outcomes (retry, cancel, dismiss) by changing them.

mod labels;
mod model;
mod styles;
mod view;

pub use labels::Labels;
pub use model::{Action, Direction, Outcome, State, Transfer, TransferList, format_bytes};
pub use styles::STYLES;
