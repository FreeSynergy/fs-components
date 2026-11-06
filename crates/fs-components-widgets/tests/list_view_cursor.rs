//! `list-view`: where the keys start when data or the selection changed, and refused clicks.
//!
//! Pattern: input tests over the router a window uses. No window is opened.

mod common;

use std::sync::{Arc, Mutex};

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

fn unselected(source: &VecRows, position: &ListPosition) -> ListView {
    ListView::new(
        vec![Column::new("Name", ColumnWidth::Share(1.0))],
        Arc::new(source.clone()),
    )
    .named("t")
    .position(position)
    .keys(true)
}

#[test]
fn without_a_selection_the_keys_follow_their_row_through_changed_data() {
    let _one = serial();
    let (chosen, sink) = recording();
    let position = ListPosition::new();
    let first = rows(100);
    let ui = unselected(&first, &position).on_select(Arc::clone(&sink));
    let mut tree = layout(ui.build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    input.key_broadcast(&mut tree, DOWN);
    let mut shifted: Vec<Row> = (0..10)
        .map(|i| Row::new(format!("new{i}"), vec![]))
        .collect();
    shifted.extend((0..100).map(|i| Row::new(format!("k{i}"), vec![])));
    let source = VecRows::new(shifted);
    let ui = unselected(&source, &position).on_select(sink);
    let mut tree = layout(ui.build(400.0, 300.0), 400.0, 300.0);
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(*chosen.lock().unwrap(), ["k0", "k1", "k2"]);
}

#[test]
fn a_selected_row_that_is_hidden_gives_the_keys_a_fresh_start() {
    let _one = serial();
    let (chosen, sink) = recording();
    let (source, position) = (rows(100), ListPosition::new());
    position.set_cursor(Some(50));
    let ui = list(&source, &position, "not-in-this-list", sink);
    let mut tree = layout(ui.build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(*chosen.lock().unwrap(), ["k0"], "not from the old cursor");
}

#[test]
fn a_refused_click_does_not_move_the_cursor() {
    let _one = serial();
    let (_, sink) = recording();
    let (source, position) = (rows(100), ListPosition::new());
    let ui = list(&source, &position, "k1", sink);
    let mut tree = layout(ui.build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.click(&mut tree, "t-k5");
    assert_eq!(position.cursor(), None, "the caller has not taken it");
    let (_, sink) = recording();
    let ui = list(&source, &position, "k5", sink);
    let _tree = layout(ui.build(400.0, 300.0), 400.0, 300.0);
    assert_eq!(position.cursor(), Some(5), "now it has");
}

#[test]
fn a_selection_without_on_select_still_lets_the_keys_move_the_list_from_that_row() {
    let _one = serial();
    let (source, position) = (rows(1_000), ListPosition::new());
    let ui = unselected(&source, &position).selected(Some("k8"));
    let mut tree = layout(ui.build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.focus(&tree, "t");
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(position.cursor(), Some(9));
    input.key_broadcast(&mut tree, DOWN);
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(position.cursor(), Some(11));
    assert!(position.offset() > 0.0, "the row is kept in view");
}

#[test]
fn an_empty_cell_stays_empty_however_narrow() {
    let _one = serial();
    blinc_layout::init_text_measurer();
    assert_eq!(fit_cell("", 20.0), "");
}
