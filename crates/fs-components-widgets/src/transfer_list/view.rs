//! The transfer list as elements.
//!
//! Pattern: Adapter — our model in, `blinc_cn` progress bar and buttons inside Blinc layout out.

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_cn::components::progress::progress;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::labels::Labels;
use super::model::{Action, Direction, Outcome, State, Transfer, TransferList, format_bytes};
use crate::Element;

/// Width of the progress bar in pixels.
const BAR: f32 = 180.0;

impl TransferList {
    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let mut list = div().class("fs-tl");
        if self.transfers.is_empty() {
            return list.child(text(self.labels.empty.clone()).class("fs-tl-empty"));
        }
        for t in self.transfers.iter().take(self.max_rows) {
            list = list.child(self.row(t));
        }
        let more = self.not_shown();
        if more > 0 {
            let line = Labels::fill(&self.labels.more, &more.to_string());
            list = list.child(text(line).class("fs-tl-more"));
        }
        list
    }

    fn row(&self, t: &Transfer) -> Div {
        let l = &self.labels;
        let way = match t.direction {
            Direction::Upload => &l.upload,
            _ => &l.download,
        };
        let status = match &t.state {
            State::Queued => l.queued.clone(),
            State::Running => l.running.clone(),
            State::Done => l.done.clone(),
            State::Failed(why) => Labels::fill(&l.failed, why),
        };
        let size = match t.total {
            Some(total) => format!("{} / {}", format_bytes(t.done), format_bytes(total)),
            None if t.state == State::Done => format_bytes(t.done),
            None => format!("{} ({})", format_bytes(t.done), l.unknown_size),
        };
        let bar = match t.fraction() {
            Some(f) => progress(f * 100.0).w(BAR),
            None => progress(0.0).w(BAR),
        };
        let main = div()
            .class("fs-tl-main")
            .child(text(t.name.clone()).class("fs-tl-name").no_wrap())
            .child(bar)
            .child(
                text(format!("{way} - {status} - {size}"))
                    .class("fs-tl-meta")
                    .no_wrap(),
            );
        let mut actions = div().class("fs-tl-actions");
        for action in t.actions() {
            actions = actions.child(self.button(t, action));
        }
        let mut row = div().class("fs-tl-row").child(main).child(actions);
        if matches!(t.state, State::Failed(_)) {
            row = row.class("fs-tl-row-failed");
        }
        row
    }

    fn button(&self, t: &Transfer, action: Action) -> impl Element + use<> {
        let l = &self.labels;
        let (word, variant, tag) = match action {
            Action::Retry => (&l.retry, ButtonVariant::Primary, "retry"),
            Action::Dismiss => (&l.dismiss, ButtonVariant::Ghost, "dismiss"),
            _ => (&l.cancel, ButtonVariant::Outline, "cancel"),
        };
        let id = format!("fs-tl-{}-{tag}", t.key);
        let mut b = ButtonBuilder::with_key(id.clone(), word.clone())
            .variant(variant)
            .size(ButtonSize::Small);
        if let Some(handler) = &self.on_outcome {
            let (handler, outcome) = (handler.clone(), Outcome::new(t.key.clone(), action));
            b = b.on_click(move |_| handler(&outcome));
        }
        b.build_component().id(&id)
    }
}
