//! The pure model of the editor: text, selection, commands, history. No Blinc.
//!
//! Pattern: none — declares modules and re-exports the public API.

mod command;
mod counts;
mod edit;
mod lines;
mod selection;
mod state;
mod wrap;

pub use command::EditCommand;
pub use counts::Counts;
pub use edit::Edit;
pub use selection::Selection;
pub use state::{EditorModel, MAX_HISTORY};
