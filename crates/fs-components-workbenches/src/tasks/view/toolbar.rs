//! Search, the two views, and New.
//!
//! Pattern: none — the view pressed is the primary button; what the source or the
//! variant cannot do is not offered. The compact variant has only the search.

use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

use super::actions::{TaskActions, button};
use crate::tasks::context::TaskView;
use crate::tasks::shown::Variant;
use crate::tasks::state::TasksState;

/// The toolbar, with `field` as its search field.
pub(in crate::tasks) fn toolbar(
    state: &TasksState,
    name: &str,
    actions: &TaskActions,
    field: impl ElementBuilder + 'static,
) -> Div {
    let labels = state.labels();
    let id = |what: &str| format!("{name}-{what}");
    let mut bar = div()
        .class("fs-tasks-bar-row")
        .flex_row()
        .flex_wrap()
        .w_full()
        .child(
            div()
                .id(id("search"))
                .class("fs-tasks-search-field")
                .child(field),
        );
    if state.variant() == Variant::Full {
        let variant = |view: TaskView| {
            if state.view() == view {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Secondary
            }
        };
        bar = bar
            .child(button(
                id("view-list"),
                &labels.view_list,
                variant(TaskView::List),
                &actions.view_list,
            ))
            .child(button(
                id("view-status"),
                &labels.view_status,
                variant(TaskView::Status),
                &actions.view_status,
            ));
    }
    if state.can_create() {
        bar = bar.child(button(
            id("create"),
            &labels.create,
            ButtonVariant::Primary,
            &actions.create,
        ));
    }
    bar
}
