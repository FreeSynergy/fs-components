//! The words of the name dialog. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

/// The texts of the dialog. The titles that talk about something (the parent page, the old name) are made by
/// [`Labels::title_about`] with the real MF2 argument.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Title for a new page.
    pub new_page: String,
    /// Shown while the field is empty.
    pub placeholder: String,
    /// Button: take the name.
    pub ok: String,
    /// Button: leave.
    pub cancel: String,
    /// The language the words were made for; the titles about something are formatted in it.
    pub locale: Locale,
}

/// The two language files of the dialog; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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

impl Localized for Labels {
    /// The dialog's words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            new_page: t.plain("name-dialog-new-page"),
            placeholder: t.plain("name-dialog-placeholder"),
            ok: t.plain("name-dialog-ok"),
            cancel: t.plain("name-dialog-cancel"),
            locale: locale.clone(),
        }
    }
}

impl Labels {
    /// The title `id` (`name-dialog-sub-page`, `name-dialog-rename`) with `about` as the MF2 argument, so formatting and
    /// bidi isolation of the message apply.
    pub(super) fn title_about(&self, id: &str, about: &str) -> String {
        Mf2Texts::new(TEXTS, &self.locale)
            .say(&TextId::new(id), &Args::new().text("about", about))
            .text()
            .to_owned()
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
