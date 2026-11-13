//! A page and the folder of its sub-pages are one node (`handbook.md` + `handbook/chapter-one.md`).
//!
//! Pattern: none (tests of the arena built by `fold`).

use super::tree::PageTree;
use crate::source::Entry;

fn entry(path: &str, title: &str) -> Entry {
    Entry::new(path, title)
}

fn handbook() -> Vec<Entry> {
    vec![
        entry("handbook.md", "Handbook"),
        entry("handbook/chapter-one.md", "Chapter one"),
        entry("handbook/chapter-two.md", "Chapter two"),
        entry("notes.md", "Notes"),
    ]
}

/// `depth label [+|-|.]` per row: `+` closed and expandable, `-` open, `.` a leaf; `*` after it for a readable page.
fn shape(t: &PageTree) -> Vec<String> {
    t.rows(100)
        .iter()
        .map(|r| {
            let n = r.node;
            let mark = match (n.folder, r.open) {
                (false, _) => '.',
                (true, true) => '-',
                (true, false) => '+',
            };
            format!("{} {} {mark}{}", n.depth, n.label, if n.page { "*" } else { "" })
        })
        .collect()
}

#[test]
fn a_page_with_sub_pages_is_one_node() {
    let mut t = PageTree::new(&handbook());
    assert_eq!(shape(&t), ["0 Handbook +*", "0 Notes .*"]);
    t.toggle("handbook.md");
    assert_eq!(
        shape(&t),
        [
            "0 Handbook -*",
            "1 Chapter one .*",
            "1 Chapter two .*",
            "0 Notes .*"
        ]
    );
    t.toggle("handbook.md");
    assert_eq!(shape(&t), ["0 Handbook +*", "0 Notes .*"]);
    assert_eq!(t.pages(), 4);
}

#[test]
fn the_order_of_the_list_does_not_matter() {
    let mut list = handbook();
    list.reverse();
    let mut t = PageTree::new(&list);
    t.toggle("handbook.md");
    assert_eq!(t.rows(100).len(), 4);
    assert_eq!(t.rows(100).iter().filter(|r| r.node.label == "handbook").count(), 0);
}

#[test]
fn a_folder_without_a_page_stays_a_folder() {
    let mut t = PageTree::new(&[entry("guides/a.md", "A"), entry("z.md", "Z")]);
    assert_eq!(shape(&t), ["0 guides +", "0 Z .*"]);
    t.toggle("guides/");
    assert_eq!(shape(&t), ["0 guides -", "1 A .*", "0 Z .*"]);
}

#[test]
fn an_index_page_still_sits_in_its_folder() {
    let mut t = PageTree::new(&[entry("x/index.md", "X home"), entry("x/y.md", "Y")]);
    t.toggle("x/");
    assert_eq!(shape(&t), ["0 x -", "1 X home .*", "1 Y .*"]);
}

#[test]
fn the_nesting_goes_deep() {
    let mut t = PageTree::new(&[
        entry("a.md", "A"),
        entry("a/b.md", "B"),
        entry("a/b/c.md", "C"),
    ]);
    assert_eq!(shape(&t), ["0 A +*"]);
    t.toggle("a.md");
    assert_eq!(shape(&t), ["0 A -*", "1 B +*"]);
    t.toggle("a/b.md");
    assert_eq!(shape(&t), ["0 A -*", "1 B -*", "2 C .*"]);
}

#[test]
fn revealing_a_sub_page_opens_its_parent_page() {
    let mut t = PageTree::new(&handbook());
    t.reveal("handbook/chapter-two.md");
    assert_eq!(t.rows(100).len(), 4);
    let mut deep = PageTree::new(&[
        entry("a.md", "A"),
        entry("a/b.md", "B"),
        entry("a/b/c.md", "C"),
    ]);
    deep.reveal("a/b/c.md");
    assert_eq!(shape(&deep), ["0 A -*", "1 B -*", "2 C .*"]);
}
