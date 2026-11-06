//! The lines about what could not be shown, above the calendar.
//!
//! Pattern: Adapter — one line per problem, and a click opens it, so that it can be read or deleted.
//! Beyond a few they are counted, so the calendar keeps its room.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::Picked;
use fs_components_widgets::list_view::fit_cell;

use super::actions::CalendarActions;
use super::detail::with_marks;
use super::frame::Places;
use crate::calendar::model::ItemProblem;
use crate::calendar::state::CalendarState;

/// Height of a line.
const PROBLEM_HEIGHT: f32 = 28.0;
/// The most lines.
const MOST_PROBLEMS: usize = 3;
/// What a mark takes of a line, about.
const MARK_ROOM: f32 = 90.0;

/// `holder` with the lines added, and the height they take.
#[allow(clippy::cast_precision_loss)]
pub(super) fn with_problems(
    mut holder: Div,
    state: &CalendarState,
    places: &Places,
    actions: &CalendarActions,
    width: f32,
) -> (Div, f32) {
    let (shown, left) = problems_shown(state.problems().len());
    for (index, problem) in state.problems().iter().take(shown).enumerate() {
        holder = holder.child(problem_row(problem, index, places, actions, width));
    }
    if left > 0 {
        let count = state
            .labels()
            .names
            .more
            .replacen("{}", &left.to_string(), 1);
        holder = holder.child(
            div()
                .class("fs-calendar-problem")
                .w(width)
                .h(PROBLEM_HEIGHT)
                .child(text(count).no_wrap().no_cursor()),
        );
    }
    let rows = shown + usize::from(left > 0);
    (holder, rows as f32 * PROBLEM_HEIGHT)
}

/// How many problems get a line, and how many are only counted.
fn problems_shown(count: usize) -> (usize, usize) {
    if count <= MOST_PROBLEMS {
        (count, 0)
    } else {
        (MOST_PROBLEMS - 1, count - (MOST_PROBLEMS - 1))
    }
}

/// One thing that could not be shown; a click opens it, so that it can be read or deleted.
#[allow(clippy::cast_precision_loss)]
fn problem_row(
    problem: &ItemProblem,
    index: usize,
    places: &Places,
    actions: &CalendarActions,
    width: f32,
) -> Div {
    let line = div()
        .id(format!("{}-problem-{index}", places.name))
        .class("fs-calendar-problem")
        .flex_row()
        .items_center()
        .overflow_clip()
        .w(width)
        .h(PROBLEM_HEIGHT);
    let room = width - MARK_ROOM * problem.marks.len() as f32;
    let line = with_marks(line, &problem.marks)
        .child(text(fit_cell(&problem.text, room)).no_wrap().no_cursor());
    match problem.key.clone() {
        Some(key) => {
            let pick = Arc::clone(&actions.pick);
            line.class("fs-calendar-problem-open")
                .on_click(move |_| pick(Picked::Item(key.clone())))
        }
        None => line,
    }
}
