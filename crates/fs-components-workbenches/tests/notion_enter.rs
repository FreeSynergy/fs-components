//! Notion frontend, editing: Enter in a block makes the next block at once and gives it the keyboard (finding of
//! 2026-11-14: the new fields showed only after Save and reload).
//!
//! Tests: event tests through the built widget, the key event is broadcast to the built tree like the window runner does.
//! The page is shown in the tree after the key: the block elements are counted, not only the model.

mod common;

use std::sync::Arc;

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Editable, Fake};
use fs_components_workbenches::{Labels, Layout, ReaderWidget, WikiLabels};

const ENTER: u32 = 13;

fn init() {
    let tmp = std::env::temp_dir().join(format!("fs-enter-{}.png", std::process::id()));
    let _ = fs_components_workbenches::snapshot(
        fs_components_workbenches::Picture::new(Arc::new(Fake), 200, 100),
        &tmp,
    );
    let _ = std::fs::remove_file(&tmp);
    blinc_theme::ThemeState::init_default();
    // What the window runner installs: the notifier that tells stateful areas which signals changed.
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
}

/// The widget editing "# Handbook\n\nfirst line\n" in the Notion layout, built into a tree.
fn editing() -> (ReaderWidget, RenderTree) {
    init();
    let source = Editable::with(&[("handbook.md", "# Handbook\n\nfirst line\n")]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let widget = ReaderWidget::new(Arc::new(source), Labels::default(), None)
        .layout(Layout::Notion, WikiLabels::default());
    widget.with_reader(|r| {
        r.open("handbook.md");
        r.begin_edit();
    });
    let ui = widget.build(1000.0, 600.0);
    let tree = RenderTree::from_element(&ui);
    (widget, tree)
}

/// Press Enter, then let the window draw what the key changed (the next frame).
fn enter(tree: &mut RenderTree) {
    tree.broadcast_key_event(KEY_DOWN, ENTER, false, false, false, false);
    tree.process_pending_subtree_rebuilds();
}

fn shown(tree: &RenderTree, i: usize) -> bool {
    tree.query_by_id(&format!("fs-wk-blk-{i}")).is_some()
}

#[test]
fn enter_at_the_end_makes_a_new_block_below_and_focuses_it() {
    let (widget, mut tree) = editing();
    assert_eq!(widget.block_texts(), ["first line"]);
    assert!(
        shown(&tree, 0) && !shown(&tree, 1),
        "before Enter: one block"
    );
    widget.focus_block(0, 10);
    assert_eq!(widget.focused_block(), Some(0));
    enter(&mut tree);
    assert_eq!(widget.block_texts(), ["first line", ""], "model");
    assert!(shown(&tree, 1), "the new block is in the window at once");
    assert_eq!(widget.focused_block(), Some(1), "the keyboard moved on");
    // Enter again in the empty block: a third one.
    enter(&mut tree);
    assert_eq!(widget.block_texts(), ["first line", "", ""]);
    assert!(shown(&tree, 2));
    assert_eq!(widget.focused_block(), Some(2));
}

#[test]
fn enter_in_the_middle_splits_the_block_and_the_draft_follows() {
    let (widget, mut tree) = editing();
    widget.focus_block(0, 5);
    enter(&mut tree);
    assert_eq!(widget.block_texts(), ["first", " line"]);
    assert!(shown(&tree, 1));
    assert_eq!(widget.focused_block(), Some(1));
    let draft = widget.with_reader(|r| r.editing().unwrap().draft.clone());
    assert_eq!(
        draft, "# Handbook\n\nfirst\n\n line\n",
        "Save would write both blocks"
    );
}

#[test]
fn enter_in_a_list_carries_the_list_on_and_an_empty_item_leaves_it() {
    init();
    let source = Editable::with(&[("l.md", "# L\n\n- a\n")]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let widget = ReaderWidget::new(Arc::new(source), Labels::default(), None)
        .layout(Layout::Notion, WikiLabels::default());
    widget.with_reader(|r| {
        r.open("l.md");
        r.begin_edit();
    });
    let mut tree = RenderTree::from_element(&widget.build(1000.0, 600.0));
    widget.focus_block(0, 3);
    enter(&mut tree);
    assert_eq!(
        widget.block_texts(),
        ["- a\n- "],
        "the list goes on in the same block"
    );
    assert_eq!(widget.focused_block(), Some(0));
    // Enter on the empty item ends the list: the item goes, a new block follows.
    enter(&mut tree);
    assert_eq!(widget.block_texts(), ["- a", ""]);
    assert_eq!(widget.focused_block(), Some(1));
    assert!(shown(&tree, 1));
}

/// The pictures of the finding: the page with one block, and the same page after Enter at the end of it (two blocks).
#[test]
fn the_window_shows_the_new_block_after_enter() {
    use common::draw::{Scratch, fonts_work, pixels};
    use fs_components_workbenches::{Picture, Step, snapshot};
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let mut shots = Vec::new();
    for (name, steps) in [
        (
            "notion-enter-vorher.png",
            vec![Step::Open("h.md".into()), Step::Edit],
        ),
        (
            "notion-enter-nachher.png",
            vec![Step::Open("h.md".into()), Step::Edit, Step::BlockEnter],
        ),
    ] {
        let source = Editable::with(&[("h.md", "# H\n\nfirst line\n")]);
        source
            .heading_titles
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let mut picture = Picture::new(Arc::new(source), 1000, 520);
        picture.layout = Layout::Notion;
        picture.steps = steps;
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
    // The edges of a block box are long light-grey lines; the second box adds its own.
    let edges = |img: &image::RgbaImage| {
        (0..img.height())
            .filter(|&y| {
                (330..920)
                    .filter(|&x| {
                        let p = img.get_pixel(x, y);
                        p[0] < 245 && p[0] > 150 && p[0].abs_diff(p[1]) < 8
                    })
                    .count()
                    > 400
            })
            .count()
    };
    assert!(
        edges(&shots[1]) >= edges(&shots[0]) + 2,
        "the second block box is drawn: {} edge rows before, {} after",
        edges(&shots[0]),
        edges(&shots[1])
    );
}
