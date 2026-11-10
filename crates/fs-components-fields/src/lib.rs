//! `fs-components-fields`: the components of the basic types (ADR 0066, item 4): input, display,
//! compact and cell, for e-mail address, name, phone number, postal address and date, in any
//! style of `fs-components-styles`.
//!
//! The rules and the structure of a type are pure ([`BasicField`], `kinds/`); the views in
//! `view/` are written once and read a [`Look`] (a style and light or dark). A program never
//! draws a field: it names a [`Kind`], a [`Mode`] and a style.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod field;
pub mod kinds;
pub mod mode;
pub mod problem;
pub mod suggest;
pub mod view;

pub use field::{BasicField, Segment};
pub use kinds::Kind;
pub use mode::Mode;
pub use problem::Problem;
pub use suggest::{Suggest, SuggestFromList};
pub use view::{FieldInput, KeysAsText, Look, Say, cell, compact, display};
