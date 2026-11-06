//! The list of people.
//!
//! Pattern: Adapter — the rows of the state as a [`RowSource`] for `list-view`. The rows are a
//! shared copy, so the list never reaches back into the locked state while it scrolls.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{Column, ColumnWidth, ListPosition, ListView, Row, VecRows};

use super::actions::PeopleActions;
use crate::people::labels::PeopleLabels;
use crate::people::state::PeopleState;

/// Height of a row of the list in pixels.
const ROW_HEIGHT: f32 = 36.0;

/// The list, or why there is none.
pub(in crate::people) fn list(
    state: &PeopleState,
    name: &str,
    position: &ListPosition,
    actions: &PeopleActions,
    width: f32,
    height: f32,
) -> Div {
    let labels = state.labels();
    let area = div().flex_col().w(width).h(height).min_h(0.0);
    if let Some(problem) = state.list_problem() {
        return area.child(div().class("fs-people-failed").child(text(problem)));
    }
    let rows: Vec<Row> = state
        .rows()
        .iter()
        .map(|r| Row::new(r.key.clone(), r.cells.clone()))
        .collect();
    let empty = if state.term().is_empty() {
        labels.nobody.clone()
    } else {
        PeopleLabels::fill(&labels.nobody_found, state.term())
    };
    let columns = labels
        .columns
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let share = if i == 0 { 2.0 } else { 1.0 };
            Column::new(title.clone(), ColumnWidth::Share(share))
        })
        .collect();
    let select = Arc::clone(&actions.select);
    area.child(
        ListView::new(columns, Arc::new(VecRows::new(rows)))
            .row_height(ROW_HEIGHT)
            .named(name)
            .position(position)
            .selected(state.selected())
            .when_empty(empty)
            .on_select(select)
            .build(width, height),
    )
}
