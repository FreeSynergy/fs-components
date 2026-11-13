//! `fs-components-words`: text a person reads is translatable (ADR 0057, coding rules §2a).
//!
//! A program asks a [`Words`] for a sentence: a [`TextId`] plus named values ([`Args`], each with
//! its kind). The template of the language decides word order, plural form and formatting. Std
//! only, no UI framework, so apps and components can both use it.
//!
//! Pattern: Strategy (`Words`, `PluralRules`) + Decorator (`Fallback`) + Newtype (`TextId`) —
//! see `DESIGN.md` for why. This file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

mod args;
mod catalog;
mod english;
mod fallback;
mod format;
mod locale;
mod localized;
mod plural_rules;
mod resolve;
mod said;
mod setting;
mod template;
mod text_id;
mod words;

pub use args::{Args, Value};
pub use catalog::{Catalog, CatalogError};
pub use english::English;
pub use fallback::Fallback;
pub use locale::{Locale, LocaleError};
pub use localized::Localized;
pub use plural_rules::{EnglishPlural, PluralCategory, PluralRules, PolishPlural};
pub use resolve::{Chosen, Env, LanguageSource, ProcessEnv, Resolved, load_and_resolve, resolve};
pub use said::Said;
pub use setting::LanguageSetting;
pub use text_id::TextId;
pub use words::Words;
