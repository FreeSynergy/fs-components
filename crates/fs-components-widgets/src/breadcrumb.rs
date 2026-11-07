//! `breadcrumb`: the way to where the person is, each step a thing to press.
//!
//! Pattern: Adapter over plain Blinc elements, with a pure plan ([`plan`]). The widget has
//! no words and no colours of its own and does not know what a step means. A step is a plain element
//! and a tab stop, so it may be drawn inside a live area and the window turns Enter and Space on it
//! into its click. The last step is where the person is: it is not a thing to press.

use std::sync::Arc;

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use crate::Element;
use crate::list_view::fit_cell;

/// About how wide a step is at most, in pixels; a longer label is cut with an ellipsis.
const MAX_STEP: f32 = 220.0;
/// What the padding and the separator of one step take, about.
const STEP_ROOM: f32 = 44.0;
/// What a character takes, about: only used to decide how many steps fit.
const CHAR_WIDTH: f32 = 8.0;

/// One step of the way.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crumb {
    /// Handed back when the step is pressed.
    pub key: String,
    /// What is shown.
    pub label: String,
}

impl Crumb {
    /// A step.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// What to draw: the steps by index, `None` standing for the ones left out.
pub fn plan(crumbs: &[Crumb], width: f32) -> Vec<Option<usize>> {
    let width_of = |c: &Crumb| {
        #[allow(clippy::cast_precision_loss)]
        let wanted = c.label.chars().count() as f32 * CHAR_WIDTH;
        wanted.min(MAX_STEP) + STEP_ROOM
    };
    let Some(last) = crumbs.len().checked_sub(1) else {
        return Vec::new();
    };
    // The first and last always; then from the end as many as fit.
    let mut used = width_of(&crumbs[last]);
    if last > 0 {
        used += width_of(&crumbs[0]) + STEP_ROOM;
    }
    let mut from = last;
    while from > 1 && used + width_of(&crumbs[from - 1]) <= width {
        from -= 1;
        used += width_of(&crumbs[from]);
    }
    let mut shown = Vec::new();
    if last > 0 {
        shown.push(Some(0));
    }
    if from > 1 {
        shown.push(None);
    }
    shown.extend((from..=last).map(Some));
    shown
}

/// Called with the key of the step pressed.
pub type OnChoose = Arc<dyn Fn(&str) + Send + Sync>;

/// The breadcrumb.
#[must_use]
pub struct Breadcrumb {
    crumbs: Vec<Crumb>,
    name: String,
    separator: String,
    hidden: String,
    on_choose: Option<OnChoose>,
}

impl Breadcrumb {
    /// A way of these steps; element ids are `<name>-crumb-<index>` and `<name>-crumb-hidden`.
    pub fn new(crumbs: Vec<Crumb>, name: impl Into<String>) -> Self {
        Self {
            crumbs,
            name: name.into(),
            separator: "/".into(),
            hidden: "\u{2026}".into(),
            on_choose: None,
        }
    }

    /// Report the key of the step pressed.
    pub fn on_choose(mut self, handler: OnChoose) -> Self {
        self.on_choose = Some(handler);
        self
    }

    /// The element, one line within `width`.
    pub fn build(&self, width: f32) -> impl Element + use<> {
        let mut row = div()
            .class("fs-crumbs")
            .flex_row()
            .items_center()
            .w(width)
            .overflow_clip();
        let shown = plan(&self.crumbs, width);
        // A left-out step leads to the last of the steps it stands for.
        let first_after_gap = shown.iter().flatten().nth(1).copied().unwrap_or(0);
        for (place, step) in shown.iter().enumerate() {
            if place > 0 {
                row = row.child(
                    div()
                        .class("fs-crumb-gap")
                        .flex_shrink_0()
                        .child(text(self.separator.clone()).no_wrap().no_cursor()),
                );
            }
            row = row.child(match step {
                Some(index) => self.step(*index, width),
                None => self.press(
                    format!("{}-crumb-hidden", self.name),
                    &self.crumbs[first_after_gap.saturating_sub(1)].key,
                    &self.hidden,
                ),
            });
        }
        row
    }

    fn step(&self, index: usize, width: f32) -> Div {
        let crumb = &self.crumbs[index];
        let label = fit_cell(&crumb.label, MAX_STEP.min(width));
        let id = format!("{}-crumb-{index}", self.name);
        if index + 1 == self.crumbs.len() {
            return div()
                .id(id)
                .class("fs-crumb-here")
                .flex_shrink_0()
                .child(text(label).no_wrap().no_cursor());
        }
        self.press(id, &crumb.key, &label)
    }

    fn press(&self, id: String, key: &str, label: &str) -> Div {
        let mut step = div()
            .id(id)
            .class("fs-crumb")
            .flex_shrink_0()
            .attr("tabindex", "0")
            .child(text(label).no_wrap().no_cursor());
        if let Some(handler) = &self.on_choose {
            let (handler, key) = (Arc::clone(handler), key.to_string());
            step = step.on_click(move |_| handler(&key));
        }
        step
    }
}

/// CSS of the breadcrumb: only theme variables.
pub const STYLES: &str = r"
.fs-crumbs { gap: var(--space-1); min-height: 28px; font-size: var(--text-sm); }
.fs-crumb { padding: var(--space-1) var(--space-2); border-radius: var(--radius-default); cursor: pointer; color: var(--accent); }
.fs-crumb:hover { background: var(--accent-subtle); }
.fs-crumb-here { padding: var(--space-1) var(--space-2); color: var(--text-primary); font-weight: 600; }
.fs-crumb-gap { color: var(--text-secondary); }
";
