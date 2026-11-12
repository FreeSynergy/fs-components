//! `search-field`: a text field that reports what is typed.
//!
//! Pattern: none (declares modules and re-exports only) — declares the module and re-exports the field.

mod field;

pub use field::{FIELD_FONT_CLASS, FieldSize, OnChange, STYLES, SearchField, with_field_font};
