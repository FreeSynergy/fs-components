//! The lightbox as elements.
//!
//! Pattern: Adapter — the state of the [`Viewer`](super::Viewer) in, Blinc layout and Blinc's
//! image out. The picture is fitted inside the stage (`contain`), never cropped or stretched.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::image::img;
use blinc_layout::prelude::text;

use super::bindings::Command;
use super::handle::LightBox;
use super::model::{Media, Viewer};
use crate::Element;
use crate::keys::bind_keys;

impl LightBox {
    /// The element to place in a layout (it fills the box it is given).
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        let root = div()
            .id(&self.id)
            .class("fs-lb-root")
            .w_full()
            .h_full()
            .child(self.live.view(move || this.render()));
        if self.bindings.is_none() {
            return root;
        }
        let this = self.clone();
        bind_keys(
            root,
            Arc::new(move |event| this.press(&event) != super::model::Outcome::Ignored),
        )
    }

    fn render(&self) -> Div {
        self.read(|viewer| {
            let l = &self.labels;
            let (at, of) = viewer.position();
            let count =
                l.counter
                    .replacen("{}", &at.to_string(), 1)
                    .replacen("{}", &of.to_string(), 1);
            let bar = div()
                .class("fs-lb-bar")
                .child(text(count).class("fs-lb-count").no_wrap())
                .child(self.button("close", &l.close, Command::Close, false));
            let at_first = !viewer.wraps() && at <= 1;
            let at_last = !viewer.wraps() && at >= of;
            let body = div()
                .class("fs-lb-body")
                .child(self.button("prev", &l.prev, Command::Prev, at_first))
                .child(self.stage(viewer))
                .child(self.button("next", &l.next, Command::Next, at_last));
            let caption = viewer
                .current()
                .map_or(String::new(), |m| m.caption.clone());
            div().class("fs-lb").child(bar).child(body).child(
                text(caption)
                    .class("fs-lb-caption")
                    .id(format!("{}-caption", self.id)),
            )
        })
    }

    fn stage(&self, viewer: &Viewer) -> Div {
        let stage = div().class("fs-lb-stage");
        let l = &self.labels;
        let note_id = format!("{}-note", self.id);
        match viewer.current() {
            None => stage.child(
                text(l.empty.clone())
                    .class("fs-lb-note")
                    .id(note_id.clone()),
            ),
            Some(m) if !m.picture => stage.child(
                text(l.not_a_picture.clone())
                    .class("fs-lb-note")
                    .id(note_id.clone()),
            ),
            Some(m) if !loadable(m) => stage.child(
                text(l.unavailable.clone())
                    .class("fs-lb-note")
                    .id(note_id.clone()),
            ),
            Some(m) => stage.child(img(m.source.clone()).w_full().h_full().contain()),
        }
    }

    fn button(
        &self,
        tag: &str,
        word: &str,
        command: Command,
        disabled: bool,
    ) -> impl Element + use<> {
        let id = format!("{}-{tag}", self.id);
        let this = self.clone();
        ButtonBuilder::with_key(id.clone(), word.to_string())
            .variant(ButtonVariant::Secondary)
            .size(ButtonSize::Medium)
            .disabled(disabled)
            .on_click(move |_| {
                this.command(command);
            })
            .build_component()
            .id(&id)
    }
}

/// Can the picture be expected to load?
fn loadable(m: &Media) -> bool {
    crate::picture::loadable(&m.source)
}
