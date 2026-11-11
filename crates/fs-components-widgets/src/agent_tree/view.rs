//! The tree as elements.
//!
//! Pattern: Adapter over Blinc's scroll container. Rows have one fixed height, so only the rows in
//! view exist as elements, placed by their index (as in `list-view`): thousands of agents cost
//! what a screenful costs. A line above the tree always says how many need attention, so a
//! failure cannot hide behind a closed branch or out of view.

use std::sync::Arc;

use blinc_core::reactive::{ReactiveGraph, computed};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::Scroll;

use super::forest::Row;
use super::handle::AgentTree;
use super::labels::Labels;
use super::model::{AgentEnd, AgentState, Attention};
use crate::Element;
use crate::list_view::visible_rows;
use crate::participant::chip;

/// Height of one row in pixels.
pub const ROW_HEIGHT: f32 = 36.0;
/// Height of the line above the tree.
const HEAD: f32 = 30.0;
/// Rows deeper than this are not indented further (the depth is written instead).
const MAX_INDENT: usize = 10;

impl AgentTree {
    /// The element to place in a layout, filling `width` x `height`.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let this = self.clone();
        self.live.view("tree", move || this.render(width, height))
    }

    #[allow(clippy::cast_precision_loss)]
    fn render(&self, width: f32, height: f32) -> Div {
        let rows = self.rows();
        let root = div().class("fs-at").w(width).h(height).min_h(0.0);
        let root = root.child(self.header());
        if rows.is_empty() {
            return root.child(
                div()
                    .class("fs-at-empty")
                    .child(text(self.labels.empty.clone())),
            );
        }
        let count = rows.len();
        let view = (height - HEAD).max(0.0);
        let total = count as f32 * ROW_HEIGHT;
        let offset = self.position.offset().clamp(0.0, (total - view).max(0.0));
        self.position.apply_offset(offset);
        let watch = fs_components_primitives::scroll_watch(&self.position.scroll, offset);
        if watch.try_get().is_none_or(|y| (y - offset).abs() > 0.5) {
            watch.set(offset);
        }
        let window = computed(move |g: &ReactiveGraph| {
            let y = g.get(watch).unwrap_or(offset);
            visible_rows(y, view, ROW_HEIGHT, count).collect::<Vec<usize>>()
        });
        let (this, shown) = (self.clone(), Arc::clone(&rows));
        let body = div().relative().w(width).h(total).flex_shrink_0().for_each(
            &window,
            |i: &usize| *i,
            move |i: usize| {
                this.row(&shown[i], width)
                    .absolute()
                    .left(0.0)
                    .top(i as f32 * ROW_HEIGHT)
                    .w(width)
                    .h(ROW_HEIGHT)
            },
        );
        root.child(
            Scroll::with_physics(Arc::clone(&self.position.physics))
                .bind(&self.position.scroll)
                .w(width)
                .h(view)
                .content(body),
        )
    }

    fn header(&self) -> Div {
        let c = self.counts();
        let line = if c.total() == 0 {
            self.labels.all_quiet.clone()
        } else {
            self.labels
                .needs_attention
                .replacen("{}", &c.total().to_string(), 1)
        };
        let mut head = div().class("fs-at-head").h(HEAD);
        if let Some(a) = c.most_urgent() {
            head = head.class(attention_class(a));
        }
        head.id(format!("{}-head", self.id))
            .child(text(line).class("fs-at-head-text"))
    }

    #[allow(clippy::cast_precision_loss)] // at most MAX_INDENT
    fn row(&self, r: &Row, _width: f32) -> Div {
        let l = &self.labels;
        let mut row = div()
            .class("fs-at-row")
            .id(format!("{}-row-{}", self.id, r.node.key));
        if let Some(a) = r.attention {
            row = row.class(attention_class(a));
        }
        row = row.child(
            div()
                .w(r.depth.min(MAX_INDENT) as f32 * 16.0)
                .flex_shrink_0(),
        );
        row = row.child(self.toggle_slot(r));
        row = row.child(chip(r.node.kind, &l.kinds).id(format!("{}-mark-{}", self.id, r.node.key)));
        let (this, key) = (self.clone(), r.node.key.clone());
        let name = div()
            .class("fs-at-main")
            .id(format!("{}-main-{}", self.id, r.node.key))
            .child(text(r.node.name.clone()).class("fs-at-name").no_wrap())
            .child(
                text(r.node.summary.clone())
                    .class("fs-at-summary")
                    .no_wrap(),
            )
            .on_click(move |_| {
                this.select(&key);
            });
        row = row.child(name);
        if r.depth > MAX_INDENT {
            let word = l.depth.replacen("{}", &r.depth.to_string(), 1);
            row = row.child(text(word).class("fs-at-meta").no_wrap());
        }
        row = row.child(
            text(state_word(l, r.node.state, r.node.end).to_owned())
                .class("fs-at-state")
                .no_wrap(),
        );
        if r.collapsed && r.below.total() > 0 {
            let word = l.below.replacen("{}", &r.below.total().to_string(), 1);
            let class = r.below.most_urgent().map_or("", attention_class);
            row = row.child(
                div()
                    .class("fs-at-below")
                    .class(class)
                    .id(format!("{}-below-{}", self.id, r.node.key))
                    .child(text(word).class("fs-at-meta").no_wrap()),
            );
        }
        row
    }

    fn toggle_slot(&self, r: &Row) -> Div {
        let slot = div().class("fs-at-toggle");
        if !r.has_children {
            return slot;
        }
        let word = if r.collapsed {
            &self.labels.expand
        } else {
            &self.labels.collapse
        };
        let (this, key) = (self.clone(), r.node.key.clone());
        slot.id(format!("{}-toggle-{}", self.id, r.node.key))
            .child(text(word.clone()).class("fs-at-name"))
            .on_click(move |_| {
                this.toggle(&key);
            })
    }
}

fn attention_class(a: Attention) -> &'static str {
    match a {
        Attention::Question => "fs-at-question",
        Attention::Failure => "fs-at-failure",
        _ => "fs-at-delivery",
    }
}

fn state_word(l: &Labels, s: AgentState, e: AgentEnd) -> &str {
    match (s, e) {
        (AgentState::Forming, _) => &l.forming,
        (AgentState::Working, _) => &l.working,
        (AgentState::Question, _) => &l.question,
        (_, AgentEnd::Delivered) => &l.delivered,
        (_, AgentEnd::Failed) => &l.failed,
        (_, AgentEnd::Cancelled) => &l.cancelled,
        _ => &l.settled,
    }
}
