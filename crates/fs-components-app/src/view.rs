//! The shell's state as elements.
//!
//! Pattern: Adapter (state to elements) + Template — fixed skeleton, the live areas are passed in,
//! so a window can make them react while a picture draws them once. Navigation entries are buttons,
//! which makes them reachable by Tab.

use std::sync::Arc;

use blinc_cn::cn;
use blinc_cn::components::button::ButtonVariant;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use blinc_layout::text;

use crate::shell::{Shell, Status};
use crate::styles::{BAR_HEIGHT, NAV_WIDTH, STATUS_HEIGHT};

/// What a click on a navigation entry does.
pub(crate) type OnSelect = Arc<dyn Fn(&str) + Send + Sync>;

/// A navigation only makes sense with something to choose from.
pub(crate) fn has_nav(shell: &Shell) -> bool {
    shell.screens().len() > 1
}

/// Bar above, navigation (if given) left of the main area, status line below.
pub(crate) fn frame(
    title: &str,
    nav: Option<impl ElementBuilder + 'static>,
    main: impl ElementBuilder + 'static,
    status: impl ElementBuilder + 'static,
    width: f32,
    height: f32,
) -> Div {
    let body = height - BAR_HEIGHT - STATUS_HEIGHT;
    let mut row = div().class("fs-app-row").w(width).h(body).min_h(0.0);
    if let Some(nav) = nav {
        row = row.child(nav);
    }
    div()
        .class("fs-app-root")
        .w(width)
        .h(height)
        .child(
            div()
                .class("fs-app-bar")
                .h(BAR_HEIGHT)
                .child(text(title).class("fs-app-title")),
        )
        .child(row.child(main))
        .child(status)
}

/// One button per screen; the shown one stands out.
pub(crate) fn nav(shell: &Shell, on_select: &OnSelect) -> Div {
    let shown = shell.current().map(|s| s.id.as_str());
    let mut column = div().class("fs-app-nav").w(NAV_WIDTH);
    for screen in shell.screens() {
        let variant = if shown == Some(screen.id.as_str()) {
            ButtonVariant::Secondary
        } else {
            ButtonVariant::Ghost
        };
        let (select, id) = (Arc::clone(on_select), screen.id.clone());
        column = column.child(
            cn::button(screen.label.clone())
                .variant(variant)
                .on_click(move |_| select(&id)),
        );
    }
    column
}

/// The surface of the shown screen, told its real size.
pub(crate) fn main_area(shell: &Shell, width: f32, height: f32) -> Div {
    let body = height - BAR_HEIGHT - STATUS_HEIGHT;
    let w = if has_nav(shell) {
        width - NAV_WIDTH
    } else {
        width
    };
    let area = div().class("fs-app-main").w(w).h(body).min_h(0.0);
    match shell.current() {
        Some(screen) => area.child(screen.surface.build(w, body)),
        None => area.child(div().class("fs-app-empty")),
    }
}

/// The status line; empty but present when there is nothing to say.
pub(crate) fn status_line(shell: &Shell, width: f32) -> Div {
    let line = div().class("fs-app-status").w(width).h(STATUS_HEIGHT);
    match shell.status() {
        Some(Status::Info(message)) => line.child(text(message.clone())),
        Some(Status::Problem(message)) => line
            .class("fs-app-status-problem")
            .child(text(message.clone())),
        None => line,
    }
}
