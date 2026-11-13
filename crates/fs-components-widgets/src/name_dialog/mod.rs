//! `name-dialog`: a small dialog that asks for one name (a page, a sub-page, a new name for a page).
//!
//! Pattern: Adapter over the search field of the primitives (its input already edits: Backspace, Delete, arrows,
//! Home, End, selection, paste, undo) + Command (`confirm`, `cancel`). The dialog knows no data: it hands the trimmed
//! name to the caller, who decides what it names. The words come from the caller as [`Labels`], made from any
//! [`Words`](fs_components_words::Words) with [`Labels::from_words`]; [`texts`] holds the English and German tables.

mod dialog;
mod labels;
mod styles;
pub mod texts;

pub use dialog::{NameDialog, Purpose};
pub use labels::Labels;
pub use styles::STYLES;
