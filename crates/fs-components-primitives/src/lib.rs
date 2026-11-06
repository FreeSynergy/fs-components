//! `fs-components-primitives`: primitives (layer 1) of FreeSynergy.
//!
//! A primitive cannot be broken down further: one value, one gesture, one state. Here:
//! [`SearchField`]. Blinc stays behind this API.
//!
//! Pattern: Facade — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod search_field;

pub use search_field::{FieldSize, SearchField};
