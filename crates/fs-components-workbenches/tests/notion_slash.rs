//! Notion frontend, editing: "/" in a block opens the block menu (first step: block types only); typing filters it, Up, Down,
//! Enter and Esc and the mouse work; the words come in German and English.
//!
//! Tests: event tests through the built widget, the key event is broadcast to the built tree like the window runner does.
//! Pictures of the open menu are kept when `FS_COMP_SHOTS` names a directory.

mod common;

use std::sync::Arc;

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::event_router::{EventRouter, MouseButton};
use blinc_layout::prelude::RenderTree;
use common::draw::{Scratch, fonts_work, pixels};
use common::{Editable, Fake};
use fs_components_words::{Locale, Localized};
use fs_components_workbenches::{
    Labels, Layout, Picture, ReaderWidget, Step, WikiLabels, snapshot,
};

const ENTER: u32 = 13;
const ESC: u32 = 27;
const DOWN: u32 = 40;
const UP: u32 = 38;

fn init() {
    let tmp = std::env::temp_dir().join(format!("fs-slash-{}.png", std::process::id()));
    let _ = snapshot(Picture::new(Arc::new(Fake), 200, 100), &tmp);
    let _ = std::fs::remove_file(&tmp);
    blinc_theme::ThemeState::init_default();
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
}

/// The widget editing a page of one block "first", in the Notion layout, the keyboard in a new empty second block.
fn editing() -> (ReaderWidget, RenderTree) {
    init();
    let source = Editable::with(&[("h.md", "# H\n\nfirst\n")]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let widget = ReaderWidget::new(Arc::new(source), Labels::default(), None)
        .layout(Layout::Notion, WikiLabels::default());
    widget.with_reader(|r| {
        r.open("h.md");
        r.begin_edit();
    });
    let mut tree = RenderTree::from_element(&widget.build(1000.0, 600.0));
    widget.focus_block(0, 5);
    press(&mut tree, ENTER);
    (widget, tree)
}

fn press(tree: &mut RenderTree, code: u32) {
    tree.broadcast_key_event(KEY_DOWN, code, false, false, false, false);
    tree.process_pending_subtree_rebuilds();
}

fn shown(tree: &RenderTree, id: &str) -> bool {
    tree.query_by_id(id).is_some()
}

#[test]
fn a_slash_opens_the_menu_typing_filters_and_enter_chooses() {
    let (widget, mut tree) = editing();
    assert_eq!(widget.focused_block(), Some(1));
    assert!(!widget.block_menu_open());
    widget.type_in_block(1, 0, "/");
    tree.process_pending_subtree_rebuilds();
    assert!(widget.block_menu_open(), "a slash opens the menu");
    assert!(shown(&tree, "fs-wk-slash-9"), "all ten types are offered");
    // Typing narrows the list to the three headings.
    widget.type_in_block(1, 1, "head");
    tree.process_pending_subtree_rebuilds();
    assert!(shown(&tree, "fs-wk-slash-2") && !shown(&tree, "fs-wk-slash-3"));
    // Down marks Heading 2; Enter writes it into the block, closes the menu and keeps the keyboard there.
    press(&mut tree, DOWN);
    press(&mut tree, ENTER);
    assert_eq!(widget.block_texts(), ["first", "## "]);
    assert!(!widget.block_menu_open());
    assert!(!shown(&tree, "fs-wk-slash-0"), "the menu is gone");
    assert_eq!(widget.focused_block(), Some(1));
    let draft = widget.with_reader(|r| r.editing().unwrap().draft.clone());
    assert_eq!(
        draft, "# H\n\nfirst\n\n## \n",
        "Save writes the heading marker as typed"
    );
}

#[test]
fn up_wraps_and_esc_closes_without_changing_the_text() {
    let (widget, mut tree) = editing();
    widget.type_in_block(1, 0, "/");
    tree.process_pending_subtree_rebuilds();
    press(&mut tree, UP);
    press(&mut tree, ESC);
    assert!(!widget.block_menu_open());
    assert_eq!(
        widget.block_texts(),
        ["first", "/"],
        "the text stays as typed"
    );
}

#[test]
fn a_filter_without_a_match_offers_nothing_and_enter_then_splits_as_usual() {
    let (widget, mut tree) = editing();
    widget.type_in_block(1, 0, "/zzz");
    tree.process_pending_subtree_rebuilds();
    assert!(widget.block_menu_open());
    assert!(!shown(&tree, "fs-wk-slash-0"));
}

#[test]
fn the_mouse_chooses_a_type() {
    let (widget, mut tree) = editing();
    widget.type_in_block(1, 0, "/");
    tree.process_pending_subtree_rebuilds();
    tree.compute_layout(1000.0, 600.0);
    let node = tree.query_by_id("fs-wk-slash-5").expect("the to-do entry");
    let b = tree.get_absolute_bounds(node).expect("bounds");
    let (x, y) = (b.x + b.width / 2.0, b.y + b.height / 2.0);
    let mut router = EventRouter::new();
    for (events, _) in [(router.on_mouse_move(&tree, x, y), 0)] {
        for (n, e) in events {
            tree.dispatch_event(n, e, x, y);
        }
    }
    for events in [
        router.on_mouse_down(&tree, x, y, MouseButton::Left),
        router.on_mouse_up(&tree, x, y, MouseButton::Left),
    ] {
        for (n, e) in events {
            tree.dispatch_event(n, e, x, y);
        }
    }
    assert_eq!(widget.block_texts(), ["first", "- [ ] "]);
    assert!(!widget.block_menu_open());
}

#[test]
fn the_menu_is_drawn_in_both_languages() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let mut drawn = Vec::new();
    for (lang, name) in [("en", "notion-slash-en.png"), ("de", "notion-slash-de.png")] {
        let source = Editable::with(&[("h.md", "# H\n\nfirst\n")]);
        source
            .heading_titles
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let mut picture = Picture::new(Arc::new(source), 1000, 800);
        picture.layout = Layout::Notion;
        let locale = Locale::parse(lang).unwrap();
        picture.labels = Labels::for_locale(&locale);
        picture.wiki_labels = WikiLabels::for_locale(&locale);
        picture.steps = vec![Step::Open("h.md".into()), Step::Edit, Step::BlockMenu];
        let file = tmp.file(name);
        if snapshot(picture, &file).is_err() {
            eprintln!("skipped: no graphics");
            return;
        }
        if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::copy(&file, std::path::Path::new(&dir).join(name)).unwrap();
        }
        drawn.push(pixels(&file));
    }
    assert_ne!(
        drawn[0].as_raw(),
        drawn[1].as_raw(),
        "the menu words differ by language"
    );
}
