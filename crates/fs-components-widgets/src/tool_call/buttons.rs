//! The answer buttons of a question and their keys.
//!
//! Pattern: Adapter + State (the guard, `Approvals`). An answer button answers on a release inside
//! it that follows a press that began on it while armed (Blinc routes a release to the pressed
//! element wherever the pointer is, so the position is checked here). A button is no tab stop and
//! takes no key. The keyboard works on the row of options, the one tab stop: Left and Right move a cursor, Enter or Space answers the option
//! under it on the key's release, only while the row has the keyboard focus; a held key (which
//! repeats the press) never answers.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::card::{Ctx, Sinks};
use super::model::{Decision, ToolCall, ToolChoice, ToolOption};
use crate::softwrap::soft_wrap;

/// What the keys of one row of options need: the question it belongs to and where to report.
#[derive(Clone)]
struct Keys {
    approvals: super::Approvals,
    place: String,
    message: String,
    call: String,
    prefix: String,
    options: Vec<ToolOption>,
    drawn: (u64, Vec<u8>),
    choose: super::card::ChooseFn,
    rearm: Arc<dyn Fn(u64) + Send + Sync>,
}

impl Keys {
    /// The option under the cursor and the id of its button.
    fn under_cursor(&self) -> Option<(&ToolOption, String)> {
        let at = self
            .approvals
            .cursor(&self.place, &self.message, &self.call)
            .min(self.options.len().checked_sub(1)?);
        let o = &self.options[at];
        Some((o, format!("{}-opt-{}-{}", self.prefix, self.call, o.id)))
    }

    /// Enter or Space went down (as the click Blinc makes of it): the first press of the key
    /// begins an answer, its repeats do not, also not after a redraw.
    fn key_down(&self) {
        let first = self
            .approvals
            .key_down(&self.place, &self.message, &self.call);
        if !first || self.approvals.has_press() {
            return;
        }
        if let Some((o, button)) = self.under_cursor() {
            let refuse = o.choice == ToolChoice::Refuse;
            self.approvals
                .press(&self.place, &self.message, &self.call, &button, refuse);
        }
    }

    /// Enter or Space went up: the answer, if the press began here and the question is still armed.
    fn key_up(&self, mine: bool) {
        let was_down = self
            .approvals
            .key_up(&self.place, &self.message, &self.call);
        let Some((o, button)) = self.under_cursor().filter(|_| was_down && mine) else {
            return;
        };
        let refuse = o.choice == ToolChoice::Refuse;
        if self
            .approvals
            .release(&self.place, &self.message, &self.call, &button, refuse)
        {
            (self.choose)(Decision {
                message: self.message.clone(),
                call: self.call.clone(),
                option: o.id.clone(),
                choice: o.choice,
                revision: self.drawn.0,
                digest: self.drawn.1.clone(),
            });
        }
    }

    fn walk(&self, delta: i32) {
        self.approvals.move_cursor(
            &self.place,
            &self.message,
            &self.call,
            delta,
            self.options.len(),
        );
        (self.rearm)(0);
    }
}

pub(super) fn options(call: &ToolCall, prefix: &str, sinks: &Sinks, ctx: &Ctx) -> Div {
    use std::sync::atomic::{AtomicBool, Ordering};
    let cursor = ctx
        .approvals
        .cursor(&ctx.place, &ctx.message, &call.id)
        .min(call.options.len().saturating_sub(1));
    let mut row = div()
        .class("fs-tc-options")
        .id(format!("{prefix}-keys-{}", call.id))
        .attr("tabindex", "0");
    for (i, o) in call.options.iter().enumerate() {
        let mut b = answer_button(call, o, prefix, sinks, ctx);
        if i == cursor {
            b = b.class("fs-tc-btn-cursor");
        }
        row = row.child(b);
    }
    // The keyboard works on the row, which is the one tab stop: Left and Right move the cursor
    // over the options, Enter or Space answers the one under it, on the key's release. Only while
    // this row has the keyboard focus; a held key (it repeats the press) never answers.
    let focused = Arc::new(AtomicBool::new(false));
    let (on, off, down, up) = (
        Arc::clone(&focused),
        Arc::clone(&focused),
        Arc::clone(&focused),
        focused,
    );
    let mine = |c: &blinc_layout::event_handler::EventContext, flag: &AtomicBool| {
        let node = blinc_layout::event_router::keyboard_focused_node();
        node.map_or_else(|| flag.load(Ordering::Relaxed), |n| n == c.node_id)
            && !blinc_layout::widgets::text_input::has_focused_text_input()
    };
    let keys = Keys {
        approvals: ctx.approvals.clone(),
        place: ctx.place.clone(),
        message: ctx.message.clone(),
        call: call.id.clone(),
        prefix: prefix.to_owned(),
        options: call.options.clone(),
        drawn: (call.revision, call.digest.clone()),
        choose: Arc::clone(&sinks.choose),
        rearm: Arc::clone(&sinks.rearm),
    };
    let (k_blur, k_click, k_down, k_up) = (keys.clone(), keys.clone(), keys.clone(), keys);
    row.on_focus(move |_| on.store(true, Ordering::Relaxed))
        .on_blur(move |_| {
            off.store(false, Ordering::Relaxed);
            k_blur
                .approvals
                .key_up(&k_blur.place, &k_blur.message, &k_blur.call);
        })
        // Enter and Space on the focused row reach it as a click, not as a key (Blinc's
        // `handle_navigation_key` presses and releases the element). A click whose receiver is the
        // row that has the keyboard is that key; its first one begins an answer (a repeat of a held
        // key is no new press), the answer comes on the key's release.
        .on_mouse_down(move |c| {
            let keyboard = blinc_layout::event_router::keyboard_focused_node() == Some(c.node_id)
                && !blinc_layout::widgets::text_input::has_focused_text_input();
            if keyboard {
                k_click.key_down();
            }
        })
        .on_key_down(move |c| {
            if mine(c, &down) && !k_down.options.is_empty() && matches!(c.key_code, 37 | 39) {
                k_down.walk(if c.key_code == 37 { -1 } else { 1 });
            }
        })
        .on_key_up(move |c| {
            if matches!(c.key_code, 13 | 32) {
                k_up.key_up(mine(c, &up));
            }
        })
}

/// Is the pointer of this event inside the element it was routed to? In doubt: no.
fn inside(ctx: &blinc_layout::event_handler::EventContext) -> bool {
    ctx.bounds_width > 0.0
        && ctx.bounds_height > 0.0
        && ctx.mouse_x >= ctx.bounds_x
        && ctx.mouse_x <= ctx.bounds_x + ctx.bounds_width
        && ctx.mouse_y >= ctx.bounds_y
        && ctx.mouse_y <= ctx.bounds_y + ctx.bounds_height
}

/// An answer button. It answers on a release **inside it** that follows a press that began on it
/// while armed (Blinc routes a release to the pressed element wherever the pointer is, so the
/// position is checked here). It is no tab stop and answers to no key: the row of options does.
fn answer_button(call: &ToolCall, o: &ToolOption, prefix: &str, sinks: &Sinks, ctx: &Ctx) -> Div {
    let refuse = o.choice == ToolChoice::Refuse;
    let armed = ctx
        .approvals
        .armed(&ctx.place, &ctx.message, &call.id, refuse);
    let id = format!("{prefix}-opt-{}-{}", call.id, o.id);
    let class = if refuse {
        "fs-tc-btn-no"
    } else {
        "fs-tc-btn-yes"
    };
    let mut b = div().class("fs-tc-btn").class(class).id(&id);
    if !armed {
        b = b.class("fs-tc-btn-off");
    }
    let decision = Decision {
        message: ctx.message.clone(),
        call: call.id.clone(),
        option: o.id.clone(),
        choice: o.choice,
        revision: call.revision,
        digest: call.digest.clone(),
    };
    let who = (
        ctx.approvals.clone(),
        ctx.place.clone(),
        ctx.message.clone(),
        call.id.clone(),
        id,
    );
    let (w1, w2, w3) = (who.clone(), who.clone(), who);
    let choose = Arc::clone(&sinks.choose);
    let (rearm, dec) = (Arc::clone(&sinks.rearm), decision);
    b.child(text(soft_wrap(&o.label)).class("fs-tc-btn-text"))
        .on_mouse_down(move |c| {
            // A repeat of the key that is held on the row is no press of a button.
            let key = w1.0.is_held(&w1.1, &w1.2, &w1.3);
            if key || !(inside(c) && w1.0.press(&w1.1, &w1.2, &w1.3, &w1.4, refuse)) {
                w1.0.cancel_press();
                rearm(0); // a disarmed button does nothing, visibly: draw it as it is now
            }
        })
        .on_hover_leave(move |_| w2.0.cancel_press())
        .on_mouse_up(move |c| {
            if inside(c) && w3.0.release(&w3.1, &w3.2, &w3.3, &w3.4, refuse) {
                choose(dec.clone());
            } else {
                w3.0.cancel_press();
            }
        })
}
