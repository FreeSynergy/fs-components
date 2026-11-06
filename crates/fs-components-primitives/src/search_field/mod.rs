//! `search-field`: a text field that reports what is typed.
//!
//! Pattern: Facade — declares the module and re-exports the field.

mod field;

pub use field::{FieldSize, OnChange, SearchField};
