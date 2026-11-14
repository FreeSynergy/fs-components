//! The page count in the side bar says "1 page" and "2 pages" (finding W3 of 2026-11-14: "1 Seiten").
//!
//! Tests: model tests over the labels, a picture test of the side bar (skipped without graphics or fonts).

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, fonts_work, pixels};
use fs_components_workbenches::{Layout, Picture, Step, WikiLabels, snapshot};

#[test]
fn one_page_is_said_in_the_singular_and_the_others_in_the_plural() {
    let l = WikiLabels::default();
    assert_eq!(
        (l.pages_said(0), l.pages_said(1), l.pages_said(2)),
        (
            "0 pages".to_owned(),
            "1 page".to_owned(),
            "2 pages".to_owned()
        )
    );
    // A program's words (German): the two forms are its own.
    let mut de = WikiLabels::default();
    de.page_count = "{} Seiten".into();
    de.page_count_one = "{} Seite".into();
    assert_eq!(
        (de.pages_said(1), de.pages_said(3)),
        ("1 Seite".to_owned(), "3 Seiten".to_owned())
    );
}

#[test]
fn the_side_bar_draws_the_form_for_the_number() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let mut shots = Vec::new();
    for (name, pages) in [
        ("seitenzahl-eine.png", &["a.md"][..]),
        ("seitenzahl-zwei.png", &["a.md", "b.md"][..]),
    ] {
        let source = Editable::with(&pages.iter().map(|p| (*p, "# T\n")).collect::<Vec<_>>());
        let mut picture = Picture::new(Arc::new(source), 700, 200);
        picture.layout = Layout::Confluence;
        picture.steps = vec![Step::Open("a.md".into())];
        let file = tmp.file(name);
        if snapshot(picture, &file).is_err() {
            eprintln!("skipped: no graphics");
            return;
        }
        if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::copy(&file, std::path::Path::new(&dir).join(name)).unwrap();
        }
        shots.push(pixels(&file));
    }
    assert_ne!(
        shots[0].as_raw(),
        shots[1].as_raw(),
        "one and two pages are drawn differently"
    );
}
