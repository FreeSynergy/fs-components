//! The second slot, writes in flight and discarding offers: more of the backup state machine.
//!
//! Pattern: State machine tests through the reader and a built widget.

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::{Editable, MemDrafts};
use fs_components_workbenches::{Draft, Labels, Reader};

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

/// A backup of `path` written by an earlier session.
fn earlier_session(store: &Arc<MemDrafts>, path: &str, text: &str, original: &str) {
    store.plant(path, Draft::new(path, text, original, Some("1".into())));
}

fn fast(r: &mut Reader) {
    r.set_backup_timing(Duration::ZERO, Duration::ZERO);
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
    r.discard_backup(); // the newer text goes; the older X waits behind it and is offered now
    assert_eq!(
        r.editing().unwrap().backup_offer.as_ref().unwrap().text,
        "X from before"
    );
    r.discard_backup();
    assert!(store.drafts.lock().unwrap().is_empty());
}

#[test]
fn discarding_the_newer_text_offers_the_older_backup_that_waited() {
    let (source, store) = setup();
    earlier_session(&store, "a", "X old", "original\n");
    let mut r = reader(&source, &store);
    fast(&mut r);
    r.begin_edit();
    r.set_draft("Y newer");
    r.backup_if_due(); // goes to the second slot, X stays
    drop(r);
    let mut r = reader(&source, &store);
    r.begin_edit();
    assert_eq!(
        r.editing().unwrap().backup_offer.as_ref().unwrap().text,
        "Y newer\n"
    );
    r.discard_backup();
    let offer = r
        .editing()
        .unwrap()
        .backup_offer
        .as_ref()
        .expect("X was lost with Y");
    assert_eq!(offer.text, "X old");
    assert_eq!(offer.path, "a", "the offer names the document");
}

#[test]
fn typing_during_a_clear_job_stays_marked_as_not_backed_up() {
    let (source, store) = setup();
    let mut r = reader(&source, &store);
    r.begin_edit();
    r.set_draft("typed");
    let write = r.backup_job_now().unwrap();
    let result = write.run();
    r.backup_done(&write, result);
    r.set_draft("original"); // typed and undone: the next job is a clear
    let clear = r.backup_job_now().expect("a clear job");
    r.set_draft("typed again"); // typing while the clear runs
    let result = clear.run();
    r.backup_done(&clear, result);
    assert!(
        r.backup_job_now().is_some(),
        "the new text is considered backed up and waits for the next keystroke"
    );
}
