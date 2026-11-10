//! `fs-components-kanban`: a board of columns with cards.
//!
//! Plain data in (`Column`, `Card`), a trait for the data (`BoardSource`, which also says which
//! operations the host supports) and one for what the reader asks (`BoardEvents`), a pure model
//! (moves by grab/hover/drop, keys, move buttons) and a Blinc view in the look of
//! `fs-components-styles`. Unsupported operations are drawn locked with a "comes later" hint.
//! Texts are translation keys answered by `Say`. Entry point: [`KanbanBoard::build`].
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]
// Pixel arithmetic over a few dozen cards (counts far below 2^23) and short names for the style and
// palette in the view functions: the casts cannot lose anything that matters here.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names,
    clippy::similar_names
)]

pub mod handle;
mod keys;
pub mod layout;
pub mod model;
pub mod source;
pub mod state;
pub mod view;

pub use handle::KanbanBoard;
pub use model::{Card, Column, Due, DueState, Label, Operation, Tone};
pub use source::{Answer, BoardEvents, BoardSource, FixedBoard, NoEvents};
pub use state::{BoardState, Held, MoveRequest, Step};
