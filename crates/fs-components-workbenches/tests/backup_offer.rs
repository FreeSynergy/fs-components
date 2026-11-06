//! Answering an offered backup: nothing the person has not answered is touched, nothing typed is
//! lost, a wrong or stale backup is reported.
//!
//! Pattern: State machine tests through the reader and a built widget.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{Editable, MemDrafts};
use fs_components_workbenches::{Draft, Labels, Reader, ReaderWidget};

fn setup() -> (Arc<Editable>, Arc<MemDrafts>) {
    (
        Arc::new(Editable::with(&[("a", "original\n"), ("b", "bee\n")])),
        Arc::new(MemDrafts::default()),
    )
}

fn reader(source: &Arc<Editable>, store: &Arc<MemDrafts>) -> Reader {
    let mut r = Reader::new(source.clone(), Labels::default());
    r.set_draft_store(Some(store.clone()));
    r
}

fn wait_for(what: &str, mut ok: impl FnMut() -> bool) {
    let until = std::time::Instant::now() + Duration::from_secs(5);
    while !ok() {
        assert!(
            std::time::Instant::now() < until,
            "timed out waiting for {what}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// A backup of `path` written by an earlier session.
fn earlier_session(store: &Arc<MemDrafts>, path: &str, text: &str, original: &str) {
    store.plant(path, Draft::new(path, text, original, Some("1".into())));
}

fn fast(r: &mut Reader) {
    r.set_backup_timing(Duration::ZERO, Duration::ZERO);
}

#[test]
fn an_unanswered_offer_is_never_overwritten_or_cleared() {
    let (source, store) = setup();
    earlier_session(&store, "a", "X from before", "original\n");
    let mut r = reader(&source, &store);
    fast(&mut r);
    r.begin_edit();
    assert!(r.editing().unwrap().backup_offer.is_some());
    r.set_draft("Y typed now");
    r.backup_if_due();
    assert_eq!(
        store.drafts.lock().unwrap()["a"].text,
        "X from before",
        "typing overwrote the offer"
    );
    r.set_draft("original"); // typed and undone
    r.backup_if_due();
    assert!(store.has("a"), "type and undo cleared the offer");
    r.set_draft("Y again");
    r.cancel_edit();
    r.cancel_edit(); // chosen discard of Y
    assert_eq!(
        store.drafts.lock().unwrap()["a"].text,
        "X from before",
        "cancel deleted the offer"
    );
}

#[test]
fn dirty_state_of_one_document_does_not_touch_the_backup_of_another() {
    let (source, store) = setup();
    earlier_session(&store, "b", "Z for bee", "bee\n");
    let mut r = reader(&source, &store);
    fast(&mut r);
    r.begin_edit();
    r.set_draft("typed in a"); // dirty, not yet written
    r.cancel_edit();
    r.cancel_edit();
    r.open("b");
    r.begin_edit();
    r.backup_if_due();
    r.cancel_edit();
    assert_eq!(store.drafts.lock().unwrap()["b"].text, "Z for bee");
}

#[test]
fn restore_over_typed_text_asks_and_discard_keeps_backing_up_what_was_typed() {
    let (source, store) = setup();
    earlier_session(&store, "a", "X from before", "original\n");
    let mut r = reader(&source, &store);
    fast(&mut r);
    r.begin_edit();
    r.set_draft("Y typed");
    r.restore_backup();
    assert_eq!(
        r.editing().unwrap().draft,
        "Y typed\n",
        "it replaced typed text without asking"
    );
    assert!(r.notice().unwrap().contains("Press Restore again"));
    r.restore_backup();
    assert_eq!(r.editing().unwrap().draft, "X from before");
    // Discard instead: the typed text is backed up from then on.
    let mut r = reader(&source, &store);
    fast(&mut r);
    r.begin_edit();
    r.set_draft("Y typed");
    r.discard_backup();
    assert!(!store.has("a"));
    r.backup_if_due();
    assert_eq!(store.drafts.lock().unwrap()["a"].text, "Y typed\n");
}

#[test]
fn a_draft_stored_for_another_document_is_reported_not_offered() {
    let (source, store) = setup();
    store.plant("a", Draft::new("x/y", "other", "o", None)); // key "a" holds a draft of "x/y"
    let mut r = reader(&source, &store);
    r.begin_edit();
    assert!(r.editing().unwrap().backup_offer.is_none());
    assert!(r.backup_problem().unwrap().contains("x/y"));
}

#[test]
fn a_failed_backup_is_retried_when_the_store_recovers() {
    use std::sync::atomic::Ordering::Relaxed;
    let (source, store) = setup();
    let mut r = reader(&source, &store);
    fast(&mut r);
    r.begin_edit();
    store.fail.store(true, Relaxed);
    r.set_draft("precious");
    r.backup_if_due();
    assert!(r.backup_problem().is_some());
    store.fail.store(false, Relaxed);
    std::thread::sleep(Duration::from_millis(2100)); // the retry wait
    r.backup_if_due();
    assert_eq!(
        store.drafts.lock().unwrap()["a"].text,
        "precious\n",
        "the draft was lost"
    );
    assert!(r.backup_problem().is_none());
}

#[test]
fn an_offer_equal_to_the_document_is_removed() {
    let (source, store) = setup();
    earlier_session(&store, "a", "original\n", "original\n");
    let mut r = reader(&source, &store);
    r.begin_edit();
    assert!(r.editing().unwrap().backup_offer.is_none());
    assert!(!store.has("a"));
}

#[test]
fn restoring_for_a_source_without_revisions_asks_before_the_first_save() {
    use std::sync::atomic::Ordering::Relaxed;
    let (source, store) = setup();
    source.no_revisions.store(true, Relaxed);
    earlier_session(&store, "a", "old draft", "an older original\n");
    let mut r = reader(&source, &store);
    r.begin_edit();
    r.restore_backup();
    r.save();
    assert_eq!(
        source.text("a"),
        "original\n",
        "the first save overwrote newer text"
    );
    assert!(
        r.editing()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .contains("changed after")
    );
    r.save();
    assert_eq!(source.text("a"), "old draft");
}

#[test]
fn a_built_widget_backs_up_in_the_background_and_shows_nothing_lost() {
    let (source, store) = setup();
    // Set the framework's state up the way a first picture does.
    let tmp = std::env::temp_dir().join(format!("fs-bk-{}.png", std::process::id()));
    let _ = fs_components_workbenches::snapshot(
        fs_components_workbenches::Picture::new(Arc::new(common::Fake), 200, 100),
        &tmp,
    );
    let _ = std::fs::remove_file(&tmp);
    let widget = ReaderWidget::new(source.clone(), Labels::default(), None)
        .backup_timing(Duration::from_millis(100), Duration::from_millis(300))
        .drafts(store.clone());
    let ui = widget.build(900.0, 600.0);
    let _tree = blinc_layout::prelude::RenderTree::from_element(&ui);
    widget.with_reader(|r| {
        r.begin_edit();
        r.set_draft("typed in a window");
    });
    // The window's state keeps the reader alive: no drop happens, the timer must write it.
    wait_for("the background backup", || store.has("a"));
    assert_eq!(
        store.drafts.lock().unwrap()["a"].text,
        "typed in a window\n"
    );
    assert_eq!(source.text("a"), "original\n");
}

#[test]
fn a_write_in_flight_across_a_save_is_taken_back() {
    let (source, store) = setup();
    let mut r = reader(&source, &store);
    r.begin_edit();
    r.set_draft("typed");
    let job = r.backup_job_now().expect("a job");
    r.set_draft("typed more"); // keeps the edit changed until the save
    r.save(); // the save clears the backup of this edit...
    let result = job.run(); // ...and only now the slow write lands
    r.backup_done(&job, result);
    assert!(
        !store.has("a"),
        "the stale text was left in the store and would be offered next time"
    );
}

#[test]
fn text_typed_while_an_offer_waits_goes_to_a_second_slot_and_is_offered_next() {
    let (source, store) = setup();
    earlier_session(&store, "a", "X from before", "original\n");
    let mut r = reader(&source, &store);
    fast(&mut r);
    r.begin_edit();
    r.set_draft("Y typed later");
    r.backup_if_due();
    assert_eq!(
        store.drafts.lock().unwrap()["a"].text,
        "X from before",
        "the offer was touched"
    );
    assert!(
        store.drafts.lock().unwrap().len() == 2,
        "the typed text is not in the store: a crash loses it"
    );
    drop(r); // the window goes away
    let mut r = reader(&source, &store);
    r.begin_edit();
    assert_eq!(
        r.editing().unwrap().backup_offer.as_ref().unwrap().text,
        "Y typed later\n"
    );
    r.discard_backup();
    assert!(store.drafts.lock().unwrap().is_empty());
}
