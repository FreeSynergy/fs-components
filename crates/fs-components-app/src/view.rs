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
use crate::surface::Surface;

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
    let body = body_height(height);
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
                .child(text(title).no_wrap().class("fs-app-title")),
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

/// The surface, told its real size, never a negative one. Runs the surface: call it without the lock.
pub(crate) fn main_area(
    surface: Option<&Arc<dyn Surface>>,
    with_nav: bool,
    width: f32,
    height: f32,
) -> Div {
    let body = body_height(height);
    let w = if with_nav {
        (width - NAV_WIDTH).max(0.0)
    } else {
        width
    };
    let area = div().class("fs-app-main").w(w).h(body).min_h(0.0);
    match surface {
        Some(surface) => area.child(surface.build(w, body)),
        None => area.child(div().class("fs-app-empty")),
    }
}

/// The status line; empty but present when there is nothing to say. The kind of message sits on an
/// inner element: a live area's own node keeps the classes of its first drawing (`DESIGN.md`).
pub(crate) fn status_line(shell: &Shell, width: f32) -> Div {
    let inner = div().class("fs-app-status").h_full().w_full();
    let inner = match shell.status() {
        Some(Status::Info(message)) => inner.child(text(message.clone()).no_wrap()),
        Some(Status::Problem(message)) => inner
            .class("fs-app-status-problem")
            .child(text(message.clone()).no_wrap()),
        None => inner,
    };
    div()
        .class("fs-app-status-line")
        .w(width)
        .h(STATUS_HEIGHT)
        .child(inner)
}

fn body_height(height: f32) -> f32 {
    (height - BAR_HEIGHT - STATUS_HEIGHT).max(0.0)
}
