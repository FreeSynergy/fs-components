//! A change made through the widget reaches the live areas, also when a picture was drawn earlier
//! in the same process (the picture sets the reactive state up without the notifier a window
//! runner installs).
//!
//! Tests: State test through the widget and the framework's pending-rebuild queue.

mod common;

use std::sync::Arc;

use blinc_layout::prelude::RenderTree;
use blinc_layout::stateful::{has_pending_subtree_rebuilds, take_pending_subtree_rebuilds};
use common::Fake;
use common::draw::Scratch;
use fs_components_workbenches::{Labels, Picture, ReaderWidget, snapshot};

#[test]
fn a_refresh_after_a_picture_reaches_the_stateful_areas() {
    let tmp = Scratch::new();
    // Order A: a picture first. (Without graphics the picture cannot be drawn; the state is
    // set up by the first widget build all the same, so the test goes on.)
    let _ = snapshot(
        Picture::new(Arc::new(Fake), 300, 200),
        &tmp.file("first.png"),
    );
    blinc_theme::ThemeState::init_default();
    // What the window runner does outside its init guard: the notifier that tells stateful areas
    // which signals changed.
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    let widget = ReaderWidget::new(Arc::new(Fake), Labels::default(), None);
    let ui = widget.build(900.0, 600.0);
    let _tree = RenderTree::from_element(&ui);
    let _ = take_pending_subtree_rebuilds();
    widget.with_reader(|r| r.open("a"));
    widget.refresh();
    assert!(
        has_pending_subtree_rebuilds(),
        "the change did not reach the stateful areas"
    );
}
