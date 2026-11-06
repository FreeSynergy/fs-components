//! The preview as elements.
//!
//! Pattern: Adapter — our model in, plain Blinc elements out. The scroller is new with every build, so
//! another file starts at the top (Blinc keys a plain scroll by its call site).

use std::sync::{Arc, Mutex};

use blinc_layout::div::{Div, div};
use blinc_layout::image::img;
use blinc_layout::prelude::text;
use blinc_layout::widgets::scroll::{Scroll, ScrollPhysics};

use super::safe::SafeText;
use crate::Element;

/// One fact about the file, in the caller's words.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fact {
    /// What it says.
    pub label: String,
    /// The value.
    pub value: String,
}

impl Fact {
    /// A fact.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// What the preview shows of the content.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PreviewContent {
    /// Nothing but the facts.
    #[default]
    Nothing,
    /// Text checked by [`safe_text`](super::safe_text), and what to say if some was left out.
    Text(SafeText, String),
    /// A picture checked by [`picture_fits`](super::picture_fits), as a data address.
    Picture(String),
    /// The content is not shown, and this says why.
    Note(String),
    /// Reading it was refused, and this says why.
    Refused(String),
}

/// The preview.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FilePreview {
    /// The name.
    pub title: String,
    /// Markings that must be seen at once, in the caller's words.
    pub marks: Vec<String>,
    /// Facts.
    pub facts: Vec<Fact>,
    /// The content.
    pub content: PreviewContent,
}

impl FilePreview {
    /// A preview of this name.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// The element, filling `width` x `height`, with `prefix` as the start of its element ids.
    pub fn build(&self, prefix: &str, width: f32, height: f32) -> impl Element + use<> {
        let mut head = div()
            .class("fs-preview-head")
            .flex_col()
            .w(width)
            .flex_shrink_0();
        head = head.child(
            div()
                .id(format!("{prefix}-title"))
                .class("fs-preview-title")
                .w(width)
                .child(text(self.title.clone())),
        );
        if !self.marks.is_empty() {
            let mut row = div()
                .id(format!("{prefix}-marks"))
                .class("fs-preview-marks")
                .flex_row()
                .flex_wrap();
            for mark in &self.marks {
                row = row.child(div().class("fs-preview-mark").child(text(mark.clone())));
            }
            head = head.child(row);
        }
        let mut page = div().flex_col().w(width);
        for fact in &self.facts {
            page = page.child(
                div()
                    .class("fs-preview-fact")
                    .flex_row()
                    .w(width)
                    .child(
                        div()
                            .class("fs-preview-label")
                            .child(text(fact.label.clone())),
                    )
                    .child(
                        div()
                            .class("fs-preview-value")
                            .flex_1()
                            .child(text(fact.value.clone())),
                    ),
            );
        }
        page = page.child(self.content(prefix, width));
        let physics = Arc::new(Mutex::new(ScrollPhysics::default()));
        div()
            .class("fs-preview")
            .flex_col()
            .w(width)
            .h(height)
            .child(head)
            .child(
                div().flex_1().min_h(0.0).w(width).child(
                    Scroll::with_physics(physics)
                        .w(width)
                        .h_full()
                        .content(page),
                ),
            )
    }

    fn content(&self, prefix: &str, width: f32) -> Div {
        let id = format!("{prefix}-content");
        match &self.content {
            PreviewContent::Nothing => div().id(id),
            PreviewContent::Note(note) => div()
                .id(id)
                .class("fs-preview-note")
                .child(text(note.clone())),
            PreviewContent::Refused(why) => div()
                .id(id)
                .class("fs-preview-refused")
                .child(text(why.clone())),
            PreviewContent::Picture(address) => div()
                .id(id)
                .w(width)
                .h(width)
                .child(img(address.clone()).w_full().h_full().contain()),
            PreviewContent::Text(safe, note) => {
                let mut area = div().id(id).class("fs-preview-text").flex_col().w(width);
                for line in &safe.lines {
                    area = area.child(text(line.clone()).no_wrap());
                }
                if safe.cut {
                    area = area.child(div().class("fs-preview-note").child(text(note.clone())));
                }
                area.overflow_clip()
            }
        }
    }
}
