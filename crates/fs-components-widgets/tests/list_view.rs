//! `list-view`: the arithmetic, and the real element driven by an in-process mouse and wheel over
//! the router and hit test a window uses. Pictures with many rows are in the tests of
//! `fs-components-workbenches`, which has the renderer as a dependency.
//!
//! Tests: Model tests + input tests. No window is opened.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::event_router::EventRouter;
use blinc_layout::prelude::RenderTree;
use common::{Input, layout, serial};
use fs_components_widgets::Key;
use fs_components_widgets::list_view::{OnSelect, column_widths, key_target, visible_rows};
use fs_components_widgets::{Column, ColumnWidth, ListPosition, ListView, Row, RowSource, VecRows};

struct Numbers(usize);

impl RowSource for Numbers {
    fn count(&self) -> usize {
        self.0
    }
    fn rows(&self, first: usize, limit: usize) -> Result<Vec<Row>, String> {
        Ok((first..(first + limit).min(self.0))
            .map(|i| Row::new(format!("k{i}"), vec![format!("Row {i}")]))
            .collect())
    }
}

fn one_column() -> Vec<Column> {
    vec![Column::new("Name", ColumnWidth::Share(1.0))]
}

fn list_of(source: impl RowSource + 'static) -> ListView {
    ListView::new(one_column(), Arc::new(source)).named("t")
}

fn has_row(tree: &RenderTree, index: usize) -> bool {
    tree.query_by_id(&format!("t-k{index}")).is_some()
}

/// Turn the wheel over the middle of the list, as a window does, and run the frame that follows.
fn wheel(tree: &mut RenderTree, down: f32) {
    let mut router = EventRouter::new();
    router.on_mouse_move(tree, 200.0, 200.0);
    let hit = router
        .hit_test(tree, 200.0, 200.0)
        .expect("something is there");
    tree.dispatch_scroll_chain(hit.node, &hit.ancestors, 200.0, 200.0, 0.0, -down);
    tree.tick_scroll_physics(1_000);
    let updates = tree.take_pending_property_updates();
    tree.apply_partial_property_updates(updates);
    tree.compute_layout(400.0, 300.0);
}

#[test]
fn the_window_covers_the_view_and_stays_inside_the_list() {
    assert_eq!(visible_rows(0.0, 100.0, 10.0, 1000), 0..14);
    assert_eq!(visible_rows(500.0, 100.0, 10.0, 1000), 46..64);
    assert_eq!(visible_rows(9_950.0, 100.0, 10.0, 1000), 991..1000);
    assert_eq!(visible_rows(1e9, 100.0, 10.0, 1000), 996..1000);
    assert_eq!(visible_rows(-5.0, 100.0, 10.0, 3), 0..3);
    assert_eq!(visible_rows(0.0, 100.0, 10.0, 0), 0..0);
    assert_eq!(
        visible_rows(0.0, 100.0, 0.0, 2),
        0..2,
        "a row is never 0 high"
    );
}

#[test]
fn columns_share_what_the_fixed_ones_leave() {
    let columns = [
        Column::new("a", ColumnWidth::Pixels(100.0)),
        Column::new("b", ColumnWidth::Share(1.0)),
        Column::new("c", ColumnWidth::Share(3.0)),
    ];
    assert_eq!(column_widths(&columns, 500.0), [100.0, 100.0, 300.0]);
    assert_eq!(
        column_widths(&columns, 50.0),
        [100.0, 0.0, 0.0],
        "never negative"
    );
    assert_eq!(column_widths(&[], 500.0), [] as [f32; 0]);
}

#[test]
fn keys_lead_to_a_row_inside_the_list() {
    use Key::{Down, End, Enter, Home, PageDown, PageUp, Up};
    assert_eq!(key_target(Enter, Some(1), 5, 2), None);
    assert_eq!(key_target(Down, None, 5, 2), Some(0));
    assert_eq!(key_target(End, None, 5, 2), Some(4));
    assert_eq!(key_target(Down, Some(4), 5, 2), Some(4));
    assert_eq!(key_target(Up, Some(0), 5, 2), Some(0));
    assert_eq!(key_target(PageDown, Some(1), 5, 2), Some(3));
    assert_eq!(key_target(PageUp, Some(1), 5, 2), Some(0));
    assert_eq!(key_target(Home, Some(3), 5, 2), Some(0));
    assert_eq!(
        key_target(Up, Some(99), 5, 2),
        Some(3),
        "a stale row is pulled in"
    );
    assert_eq!(key_target(Down, None, 0, 2), None);
}

#[test]
fn only_the_rows_in_view_exist_and_the_wheel_brings_the_others() {
    let _one = serial();
    let list = list_of(Numbers(100_000));
    let mut tree = layout(list.build(400.0, 300.0), 400.0, 300.0);
    assert!(has_row(&tree, 0) && has_row(&tree, 8));
    assert!(!has_row(&tree, 40), "not in view, so not built");
    assert!(!has_row(&tree, 99_999));

    wheel(&mut tree, 3_200.0);
    assert!(has_row(&tree, 100), "the row now in view exists");
    assert!(!has_row(&tree, 0), "the row that left is gone");

    wheel(&mut tree, 3_200.0);
    assert!(has_row(&tree, 200));
    assert!(!has_row(&tree, 100));
}

#[test]
fn a_click_reports_the_key_of_the_row_under_the_mouse_also_after_scrolling() {
    let _one = serial();
    let chosen = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&chosen);
    let position = ListPosition::new();
    let list = list_of(Numbers(100_000))
        .position(&position)
        .on_select(Arc::new(move |key| sink.lock().unwrap().push(key.into())));
    let mut tree = layout(list.build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.click(&mut tree, "t-k2");
    assert_eq!(position.cursor(), Some(2));

    wheel(&mut tree, 3_200.0);
    input.click(&mut tree, "t-k103");
    assert_eq!(*chosen.lock().unwrap(), ["k2", "k103"]);
    assert_eq!(position.cursor(), Some(103));
}

#[test]
fn a_rebuild_keeps_the_position_and_fewer_rows_pull_it_back() {
    let _one = serial();
    let position = ListPosition::new();
    let many = || list_of(Numbers(1_000)).position(&position);
    let mut tree = layout(many().build(400.0, 300.0), 400.0, 300.0);
    wheel(&mut tree, 3_200.0);
    assert!((position.offset() - 3_200.0).abs() < 1.0);

    let tree = layout(many().build(400.0, 300.0), 400.0, 300.0);
    assert!(has_row(&tree, 100), "the same place after a rebuild");

    let few = list_of(Numbers(20)).position(&position);
    let tree = layout(few.build(400.0, 300.0), 400.0, 300.0);
    assert!(has_row(&tree, 19), "the end of the shorter list");
    assert!(position.offset() <= 20.0 * 32.0);
}

#[test]
fn rows_in_memory_are_a_source_too_and_the_selected_row_has_its_class() {
    let _one = serial();
    let rows = VecRows::new(vec![
        Row::new("a", vec!["A".into()]),
        Row::new("b", vec!["B".into()]),
    ]);
    assert_eq!(rows.rows(1, 5).unwrap().len(), 1);
    let list = list_of(rows).selected(Some("b"));
    let tree = layout(list.build(400.0, 300.0), 400.0, 300.0);
    assert!(tree.query_by_id("t-b").is_some());
}

#[test]
fn keys_are_off_by_default_and_when_on_move_the_choice_and_keep_it_in_view() {
    let _one = serial();
    let chosen = Arc::new(Mutex::new(Vec::<String>::new()));
    let reporting = |chosen: &Arc<Mutex<Vec<String>>>| -> OnSelect {
        let sink = Arc::clone(chosen);
        Arc::new(move |key| sink.lock().unwrap().push(key.into()))
    };
    let mut input = Input::new();

    let silent = list_of(Numbers(1_000)).on_select(reporting(&chosen));
    let mut tree = layout(silent.build(400.0, 300.0), 400.0, 300.0);
    input.key_broadcast(&mut tree, 0x28);
    assert!(chosen.lock().unwrap().is_empty(), "keys are off");

    let position = ListPosition::new();
    let keyed = list_of(Numbers(1_000))
        .on_select(reporting(&chosen))
        .position(&position)
        .keys(true);
    let mut tree = layout(keyed.build(400.0, 300.0), 400.0, 300.0);
    input.key_broadcast(&mut tree, 0x28);
    assert!(chosen.lock().unwrap().is_empty(), "it has not the keyboard");
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, 0x28); // down: the first row
    input.key_broadcast(&mut tree, 0x28); // down: the second
    input.key_broadcast(&mut tree, 0x23); // end
    assert_eq!(*chosen.lock().unwrap(), ["k0", "k1", "k999"]);
    tree.process_pending_scroll_refs();
    tree.tick_scroll_physics(1_000);
    let updates = tree.take_pending_property_updates();
    tree.apply_partial_property_updates(updates);
    tree.compute_layout(400.0, 300.0);
    assert!(has_row(&tree, 999), "the end is in view");
    assert!(!has_row(&tree, 0));
}

#[test]
fn a_list_may_be_told_no_room_at_all() {
    let _one = serial();
    let list = list_of(Numbers(100));
    let tree = layout(list.build(0.0, 0.0), 0.0, 0.0);
    drop(tree);
}
