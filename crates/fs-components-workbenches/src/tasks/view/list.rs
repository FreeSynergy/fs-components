//! The list of tasks and what follows it: Show more, a failure, or nothing found.
//!
//! Pattern: Adapter — the rows of the state as a [`RowSource`](fs_components_widgets::RowSource) for
//! `list-view`. The rows are a shared copy, so the list never reaches back into the locked state
//! while it scrolls. Everything here is drawn in a live area, so it holds only plain elements.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{Column, ColumnWidth, ListPosition, ListView, Row, VecRows};

use super::actions::TaskActions;
use crate::tasks::context::TaskView;
use crate::tasks::labels::TaskLabels;
use crate::tasks::state::TasksState;

/// Height of a row of the list in pixels.
const ROW_HEIGHT: f32 = 36.0;
/// Height of the bar with Show more, in pixels.
const MORE_HEIGHT: f32 = 52.0;
/// Height of the line saying a further page failed, in pixels.
const PROBLEM_HEIGHT: f32 = 64.0;

fn empty_text(state: &TasksState) -> String {
    let labels = state.labels();
    if state.term().is_empty() {
        labels.nothing.clone()
    } else {
        TaskLabels::fill(&labels.nothing_found, state.term())
    }
}

/// How much the lines below the rows take.
pub(super) fn footer_height(state: &TasksState) -> f32 {
    let problem = if state.list_problem().is_some() {
        PROBLEM_HEIGHT
    } else {
        0.0
    };
    problem + if state.has_more() { MORE_HEIGHT } else { 0.0 }
}

/// The lines below the rows. Show more is a plain element and a tab stop: the window turns Enter and
/// Space on it into this click.
pub(super) fn footer(state: &TasksState, name: &str, actions: &TaskActions, width: f32) -> Div {
    let mut area = div().flex_col().flex_shrink_0().w(width);
    if let Some(problem) = state.list_problem() {
        area = area.child(
            div()
                .class("fs-tasks-failed")
                .w(width)
                .h(PROBLEM_HEIGHT)
                .overflow_clip()
                .child(text(problem)),
        );
    }
    if state.has_more() {
        let more = Arc::clone(&actions.more);
        area = area.child(
            div()
                .class("fs-tasks-more")
                .flex_row()
                .h(MORE_HEIGHT)
                .w(width)
                .child(
                    div()
                        .id(format!("{name}-more"))
                        .class("fs-tasks-more-button")
                        .attr("tabindex", "0")
                        .on_click(move |_| more())
                        .child(text(&state.labels().more).no_cursor()),
                ),
        );
    }
    area
}

/// In the status view only what follows the columns; in the list view the list too.
pub(in crate::tasks) fn list(
    state: &TasksState,
    name: &str,
    position: &ListPosition,
    actions: &TaskActions,
    width: f32,
    height: f32,
) -> Div {
    if state.view() == TaskView::Status {
        let mut area = div().flex_col().w(width);
        if state.rows().is_empty() && state.list_problem().is_none() {
            area = area.child(div().class("fs-tasks-empty").child(text(empty_text(state))));
        }
        return area.child(footer(state, name, actions, width));
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
    let columns = state
        .labels()
        .columns
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let share = if i == 0 { 3.0 } else { 1.0 };
            Column::new(title.clone(), ColumnWidth::Share(share))
        })
        .collect();
    div()
        .flex_col()
        .w(width)
        .h(height)
        .min_h(0.0)
        .child(
            ListView::new(columns, Arc::new(VecRows::new(rows)))
                .row_height(ROW_HEIGHT)
                .named(name)
                .position(position)
                .selected(state.selected())
                .when_empty(empty_text(state))
                .on_select(Arc::clone(&actions.select))
                .keys(true)
                .build(width, (height - footer_height(state)).max(0.0)),
        )
        .child(footer(state, name, actions, width))
}
