//! One task as a line of the day list: a round checkbox, the title, a small line under it, a chip on the right; a line
//! can open in place and show its details under itself.
//!
//! Pattern: Adapter — the data of one task in, an element out; the box and the title are two click targets with two
//! requests (`tick`, `open`), so ticking never opens and opening never ticks.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::chip::{Chip, chip_element};
use super::events::{Events, tell};
use super::model::Detail;

/// One line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskLine {
    /// Names the task for the host.
    pub key: String,
    /// The title.
    pub title: String,
    /// A small line under the title (the project, the people).
    pub sub: Option<String>,
    /// A chip on the right (the due date).
    pub chip: Option<Chip>,
    /// The box is ticked: the task is finished.
    pub checked: bool,
    /// Drawn pale: it cannot be started; `reason` says why in small text.
    pub muted: bool,
    /// Why it waits.
    pub reason: Option<String>,
    /// The details are shown under the line.
    pub open: bool,
    /// The details.
    pub details: Vec<Detail>,
}

impl TaskLine {
    /// A line with a title.
    pub fn new(key: &str, title: &str) -> Self {
        Self {
            key: key.into(),
            title: title.into(),
            ..Self::default()
        }
    }

    /// The element; `id` prefixes its element ids.
    pub fn build(&self, id: &str, events: &Events) -> Div {
        let (tick, open) = (events.tick.clone(), events.open.clone());
        let (k1, k2) = (self.key.clone(), self.key.clone());
        let mut boxed = div()
            .class("fs-plan-check")
            .id(format!("{id}-{}-check", self.key));
        if self.checked {
            boxed = boxed
                .class("fs-plan-check-on")
                .child(text("\u{2713}").class("fs-plan-check-mark"));
        }
        let mut body = div().class("fs-plan-line-body").flex_col().flex_1();
        body = body.child(text(self.title.clone()).class("fs-plan-line-title"));
        if let Some(reason) = &self.reason {
            body = body.child(text(reason.clone()).class("fs-plan-line-reason"));
        } else if let Some(sub) = &self.sub {
            body = body.child(text(sub.clone()).class("fs-plan-line-sub"));
        }
        let mut head = div().class("fs-plan-line-head").flex_row().w_full();
        head = head.child(boxed.on_click(move |_| tell(tick.as_ref(), &k1)));
        let mut main = div()
            .class("fs-plan-line-main")
            .flex_row()
            .flex_1()
            .id(format!("{id}-{}", self.key));
        main = main.child(body);
        if let Some(chip) = &self.chip {
            main = main.child(chip_element(chip));
        }
        head = head.child(main.on_click(move |_| tell(open.as_ref(), &k2)));
        let mut line = div().class("fs-plan-line").flex_col().w_full();
        if self.muted {
            line = line.class("fs-plan-muted");
        }
        line = line.child(head);
        if self.open {
            let mut more = div().class("fs-plan-line-more").flex_col();
            for d in &self.details {
                more = more.child(
                    div()
                        .class("fs-plan-detail")
                        .flex_row()
                        .child(text(d.label.clone()).class("fs-plan-detail-label"))
                        .child(text(d.value.clone()).class("fs-plan-detail-value")),
                );
            }
            line = line.child(more);
        }
        line
    }
}
