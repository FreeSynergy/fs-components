//! Backup of drafts: survives a dropped widget, is offered and never applied by itself, is
//! cleared by a save or a chosen discard, and failures are shown.
//!
//! Pattern: State machine tests through the reader and the widget.

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{Editable, MemDrafts};
use fs_components_workbenches::{Labels, Reader, ReaderWidget};

fn setup() -> (Arc<Editable>, Arc<MemDrafts>) {
    (
        Arc::new(Editable::with(&[("a", "original\n")])),
        Arc::new(MemDrafts::default()),
    )
}

fn reader(source: &Arc<Editable>, store: &Arc<MemDrafts>) -> Reader {
    let mut r = Reader::new(source.clone(), Labels::default());
    r.set_draft_store(Some(store.clone()));
    r
}

fn wait_for(what: &str, mut ok: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(5);
    while !ok() {
        assert!(Instant::now() < until, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn a_dropped_widget_leaves_its_draft_and_the_next_start_offers_it() {
    let (source, store) = setup();
    let widget = ReaderWidget::new(source.clone(), Labels::default(), None).drafts(store.clone());
    widget.with_reader(|r| {
        r.begin_edit();
        r.set_draft("unsaved words");
    });
    drop(widget); // the window is closed without saving
    wait_for("the backup", || store.has("a"));
    assert_eq!(
        source.text("a"),
        "original\n",
        "a backup never changes the document"
    );

    let widget = ReaderWidget::new(source.clone(), Labels::default(), None).drafts(store.clone());
    widget.with_reader(Reader::begin_edit);
    assert_eq!(
        widget.editor_text().as_deref(),
        Some("original"),
        "the backup is offered, not applied"
    );
    assert!(widget.with_reader(|r| r.editing().unwrap().backup_offer.is_some()));
    widget.with_reader(Reader::restore_backup);
    assert_eq!(widget.editor_text().as_deref(), Some("unsaved words"));
    widget.with_reader(Reader::save);
    assert_eq!(source.text("a"), "unsaved words\n");
    assert!(!store.has("a"), "a save clears the backup");
}

#[test]
fn backup_follows_the_pause_and_the_interval() {
    let (source, store) = setup();
    let mut r = reader(&source, &store);
    r.begin_edit();
    r.set_draft("typing");
    r.backup_if_due();
    assert!(!store.has("a"), "not yet: typing has not paused");
    r.set_backup_timing(Duration::ZERO, Duration::from_secs(10));
    r.backup_if_due();
    assert!(store.has("a"));
}

#[test]
fn a_chosen_discard_clears_but_an_unchanged_cancel_keeps_it() {
    let (source, store) = setup();
    let mut r = reader(&source, &store);
    r.set_backup_timing(Duration::ZERO, Duration::ZERO);
    r.begin_edit();
    r.set_draft("draft");
    r.backup_if_due();
    r.cancel_edit();
    r.cancel_edit(); // the person chose to discard
    assert!(!store.has("a"));
    r.begin_edit();
    r.set_draft("again");
    r.backup_if_due();
    drop(r);
    let mut r = reader(&source, &store);
    r.begin_edit();
    assert!(r.editing().unwrap().backup_offer.is_some());
    r.cancel_edit(); // looked at nothing, changed nothing
    assert!(
        store.has("a"),
        "the offer was not answered, the backup stays"
    );
    r.begin_edit();
    r.discard_backup();
    assert!(!store.has("a"));
}

#[test]
fn a_draft_for_a_changed_document_is_a_conflict_like_a_save() {
    let (source, store) = setup();
    let mut r = reader(&source, &store);
    r.set_backup_timing(Duration::ZERO, Duration::ZERO);
    r.begin_edit();
    r.set_draft("my old draft");
    r.backup_if_due();
    drop(r);
    source.other_saves("a", "theirs\n");
    let mut r = reader(&source, &store);
    r.begin_edit();
    r.restore_backup();
    let e = r.editing().unwrap();
    assert_eq!(e.draft, "my old draft\n");
    assert!(
        e.error
            .as_ref()
            .unwrap()
            .contains("changed after this draft")
    );
    r.save();
    assert_eq!(
        source.text("a"),
        "theirs\n",
        "their version survives the first save"
    );
    assert!(
        r.editing()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .contains("Press Save again")
    );
    r.save();
    assert_eq!(source.text("a"), "my old draft\n");
}

#[test]
fn a_failing_store_is_shown_not_hidden() {
    let (source, store) = setup();
    let mut r = reader(&source, &store);
    r.set_backup_timing(Duration::ZERO, Duration::ZERO);
    store.fail.store(true, std::sync::atomic::Ordering::Relaxed);
    r.begin_edit();
    assert!(
        r.backup_problem().unwrap().contains("disk is full"),
        "load failed"
    );
    r.set_draft("x");
    r.backup_if_due();
    assert!(
        r.backup_problem()
            .unwrap()
            .contains("Backup of your draft failed")
    );
}
