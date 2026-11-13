//! The words of the standard shell: its own menus in German and English.
//!
//! Pattern: Factory over embedded `MessageFormat` 2 files (`texts/en.mf2`, `texts/de.mf2`, ADR 0057); the shell has words of its
//! own because ADR 0065 fixes its menus (Program, View, Help, Settings, Quit, About), so no program types them again and a
//! menu bar can never mix two languages. A language without a file comes in English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::Locale;

/// The two language files of the shell; `fs_components_mf2::check_pairs(TEXTS)` must be empty (a test asserts it).
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

/// The shell's words in `locale`.
pub(crate) fn shell_texts(locale: &Locale) -> Mf2Texts {
    Mf2Texts::new(TEXTS, locale)
}

/// The name of a language in that language (`Deutsch`); the tag itself for a language the shell has no name for.
pub(crate) fn language_name(texts: &Mf2Texts, language: &Locale) -> String {
    let id = format!("language-name-{}", language.as_str().to_ascii_lowercase());
    let said = fs_components_words::Words::say(texts, &id.as_str().into(), &fs_components_words::Args::new());
    if said.is_found() {
        said.text().to_owned()
    } else {
        language.to_string()
    }
}
