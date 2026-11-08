//! A failing backup is shown: the timer thread redraws the page when the message changes.
//!
//! Tests: Observer test through the framework's pending-rebuild queue.

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use blinc_layout::prelude::RenderTree;
use blinc_layout::stateful::{has_pending_subtree_rebuilds, take_pending_subtree_rebuilds};
use common::{Editable, Fake, MemDrafts};
use fs_components_workbenches::{Labels, ReaderWidget};

#[test]
fn a_backup_failure_asks_the_page_to_redraw() {
    let tmp = std::env::temp_dir().join(format!("fs-redraw-{}.png", std::process::id()));
    let _ = fs_components_workbenches::snapshot(
        fs_components_workbenches::Picture::new(Arc::new(Fake), 200, 100),
        &tmp,
    );
    let _ = std::fs::remove_file(&tmp);
    blinc_core::reactive::set_stateful_deps_notifier(|ids| {
        blinc_layout::check_stateful_deps(ids);
    });
    let source = Arc::new(Editable::with(&[("a", "text\n")]));
    let store = Arc::new(MemDrafts::default());
    let widget = ReaderWidget::new(source, Labels::default(), None)
        .backup_timing(Duration::from_millis(50), Duration::from_millis(100))
        .drafts(store.clone());
    let ui = widget.build(900.0, 600.0);
    let _tree = RenderTree::from_element(&ui);
    let _ = take_pending_subtree_rebuilds();
    widget.refresh();
    assert!(
        has_pending_subtree_rebuilds(),
        "sanity: refresh queues a rebuild"
    );
    let _ = take_pending_subtree_rebuilds();
    widget.with_reader(fs_components_workbenches::Reader::begin_edit);
    let _ = take_pending_subtree_rebuilds();
    // The store breaks after the edit began: only the timer thread can notice.
    store.fail.store(true, std::sync::atomic::Ordering::Relaxed);
    widget.with_reader(|r| r.set_draft("typed"));
    let until = Instant::now() + Duration::from_secs(5);
    while widget.with_reader(|r| r.backup_problem().is_none()) {
        assert!(Instant::now() < until, "the backup never failed");
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(100));
    assert!(
        has_pending_subtree_rebuilds(),
        "the message about the failure is never drawn"
    );
}
