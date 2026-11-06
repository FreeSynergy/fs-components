//! One tool call drawn as a card.
//!
//! Pattern: Adapter — the call in, Blinc elements out. The kind is the first thing in the header.
//! A question shows, without opening anything, the agent's summary (labelled as the agent's words)
//! and what the engine says it will do (`detail`); the text scrolls in a small window and the
//! answer buttons stay below it, always visible. Answer buttons are the guard's (`Approvals`): a
//! new or changed question is disarmed for a moment, and a disarmed button looks it. Nothing here
//! decides or executes anything.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::Scroll;

use super::guard::Approvals;
use super::labels::Labels;
use super::model::{
    Decision, ToolCall, ToolChoice, ToolEnd, ToolKind, ToolOption, ToolRisk, ToolState,
};
use crate::softwrap::soft_wrap;

/// Called with a message and a call id.
pub(crate) type ToggleFn = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Called with an answer that passed the guard.
pub(crate) type ChooseFn = Arc<dyn Fn(Decision) + Send + Sync>;

/// Where a card reports clicks. The card keeps no state of its own.
#[derive(Clone)]
pub(crate) struct Sinks {
    /// The detail button was pressed: the message and the call's id.
    pub toggle: ToggleFn,
    /// An answer passed the guard.
    pub choose: ChooseFn,
    /// A new or changed question was drawn: draw again after this many milliseconds.
    pub rearm: Arc<dyn Fn(u64) + Send + Sync>,
}

/// The guard and the message a card belongs to.
#[derive(Clone)]
pub(crate) struct Ctx {
    pub approvals: Approvals,
    pub message: String,
}

fn kind_class(k: ToolKind) -> &'static str {
    match k {
        ToolKind::Reads => "fs-tc-kind-reads",
        ToolKind::Writes => "fs-tc-kind-writes",
        ToolKind::RunsCommand => "fs-tc-kind-runs",
        ToolKind::ConnectedTool => "fs-tc-kind-connected",
        _ => "fs-tc-kind-unknown",
    }
}

fn risk_class(r: ToolRisk) -> &'static str {
    match r {
        ToolRisk::ReadsOnly => "fs-tc-risk-ok",
        ToolRisk::Changes => "fs-tc-risk-changes",
        ToolRisk::Destroys | ToolRisk::ActsForPerson => "fs-tc-risk-high",
        _ => "fs-tc-risk-unknown",
    }
}

fn chip(word: &str, classes: &[&str]) -> Div {
    let mut d = div();
    for c in classes {
        d = d.class(*c);
    }
    d.child(text(soft_wrap(word)).class("fs-tc-chip-text"))
}

/// Draw `call`. `prefix` makes the element ids unique per place.
pub(crate) fn card(
    call: &ToolCall,
    expanded: bool,
    labels: &Labels,
    prefix: &str,
    sinks: &Sinks,
    ctx: &Ctx,
) -> Div {
    let question = call.state == ToolState::Question;
    if question {
        ctx.approvals.show(&ctx.message, call);
        if !ctx.approvals.armed(&ctx.message, &call.id) {
            (sinks.rearm)(ctx.approvals.delay());
        }
    } else if call.state == ToolState::Settled {
        ctx.approvals.forget(&ctx.message, &call.id);
    }
    let mut root = div()
        .class("fs-tc")
        .id(format!("{prefix}-card-{}", call.id));
    if question {
        root = root.class("fs-tc-question");
    }
    if call.state == ToolState::Settled && call.end == ToolEnd::Failed {
        root = root.class("fs-tc-failed");
    }
    let state = if call.state == ToolState::Settled {
        format!("{} - {}", labels.state(call.state), labels.end(call.end))
    } else {
        labels.state(call.state).to_string()
    };
    root = root.child(
        div()
            .class("fs-tc-head")
            .child(chip(
                labels.kind(call.kind),
                &["fs-tc-kind", kind_class(call.kind)],
            ))
            .child(text(soft_wrap(&call.name)).class("fs-tc-name"))
            .child(chip(&state, &["fs-tc-state"]))
            .child(chip(
                labels.risk(call.risk),
                &["fs-tc-risk", risk_class(call.risk)],
            )),
    );
    if question {
        root = root.child(question_body(call, labels, prefix));
        if let Some(why) = ctx.approvals.refusal(&ctx.message, &call.id) {
            let line = labels.refused.replacen("{}", &soft_wrap(&why), 1);
            root = root.child(
                div()
                    .class("fs-tc-refused")
                    .id(format!("{prefix}-refused-{}", call.id))
                    .child(text(line).class("fs-tc-summary")),
            );
        }
        return root.child(options(call, prefix, sinks, ctx));
    }
    if !call.summary.is_empty() {
        root = root.child(text(soft_wrap(&call.summary)).class("fs-tc-summary"));
    }
    if !call.detail.is_empty() || !call.result.is_empty() || !call.summary.is_empty() {
        root = root.child(toggle(call, expanded, labels, prefix, sinks, &ctx.message));
    }
    if expanded {
        root = root.child(details(call, labels));
    }
    root
}

/// A window on the agent's words and the engine's text, about this many pixels high.
fn body_height(call: &ToolCall) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let lines = (call.summary.chars().count() + call.detail.chars().count()) as f32 / 48.0 + 3.0;
    (lines * 18.0).clamp(100.0, 150.0)
}

fn question_body(call: &ToolCall, labels: &Labels, prefix: &str) -> Scroll {
    let mut body = div().class("fs-tc-details").w_full();
    // what the engine says it will do comes first: that is what is being allowed
    if !call.detail.is_empty() {
        body = body
            .child(text(labels.engine_detail.clone()).class("fs-tc-caption"))
            .child(
                text(soft_wrap(&call.detail))
                    .class("fs-tc-code")
                    .id(format!("{prefix}-detail-{}", call.id)),
            );
    }
    if !call.summary.is_empty() {
        body = body
            .child(text(labels.by_agent.clone()).class("fs-tc-caption"))
            .child(text(soft_wrap(&call.summary)).class("fs-tc-summary"));
    }
    Scroll::new().w_full().h(body_height(call)).content(body)
}

fn toggle(
    call: &ToolCall,
    expanded: bool,
    labels: &Labels,
    prefix: &str,
    sinks: &Sinks,
    message: &str,
) -> impl crate::Element + use<> {
    let word = if expanded {
        &labels.hide_details
    } else {
        &labels.show_details
    };
    let id = format!("{prefix}-toggle-{}", call.id);
    let (toggle, call_id, message) = (
        Arc::clone(&sinks.toggle),
        call.id.clone(),
        message.to_owned(),
    );
    ButtonBuilder::with_key(id.clone(), word.clone())
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Small)
        .on_click(move |_| toggle(&message, &call_id))
        .build_component()
        .id(&id)
}

fn details(call: &ToolCall, labels: &Labels) -> Div {
    let mut d = div().class("fs-tc-details");
    d = d.child(text(labels.by_agent.clone()).class("fs-tc-caption"));
    if !call.detail.is_empty() {
        d = d
            .child(text(labels.engine_detail.clone()).class("fs-tc-caption"))
            .child(text(soft_wrap(&call.detail)).class("fs-tc-code"));
    }
    if !call.result.is_empty() {
        d = d
            .child(text(labels.result.clone()).class("fs-tc-caption"))
            .child(text(soft_wrap(&call.result)).class("fs-tc-code"));
    }
    d
}

fn options(call: &ToolCall, prefix: &str, sinks: &Sinks, ctx: &Ctx) -> Div {
    let mut row = div().class("fs-tc-options");
    for o in &call.options {
        row = row.child(answer_button(call, o, prefix, sinks, ctx));
    }
    row
}

/// An answer button: it answers on a release that follows a press that began on it while armed.
fn answer_button(call: &ToolCall, o: &ToolOption, prefix: &str, sinks: &Sinks, ctx: &Ctx) -> Div {
    let armed = ctx.approvals.armed(&ctx.message, &call.id);
    let id = format!("{prefix}-opt-{}-{}", call.id, o.id);
    let class = if o.choice == ToolChoice::Refuse {
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
    let drawn = (
        ctx.message.clone(),
        call.id.clone(),
        call.revision,
        call.digest.clone(),
    );
    let decision = Decision {
        message: ctx.message.clone(),
        call: call.id.clone(),
        option: o.id.clone(),
        choice: o.choice,
        revision: call.revision,
        digest: call.digest.clone(),
    };
    let (a1, a2, a3, a4, a5) = (
        ctx.approvals.clone(),
        ctx.approvals.clone(),
        ctx.approvals.clone(),
        ctx.approvals.clone(),
        ctx.approvals.clone(),
    );
    let (d1, d2, d3, d4) = (drawn.clone(), drawn.clone(), drawn.clone(), drawn);
    let (i1, i2, i3, i4) = (id.clone(), id.clone(), id.clone(), id);
    let (choose, choose_key) = (Arc::clone(&sinks.choose), Arc::clone(&sinks.choose));
    let (dec, dec_key) = (decision.clone(), decision);
    b.child(text(soft_wrap(&o.label)).class("fs-tc-btn-text"))
        .on_mouse_down(move |_| {
            a1.press(&d1.0, &d1.1, &i1, d1.2, &d1.3);
        })
        .on_hover_leave(move |_| a2.cancel_press())
        .on_mouse_up(move |_| {
            if a3.release(&d2.0, &d2.1, &i2, d2.2, &d2.3) {
                choose(dec.clone());
            }
        })
        // the keyboard answers on the key's release, and a held key (which repeats the press) never does
        .on_key_down(move |ctx| {
            if matches!(ctx.key_code, 13 | 32) && !a4.has_press() {
                a4.press(&d3.0, &d3.1, &i3, d3.2, &d3.3);
            }
        })
        .on_key_up(move |ctx| {
            if matches!(ctx.key_code, 13 | 32) && a5.release(&d4.0, &d4.1, &i4, d4.2, &d4.3) {
                choose_key(dec_key.clone());
            }
        })
}
