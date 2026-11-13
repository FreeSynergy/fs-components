//! Notion frontend, editing a page: the title is a big input field above the blocks, not a raw `# ` block.
//!
//! Tests: pixel tests, skipped where there is no graphics adapter or no font. Pictures are kept when `FS_COMP_SHOTS`
//! names a directory. A title is a run of 20 or more pixel rows holding ink (a 40 px title); text of 14 px stays below.

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, dark, fonts_work, pixels};
use fs_components_widgets::TitleLabels;
use fs_components_words::{Locale, Localized};
use fs_components_workbenches::{Layout, Picture, Step, snapshot};

/// The page "handbook" edited, in `language` ("en" or "de"); `text` is its source. Saved as `name` if asked.
fn shoot(text: &str, language: &str, name: &str) -> Option<image::RgbaImage> {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return None;
    }
    let source = Editable::with(&[("handbook.md", text)]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let mut picture = Picture::new(Arc::new(source), 1000, 520);
    picture.layout = Layout::Notion;
    picture.wiki_labels.title_hint =
        TitleLabels::for_locale(&Locale::parse(language).unwrap()).placeholder;
    picture.steps = vec![Step::Open("handbook.md".into()), Step::Edit];
    let file = tmp.file(name);
    if snapshot(picture, &file).is_err() {
        eprintln!("skipped: no graphics");
        return None;
    }
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(&file, std::path::Path::new(&dir).join(name)).unwrap();
    }
    Some(pixels(&file))
}

/// Runs of consecutive rows (y0, y1) in the columns `from_x..760` of the page that hold a pixel `ink` says is ink.
fn runs(
    img: &image::RgbaImage,
    from_x: u32,
    ink: impl Fn(image::Rgba<u8>) -> bool,
) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for y in 0..img.height() {
        if !(from_x..760).any(|x| ink(*img.get_pixel(x, y))) {
            continue;
        }
        match out.last_mut() {
            Some(r) if r.1 + 1 == y => r.1 = y,
            _ => out.push((y, y)),
        }
    }
    out
}

fn big(runs: &[(u32, u32)]) -> Vec<(u32, u32)> {
    runs.iter().copied().filter(|r| r.1 - r.0 >= 20).collect()
}

#[test]
fn the_title_is_a_big_field_above_the_first_block() {
    for (language, name) in [
        ("en", "notion-titel-edit-en.png"),
        ("de", "notion-titel-edit-de.png"),
    ] {
        let Some(img) = shoot("# Handbook\n\nBody text\n", language, name) else {
            return;
        };
        let all = runs(&img, 345, dark);
        let titles = big(&all);
        assert_eq!(titles.len(), 1, "one big title ({language}): {all:?}");
        // The blocks come after it: the title is not the raw `# ` block (that block is small text below nothing big).
        assert!(
            all.iter().any(|r| r.0 > titles[0].1),
            "a block below the title ({language}): {all:?}"
        );
        assert!(
            all.iter().all(|r| r.0 >= titles[0].0 || r.1 - r.0 < 20),
            "nothing big above the title"
        );
    }
}

#[test]
fn without_a_title_the_placeholder_shows_in_both_languages() {
    let (Some(en), Some(de)) = (
        shoot("Body text\n", "en", "notion-titel-leer-en.png"),
        shoot("Body text\n", "de", "notion-titel-leer-de.png"),
    ) else {
        return;
    };
    // The placeholder is light, big text: no dark big run, but a big run of pale ink (right of the page icon).
    let pale = |p: image::Rgba<u8>| p[0] < 215 && p[1] < 215 && p[2] < 215;
    for (language, img) in [("en", &en), ("de", &de)] {
        assert!(
            big(&runs(img, 345, dark)).is_empty(),
            "no dark title ({language})"
        );
        assert_eq!(
            big(&runs(img, 345, pale)).len(),
            1,
            "a big placeholder ({language})"
        );
    }
    // Counter-test: the two languages draw different words.
    assert_ne!(en.as_raw(), de.as_raw(), "placeholder differs by language");
}
