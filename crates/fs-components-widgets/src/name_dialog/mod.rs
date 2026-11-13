//! `name-dialog`: a small dialog that asks for one name (a page, a sub-page, a new name for a page).
//!
//! Pattern: Adapter over the search field of the primitives (its input already edits: Backspace, Delete, arrows,
//! Home, End, selection, paste, undo) + Command (`confirm`, `cancel`). The dialog knows no data: it hands the trimmed
//! name to the caller, who decides what it names. The words come from the caller as [`Labels`], made for a language with `Localized::for_locale` from the
//! `MessageFormat` 2 files in `texts/` (ADR 0057).

mod dialog;
mod labels;
mod styles;

pub use dialog::{NameDialog, Purpose};
pub use labels::{Labels, TEXTS};
pub use styles::STYLES;
