//! `Localized`: what a `*Labels` struct of a component offers so that the program can ask for its words in a language.
//!
//! Pattern: Factory method — `for_locale` builds the struct in a language; `Default` stays the English one. The component
//! knows no program and no language list: the program passes the `Locale` it resolved (`resolve`).

use crate::Locale;

/// A struct of words that can be made in a language.
pub trait Localized: Sized {
    /// The words in `locale`; a language without texts comes in English.
    fn for_locale(locale: &Locale) -> Self;
}
