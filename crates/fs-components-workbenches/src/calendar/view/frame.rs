//! The outer layout: the bar on top, the calendar, and the item beside it or in its place.
//!
//! Pattern: Template — fixed skeleton. Sizes are worked out here because the calendar view must be told
//! them in pixels. Where there is no room for both, an open item takes the calendar's place and has a
//! way back; the calendar view itself draws no menu, no form and no export (ADR 0005).

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::list_view::fit_cell;
use fs_components_widgets::{CalendarPosition, CalendarView, Picked};

use super::actions::CalendarActions;
use super::detail::detail;
use super::form::InputSlots;
use super::page_scroll::PageScroll;
use super::toolbar::toolbar;
use crate::calendar::model::ItemProblem;
use crate::calendar::shown::Shown;
use crate::calendar::state::CalendarState;

/// Height of the bar in pixels on one line.
const BAR_HEIGHT: f32 = 56.0;
/// Below this width the view buttons are on a line of their own.
const WIDE_BAR: f32 = 960.0;
/// Below this width the item takes the calendar's place.
const WIDE_FOR_BOTH: f32 = 860.0;
/// Width of the item beside the calendar.
const DETAIL_WIDTH: f32 = 380.0;
/// Height of a line about something that could not be shown.
const PROBLEM_HEIGHT: f32 = 28.0;
/// The most such lines; beyond that they are counted, so the calendar keeps its room.
const MOST_PROBLEMS: usize = 3;
/// The hour at the top of the axis at first.
const START_HOUR: u16 = 7;

/// What the widget keeps across builds.
pub(in crate::calendar) struct Places {
    /// What element ids begin with.
    pub name: String,
    /// Where the calendar is scrolled to.
    pub position: CalendarPosition,
    /// The text fields of the form.
    pub slots: InputSlots,
    /// Where the page of the detail area is scrolled to.
    pub page: PageScroll,
}

/// Everything, filling `width` x `height`.
pub(in crate::calendar) fn workbench(
    state: &CalendarState,
    places: &Places,
    actions: &CalendarActions,
    width: f32,
    height: f32,
) -> Div {
    let wide_bar = width >= WIDE_BAR;
    let bar_height = if wide_bar {
        BAR_HEIGHT
    } else {
        2.0 * BAR_HEIGHT
    };
    let below = (height - bar_height).max(0.0);
    let bar = div()
        .class("fs-calendar-top")
        .flex_col()
        .w(width)
        .h(bar_height)
        .child(toolbar(state, &places.name, actions, wide_bar));
    let root = div()
        .class("fs-calendar-root")
        .flex_col()
        .w(width)
        .h(height)
        .child(bar);
    let open = state.editing().is_some() || !matches!(state.shown(), Shown::Nothing);
    let body = div().flex_row().w(width).h(below).min_h(0.0);
    let body = if width < WIDE_FOR_BOTH && open {
        body.child(
            div()
                .class("fs-calendar-detail")
                .w(width)
                .h(below)
                .child(detail(
                    state,
                    &places.name,
                    actions,
                    &places.slots,
                    &places.page,
                    true,
                )),
        )
    } else if width < WIDE_FOR_BOTH {
        body.child(calendar(state, places, actions, width, below))
    } else {
        let main = (width - DETAIL_WIDTH).max(0.0);
        body.child(calendar(state, places, actions, main, below))
            .child(
                div()
                    .class("fs-calendar-detail")
                    .w(DETAIL_WIDTH)
                    .h(below)
                    .child(detail(
                        state,
                        &places.name,
                        actions,
                        &places.slots,
                        &places.page,
                        false,
                    )),
            )
    };
    root.child(body)
}

fn calendar(
    state: &CalendarState,
    places: &Places,
    actions: &CalendarActions,
    width: f32,
    height: f32,
) -> Div {
    let holder = div().flex_col().w(width).h(height).min_h(0.0);
    if let Some(problem) = state.problem() {
        return holder.child(div().class("fs-calendar-failed").child(text(problem)));
    }
    let (shown, left) = problems_shown(state.problems().len());
    let mut holder = holder;
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
    #[allow(clippy::cast_precision_loss)]
    let height = (height - rows as f32 * PROBLEM_HEIGHT).max(0.0);
    let view = CalendarView::new(
        state.view(),
        state.anchor(),
        Arc::new(state.labels().names.clone()),
        Arc::clone(state.items()),
    )
    .named(places.name.clone())
    .week_start(state.week_start())
    .now(state.now())
    .selected_day(state.selected_day())
    .selected_item(state.selected().map(str::to_string))
    .position(&places.position)
    .start_hour(START_HOUR)
    .on_pick(Arc::clone(&actions.pick));
    holder.child(view.build(width, height))
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
        .h(PROBLEM_HEIGHT)
        .child(text(fit_cell(&problem.text, width)).no_wrap().no_cursor());
    match problem.key.clone() {
        Some(key) => {
            let pick = Arc::clone(&actions.pick);
            line.class("fs-calendar-problem-open")
                .on_click(move |_| pick(Picked::Item(key.clone())))
        }
        None => line,
    }
}
