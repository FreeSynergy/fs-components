//! The file list: size text, order, rows, and the drawn element.
//!
//! Tests: Model tests + one offscreen picture. No window is opened.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use fs_components_file_list::{FileEntry, FileList, FileListWords, SortBy, human_size, sorted};

fn words() -> FileListWords {
    FileListWords::new(
        ("Name", "Size", "Type", "Modified"),
        "Folder",
        "Nothing here",
    )
}

fn entries() -> Vec<FileEntry> {
    vec![
        FileEntry::file("b", "beta.txt", 2048)
            .with_media_type("text/plain")
            .with_modified("2026-11-02T10:00:00Z"),
        FileEntry::file("a", "Alpha.png", 10)
            .with_media_type("image/png")
            .with_modified("2026-11-03T10:00:00Z"),
        FileEntry::folder("f", "Zeta").with_modified("2026-11-01T10:00:00Z"),
    ]
}

fn keys(sort: SortBy, descending: bool) -> Vec<String> {
    sorted(entries(), sort, descending)
        .into_iter()
        .map(|e| e.key)
        .collect()
}

#[test]
fn sizes_are_written_for_people() {
    assert_eq!(human_size(0), "0 B");
    assert_eq!(human_size(1023), "1023 B");
    assert_eq!(human_size(1536), "1.5 KB");
    assert_eq!(human_size(10 * 1024 * 1024), "10 MB");
    assert_eq!(human_size(1 << 50), "1024 TB");
}

#[test]
fn the_order_is_stable_and_follows_the_choice() {
    assert_eq!(keys(SortBy::Name, false), ["f", "a", "b"]); // folders first, then case-insensitive
    assert_eq!(keys(SortBy::Name, true), ["f", "b", "a"]);
    assert_eq!(keys(SortBy::Size, false), ["f", "a", "b"]);
    assert_eq!(keys(SortBy::Size, true), ["b", "a", "f"]);
    assert_eq!(keys(SortBy::Modified, true), ["a", "b", "f"]);
    assert_eq!(keys(SortBy::Kind, false), ["a", "f", "b"]); // image/png < inode/directory < text/plain
}

#[test]
fn rows_have_the_texts_and_a_folder_has_its_mark_and_no_size() {
    let list = FileList::new(entries(), SortBy::Name, false, words());
    let rows = list.rows();
    assert_eq!(rows[0].cells, ["Zeta", "", "", "2026-11-01T10:00:00Z"]);
    assert_eq!(rows[0].mark.as_deref(), Some("Folder"));
    assert_eq!(rows[2].cells[1], "2.0 KB");
    assert_eq!(rows[1].mark, None);
}

#[test]
fn the_list_is_drawn_with_its_headings() {
    let _g = ui::serial();
    let tree = ui::layout(
        FileList::new(entries(), SortBy::Name, false, words()).build(600.0, 300.0),
        600.0,
        300.0,
    );
    assert!(tree.query_by_id("fs-file-list").is_some());
    let _ = ui::draw_tree(&tree, 600, 300, "file-list");
}
