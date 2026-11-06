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

use super::buttons::options;
use super::guard::Approvals;
use super::labels::Labels;
use super::model::{Decision, ToolCall, ToolEnd, ToolKind, ToolRisk, ToolState};
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
    /// The place that draws the card (a list, the dock).
    pub place: String,
    /// Its position among the questions of that place.
    pub slot: usize,
    /// The height of the window on the text of a question, if the place decides it.
    pub body_h: Option<f32>,
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
    let mut body_state = None;
    if question {
        let (lines, window) = text_lines(call, ctx.body_h);
        let more = lines.saturating_sub(window);
        let body = ctx
            .approvals
            .show(&ctx.place, &ctx.message, call, ctx.slot, more > 0);
        body_state = Some((body, more));
        if !ctx
            .approvals
            .armed(&ctx.place, &ctx.message, &call.id, false)
        {
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
        if let Some((body, more)) = body_state {
            root = root.child(question_body(call, labels, prefix, &body, ctx.body_h));
            if more > 0 {
                let note = labels.more_lines.replacen("{}", &more.to_string(), 1);
                root = root.child(
                    text(note)
                        .class("fs-tc-caption")
                        .id(format!("{prefix}-more-{}", call.id)),
                );
            }
        }
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

/// How many lines the text of a question takes (an estimate) and how many its window shows.
fn text_lines(call: &ToolCall, height: Option<f32>) -> (usize, usize) {
    const PER_LINE: usize = 56;
    let count = |t: &str| {
        t.split('\n')
            .map(|l| l.chars().count().div_ceil(PER_LINE).max(1))
            .sum::<usize>()
    };
    let lines = 2
        + count(&call.detail)
        + if call.summary.is_empty() {
            0
        } else {
            1 + count(&call.summary)
        };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let window = (height.unwrap_or_else(|| body_height(call)) / 18.0) as usize;
    (lines, window)
}

/// A window on the agent's words and the engine's text, about this many pixels high.
fn body_height(call: &ToolCall) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let lines = (call.summary.chars().count() + call.detail.chars().count()) as f32 / 48.0 + 3.0;
    (lines * 18.0).clamp(100.0, 150.0)
}

fn question_body(
    call: &ToolCall,
    labels: &Labels,
    prefix: &str,
    state: &super::entry::Body,
    height: Option<f32>,
) -> Scroll {
    let mut inner = div().class("fs-tc-details").w_full();
    // what the engine says it will do comes first: that is what is being allowed
    if !call.detail.is_empty() {
        inner = inner
            .child(text(labels.engine_detail.clone()).class("fs-tc-caption"))
            .child(
                text(soft_wrap(&call.detail))
                    .class("fs-tc-code")
                    .id(format!("{prefix}-detail-{}", call.id)),
            );
    }
    if !call.summary.is_empty() {
        inner = inner
            .child(text(labels.by_agent.clone()).class("fs-tc-caption"))
            .child(text(soft_wrap(&call.summary)).class("fs-tc-summary"));
    }
    Scroll::with_physics(Arc::clone(&state.physics))
        .bind(&state.scroll)
        .w_full()
        .h(height.unwrap_or_else(|| body_height(call)))
        .content(inner)
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
