//! The words of the reader and of the wiki layouts in a chosen language.
//!
//! Pattern: Factory method (`Localized::for_locale`) over embedded `MessageFormat` 2 files (ADR 0057), like the title field and
//! the block menu; `Default` is English. A program may still fill the fields with its own words (the wiki does, from its own
//! files); the point of these files is that a frontend shown in German is German everywhere, also without a program's words.

use fs_components_mf2::{Mf2Texts, Source, check_pairs};
use fs_components_widgets::{SlashLabels, TitleLabels};
use fs_components_words::{Args, Locale, Localized, TextId, Words};

use crate::{Labels, WikiLabels};

/// The two language files; `check_pairs(TEXTS)` must be empty (a test asserts it).
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

/// The problems of the two files (empty is sound).
pub fn check() -> Vec<String> {
    check_pairs(TEXTS)
}

/// A sentence with its one hole as the two characters `{}`.
fn holey(texts: &Mf2Texts, id: &str) -> String {
    texts
        .say(&TextId::new(id), &Args::new().text("hole", "{}"))
        .text()
        .to_owned()
}

impl Localized for Labels {
    /// The words of the reader in `locale`.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        Self {
            search: t.plain("rd-search"),
            back: t.plain("rd-back"),
            edit: t.plain("rd-edit"),
            save: t.plain("rd-save"),
            cancel: t.plain("rd-cancel"),
            export: t.plain("rd-export"),
            nothing_open: t.plain("rd-nothing-open"),
            nothing_found: holey(&t, "rd-nothing-found"),
            unreadable: holey(&t, "rd-unreadable"),
            missing_target: holey(&t, "rd-missing-target"),
            unsaved: t.plain("rd-unsaved"),
            discard_confirm: t.plain("rd-discard-confirm"),
            saved: t.plain("rd-saved"),
            no_changes: t.plain("rd-no-changes"),
            not_saved: holey(&t, "rd-not-saved"),
            tasks_locked: t.plain("rd-tasks-locked"),
            cannot_edit: holey(&t, "rd-cannot-edit"),
            not_exported: holey(&t, "rd-not-exported"),
            overwrite_confirm: holey(&t, "rd-overwrite-confirm"),
            lossy: holey(&t, "rd-lossy"),
            conflict_confirm: holey(&t, "rd-conflict-confirm"),
            backup_found: t.plain("rd-backup-found"),
            restore: t.plain("rd-restore"),
            discard: t.plain("rd-discard"),
            backup_stale: t.plain("rd-backup-stale"),
            restore_confirm: t.plain("rd-restore-confirm"),
            backup_mismatch: holey(&t, "rd-backup-mismatch"),
            backup_failed: holey(&t, "rd-backup-failed"),
            glyphs_missing: holey(&t, "rd-glyphs-missing"),
            glyphs_titles: holey(&t, "rd-glyphs-titles"),
            glyphs_more: holey(&t, "rd-glyphs-more"),
            list_failed: holey(&t, "rd-list-failed"),
            search_failed: holey(&t, "rd-search-failed"),
        }
    }
}

impl Default for Labels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}

impl Localized for WikiLabels {
    /// The words of the wiki layouts in `locale`, with the title field and the block menu.
    fn for_locale(locale: &Locale) -> Self {
        let t = Mf2Texts::new(TEXTS, locale);
        let count = |n: u64| {
            t.say(
                &TextId::new("wk-page-count"),
                &Args::new().count("n", n).text("hole", "{}"),
            )
            .text()
            .to_owned()
        };
        Self {
            program: t.plain("wk-program"),
            pages: t.plain("wk-pages"),
            filter: t.plain("wk-filter"),
            outline: t.plain("wk-outline"),
            no_headings: t.plain("wk-no-headings"),
            history: t.plain("wk-history"),
            no_history: t.plain("wk-no-history"),
            state_current: t.plain("wk-state-current"),
            state_outdated: t.plain("wk-state-outdated"),
            state_fallback: t.plain("wk-state-fallback"),
            language: t.plain("wk-language"),
            edit: t.plain("wk-edit"),
            share: t.plain("wk-share"),
            block_hint: t.plain("wk-block-hint"),
            title_hint: TitleLabels::for_locale(locale).placeholder,
            page_count: count(2),
            page_count_one: count(1),
            slash: SlashLabels::for_locale(locale),
        }
    }
}

impl Default for WikiLabels {
    fn default() -> Self {
        Self::for_locale(&Locale::english())
    }
}
