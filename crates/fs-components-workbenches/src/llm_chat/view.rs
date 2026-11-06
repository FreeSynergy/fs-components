//! The chat as elements.
//!
//! Pattern: Adapter — the parts' views arranged in a column (and a side panel for the agents in the
//! full form). Questions wait in a dock above the text field, so they can never scroll away with
//! a long conversation. The stop control is the host's; the workbench draws it and reports a click.

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::Scroll;

use super::handle::LlmChat;
use super::outcome::Outcome;
use super::source::{StopControl, StopState};
use fs_components_widgets::Element;

const HEAD: f32 = 40.0;
const SIDE: f32 = 300.0;
const COMPOSER: f32 = 204.0;
const COMPOSER_COMPACT: f32 = 152.0;
const QUESTION: f32 = 175.0;

impl LlmChat {
    /// The element to place in a layout, filling `width` x `height`.
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let this = self.clone();
        self.layout
            .view(move || this.arrange(width, height))
            .h(height)
    }

    #[allow(clippy::cast_precision_loss)]
    fn arrange(&self, width: f32, height: f32) -> Div {
        let side = if self.compact {
            0.0
        } else {
            SIDE.min(width * 0.4)
        };
        let main = width - side;
        let composer = if self.compact {
            COMPOSER_COMPACT
        } else {
            COMPOSER
        };
        let questions = self.dock.calls().len();
        let dock = if questions == 0 {
            0.0
        } else {
            (questions as f32 * QUESTION).min(height * 0.4)
        };
        let list = (height - HEAD - composer - dock).max(80.0);
        let this = self.clone();
        let mut column = div()
            .class("fs-llm-main")
            .w(main)
            .h(height)
            .child(self.parts.view(move || this.header()).h(HEAD))
            .child(self.list.build(main, list));
        if questions > 0 {
            column = column.child(self.question_dock(main, dock, questions));
        }
        column = column.child(self.composer.build());
        let mut row = div()
            .class("fs-llm")
            .flex_row()
            .w(width)
            .h(height)
            .child(column);
        if !self.compact {
            let tree = self.tree.build(side, height - HEAD);
            let heading = text(self.labels.agents.clone()).class("fs-llm-title");
            row = row.child(
                div()
                    .class("fs-llm-side")
                    .w(side)
                    .h(height)
                    .child(div().class("fs-llm-sidehead").h(HEAD).child(heading))
                    .child(tree),
            );
        }
        row
    }

    fn question_dock(&self, width: f32, height: f32, waiting: usize) -> Div {
        let heading = self.labels.waiting.replacen("{}", &waiting.to_string(), 1);
        div()
            .class("fs-llm-dock")
            .w(width)
            .h(height)
            .flex_shrink_0()
            .child(text(heading).class("fs-llm-title"))
            .child(
                Scroll::new()
                    .w(width)
                    .h((height - 48.0).max(40.0))
                    .content(self.dock.build()),
            )
    }

    fn header(&self) -> Div {
        let mut head = div()
            .class("fs-llm-head")
            .child(text(self.source.title()).class("fs-llm-title"));
        if let Some(usage) = self.source.usage() {
            head = head.child(text(usage).class("fs-llm-meta").no_wrap());
        }
        if self.compact {
            let n = self.tree.counts().total();
            if n > 0 {
                let line = self
                    .labels
                    .tree
                    .needs_attention
                    .replacen("{}", &n.to_string(), 1);
                head = head.child(text(line).class("fs-llm-attention").no_wrap());
            }
        }
        match self.source.stop_control() {
            Some(control) => head.child(self.stop(&control)),
            None => head,
        }
    }

    fn stop(&self, control: &StopControl) -> Div {
        let l = &self.labels;
        let slot = div().class("fs-llm-stop");
        match control.state {
            StopState::Ready => {
                let id = format!("{}-stop", self.id);
                let this = self.clone();
                slot.child(
                    ButtonBuilder::with_key(id.clone(), control.label.clone())
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Small)
                        .on_click(move |_| this.lock_sink_emit(&Outcome::StopRequested))
                        .build_component()
                        .id(&id),
                )
            }
            StopState::Stopping => slot.child(text(l.stopping.clone()).class("fs-llm-meta")),
            StopState::Stopped => slot.child(text(l.stopped.clone()).class("fs-llm-attention")),
            _ => slot.child(text(l.stop_unavailable.clone()).class("fs-llm-meta")),
        }
    }
}
