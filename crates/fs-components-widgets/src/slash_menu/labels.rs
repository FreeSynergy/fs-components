//! The words of the block menu.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057); `Default` is English.

use fs_components_mf2::{Mf2Texts, Source};
use fs_components_words::{Locale, Localized};

/// The names of the block types in the menu, in the order the menu shows them, and the empty-result line.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlashLabels {
    /// Heading 1, 2 and 3.
    pub headings: [String; 3],
    /// Bulleted list.
    pub bullet: String,
    /// Numbered list.
    pub numbered: String,
    /// List of tasks (checkboxes).
    pub task: String,
    /// Table.
    pub table: String,
    /// Quote.
    pub quote: String,
    /// Code block.
    pub code: String,
    /// Divider (horizontal rule).
    pub divider: String,
    /// Shown when the typed filter matches no block type.
    pub none: String,
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

impl Localized for SlashLabels {
    /// The words in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            headings: [
                t.plain("slash-heading-1"),
                t.plain("slash-heading-2"),
                t.plain("slash-heading-3"),
            ],
            bullet: t.plain("slash-bullet"),
            numbered: t.plain("slash-numbered"),
            task: t.plain("slash-task"),
            table: t.plain("slash-table"),
            quote: t.plain("slash-quote"),
            code: t.plain("slash-code"),
            divider: t.plain("slash-divider"),
            none: t.plain("slash-none"),
        }
    }
}

impl Default for SlashLabels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
