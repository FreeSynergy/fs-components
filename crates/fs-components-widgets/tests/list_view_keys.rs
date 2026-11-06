//! `list-view`: the keys start from the selected row and a refusal moves nothing; a long text is
//! cut with an ellipsis; rebuilding does not grow the reactive graph.
//!
//! Pattern: input tests over the router a window uses. No window is opened.

mod common;

use std::sync::{Arc, Mutex};

use blinc_core::owner::BuildScope;
use blinc_core::reactive::global_graph;
use blinc_layout::event_router::EventRouter;
use common::{Input, layout, serial};
use fs_components_widgets::list_view::{OnSelect, fit_cell};
use fs_components_widgets::{Column, ColumnWidth, ListPosition, ListView, Row, VecRows};

const DOWN: u32 = 0x28;

fn rows(count: usize) -> VecRows {
    VecRows::new(
        (0..count)
            .map(|i| Row::new(format!("k{i}"), vec![format!("Row {i}")]))
            .collect(),
    )
}

fn list(source: &VecRows, position: &ListPosition, selected: &str, sink: OnSelect) -> ListView {
    ListView::new(
        vec![Column::new("Name", ColumnWidth::Share(1.0))],
        Arc::new(source.clone()),
    )
    .named("t")
    .position(position)
    .selected(Some(selected))
    .on_select(sink)
    .keys(true)
}

fn recording() -> (Arc<Mutex<Vec<String>>>, OnSelect) {
    let chosen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&chosen);
    (
        chosen,
        Arc::new(move |key: &str| sink.lock().unwrap().push(key.to_string())),
    )
}

#[test]
fn the_keys_start_from_the_selected_row() {
    let _one = serial();
    let (chosen, sink) = recording();
    let (source, position) = (rows(1_000), ListPosition::new());
    let mut tree = layout(
        list(&source, &position, "k500", sink).build(400.0, 300.0),
        400.0,
        300.0,
    );
    let mut input = Input::new();
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(*chosen.lock().unwrap(), ["k501"]);
}

#[test]
fn a_caller_that_refuses_keeps_the_row_and_the_scroll_position() {
    let _one = serial();
    let (chosen, sink) = recording();
    let (source, position) = (rows(1_000), ListPosition::new());
    let mut tree = layout(
        list(&source, &position, "k8", sink).build(400.0, 300.0),
        400.0,
        300.0,
    );
    let mut input = Input::new();
    input.focus(&tree, "t");
    for _ in 0..3 {
        input.key_broadcast(&mut tree, DOWN);
    }
    assert_eq!(
        *chosen.lock().unwrap(),
        ["k9", "k9", "k9"],
        "it does not walk on"
    );
    assert!(position.offset().abs() < 0.5, "and the list has not moved");
}

#[test]
fn a_caller_that_accepts_brings_the_row_into_view_and_the_next_key_goes_on_from_there() {
    let _one = serial();
    let (chosen, sink) = recording();
    let (source, position) = (rows(1_000), ListPosition::new());
    let mut tree = layout(
        list(&source, &position, "k8", Arc::clone(&sink)).build(400.0, 300.0),
        400.0,
        300.0,
    );
    let mut input = Input::new();
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    // The caller takes k9 and builds again; the keys now take k10, out of view.
    let again = list(&source, &position, "k9", Arc::clone(&sink));
    tree = layout(again.build(400.0, 300.0), 400.0, 300.0);
    input.focus(&tree, "t");
    assert!(position.offset().abs() < 40.0 || position.offset() > 0.0);
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(*chosen.lock().unwrap(), ["k9", "k10"]);
    let again = list(&source, &position, "k10", sink);
    let _tree = layout(again.build(400.0, 300.0), 400.0, 300.0);
    assert!(
        position.offset() > 0.0,
        "k10 is below the first view, so the list moved"
    );
}

#[test]
fn data_that_changed_is_followed_by_the_selected_key() {
    let _one = serial();
    let (chosen, sink) = recording();
    let position = ListPosition::new();
    let first = rows(100);
    let mut tree = layout(
        list(&first, &position, "k5", Arc::clone(&sink)).build(400.0, 300.0),
        400.0,
        300.0,
    );
    let mut input = Input::new();
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    // Rows were added above: k6 is now further down.
    let mut shifted: Vec<Row> = (0..10)
        .map(|i| Row::new(format!("new{i}"), vec![]))
        .collect();
    shifted.extend((0..100).map(|i| Row::new(format!("k{i}"), vec![])));
    let source = VecRows::new(shifted);
    let mut tree = layout(
        list(&source, &position, "k6", sink).build(400.0, 300.0),
        400.0,
        300.0,
    );
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(*chosen.lock().unwrap(), ["k6", "k7"]);
}

#[test]
fn a_long_text_is_cut_with_an_ellipsis_and_a_short_one_is_not() {
    let _one = serial();
    blinc_layout::init_text_measurer();
    assert_eq!(fit_cell("short", 300.0), "short");
    let long = "a.very.long.address.that.goes.on.and.on@example.org";
    let cut = fit_cell(long, 150.0);
    assert!(cut.ends_with('\u{2026}'), "{cut}");
    assert!(cut.chars().count() < long.chars().count());
    assert_eq!(fit_cell(long, 0.0), "\u{2026}", "no room, no panic");
}

#[test]
fn rebuilding_with_a_position_leaves_no_nodes_of_ours_in_the_reactive_graph() {
    let _one = serial();
    let (source, position) = (rows(1_000), ListPosition::new());
    let (_, sink) = recording();
    let build = |sink: &OnSelect| {
        let ui = list(&source, &position, "k1", Arc::clone(sink)).build(400.0, 300.0);
        layout(ui, 400.0, 300.0)
    };
    let tree = build(&sink);
    let before = global_graph().lock().unwrap().stats();
    let mut last = tree;
    for _ in 0..200 {
        last = build(&sink);
    }
    let after = global_graph().lock().unwrap().stats();
    eprintln!("graph before {before:?}\ngraph after 200 rebuilds {after:?}");
    assert!(
        after.derived_count <= before.derived_count + 5,
        "{before:?} -> {after:?}"
    );
    drop(last);
}

#[test]
fn rebuilding_without_a_position_leaves_no_nodes_either() {
    let _one = serial();
    let source = rows(100);
    let build = || {
        let columns = vec![Column::new("Name", ColumnWidth::Share(1.0))];
        let ui = ListView::new(columns, Arc::new(source.clone()));
        layout(ui.build(400.0, 300.0), 400.0, 300.0)
    };
    drop(build());
    let before = global_graph().lock().unwrap().stats();
    for _ in 0..200 {
        drop(build());
    }
    let after = global_graph().lock().unwrap().stats();
    assert!(
        after.derived_count <= before.derived_count + 2,
        "{before:?} -> {after:?}"
    );
    assert!(
        after.signal_count <= before.signal_count + 2,
        "{before:?} -> {after:?}"
    );
}

/// A window owns what a build creates and gives it back when the next build replaces it. The list
/// must still follow the wheel after that.
#[test]
fn the_window_follows_the_wheel_after_builds_in_a_window_scope_are_replaced() {
    let _one = serial();
    let (source, position) = (rows(1_000), ListPosition::new());
    let (_, sink) = recording();
    let mut scope = BuildScope::new();
    let mut tree = None;
    for _ in 0..3 {
        let pass = scope.begin();
        let ui = list(&source, &position, "k1", Arc::clone(&sink)).build(400.0, 300.0);
        tree = Some(layout(ui, 400.0, 300.0));
        scope.finish(pass);
    }
    let mut tree = tree.unwrap();
    assert!(tree.query_by_id("t-k0").is_some());
    let mut router = EventRouter::new();
    router.on_mouse_move(&tree, 200.0, 200.0);
    let hit = router
        .hit_test(&tree, 200.0, 200.0)
        .expect("something is there");
    tree.dispatch_scroll_chain(hit.node, &hit.ancestors, 200.0, 200.0, 0.0, -3_200.0);
    tree.tick_scroll_physics(1_000);
    let updates = tree.take_pending_property_updates();
    tree.apply_partial_property_updates(updates);
    tree.compute_layout(400.0, 300.0);
    assert!(
        tree.query_by_id("t-k100").is_some(),
        "the row now in view exists"
    );
    assert!(
        tree.query_by_id("t-k0").is_none(),
        "the row that left is gone"
    );
}
