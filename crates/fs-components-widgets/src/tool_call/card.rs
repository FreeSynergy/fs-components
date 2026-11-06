//! One tool call drawn as a card.
//!
//! Pattern: Adapter — the call in, Blinc elements out. The kind is the first thing in the header,
//! before the name and before any detail. Questions are never folded away: the buttons of a
//! question are always drawn. Nothing here decides or executes anything: a click only reports
//! through the [`Sinks`].

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::labels::Labels;
use super::model::{ToolCall, ToolChoice, ToolEnd, ToolKind, ToolRisk, ToolState};

/// Called with a call id and an option id.
pub(crate) type ChooseFn = Arc<dyn Fn(&str, &str) + Send + Sync>;

/// Where a card reports clicks. The card keeps no state of its own.
#[derive(Clone)]
pub(crate) struct Sinks {
    /// The detail button was pressed: the call's id.
    pub toggle: Arc<dyn Fn(&str) + Send + Sync>,
    /// An option button was pressed: call id, option id.
    pub choose: ChooseFn,
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

/// Draw `call`. `prefix` makes the element ids unique per place.
pub(crate) fn card(
    call: &ToolCall,
    expanded: bool,
    labels: &Labels,
    prefix: &str,
    sinks: &Sinks,
) -> Div {
    let mut root = div()
        .class("fs-tc")
        .id(format!("{prefix}-card-{}", call.id));
    if call.state == ToolState::Question {
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
    let head = div()
        .class("fs-tc-head")
        .child(chip(
            labels.kind(call.kind),
            &["fs-tc-kind", kind_class(call.kind)],
        ))
        .child(text(call.name.clone()).class("fs-tc-name"))
        .child(chip(&state, &["fs-tc-state"]))
        .child(chip(
            labels.risk(call.risk),
            &["fs-tc-risk", risk_class(call.risk)],
        ));
    root = root.child(head);
    if !call.summary.is_empty() {
        root = root.child(text(call.summary.clone()).class("fs-tc-summary"));
    }
    if call.state == ToolState::Question {
        root = root.child(options(call, prefix, sinks));
    }
    if !call.detail.is_empty() || !call.result.is_empty() || !call.summary.is_empty() {
        root = root.child(toggle(call, expanded, labels, prefix, sinks));
    }
    if expanded {
        root = root.child(details(call, labels));
    }
    root
}

fn chip(word: &str, classes: &[&str]) -> Div {
    let mut d = div();
    for c in classes {
        d = d.class(*c);
    }
    d.child(text(word.to_string()).class("fs-tc-chip-text"))
}

fn toggle(
    call: &ToolCall,
    expanded: bool,
    labels: &Labels,
    prefix: &str,
    sinks: &Sinks,
) -> impl crate::Element + use<> {
    let word = if expanded {
        &labels.hide_details
    } else {
        &labels.show_details
    };
    let id = format!("{prefix}-toggle-{}", call.id);
    let (toggle, call_id) = (Arc::clone(&sinks.toggle), call.id.clone());
    ButtonBuilder::with_key(id.clone(), word.clone())
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Small)
        .on_click(move |_| toggle(&call_id))
        .build_component()
        .id(&id)
}

fn details(call: &ToolCall, labels: &Labels) -> Div {
    let mut d = div().class("fs-tc-details");
    d = d.child(text(labels.by_agent.clone()).class("fs-tc-caption"));
    if !call.detail.is_empty() {
        d = d
            .child(text(labels.detail.clone()).class("fs-tc-caption"))
            .child(text(call.detail.clone()).class("fs-tc-code"));
    }
    if !call.result.is_empty() {
        d = d
            .child(text(labels.result.clone()).class("fs-tc-caption"))
            .child(text(call.result.clone()).class("fs-tc-code"));
    }
    d
}

fn options(call: &ToolCall, prefix: &str, sinks: &Sinks) -> Div {
    let mut row = div().class("fs-tc-options");
    for o in &call.options {
        let id = format!("{prefix}-opt-{}-{}", call.id, o.id);
        let variant = match o.choice {
            ToolChoice::Refuse => ButtonVariant::Outline,
            _ => ButtonVariant::Primary,
        };
        let (choose, call_id, option) = (Arc::clone(&sinks.choose), call.id.clone(), o.id.clone());
        row = row.child(
            ButtonBuilder::with_key(id.clone(), o.label.clone())
                .variant(variant)
                .size(ButtonSize::Small)
                .on_click(move |_| choose(&call_id, &option))
                .build_component()
                .id(&id),
        );
    }
    row
}
