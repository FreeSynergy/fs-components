//! A key at the edge of a list or a grid leads nowhere: it does not report the row already chosen
//! (which callers read as "open it").
//!
//! Tests: input tests over the router a window uses. No window is opened.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout, serial};
use fs_components_widgets::file_tile::{FileTile, TileGrid};
use fs_components_widgets::list_view::OnSelect;
use fs_components_widgets::{Column, ColumnWidth, ListPosition, ListView, Row, VecRows};

const UP: u32 = 0x26;
const DOWN: u32 = 0x28;
const HOME: u32 = 0x24;
const END: u32 = 0x23;
const LEFT: u32 = 0x25;
const RIGHT: u32 = 0x27;

fn recording() -> (Arc<Mutex<Vec<String>>>, OnSelect) {
    let chosen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&chosen);
    (
        chosen,
        Arc::new(move |key: &str| sink.lock().unwrap().push(key.to_string())),
    )
}

fn list(selected: &str, sink: OnSelect) -> ListView {
    let rows = (0..3)
        .map(|i| Row::new(format!("k{i}"), vec![format!("Row {i}")]))
        .collect();
    ListView::new(
        vec![Column::new("Name", ColumnWidth::Share(1.0))],
        Arc::new(VecRows::new(rows)),
    )
    .named("e")
    .position(&ListPosition::new())
    .selected(Some(selected))
    .on_select(sink)
    .keys(true)
}

#[test]
fn a_list_reports_nothing_for_a_key_that_leads_to_the_row_already_chosen() {
    let _one = serial();
    let (chosen, sink) = recording();
    let mut tree = layout(list("k2", sink).build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.focus(&tree, "e");
    input.key_broadcast(&mut tree, DOWN);
    input.key_broadcast(&mut tree, END);
    assert!(chosen.lock().unwrap().is_empty(), "{:?}", chosen.lock());
    let (chosen, sink) = recording();
    let mut tree = layout(list("k0", sink).build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.focus(&tree, "e");
    input.key_broadcast(&mut tree, UP);
    input.key_broadcast(&mut tree, HOME);
    assert!(chosen.lock().unwrap().is_empty(), "{:?}", chosen.lock());
    input.key_broadcast(&mut tree, DOWN);
    assert_eq!(*chosen.lock().unwrap(), ["k1"]);
}

#[test]
fn a_grid_reports_nothing_for_a_key_that_leads_to_the_tile_already_chosen() {
    let _one = serial();
    let (chosen, sink) = recording();
    let tiles: Vec<FileTile> = (0..3)
        .map(|i| FileTile::new(format!("t{i}"), format!("name {i}"), "txt"))
        .collect();
    let grid = TileGrid::new(Arc::new(tiles), (160.0, 120.0), &ListPosition::new(), "g")
        .selected(Some("t2"))
        .on_select(sink)
        .build(800.0, 400.0);
    let mut tree = layout(grid, 800.0, 400.0);
    let mut input = Input::new();
    input.focus(&tree, "g");
    for key in [RIGHT, DOWN, END] {
        input.key_broadcast(&mut tree, key);
    }
    assert!(chosen.lock().unwrap().is_empty(), "{:?}", chosen.lock());
    input.key_broadcast(&mut tree, LEFT);
    assert_eq!(*chosen.lock().unwrap(), ["t1"]);
}

#[test]
fn walking_without_on_select_reports_the_step_back_onto_the_row_it_began_on() {
    let _one = serial();
    let (walked, walk) = recording();
    let (_, open) = recording();
    let rows = (0..3)
        .map(|i| Row::new(format!("k{i}"), vec![format!("Row {i}")]))
        .collect();
    let list = ListView::new(
        vec![Column::new("Name", ColumnWidth::Share(1.0))],
        Arc::new(VecRows::new(rows)),
    )
    .named("w")
    .position(&ListPosition::new())
    .selected(Some("k0"))
    .walking(walk, open)
    .keys(true);
    let mut tree = layout(list.build(400.0, 300.0), 400.0, 300.0);
    let mut input = Input::new();
    input.focus(&tree, "w");
    for key in [DOWN, DOWN, UP, UP] {
        input.key_broadcast(&mut tree, key);
    }
    assert_eq!(*walked.lock().unwrap(), ["k1", "k2", "k1", "k0"]);
    input.key_broadcast(&mut tree, UP); // the edge: nothing more to report
    assert_eq!(walked.lock().unwrap().len(), 4);
}
