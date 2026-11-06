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
/// Below this width the side panel gives way to the compact form (the stop must stay reachable).
const SIDE_MIN_WIDTH: f32 = 760.0;
const COMPOSER: f32 = 214.0;
const COMPOSER_COMPACT: f32 = 160.0;
const QUESTION: f32 = 270.0;
const MIN_QUESTION: f32 = 130.0;
/// The smallest layout the chat makes; a window smaller than this clips it.
pub const MIN_SIZE: (f32, f32) = (300.0, 360.0);
/// The width of the place of the stop control.
const STOP_W: f32 = 172.0;

impl LlmChat {
    /// The element to place in a layout, filling `width` x `height` (at least [`MIN_SIZE`]).
    pub fn build(&self, width: f32, height: f32) -> impl Element + use<> {
        let this = self.clone();
        let (w, h) = (width.max(MIN_SIZE.0), height.max(MIN_SIZE.1));
        div()
            .h(h)
            .child(self.layout.region("layout", move || this.arrange(w, h)))
    }

    #[allow(clippy::cast_precision_loss)]
    fn arrange(&self, width: f32, height: f32) -> Div {
        let compact = self.compact || width < SIDE_MIN_WIDTH;
        let side = if compact { 0.0 } else { SIDE };
        let main = width - side;
        let composer = if compact { COMPOSER_COMPACT } else { COMPOSER };
        let questions = self.dock.calls().len();
        let dock = if questions == 0 {
            0.0
        } else {
            let room = height - HEAD - composer - 60.0;
            (questions as f32 * QUESTION).min(room).max(MIN_QUESTION)
        };
        let list = (height - HEAD - composer - dock).max(40.0);
        let this = self.clone();
        let mut column = div()
            .class("fs-llm-main")
            .relative()
            .w(main)
            .h(height)
            .child(
                div().h(HEAD).child(
                    self.parts
                        .region("header", move || this.header(main, compact)),
                ),
            )
            .child(self.list.build(main, list));
        if questions > 0 {
            column = column.child(self.question_dock(main, dock, questions));
        }
        column = column.child(self.composer.build());
        // last, so that nothing is drawn over it and nothing can take its clicks
        column = column.child(self.stop_place());
        let mut row = div()
            .class("fs-llm")
            .flex_row()
            .w(width)
            .h(height)
            .child(column);
        if !compact {
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
            .id(format!("{}-dockbox", self.id))
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

    /// Title, use, and what needs attention, clipped to the room left of the stop.
    fn header(&self, main: f32, compact: bool) -> Div {
        let room = (main - STOP_W - 24.0).max(40.0);
        let mut line = div()
            .class("fs-llm-headtext")
            .w(room)
            .child(text(self.source.title()).class("fs-llm-title").no_wrap());
        if let Some(usage) = self.source.usage() {
            line = line.child(text(usage).class("fs-llm-meta").no_wrap());
        }
        if compact {
            let n = self.tree.counts().total();
            if n > 0 {
                let words = self
                    .labels
                    .tree
                    .needs_attention
                    .replacen("{}", &n.to_string(), 1);
                line = line.child(text(words).class("fs-llm-attention").no_wrap());
            }
            let working = self.tree.working();
            if working > 0 {
                let words = self.labels.working.replacen("{}", &working.to_string(), 1);
                line = line.child(text(words).class("fs-llm-meta").no_wrap());
            }
        }
        div().class("fs-llm-head").w(main).child(line)
    }

    /// The stop control: a fixed place at the top right of the column.
    fn stop_place(&self) -> Div {
        let this = self.clone();
        div()
            .absolute()
            .right(0.0)
            .top(0.0)
            .w(STOP_W + 12.0)
            .child(self.parts.region("stop", move || {
                let slot = div()
                    .class("fs-llm-stop")
                    .w(STOP_W)
                    .absolute()
                    .right(6.0)
                    .top(5.0);
                match this.source.stop_control() {
                    Some(control) => slot.child(this.stop(&control)),
                    None => slot,
                }
            }))
    }

    fn stop(&self, control: &StopControl) -> Div {
        let l = &self.labels;
        let slot = div().class("fs-llm-stopbox");
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
