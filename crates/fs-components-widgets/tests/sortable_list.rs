//! The sortable list drawn, clicked and typed at, in process (no window).
//!
//! Pattern: Pixel tests plus an in-process mouse and keyboard over the real router and hit test.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, background, draw, ink, layout, serial};
use fs_components_widgets::SortableList;
use fs_components_widgets::sortable_list::{Bindings, Command, Item, Lane, Moved, Outcome, Place};

fn lane(key: &str, items: &[&str]) -> Lane {
    Lane::new(
        key,
        key.to_uppercase(),
        items
            .iter()
            .map(|k| Item::new(*k, format!("Card {k}")))
            .collect(),
    )
}

fn board(id: &str) -> SortableList {
    SortableList::new(vec![
        lane("todo", &["a", "b", "c"]),
        lane("doing", &["x"]),
        lane("done", &[]),
    ])
    .id(id)
}

fn y_of(tree: &blinc_layout::prelude::RenderTree, id: &str) -> f32 {
    let node = tree
        .query_by_id(id)
        .unwrap_or_else(|| panic!("no row {id}"));
    tree.get_absolute_bounds(node).unwrap().y
}

#[test]
fn the_board_is_drawn_with_cursor_and_held_row() {
    let _g = serial();
    let list = board("b1");
    let img = draw(list.build(), 600, 260, "sortable-board-idle.png");
    assert!(ink(&img, (0, 0, 600, 260), background(&img)) > 5000);
    list.command(Command::Toggle); // pick up "a"
    list.command(Command::Down);
    list.command(Command::Right);
    let held = list.build();
    let tree = layout(held, 600.0, 260.0);
    let img2 = common::draw_tree(&tree, 600, 260, "sortable-board-carrying.png");
    assert_ne!(img, img2, "carrying must change the picture");
    // "a" now sits in the second lane, below "x": the lane lists x, then a
    let (x, a) = (y_of(&tree, "b1-doing-x"), y_of(&tree, "b1-doing-a"));
    assert!(a > x, "a at {a}, x at {x}");
    assert!(tree.query_by_id("b1-todo-a").is_none());
}

#[test]
fn keys_work_only_when_bound_and_only_on_the_list_that_has_the_keyboard() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let a = board("ka")
        .keys(Bindings::default())
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let b = board("kb").keys(Bindings::default());
    let plain = board("kc"); // no keys bound at all
    let ui = blinc_layout::div::div()
        .flex_col()
        .child(a.build())
        .child(b.build())
        .child(plain.build());
    let mut tree = layout(ui, 600.0, 600.0);
    let mut input = Input::new();

    // nobody has the keyboard: nothing acts
    input.key_broadcast(&mut tree, 32);
    assert_eq!(
        (a.grabbed(), b.grabbed(), plain.grabbed()),
        (None, None, None)
    );

    // `a` has it; the broadcast reaches every handler but only `a` acts
    input.focus(&tree, "ka");
    input.key_broadcast(&mut tree, 32); // Space: pick up
    assert_eq!(
        (a.grabbed().as_deref(), b.grabbed(), plain.grabbed()),
        (Some("a"), None, None)
    );
    input.key_broadcast(&mut tree, 40); // Down
    input.key_broadcast(&mut tree, 32); // Space: drop
    let lanes = a.lanes();
    assert_eq!(
        lanes[0]
            .items
            .iter()
            .map(|i| i.key.as_str())
            .collect::<Vec<_>>(),
        ["b", "a", "c"]
    );
    assert_eq!(
        b.lanes()[0].items[0].key,
        "a",
        "the other list did not move"
    );
    assert_eq!(
        got.lock().unwrap().last(),
        Some(&Outcome::Moved(Moved::new(
            "a",
            Place::new("todo", 0),
            Place::new("todo", 1)
        )))
    );

    // focus moves to the one without bindings: it takes no key
    input.focus(&tree, "kc");
    input.key(&mut tree, 32);
    input.key_broadcast(&mut tree, 32);
    assert_eq!(plain.grabbed(), None);
    assert_eq!(a.grabbed(), None);
}

#[test]
fn a_click_puts_the_cursor_on_the_row() {
    let _g = serial();
    let list = board("cl");
    let mut tree = layout(list.build(), 600.0, 260.0);
    assert_eq!(list.cursor_key().as_deref(), Some("a"));
    Input::new().click(&mut tree, "cl-todo-c");
    assert_eq!(list.cursor_key().as_deref(), Some("c"));
}

#[test]
fn a_huge_lane_draws_a_window_that_follows_the_cursor() {
    let _g = serial();
    let items: Vec<Item> = (0..100_000)
        .map(|i| Item::new(format!("k{i}"), format!("Row {i}")))
        .collect();
    let list = SortableList::new(vec![Lane::new("big", "Big", items)])
        .id("hg")
        .max_rows(30);
    let tree = layout(list.build(), 400.0, 900.0);
    assert!(tree.query_by_id("hg-big-k0").is_some());
    assert!(tree.query_by_id("hg-big-k29").is_some());
    assert!(tree.query_by_id("hg-big-k30").is_none());
    list.command(Command::Last);
    let img = draw(list.build(), 400, 900, "sortable-huge-end.png");
    let tree = layout(list.build(), 400.0, 900.0);
    assert!(tree.query_by_id("hg-big-k99999").is_some());
    assert!(tree.query_by_id("hg-big-k0").is_none());
    assert!(ink(&img, (0, 0, 400, 900), background(&img)) > 5000);
}

#[test]
fn odd_content_and_sizes_do_not_crash() {
    let _g = serial();
    let odd = SortableList::new(vec![
        Lane::new(
            "",
            "",
            vec![
                Item::new("", ""),
                Item::new("u", "ein sehr langer Name äöü 李雷 😀 ".repeat(5)),
            ],
        ),
        Lane::new("l\u{0}", "Tïtle ".repeat(50), vec![]),
    ])
    .id("odd");
    let img = draw(odd.build(), 300, 200, "sortable-odd.png");
    assert!(ink(&img, (0, 0, 300, 200), background(&img)) > 1000);
    // nothing drawn outside the box to the right of the second lane
    let _ = layout(odd.build(), 0.0, 0.0);
    let empty = SortableList::new(vec![]);
    let img = draw(empty.build(), 100, 50, "sortable-empty.png");
    assert_eq!(ink(&img, (0, 0, 100, 50), background(&img)), 0);
    assert_eq!(
        empty.press(&fs_components_widgets::KeyEvent::new(
            fs_components_widgets::Key::Down
        )),
        Outcome::Ignored
    );
    assert_eq!(
        empty.command(Command::Down),
        Outcome::Refused(fs_components_widgets::sortable_list::Refusal::Empty)
    );
}

#[test]
fn a_long_label_wraps_instead_of_being_cut() {
    let _g = serial();
    let long = SortableList::new(vec![Lane::new(
        "l",
        "",
        vec![Item::new(
            "a",
            "Write the very long report about the thing that went wrong last quarter in detail",
        )],
    )])
    .id("wl");
    let short =
        SortableList::new(vec![Lane::new("l", "", vec![Item::new("a", "Report")])]).id("ws");
    let h = |ui, id: &str| {
        let t = layout(ui, 220.0, 400.0);
        t.get_absolute_bounds(t.query_by_id(id).unwrap())
            .unwrap()
            .height
    };
    assert!(h(long.build(), "wl-l-a") > h(short.build(), "ws-l-a") + 15.0);
}

#[test]
fn every_default_key_does_its_job_through_the_element() {
    let _g = serial();
    let list = board("ek").keys(Bindings::default());
    let mut tree = layout(list.build(), 600.0, 260.0);
    let mut input = Input::new();
    input.focus(&tree, "ek");
    let cursor = |l: &SortableList| l.cursor_key().unwrap_or_default();
    input.key(&mut tree, 40); // Down
    assert_eq!(cursor(&list), "b");
    input.key(&mut tree, 35); // End
    assert_eq!(cursor(&list), "c");
    input.key(&mut tree, 36); // Home
    assert_eq!(cursor(&list), "a");
    input.key(&mut tree, 39); // Right: next lane
    assert_eq!(cursor(&list), "x");
    input.key(&mut tree, 37);
    input.key(&mut tree, 32); // pick up a
    assert_eq!(list.grabbed().as_deref(), Some("a"));
    input.key(&mut tree, 40);
    input.key(&mut tree, 27); // Escape: back
    assert_eq!(list.grabbed(), None);
    assert_eq!(list.lanes()[0].items[0].key, "a");
    input.key(&mut tree, 13); // Enter picks up too
    assert_eq!(list.grabbed().as_deref(), Some("a"));
    input.key(&mut tree, 13);
    assert_eq!(list.grabbed(), None);
    input.key(&mut tree, 65); // an unbound key does nothing
    assert_eq!(list.lanes()[0].items.len(), 3);
}
