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
            return list.child(
                text(self.labels.empty.clone())
                    .class("fs-tl-empty")
                    .id("fs-tl-empty"),
            );
        }
        for i in self.shown() {
            list = list.child(self.row(&self.transfers[i]));
        }
        let more = self.not_shown();
        if more > 0 {
            let mut line = Labels::fill(&self.labels.more, &more.to_string());
            let failed = self.failed_not_shown();
            if failed > 0 {
                line = format!(
                    "{line} {}",
                    Labels::fill(&self.labels.more_failed, &failed.to_string())
                );
            }
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
        // Names and failure reasons wrap: they must stay readable to the end.
        let failed = matches!(t.state, State::Failed(_));
        let meta = if failed {
            format!("{way} - {size}")
        } else {
            format!("{way} - {status} - {size}")
        };
        let mut main = div()
            .class("fs-tl-main")
            .child(text(t.name.clone()).class("fs-tl-name"))
            .child(bar)
            .child(text(meta).class("fs-tl-meta").no_wrap());
        if failed {
            main = main.child(text(status).class("fs-tl-reason"));
        }
        let mut actions = div().class("fs-tl-actions");
        for action in t.actions().into_iter().filter(|a| !self.without.contains(a)) {
            actions = actions.child(self.button(t, action));
        }
        let mut row = div().class("fs-tl-row").child(main).child(actions);
        if failed {
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
