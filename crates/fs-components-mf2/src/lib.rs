//! `fs-components-mf2`: the texts of FreeSynergy as Unicode `MessageFormat` 2 (ADR 0057), behind the `Words` trait.
//!
//! One text file per language next to the code that shows the text (`texts/en.mf2`, `texts/de.mf2`), embedded with
//! `include_str!`. [`Mf2Texts`] is a [`Words`](fs_components_words::Words) over one locale and its fallback chain (`de-AT`,
//! `de`, English); [`check_pairs`] is the build-time check every crate with texts runs as a test: each id in `en` and `de`,
//! the same variables, every message compiles. The library is `mf2` (pinned to `=3.0.0`; see `DESIGN` for the licence and the
//! reservations of ADR 0057). It is a separate crate so that `fs-components-words` stays std only.
//!
//! Pattern: Adapter (`mf2` behind `Words`) + Chain (the languages) — this file only declares modules and re-exports.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod check;
mod date;
mod source;
mod texts;

pub use check::check_pairs;
pub use source::Source;
pub use texts::Mf2Texts;
