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

/// Height of the bar below the list, when there are more rows, in pixels.
const MORE_HEIGHT: f32 = 52.0;

/// The list and the More button, or why there is none. The button is a plain element, not a `cn`
/// one, because this is drawn in a live area (see the Blinc notes in `DESIGN.md`).
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
    let height = if state.has_more() {
        (height - MORE_HEIGHT).max(0.0)
    } else {
        height
    };
    if let Some(problem) = state.list_problem() {
        return area.child(div().class("fs-people-failed").child(text(problem)));
    }
    let rows: Vec<Row> = state
        .rows()
        .iter()
        .map(|r| {
            let row = Row::new(r.key.clone(), r.cells.clone());
            if r.marks.is_empty() {
                row
            } else {
                row.with_mark(r.marks.join(" "))
            }
        })
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
    let area = area.child(
        ListView::new(columns, Arc::new(VecRows::new(rows)))
            .row_height(ROW_HEIGHT)
            .named(name)
            .position(position)
            .selected(state.selected())
            .when_empty(empty)
            .on_select(select)
            .build(width, height),
    );
    if !state.has_more() {
        return area;
    }
    let more = Arc::clone(&actions.more);
    area.child(
        div()
            .class("fs-people-more")
            .flex_row()
            .h(MORE_HEIGHT)
            .w(width)
            .child(
                div()
                    .id(format!("{name}-more"))
                    .class("fs-people-more-button")
                    .on_click(move |_| more())
                    .child(text(&labels.more).no_cursor()),
            ),
    )
}
