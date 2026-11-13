//! The words of the composer. They come from the caller; English by default.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files; `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

/// Every word the composer draws.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Placeholder of the text area.
    pub placeholder: String,
    /// Button: send.
    pub send: String,
    /// Button label while a send waits for its answer.
    pub sending: String,
    /// Button: attach.
    pub attach: String,
    /// Button: remove an attachment.
    pub remove: String,
    /// Not sent; `{}` = the reason. The text is still here.
    pub failed: String,
    /// A saved draft was put back.
    pub restored: String,
    /// The draft could not be saved; `{}` = the reason.
    pub draft_failed: String,
}

/// The two language files of the composer; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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
    /// The composer's words in `locale`. `failed` and `draft_failed` keep their `{}` where the reason goes, as before.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        let with_reason = |id: &str| {
            t.say(&TextId::new(id), &Args::new().text("reason", "{}"))
                .text()
                .to_owned()
        };
        Self {
            placeholder: t.plain("composer-placeholder"),
            send: t.plain("composer-send"),
            sending: t.plain("composer-sending"),
            attach: t.plain("composer-attach"),
            remove: t.plain("composer-remove"),
            failed: with_reason("composer-failed"),
            restored: t.plain("composer-restored"),
            draft_failed: with_reason("composer-draft-failed"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
