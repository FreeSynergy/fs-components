//! A frontend shown in German is German everywhere: the words of the reader and of the wiki layouts come from `MessageFormat` 2
//! files in both languages (finding E3 of 2026-11-14: a German block menu beside an English side bar and an English "Save").
//!
//! Tests: model tests over the words (every field differs between the languages, the files are paired) and picture tests of the
//! three layouts with the German words (skipped without graphics or fonts); the counter-test is the English picture.

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, fonts_work, pixels};
use fs_components_words::{Locale, Localized};
use fs_components_workbenches::words;
use fs_components_workbenches::{Labels, Layout, Picture, Step, WikiLabels, snapshot};

fn german() -> Locale {
    Locale::parse("de").unwrap()
}

/// Names and short words that are the same in both languages on purpose.
const SAME: &[&str] = &["FreeSynergy Wiki", "Code"];

#[test]
fn the_files_are_paired_and_every_visible_word_differs_between_the_languages() {
    assert_eq!(words::check(), Vec::<String>::new());
    let (en, de) = (
        Labels::for_locale(&Locale::english()),
        Labels::for_locale(&german()),
    );
    let (wen, wde) = (
        WikiLabels::for_locale(&Locale::english()),
        WikiLabels::for_locale(&german()),
    );
    // Every field of both structures, English against German.
    let pairs: Vec<(&str, &String, &String)> = vec![
        ("search", &en.search, &de.search),
        ("back", &en.back, &de.back),
        ("edit", &en.edit, &de.edit),
        ("save", &en.save, &de.save),
        ("cancel", &en.cancel, &de.cancel),
        ("export", &en.export, &de.export),
        ("nothing_open", &en.nothing_open, &de.nothing_open),
        ("nothing_found", &en.nothing_found, &de.nothing_found),
        ("unreadable", &en.unreadable, &de.unreadable),
        ("missing_target", &en.missing_target, &de.missing_target),
        ("unsaved", &en.unsaved, &de.unsaved),
        ("discard_confirm", &en.discard_confirm, &de.discard_confirm),
        ("saved", &en.saved, &de.saved),
        ("no_changes", &en.no_changes, &de.no_changes),
        ("not_saved", &en.not_saved, &de.not_saved),
        ("tasks_locked", &en.tasks_locked, &de.tasks_locked),
        ("cannot_edit", &en.cannot_edit, &de.cannot_edit),
        ("not_exported", &en.not_exported, &de.not_exported),
        (
            "overwrite_confirm",
            &en.overwrite_confirm,
            &de.overwrite_confirm,
        ),
        ("lossy", &en.lossy, &de.lossy),
        (
            "conflict_confirm",
            &en.conflict_confirm,
            &de.conflict_confirm,
        ),
        ("backup_found", &en.backup_found, &de.backup_found),
        ("restore", &en.restore, &de.restore),
        ("discard", &en.discard, &de.discard),
        ("backup_stale", &en.backup_stale, &de.backup_stale),
        ("restore_confirm", &en.restore_confirm, &de.restore_confirm),
        ("backup_mismatch", &en.backup_mismatch, &de.backup_mismatch),
        ("backup_failed", &en.backup_failed, &de.backup_failed),
        ("glyphs_missing", &en.glyphs_missing, &de.glyphs_missing),
        ("glyphs_titles", &en.glyphs_titles, &de.glyphs_titles),
        ("glyphs_more", &en.glyphs_more, &de.glyphs_more),
        ("list_failed", &en.list_failed, &de.list_failed),
        ("search_failed", &en.search_failed, &de.search_failed),
        ("program", &wen.program, &wde.program),
        ("pages", &wen.pages, &wde.pages),
        ("filter", &wen.filter, &wde.filter),
        ("outline", &wen.outline, &wde.outline),
        ("no_headings", &wen.no_headings, &wde.no_headings),
        ("history", &wen.history, &wde.history),
        ("no_history", &wen.no_history, &wde.no_history),
        ("state_current", &wen.state_current, &wde.state_current),
        ("state_outdated", &wen.state_outdated, &wde.state_outdated),
        ("state_fallback", &wen.state_fallback, &wde.state_fallback),
        ("language", &wen.language, &wde.language),
        ("share", &wen.share, &wde.share),
        ("wiki edit", &wen.edit, &wde.edit),
        ("block_hint", &wen.block_hint, &wde.block_hint),
        ("title_hint", &wen.title_hint, &wde.title_hint),
        ("page_count", &wen.page_count, &wde.page_count),
        ("page_count_one", &wen.page_count_one, &wde.page_count_one),
        ("slash code", &wen.slash.code, &wde.slash.code),
    ];
    for (name, english, german) in pairs {
        assert_ne!(*english, "", "{name} is empty in English");
        assert_ne!(*german, "", "{name} is empty in German");
        if !SAME.contains(&english.as_str()) {
            assert_ne!(
                english, german,
                "{name} is the same in both languages: {english:?}"
            );
        }
    }
    assert_eq!(
        (wde.pages_said(1), wde.pages_said(2)),
        ("1 Seite".into(), "2 Seiten".into())
    );
}

#[test]
fn the_defaults_are_the_english_words() {
    assert_eq!(Labels::default(), Labels::for_locale(&Locale::english()));
    assert_eq!(
        WikiLabels::default(),
        WikiLabels::for_locale(&Locale::english())
    );
    assert_eq!(Labels::default().save, "Save");
    assert_eq!(WikiLabels::default().pages, "Pages");
}

#[test]
fn every_layout_drawn_with_the_german_words_differs_from_the_english_picture() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    for (layout, name) in [
        (Layout::Notion, "notion"),
        (Layout::Confluence, "confluence"),
        (Layout::Classic, "classic"),
    ] {
        let mut drawn = Vec::new();
        for (locale, tag) in [(Locale::english(), "en"), (german(), "de")] {
            let source = Editable::with(&[("h.md", "# H\n\nText\n")]);
            source
                .heading_titles
                .store(true, std::sync::atomic::Ordering::Relaxed);
            let mut picture = Picture::new(Arc::new(source), 1000, 520);
            picture.layout = layout;
            picture.labels = Labels::for_locale(&locale);
            picture.wiki_labels = WikiLabels::for_locale(&locale);
            picture.steps = vec![Step::Open("h.md".into())];
            let file = tmp.file(&format!("{name}-{tag}.png"));
            if snapshot(picture, &file).is_err() {
                eprintln!("skipped: no graphics");
                return;
            }
            if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::copy(
                    &file,
                    std::path::Path::new(&dir).join(format!("sprache-{name}-{tag}.png")),
                )
                .unwrap();
            }
            drawn.push(pixels(&file));
        }
        assert_ne!(
            drawn[0].as_raw(),
            drawn[1].as_raw(),
            "{name}: the German words are drawn"
        );
    }
}
