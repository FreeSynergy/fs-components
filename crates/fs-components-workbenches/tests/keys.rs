//! The reader's keys, through a built widget: Alt+Left goes back, Ctrl+S saves, Esc does nothing
//! (it must never throw a draft away), and a host can switch the keys off.
//!
//! Pattern: Event tests — the key event is broadcast to the built tree like the window runner does.

mod common;

use std::sync::Arc;

use blinc_core::events::event_types::KEY_DOWN;
use blinc_layout::prelude::RenderTree;
use common::{Editable, Fake};
use fs_components_workbenches::{Labels, Reader, ReaderWidget};

const LEFT: u32 = 0x25;
const S: u32 = 0x53;
const ESC: u32 = 0x1B;

/// Set the framework's state up the way a first picture does.
fn init() {
    let tmp = std::env::temp_dir().join(format!("fs-keys-{}.png", std::process::id()));
    let _ = fs_components_workbenches::snapshot(
        fs_components_workbenches::Picture::new(Arc::new(Fake), 200, 100),
        &tmp,
    );
    let _ = std::fs::remove_file(&tmp);
}

fn built(widget: &ReaderWidget) -> RenderTree {
    let ui = widget.build(900.0, 600.0);
    RenderTree::from_element(&ui)
}

fn key(tree: &mut RenderTree, code: u32, ctrl: bool, alt: bool) {
    tree.broadcast_key_event(KEY_DOWN, code, false, ctrl, alt, false);
}

fn editable() -> (Arc<Editable>, ReaderWidget) {
    let source = Arc::new(Editable::with(&[("a", "first\n"), ("b", "second\n")]));
    let widget = ReaderWidget::new(source.clone(), Labels::default(), None);
    (source, widget)
}

#[test]
fn ctrl_s_saves_and_alt_left_goes_back() {
    init();
    let (source, widget) = editable();
    let mut tree = built(&widget);
    widget.with_reader(|r| {
        r.begin_edit();
        r.set_draft("changed");
    });
    key(&mut tree, S, true, false);
    assert_eq!(source.text("a"), "changed\n", "Ctrl+S did not save");
    widget.with_reader(|r| r.open("b"));
    assert_eq!(
        widget.with_reader(|r| r.current_path().map(str::to_string)),
        Some("b".into())
    );
    key(&mut tree, LEFT, false, true);
    assert_eq!(
        widget.with_reader(|r| r.current_path().map(str::to_string)),
        Some("a".into())
    );
}

#[test]
fn escape_never_discards_a_draft() {
    init();
    let (_source, widget) = editable();
    let mut tree = built(&widget);
    widget.with_reader(|r| {
        r.begin_edit();
        r.set_draft("precious");
    });
    for _ in 0..3 {
        key(&mut tree, ESC, false, false);
    }
    assert!(
        widget.with_reader(|r| r.editing().is_some()),
        "Esc ended the edit"
    );
    assert_eq!(
        widget.with_reader(|r| r.editing().unwrap().draft.clone()),
        "precious\n"
    );
}

#[test]
fn a_host_can_switch_the_keys_off() {
    init();
    let (source, widget) = editable();
    let widget = widget.keys(false);
    let mut tree = built(&widget);
    widget.with_reader(|r| {
        r.begin_edit();
        r.set_draft("changed");
    });
    key(&mut tree, S, true, false);
    assert_eq!(
        source.text("a"),
        "first\n",
        "the reader took a key it was told to leave"
    );
    widget.with_reader(Reader::save);
    assert_eq!(source.text("a"), "changed\n");
}
