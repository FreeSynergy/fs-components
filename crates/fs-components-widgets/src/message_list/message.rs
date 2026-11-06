//! One message drawn as a row.
//!
//! Pattern: Adapter — a [`ChatMessage`] in, Blinc elements out. The header (name, kind mark,
//! operator, time, delivery) is drawn by one function for every sender, so an AI's message can
//! never come out looking like a person's. Text is the text core's `TextView`; what it cannot draw
//! is shown as text. Tool calls are the `tool-call` cards.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::handle::{MessageList, Outcome};
use super::labels::Labels;
use super::model::{ChatMessage, Delivery};
use crate::participant::{SenderKind, chip};
use crate::text_view::TextView;
use crate::tool_call::{Ctx, Sinks, card};

fn delivery_word(l: &Labels, d: Delivery) -> &str {
    match d {
        Delivery::Sending => &l.sending,
        Delivery::Sent => &l.sent,
        Delivery::Delivered => &l.delivered,
        Delivery::Read => &l.read,
        _ => &l.failed,
    }
}

fn kind_class(k: SenderKind) -> &'static str {
    match k {
        SenderKind::Person => "fs-ml-k-person",
        SenderKind::Software => "fs-ml-k-software",
        SenderKind::Ai => "fs-ml-k-ai",
        _ => "fs-ml-k-unknown",
    }
}

impl MessageList {
    pub(super) fn message_row(&self, m: &ChatMessage) -> Div {
        let l = &self.labels;
        let mut row = div()
            .class("fs-ml-msg")
            .class(kind_class(m.sender.kind))
            .id(self.row_id(&m.key));
        if m.mine {
            row = row.class("fs-ml-mine");
        }
        // a coloured edge in the kind's colour: a person, software, an AI and an unknown sender
        // each have their own, and the tint of "mine" is another thing
        row = row.relative().child(
            div()
                .class("fs-ml-edge")
                .class(kind_class(m.sender.kind))
                .absolute()
                .left(0.0)
                .top(0.0)
                .bottom(0.0)
                .w(5.0),
        );
        row = row.child(self.header(m));
        if m.blocks.is_empty() && m.tool_calls.is_empty() {
            row = row.child(text(l.empty_message.clone()).class("fs-ml-note"));
        } else if !m.blocks.is_empty() {
            row = row.child(
                div()
                    .class("fs-ml-body")
                    .child(TextView::new(m.blocks.clone()).build()),
            );
        }
        if m.streaming {
            row = row.child(text(l.writing.clone()).class("fs-ml-note"));
        }
        if !m.reasoning.is_empty() {
            row = row.child(self.reasoning(m));
        }
        if !m.tool_calls.is_empty() {
            row = row.child(self.tools(m));
        }
        if !m.references.is_empty() {
            row = row.child(self.references(m));
        }
        if m.delivery == Some(Delivery::Failed) {
            row = row.child(self.failure(m));
        }
        row
    }

    fn header(&self, m: &ChatMessage) -> Div {
        let l = &self.labels;
        let mut head = div()
            .class("fs-ml-head")
            .child(text(m.sender.name.clone()).class("fs-ml-name"))
            .child(chip(m.sender.kind, &l.kinds).id(format!("{}-mark-{}", self.id, m.key)));
        if !m.sender.operator.is_empty() {
            head = head.child(text(m.sender.operator.clone()).class("fs-ml-meta"));
        }
        if !m.time.is_empty() {
            head = head.child(text(m.time.clone()).class("fs-ml-meta").no_wrap());
        }
        if let Some(d) = m.delivery {
            let mut state = div().class("fs-ml-delivery");
            if d == Delivery::Failed {
                state = state.class("fs-ml-delivery-failed");
            }
            head = head.child(
                state.child(
                    text(delivery_word(l, d).to_string())
                        .class("fs-ml-meta")
                        .no_wrap(),
                ),
            );
        }
        head
    }

    fn reasoning(&self, m: &ChatMessage) -> Div {
        let tag = format!("r:{}", m.key);
        let open = self.is_open(&tag);
        let word = if open {
            &self.labels.hide_reasoning
        } else {
            &self.labels.show_reasoning
        };
        let id = format!("{}-reasoning-{}", self.id, m.key);
        let this = self.clone();
        let button = ButtonBuilder::with_key(id.clone(), word.clone())
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Small)
            .on_click(move |_| this.toggle_open(tag.clone()))
            .build_component()
            .id(&id);
        let mut d = div().class("fs-ml-reasoning").child(button);
        if open {
            d = d.child(
                div()
                    .class("fs-ml-body")
                    .child(TextView::new(m.reasoning.clone()).build()),
            );
        }
        d
    }

    fn tools(&self, m: &ChatMessage) -> Div {
        let (toggler, chooser, live) = (self.clone(), self.clone(), self.live);
        let key = m.key.clone();
        let sinks = Sinks {
            toggle: Arc::new(move |_message, call| toggler.toggle_open(format!("t:{key}:{call}"))),
            choose: Arc::new(move |decision| {
                chooser.answer(decision);
            }),
            rearm: Arc::new(move |ms| crate::tool_call::refresh_later(live, ms)),
        };
        let ctx = Ctx {
            approvals: self.approvals.clone(),
            message: m.key.clone(),
        };
        let mut col = div().class("fs-ml-tools");
        let prefix = format!("{}-{}", self.id, m.key);
        for c in &m.tool_calls {
            let open = self.is_open(&format!("t:{}:{}", m.key, c.id));
            col = col.child(card(c, open, &self.labels.tools, &prefix, &sinks, &ctx));
        }
        col
    }

    fn references(&self, m: &ChatMessage) -> Div {
        let mut row = div().class("fs-ml-refs");
        for r in &m.references {
            let id = format!("{}-ref-{}-{}", self.id, m.key, r.key);
            let (this, message, reference) = (self.clone(), m.key.clone(), r.key.clone());
            row = row.child(
                ButtonBuilder::with_key(id.clone(), r.label.clone())
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Small)
                    .on_click(move |_| {
                        this.emit(&Outcome::Reference {
                            message: message.clone(),
                            reference: reference.clone(),
                        });
                    })
                    .build_component()
                    .id(&id),
            );
        }
        row
    }

    fn failure(&self, m: &ChatMessage) -> Div {
        let id = format!("{}-retry-{}", self.id, m.key);
        let (this, message) = (self.clone(), m.key.clone());
        let retry = ButtonBuilder::with_key(id.clone(), self.labels.retry.clone())
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Small)
            .on_click(move |_| {
                this.emit(&Outcome::Retry {
                    message: message.clone(),
                });
            })
            .build_component()
            .id(&id);
        div()
            .class("fs-ml-failure")
            .child(text(m.failure.clone()).class("fs-ml-failure-text"))
            .child(retry)
    }
}
