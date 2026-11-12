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
use fs_components_widgets::list_view::fit_cell;

use crate::shell::{Shell, Status};
use crate::status_panel::STATUS_ID;
use crate::styles::{BAR_HEIGHT, NAV_WIDTH, STATUS_HEIGHT};
use crate::surface::Surface;

/// The padding of the status text together, in pixels (`.fs-app-status`), with some room to spare.
const STATUS_PADDING: f32 = 32.0;

/// What a click on a navigation entry does.
pub(crate) type OnSelect = Arc<dyn Fn(&str) + Send + Sync>;

/// A navigation only makes sense with something to choose from.
pub(crate) fn has_nav(shell: &Shell) -> bool {
    shell.screens().len() > 1
}

/// The title bar: the title, and the menus if the program has any. A long title gives way to them.
pub(crate) fn title_bar(title: &str, menus: Option<Div>) -> Div {
    let mut bar = div().class("fs-app-bar").h(BAR_HEIGHT).child(
        div()
            .class("fs-app-title-box")
            .child(text(title).no_wrap().class("fs-app-title")),
    );
    if let Some(menus) = menus {
        bar = bar.child(menus);
    }
    bar
}

/// The heights of what stands above and below the main area: the bar (the shell's title bar or the surface's own) and the
/// status line (0 when the surface has a footer of its own).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Chrome {
    pub(crate) bar: f32,
    pub(crate) status: f32,
}

impl Default for Chrome {
    fn default() -> Self {
        Self {
            bar: BAR_HEIGHT,
            status: STATUS_HEIGHT,
        }
    }
}

/// Bar above, navigation (if given) left of the main area, status line below (if given).
pub(crate) fn frame(
    bar: Div,
    nav: Option<impl ElementBuilder + 'static>,
    main: impl ElementBuilder + 'static,
    status: Option<impl ElementBuilder + 'static>,
    (width, height): (f32, f32),
    chrome: Chrome,
) -> Div {
    let body = body_height(height, chrome);
    let mut row = div().class("fs-app-row").w(width).h(body).min_h(0.0);
    if let Some(nav) = nav {
        row = row.child(nav);
    }
    let root = div()
        .class("fs-app-root")
        .w(width)
        .h(height)
        .child(bar)
        .child(row.child(main));
    match status {
        Some(status) => root.child(status),
        None => root,
    }
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
    (width, height): (f32, f32),
    chrome: Chrome,
) -> Div {
    let body = body_height(height, chrome);
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

/// What a press on a cut status line does: the left and the top of the line.
pub(crate) type OnStatus = Arc<dyn Fn(f32, f32) + Send + Sync>;

/// The status line; empty but present when there is nothing to say. The kind of message sits on an
/// inner element: a live area's own node keeps the classes of its first drawing (`DESIGN.md`). A text
/// that does not fit ends in an ellipsis, and a press or Enter shows it whole. The press handler is always
/// there: a child without one could keep the handler of the one it replaces (`DESIGN.md`).
pub(crate) fn status_line(status: Option<&Status>, width: f32, on_status: &OnStatus) -> Div {
    let room = (width - STATUS_PADDING).max(0.0);
    let (message, problem) = match status {
        Some(Status::Info(message)) => (message.as_str(), false),
        Some(Status::Problem(message)) => (message.as_str(), true),
        None => ("", false),
    };
    let fitted = fit_cell(message, room);
    let cut = fitted != message;
    let on_status = Arc::clone(on_status);
    let mut inner = div()
        .id(STATUS_ID)
        .class("fs-app-status")
        .h_full()
        .w_full()
        .on_click(move |ctx| {
            if cut {
                on_status(ctx.bounds_x, ctx.bounds_y);
            }
        });
    if problem {
        inner = inner.class("fs-app-status-problem");
    }
    if cut {
        inner = inner.class("fs-app-status-more").attr("tabindex", "0");
    }
    if !fitted.is_empty() {
        inner = inner.child(text(fitted).no_wrap().no_cursor());
    }
    div()
        .class("fs-app-status-line")
        .w(width)
        .h(STATUS_HEIGHT)
        .child(inner)
}

fn body_height(height: f32, chrome: Chrome) -> f32 {
    (height - chrome.bar - chrome.status).max(0.0)
}
