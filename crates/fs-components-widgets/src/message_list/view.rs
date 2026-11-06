//! The list as elements.
//!
//! Pattern: Adapter over Blinc's scroll container. Only the messages near the view exist: a top
//! spacer and a bottom spacer stand for the rest, with the heights the list knows (estimated, then
//! measured). The built messages lie in a normal flow, so a wrong estimate can never cut text or
//! make two messages overlap; it only moves the scrollbar a little until the message was seen.
//! A watcher on the scroll position builds again when the view leaves what is built.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::Scroll;

use super::estimate::estimate;
use super::handle::MessageList;
use super::window::window;
use crate::Element;

impl MessageList {
    /// The element to place in a layout, filling `width` x `height`.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        self.lock().size = (width, height);
        self.watch_scroll();
        let this = self.clone();
        let content = self.live.view(move || this.render());
        let scroll = Scroll::with_physics(Arc::clone(&self.position.physics))
            .bind(&self.position.scroll)
            .w(width)
            .h(height)
            .content(content);
        let this = self.clone();
        let notice = self
            .live
            .view(move || this.indicator())
            .absolute()
            .left(0.0)
            .bottom(8.0);
        div()
            .class("fs-ml")
            .relative()
            .w(width)
            .h(height)
            .child(scroll)
            .child(notice)
    }

    /// Start the scroll watcher once per list.
    fn watch_scroll(&self) {
        let mut slot = self
            .watch
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if slot.is_some() {
            return;
        }
        let watch = self.position.scroll.watch_y(self.position.offset());
        let this = self.clone();
        *slot = Some(blinc_core::reactive::effect(move |graph| {
            this.on_scroll(graph.get(watch).unwrap_or(0.0));
        }));
    }

    fn render(&self) -> Div {
        let l = &self.labels;
        let (source, range, width, view) = {
            let mut inner = self.lock();
            let count = inner.source.count();
            if count > inner.heights.len() {
                inner.heights.grow(count);
                inner.revs.resize(count, 0);
            }
            let view = inner.size.1;
            let offset = if inner.follow {
                (inner.heights.total() - view).max(0.0)
            } else {
                self.position.offset()
            };
            let range = window(&inner.heights, offset, view);
            (inner.source.clone(), range, inner.size.0, view)
        };
        if source.count() == 0 {
            return div()
                .class("fs-ml-empty")
                .child(text(l.no_messages.clone()));
        }
        let messages = match source.messages(range.start, range.len()) {
            Ok(m) => m,
            Err(why) => {
                let note = l.load_failed.replacen("{}", &why.0, 1);
                return div().class("fs-ml-empty").child(text(note));
            }
        };
        let (top, bottom) = {
            let mut inner = self.lock();
            let mut shown = Vec::with_capacity(messages.len());
            for (i, m) in messages.iter().enumerate() {
                let index = range.start + i;
                if index >= inner.heights.len() {
                    break;
                }
                if inner.revs[index] != m.revision || !inner.heights.is_exact(index) {
                    let fresh = inner.revs[index] != m.revision;
                    let guess = estimate(m, width);
                    if fresh {
                        inner.heights.reestimate(index, guess);
                    } else {
                        inner.heights.set(index, guess, false);
                    }
                    inner.revs[index] = m.revision;
                }
                shown.push((index, m.key.clone()));
            }
            let end = range.start + shown.len();
            inner.built = range.start..end;
            inner.shown = shown;
            let total = inner.heights.total();
            if inner.follow {
                self.position.apply_offset((total - view).max(0.0));
            }
            (
                inner.heights.top(range.start),
                total - inner.heights.top(end),
            )
        };
        let mut col = div().class("fs-ml-col").child(spacer(top));
        for m in &messages {
            let row = self.message_row(m);
            if let Some(handle) = blinc_layout::selector::query(&self.row_id(&m.key)) {
                let this = self.clone();
                handle.on_ready(move |_| {
                    this.measure();
                });
            }
            col = col.child(row);
        }
        col.child(spacer(bottom))
    }

    fn indicator(&self) -> Div {
        let unseen = self.unseen();
        let mut d = div().class("fs-ml-notice");
        if unseen > 0 && !self.is_following() {
            let id = format!("{}-new", self.id);
            let this = self.clone();
            let word = self
                .labels
                .new_messages
                .replacen("{}", &unseen.to_string(), 1);
            d = d.child(
                ButtonBuilder::with_key(id.clone(), word)
                    .variant(ButtonVariant::Primary)
                    .size(ButtonSize::Small)
                    .on_click(move |_| this.jump_to_end())
                    .build_component()
                    .id(&id),
            );
        }
        d
    }
}

fn spacer(height: f32) -> Div {
    div().w_full().h(height.max(0.0)).flex_shrink_0()
}
