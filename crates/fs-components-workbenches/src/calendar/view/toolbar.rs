//! The bar above the calendar: where to go, what to see, what to make.
//!
//! Pattern: Template — what the state allows is offered. The compact variant has no view buttons, and
//! only what a source can do is there.

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::ViewRange;

use super::actions::{Act, CalendarActions};
use crate::calendar::shown::Variant;
use crate::calendar::state::CalendarState;

/// A button with an element id on a wrapper, so a program or a test can find it.
pub(super) fn button(id: String, label: &str, variant: ButtonVariant, act: &Act) -> Div {
    let act = std::sync::Arc::clone(act);
    div().id(id).w_fit().flex_shrink_0().child(
        cn::button(label.to_string())
            .variant(variant)
            .on_click(move |_| act()),
    )
}

/// The title of what is shown.
pub(in crate::calendar) fn range_title(state: &CalendarState) -> String {
    let labels = state.labels();
    let names = &labels.names;
    let (first, last) = state.span();
    match state.view() {
        ViewRange::Month => labels
            .month_title
            .replace("{month}", names.month_of(state.anchor()))
            .replace("{year}", &state.anchor().year().to_string()),
        ViewRange::Week | ViewRange::Agenda => {
            format!(
                "{} \u{2013} {}",
                names.date_text(first),
                names.date_text(last)
            )
        }
        _ => names.date_text(state.anchor()),
    }
}

/// How it is placed: on one line, or the view buttons below where there is no room.
pub(super) fn toolbar(
    state: &CalendarState,
    name: &str,
    actions: &CalendarActions,
    wide: bool,
) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let nav = div()
        .class("fs-calendar-bar")
        .flex_row()
        .items_center()
        .child(button(
            id("today"),
            &labels.today,
            ButtonVariant::Secondary,
            &actions.today,
        ))
        .child(button(
            id("previous"),
            &labels.previous,
            ButtonVariant::Ghost,
            &actions.previous,
        ))
        .child(button(
            id("next"),
            &labels.next,
            ButtonVariant::Ghost,
            &actions.next,
        ))
        .child(
            div()
                .id(id("title"))
                .class("fs-calendar-range")
                .flex_1()
                .overflow_clip()
                .child(text(range_title(state)).no_wrap()),
        );
    let mut views = div().class("fs-calendar-bar").flex_row().items_center();
    if state.variant() == Variant::Full {
        for (range, what, label, act) in [
            (
                ViewRange::Month,
                "view-month",
                &labels.month,
                &actions.month,
            ),
            (ViewRange::Week, "view-week", &labels.week, &actions.week),
            (ViewRange::Day, "view-day", &labels.day, &actions.day),
            (
                ViewRange::Agenda,
                "view-agenda",
                &labels.agenda,
                &actions.agenda,
            ),
        ] {
            let look = if state.view() == range {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Ghost
            };
            views = views.child(button(id(what), label, look, act));
        }
    }
    if state.can_create() {
        views = views.child(button(
            id("create"),
            &labels.create,
            ButtonVariant::Primary,
            &actions.create,
        ));
    }
    if wide {
        nav.child(views)
    } else {
        div().flex_col().child(nav).child(views)
    }
}
