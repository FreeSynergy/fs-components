//! `page-title`: the big input field above the content of a page, as the title of a page in Notion.
//!
//! Pattern: Adapter over the search field of the primitives (size `Title`: big text, no frame) + Command (`on_enter`).
//! The field knows no page: the caller gives the title and takes it back. Enter does not add a line, it hands the
//! keyboard on (the caller jumps to the first block). The words come from the caller as [`TitleLabels`], made for a
//! language with `Localized::for_locale` from the `MessageFormat` 2 files in `texts/` (ADR 0057).

mod field;
mod labels;

pub use field::PageTitle;
pub use labels::{TEXTS, TitleLabels};
