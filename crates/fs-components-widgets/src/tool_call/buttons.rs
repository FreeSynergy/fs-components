//! The answer buttons of a question and their keys.
//!
//! Pattern: Adapter + State (the guard, `Approvals`). An answer button answers on a release inside
//! it that follows a press that began on it while armed (Blinc routes a release to the pressed
//! element wherever the pointer is, so the position is checked here). The keyboard works on the row
//! of options, the one tab stop: Left and Right move a cursor, Enter or Space answers the option
//! under it on the key's release, only while the row has the keyboard focus; a held key (which
//! repeats the press) never answers.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::card::{Ctx, Sinks};
use super::model::{Decision, ToolCall, ToolChoice, ToolOption};
use crate::softwrap::soft_wrap;

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
    let (a1, a2) = (ctx.approvals.clone(), ctx.approvals.clone());
    let (place, message, id) = (ctx.place.clone(), ctx.message.clone(), call.id.clone());
    let (p2, m2, i2) = (place.clone(), message.clone(), id.clone());
    let options = call.options.clone();
    let opt2 = options.clone();
    let prefix = prefix.to_owned();
    let (prefix2, choose, rearm) = (
        prefix.clone(),
        Arc::clone(&sinks.choose),
        Arc::clone(&sinks.rearm),
    );
    let drawn = (call.revision, call.digest.clone());
    row.on_focus(move |_| on.store(true, Ordering::Relaxed))
        .on_blur(move |_| off.store(false, Ordering::Relaxed))
        .on_key_down(move |c| {
            if !mine(c, &down) || options.is_empty() {
                return;
            }
            match c.key_code {
                37 | 39 => {
                    let delta = if c.key_code == 37 { -1 } else { 1 };
                    a1.move_cursor(&place, &message, &id, delta, options.len());
                    rearm(0);
                }
                13 | 32 if !a1.has_press() => {
                    let o = &options[a1.cursor(&place, &message, &id).min(options.len() - 1)];
                    a1.press(
                        &place,
                        &message,
                        &id,
                        &format!("{prefix}-opt-{id}-{}", o.id),
                        o.choice == ToolChoice::Refuse,
                    );
                }
                _ => {}
            }
        })
        .on_key_up(move |c| {
            if !matches!(c.key_code, 13 | 32) || !mine(c, &up) || opt2.is_empty() {
                return;
            }
            let o = &opt2[a2.cursor(&p2, &m2, &i2).min(opt2.len() - 1)];
            let refuse = o.choice == ToolChoice::Refuse;
            if a2.release(
                &p2,
                &m2,
                &i2,
                &format!("{prefix2}-opt-{i2}-{}", o.id),
                refuse,
            ) {
                choose(Decision {
                    message: m2.clone(),
                    call: i2.clone(),
                    option: o.id.clone(),
                    choice: o.choice,
                    revision: drawn.0,
                    digest: drawn.1.clone(),
                });
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
/// position is checked here). The keyboard answers only while this very button has the keyboard
/// focus, on the key's release; a held key (which repeats the press) never answers.
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
    let mut b = div()
        .class("fs-tc-btn")
        .class(class)
        .id(&id)
        .attr("tabindex", "0");
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
            if !(inside(c) && w1.0.press(&w1.1, &w1.2, &w1.3, &w1.4, refuse)) {
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
