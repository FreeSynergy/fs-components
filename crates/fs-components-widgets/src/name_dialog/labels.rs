//! The words of the name dialog. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

/// The texts of the dialog; `{}` in a title is replaced by what the dialog is about (the parent page, the old name).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Title for a new page.
    pub new_page: String,
    /// Title for a sub-page; `{}` = the parent.
    pub sub_page: String,
    /// Title for a new name; `{}` = the old name.
    pub rename: String,
    /// Shown while the field is empty.
    pub placeholder: String,
    /// Button: take the name.
    pub ok: String,
    /// Button: leave.
    pub cancel: String,
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
    /// The dialog's words in `locale`; a title keeps `{}` where the thing it talks about goes, filled by the dialog.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        let hole = |id: &str| {
            t.say(&TextId::new(id), &Args::new().text("about", "{}"))
                .text()
                .to_owned()
        };
        Self {
            new_page: t.plain("name-dialog-new-page"),
            sub_page: hole("name-dialog-sub-page"),
            rename: hole("name-dialog-rename"),
            placeholder: t.plain("name-dialog-placeholder"),
            ok: t.plain("name-dialog-ok"),
            cancel: t.plain("name-dialog-cancel"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
