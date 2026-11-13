//! The words of the page title field.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

/// The texts of the field.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleLabels {
    /// Shown while the title is empty.
    pub placeholder: String,
}

/// The two language files; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
pub const TEXTS: &[Source] = &[
    Source {
        language: "en",
        text: include_str!("texts/en.mf2"),
    },
    Source {
        language: "de",
        text: include_str!("texts/de.mf2"),
    },
];

impl Localized for TitleLabels {
    /// The words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            placeholder: t.plain("page-title-placeholder"),
        }
    }
}

impl Default for TitleLabels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
