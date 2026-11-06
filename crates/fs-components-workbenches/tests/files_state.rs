//! The state machine without a window.
//!
//! Pattern: State tests over a source in memory. What only the real core can show is in `fs-files`.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::Shelf;
use fs_components_widgets::transfer_list::{Action, Outcome};
use fs_components_workbenches::file_manager::{
    Ask, CarryKind, Confirm, FileLabels, FilePicker, FileProblem, FileView, FilesState, Notice,
    PendingUpload, PreviewContent, Side, SortKey, Variant,
};

fn state(shelf: Shelf) -> (Arc<Shelf>, FilesState) {
    let shelf = Arc::new(shelf);
    let state = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    (shelf, state)
}

fn transfer_list_outcome(key: &str, retry: bool) -> Outcome {
    Outcome::new(
        key,
        if retry {
            Action::Retry
        } else {
            Action::Dismiss
        },
    )
}

fn notice(state: &FilesState) -> String {
    state
        .notice()
        .map(|n| n.text().to_string())
        .unwrap_or_default()
}

#[test]
fn nothing_is_selected_or_read_at_the_start_and_the_context_is_a_plain_value() {
    let (_, s) = state(Shelf::new().file("a"));
    let context = s.context();
    assert_eq!(context.selected, None);
    assert_eq!(context.folder, None);
    assert!(!context.in_trash);
    assert_eq!(context.view, FileView::List);
}

#[test]
fn a_taken_name_asks_and_changes_nothing_and_back_keeps_what_was_typed() {
    let (shelf, mut s) = state(Shelf::new().file("a").file("b"));
    s.select("a");
    s.begin_rename();
    s.set_input("b.txt");
    s.submit_input();
    assert!(matches!(s.ask(), Ask::NameTaken(_)));
    assert_eq!(shelf.thing("a").unwrap().name, "a.txt");
    s.name_taken_back();
    match s.ask() {
        Ask::Input(input) => assert_eq!(input.value, "b.txt"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_name_the_backend_cannot_hold_is_said_as_such_and_the_input_stays() {
    let (shelf, mut s) = state(Shelf::new().file("a"));
    s.select("a");
    s.begin_rename();
    s.set_input("../x");
    s.submit_input();
    match s.ask() {
        Ask::Input(input) => {
            assert_eq!(input.value, "../x");
            assert!(input.error.as_deref().unwrap().contains("cannot be used"));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(shelf.writes(), 0);
}

#[test]
fn a_failed_move_says_why_and_changes_nothing_and_a_thing_gone_leaves_the_view() {
    let (shelf, mut s) = state(Shelf::new().folder("d", None).file("a").file("b"));
    s.select("a");
    s.carry(CarryKind::Move);
    s.open_folder(Some("d"));
    assert!(s.carrying().is_some());
    shelf.break_it(Some("disk full"));
    s.put_down();
    assert!(notice(&s).contains("disk full"));
    shelf.break_it(None);
    assert_eq!(shelf.thing("a").unwrap().folder, None);
    s.open_folder(None);
    s.select("b");
    shelf.things.lock().unwrap().remove("b");
    s.retry();
    s.select("a");
    s.select("a");
    s.begin_rename();
    shelf.things.lock().unwrap().remove("a");
    s.set_input("c.txt");
    s.submit_input();
    assert!(s.rows().iter().all(|r| r.key != "a"), "gone from the list");
    assert_eq!(s.selected_key(), None);
}

#[test]
fn a_failed_read_keeps_the_rows_and_a_retry_comes_back() {
    let (shelf, mut s) = state(Shelf::new().numbered(10));
    assert_eq!(s.rows().len(), 10);
    shelf.break_it(Some("the store is gone"));
    s.retry();
    assert_eq!(s.rows().len(), 10, "the rows stay");
    assert!(s.list_problem().unwrap().contains("the store is gone"));
    shelf.break_it(None);
    s.retry();
    assert_eq!(s.list_problem(), None);
}

#[test]
fn one_bad_entry_is_named_and_the_rest_stays() {
    let shelf = Shelf::new().numbered(3);
    shelf.unreadable.lock().unwrap().push("broken.bin".into());
    let (_, s) = state(shelf);
    assert_eq!(s.rows().len(), 3);
    assert_eq!(s.unreadable(), ["broken.bin"]);
}

#[test]
fn pages_of_the_source_are_put_together_and_more_keeps_on_after_a_failure() {
    let (shelf, mut s) = state(Shelf::new().numbered(450));
    assert_eq!(s.rows().len(), 200);
    assert!(s.has_more());
    shelf.break_it(Some("gone"));
    s.load_more();
    assert_eq!(s.rows().len(), 200);
    assert!(s.list_problem().is_some());
    assert!(s.has_more(), "it can be tried again");
    shelf.break_it(None);
    s.load_more();
    assert_eq!(s.rows().len(), 400);
    s.load_more();
    assert_eq!(s.rows().len(), 450);
    assert!(!s.has_more());
}

#[test]
fn a_marked_file_is_not_read_until_the_person_asks_and_not_by_a_second_selection() {
    let shelf = Shelf::new().file("m").file("p");
    shelf.change("m", |t| t.marked = true);
    let (_, mut s) = state(shelf);
    s.select("m");
    assert!(s.preview_held_back());
    assert_eq!(
        s.content(),
        Some(&PreviewContent::Note(FileLabels::default().preview_marked))
    );
    let preview = s.preview().unwrap();
    assert_eq!(preview.marks, [FileLabels::default().marked]);
    s.release_preview();
    assert!(matches!(s.content(), Some(PreviewContent::Text(..))));
    assert!(!s.preview_held_back());
    s.select("p");
    s.select("m");
    assert!(s.preview_held_back(), "the release does not stick");
}

#[test]
fn a_marked_file_says_so_when_named_and_its_download_needs_a_second_act() {
    let shelf = Shelf::new().file("m");
    shelf.change("m", |t| t.marked = true);
    let (shelf, mut s) = state(shelf);
    s.select("m");
    s.ask_delete();
    assert!(s.confirm_words().unwrap().0.contains("[Sensitive]"));
    s.keep();
    s.begin_download();
    s.set_input("/there/m.txt");
    s.submit_input();
    assert!(matches!(
        s.ask(),
        Ask::Confirm(Confirm::DownloadMarked { .. })
    ));
    assert!(
        shelf.downloads.lock().unwrap().is_empty(),
        "nothing written yet"
    );
    s.keep();
    assert!(shelf.downloads.lock().unwrap().is_empty());
    s.begin_download();
    s.set_input("/there/m.txt");
    s.submit_input();
    s.really();
    assert_eq!(
        shelf.downloads.lock().unwrap().as_slice(),
        [("m".to_string(), "/there/m.txt".to_string(), true)]
    );
}

#[test]
fn a_changed_file_is_refused_and_shown_as_changed_outside() {
    let shelf = Shelf::new().file("a");
    shelf.change("a", |t| t.changed_outside = true);
    let (_, mut s) = state(shelf);
    s.select("a");
    match s.content() {
        Some(PreviewContent::Refused(why)) => assert!(why.contains("changed outside"), "{why}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn empty_trash_counts_again_and_a_changed_number_deletes_nothing() {
    let shelf = Shelf::new().file("a").file("b");
    shelf.change("a", |t| t.trashed = true);
    let (shelf, mut s) = state(shelf);
    s.open_trash();
    s.ask_empty_trash();
    assert!(s.confirm_words().unwrap().0.contains('1'));
    shelf.change("b", |t| t.trashed = true);
    s.really();
    assert!(
        shelf.thing("a").is_some(),
        "the number told was wrong: nothing goes"
    );
    assert!(
        s.confirm_words().unwrap().0.contains('2'),
        "asks with the new number"
    );
    s.really();
    assert!(shelf.thing("a").is_none() && shelf.thing("b").is_none());
}

#[test]
fn really_without_a_question_does_nothing_and_another_selection_takes_the_question_back() {
    let (shelf, mut s) = state(Shelf::new().file("a").file("b"));
    s.select("a");
    s.really();
    assert_eq!(shelf.writes(), 0);
    s.ask_delete();
    s.select("b");
    s.really();
    assert_eq!(shelf.writes(), 0, "the question was about a");
    s.ask_delete();
    s.open_folder(None);
    s.really();
    assert_eq!(shelf.writes(), 0, "going elsewhere takes it back too");
}

#[test]
fn restore_asks_on_a_taken_name_and_restore_to_goes_to_the_place_chosen() {
    let shelf = Shelf::new().folder("d", None).file("a");
    shelf.change("a", |t| t.trashed = true);
    let (shelf, mut s) = state(shelf.file("c"));
    shelf.change("c", |t| t.name = "a.txt".into());
    s.open_trash();
    s.select("a");
    s.restore_selected();
    assert!(matches!(s.ask(), Ask::NameTaken(_)));
    assert!(shelf.thing("a").unwrap().trashed);
    s.name_taken_back();
    s.carry(CarryKind::Restore);
    assert!(!s.in_trash());
    assert!(s.carrying().is_some());
    s.open_folder(Some("d"));
    s.put_down();
    let a = shelf.thing("a").unwrap();
    assert!(!a.trashed);
    assert_eq!(a.folder.as_deref(), Some("d"));
    assert_eq!(a.name, "a.txt");
}

#[test]
fn a_filter_that_cannot_be_read_says_so_and_the_list_stays() {
    let (_, mut s) = state(Shelf::new().file("a"));
    s.set_filter_field("min", "a lot");
    s.apply_filter();
    assert!(notice(&s).contains("a lot"));
    assert!(!s.filter().is_active());
    s.clear_filter();
    s.set_filter_field("from", "2026-02-30");
    s.apply_filter();
    assert!(matches!(
        s.notice(),
        Some(Notice::Problem(FileProblem::Failed, _))
    ));
    s.clear_filter();
    s.set_filter_field("min", "10 KB");
    s.set_filter_field("from", "2024-02-29");
    s.apply_filter();
    assert_eq!(s.filter().min_size, Some(10_000));
    assert!(s.rows().is_empty(), "a.txt is smaller");
    s.clear_filter();
    assert_eq!(s.rows().len(), 1);
}

#[test]
fn sort_by_type_turns_only_if_the_source_says_so_and_every_new_list_starts_over() {
    let (_, mut s) = state(Shelf::new().file("a"));
    let before = s.list_generation();
    s.set_sort(SortKey::Size);
    assert!(s.list_generation() > before);
    s.toggle_direction();
    assert!(s.sort().descending);
    let before = s.list_generation();
    s.search("a");
    assert!(s.list_generation() > before);
}

#[test]
fn an_unfinished_upload_can_go_on_and_dismissing_it_only_asks() {
    let shelf = Shelf::new();
    shelf
        .pending
        .lock()
        .unwrap()
        .push(PendingUpload::new("u1", "big.iso", 500));
    let (shelf, mut s) = state(shelf);
    s.open_transfers();
    assert_eq!(s.side(), Side::Transfers);
    let key = s.transfers()[0].shown.key.clone();
    s.transfer_outcome(&transfer_list_outcome(&key, false));
    assert!(matches!(s.ask(), Ask::Confirm(Confirm::DropUpload { .. })));
    assert_eq!(shelf.pending.lock().unwrap().len(), 1, "still there");
    s.keep();
    s.transfer_outcome(&transfer_list_outcome(&key, true));
    assert!(matches!(s.ask(), Ask::Input(_)));
    s.set_input("/there/big.iso");
    s.submit_input();
    assert!(shelf.pending.lock().unwrap().is_empty());
    assert!(notice(&s).contains("u1"));
}

#[test]
fn the_picker_changes_nothing_and_a_marked_file_is_a_second_act() {
    let shelf = Shelf::new().folder("d", None).file("a").file("m");
    shelf.change("m", |t| t.marked = true);
    let shelf = Arc::new(shelf);
    let picker = FilePicker::new(shelf.clone(), FileLabels::default());
    picker.with_state(|s| {
        assert!(!s.capabilities().change && !s.capabilities().trash && !s.capabilities().upload);
        s.select("a");
        s.ask_delete();
        s.really();
        s.begin_rename();
        assert_eq!(*s.ask(), Ask::Nothing);
        s.choose();
    });
    assert_eq!(picker.picked().unwrap().key, "a");
    assert_eq!(shelf.writes(), 0);
    picker.with_state(|s| {
        s.select("m");
        s.choose();
    });
    assert_eq!(picker.picked().unwrap().key, "a", "not yet");
    picker.with_state(FilesState::really);
    let picked = picker.picked().unwrap();
    assert_eq!(picked.key, "m");
    assert!(picked.marked);
}
