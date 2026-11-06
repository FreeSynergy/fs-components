//! The whole status text, in a panel above the status line.
//!
//! Pattern: Facade over the overlay stack, the same way as an open menu (`menu_overlay.rs`). A press or
//! Enter on a cut line opens it; a second press, a press outside, Escape or a new message closes it. The
//! lock of the shell is never held while the stack is touched, because the stack runs closures that
//! take that lock. It has no words of its own: it shows what the program said, whole.

use std::sync::{Arc, Mutex};

use blinc_layout::click_outside;
use blinc_layout::div::{Div, div};
use blinc_layout::overlay_state::overlay_stack;
use blinc_layout::text;
use blinc_layout::widgets::overlay_stack::{OverlayBuilder, OverlayHandle};
use blinc_layout::widgets::scroll::scroll;
use blinc_theme::{RadiusToken, ThemeState};

use crate::shell::{Shell, Status};
use crate::styles::{MENU_GAP, STATUS_PANEL_WIDTH, STATUS_TEXT};
use crate::view::OnStatus;
use crate::widget::locked;

/// The element id of the status line's text.
pub(crate) const STATUS_ID: &str = "fs-app-status";
/// The element id of the open panel.
pub(crate) const PANEL_ID: &str = "fs-app-status-panel";
/// The element id of the text in it.
const PANEL_TEXT_ID: &str = "fs-app-status-text";
/// The key of the one registration for a press outside.
const OUTSIDE_KEY: &str = "fs-app-status-outside";
/// Room around the panel in pixels.
const MARGIN: f32 = 8.0;
/// Height of a line of the text in pixels, for keeping the panel inside the window.
const LINE_HEIGHT: f32 = 20.0;
/// Average width of a character at this size in pixels; a floor for the height, the panel grows.
const CHARACTER_WIDTH: f32 = 7.0;

/// Close the panel, if it is open.
pub(crate) fn close(shell: &Mutex<Shell>) {
    let overlay = {
        let mut shell = locked(shell);
        let overlay = shell.status_overlay();
        shell.set_status_overlay(None);
        overlay
    };
    if let Some(raw) = overlay {
        click_outside::unregister_click_outside(OUTSIDE_KEY);
        OverlayHandle::from_raw(raw).close();
    }
}

/// What a press on the cut line does.
pub(crate) fn on_status(shell: &Arc<Mutex<Shell>>, window_width: f32) -> OnStatus {
    let shell = Arc::clone(shell);
    Arc::new(move |x, y| toggle(&shell, x, y, window_width))
}

fn toggle(shell: &Arc<Mutex<Shell>>, x: f32, line_top: f32, window_width: f32) {
    let open = locked(shell).status_overlay().is_some();
    close(shell);
    if open {
        return;
    }
    let message = match locked(shell).status() {
        Some(Status::Info(message) | Status::Problem(message)) => message.clone(),
        None => return,
    };
    let width = STATUS_PANEL_WIDTH.min(window_width - 2.0 * MARGIN).max(1.0);
    let height = height_of(&message, width).min((line_top - 2.0 * MARGIN).max(1.0));
    let raw = show(
        shell,
        message,
        (x + MARGIN, line_top - MENU_GAP - height),
        (width, height),
    );
    locked(shell).set_status_overlay(Some(raw));
    watch_outside(shell);
}

#[allow(clippy::cast_precision_loss)] // text lengths far below 2^23
fn height_of(message: &str, width: f32) -> f32 {
    let per_line = ((width - 2.0 * MARGIN) / CHARACTER_WIDTH).max(1.0);
    let lines = (message.chars().count() as f32 / per_line).ceil().max(1.0);
    lines * LINE_HEIGHT + 2.0 * MARGIN
}

/// Ask for a press outside the line and the panel. Blinc forgets this at every full rebuild, so the
/// build asks again while the panel is open.
pub(crate) fn watch_outside(shell: &Arc<Mutex<Shell>>) {
    let shell = Arc::clone(shell);
    let inside = [STATUS_ID.to_owned(), PANEL_ID.to_owned()];
    click_outside::register_click_outside_multi(OUTSIDE_KEY, &inside, move || close(&shell));
}

fn show(shell: &Arc<Mutex<Shell>>, message: String, (x, y): (f32, f32), (w, h): (f32, f32)) -> u64 {
    let raw = overlay_stack()
        .lock()
        .map_or(0, |stack| stack.peek_next_handle_id());
    let shell = Arc::clone(shell);
    OverlayBuilder::dropdown()
        .at(x, y)
        .size(w, h)
        .dismissable_by_escape(true)
        .on_close(move |_reason| {
            // Escape comes this way. A panel opened since then is not this one.
            let mut shell = locked(&shell);
            if shell.status_overlay() == Some(raw) {
                shell.set_status_overlay(None);
                click_outside::unregister_click_outside(OUTSIDE_KEY);
            }
        })
        .content(move || panel(&message, w, h))
        .show()
        .raw()
}

/// Plain elements and a scroll: this runs under the lock of Blinc's overlay stack. Text beyond the
/// height there is above the line scrolls.
fn panel(message: &str, width: f32, height: f32) -> Div {
    let radius = ThemeState::get().radius(RadiusToken::Md);
    div()
        .id(PANEL_ID)
        .class("cn-menu")
        .class("cn-menubar-content")
        .rounded(radius)
        .overflow_clip()
        .w(width)
        .h(height)
        .child(
            scroll().w_full().h_full().content(
                div()
                    .id(PANEL_TEXT_ID)
                    .w_full()
                    .p(2.0)
                    .child(text(message).size(STATUS_TEXT).no_cursor()),
            ),
        )
}
