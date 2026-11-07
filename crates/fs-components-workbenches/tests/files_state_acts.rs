//! The state machine without a window: marked files, destructive acts, restore, filter, transfers, picker.
//!
//! Pattern: State tests over a source in memory. What only the real core can show is in `fs-files`.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::Shelf;
use fs_components_widgets::transfer_list::{Action, Outcome};
use fs_components_workbenches::file_manager::{
    Ask, CarryKind, Confirm, FileLabels, FilePicker, FileProblem, FilesState, Notice,
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

#[test]
fn another_press_never_throws_away_what_was_typed() {
    let (shelf, mut s) = state(Shelf::new().folder("d", None).file("a"));
    s.select("a");
    s.begin_new_folder();
    s.set_input("typed");
    for press in [
        (|s: &mut FilesState| s.ask_delete()) as fn(&mut FilesState),
        |s| s.carry(CarryKind::Move),
        |s| s.begin_rename(),
        |s| s.begin_download(),
        |s| s.open_folder(Some("d")),
    ] {
        press(&mut s);
        match s.ask() {
            Ask::Input(input) => assert_eq!(input.value, "typed"),
            other => panic!("{other:?}"),
        }
    }
    s.open_folder(None);
    s.discard_input();
    assert_eq!(
        *s.ask(),
        Ask::Nothing,
        "the band asked: this was the second act"
    );
    assert_eq!(shelf.writes(), 0);
}
