//! The outer layout: the bar on top, the calendar, and the item beside it or in its place.
//!
//! Pattern: Template — fixed skeleton. Sizes are worked out here because the calendar view must be told
//! them in pixels. Where there is no room for both, an open item takes the calendar's place and has a
//! way back; the calendar view itself draws no menu, no form and no export (ADR 0005).

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::{CalendarPosition, CalendarView};

use super::actions::CalendarActions;
use super::detail::detail;
use super::page_scroll::PageScroll;
use super::problems::with_problems;
use super::toolbar::toolbar;
use crate::calendar::shown::Shown;
use crate::calendar::state::CalendarState;
use crate::input_slots::InputSlots;

/// Height of the bar in pixels on one line.
const BAR_HEIGHT: f32 = 56.0;
/// Below this width the view buttons are on a line of their own.
const WIDE_BAR: f32 = 960.0;
/// Below this width the item takes the calendar's place.
const WIDE_FOR_BOTH: f32 = 860.0;
/// Width of the item beside the calendar.
const DETAIL_WIDTH: f32 = 380.0;
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
    let (holder, used) = with_problems(holder, state, places, actions, width);
    let height = (height - used).max(0.0);
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
