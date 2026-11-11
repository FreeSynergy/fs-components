//! `fs-components-primitives`: primitives (layer 1) of FreeSynergy.
//!
//! A primitive cannot be broken down further: one value, one gesture, one state. Here:
//! [`SearchField`], [`email_input`] and [`phone_input`]. Blinc stays behind this API.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod contact_input;
pub mod field_data;
pub mod search_field;

pub use contact_input::{
    OnChange as ContactOnChange, email_input, email_problem, phone_input, phone_problem,
};
pub use field_data::{
    FieldData, build_scoped_text_area, collect as collect_field_data, dispose_effect_later,
    scroll_ref, scroll_watch,
};
pub use search_field::{FieldSize, SearchField};
