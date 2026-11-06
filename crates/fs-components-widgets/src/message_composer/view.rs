//! The composer as elements.
//!
//! Pattern: Adapter over `blinc_cn` (text area, buttons). The text area sits outside the part
//! that is drawn again; the attachment list, the notes and the buttons are drawn again after a
//! change of state.

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_cn::components::textarea::textarea;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;
use blinc_layout::widgets::text_area::SharedTextAreaState;

use super::handle::MessageComposer;
use crate::Element;

impl MessageComposer {
    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let (top, bottom) = (self.clone(), self.clone());
        let typed = self.clone();
        let area: &SharedTextAreaState = &self.area;
        let field = textarea(area)
            .w_full()
            .rows(if self.compact { 2 } else { 5 })
            .placeholder(self.labels.placeholder.clone())
            .on_change(move |value| typed.typed(value));
        div()
            .class("fs-mc")
            .id(&self.id)
            .child(self.live.view(move || top.notes()))
            .child(div().class("fs-mc-area").child(field))
            .child(self.live.view(move || bottom.buttons()))
    }

    fn notes(&self) -> Div {
        let inner = self.lock();
        let l = &self.labels;
        let mut col = div().class("fs-mc-notes");
        if !inner.attachments.is_empty() {
            let mut row = div().class("fs-mc-files");
            for a in &inner.attachments {
                let id = format!("{}-remove-{}", self.id, a.key);
                let (this, key) = (self.clone(), a.key.clone());
                let name = if a.detail.is_empty() {
                    a.name.clone()
                } else {
                    format!("{} ({})", a.name, a.detail)
                };
                row = row.child(
                    div()
                        .class("fs-mc-file")
                        .child(text(name).class("fs-mc-file-name"))
                        .child(
                            ButtonBuilder::with_key(id.clone(), l.remove.clone())
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Small)
                                .on_click(move |_| {
                                    this.remove_attachment(&key);
                                })
                                .build_component()
                                .id(&id),
                        ),
                );
            }
            col = col.child(row);
        }
        if let Some(why) = inner.state.error() {
            let line = l.failed.replacen("{}", why, 1);
            col = col.child(
                div()
                    .class("fs-mc-failed")
                    .child(text(line).class("fs-mc-note")),
            );
        }
        if let Some(why) = inner
            .job
            .as_ref()
            .and_then(|j| j.error.lock().ok().and_then(|e| e.clone()))
        {
            let line = l.draft_failed.replacen("{}", &why, 1);
            col = col.child(
                div()
                    .class("fs-mc-failed")
                    .child(text(line).class("fs-mc-note")),
            );
        }
        if let Some(note) = &inner.note {
            col = col.child(text(note.clone()).class("fs-mc-note"));
        }
        col
    }

    fn buttons(&self) -> Div {
        let l = &self.labels;
        let sending = self.is_sending();
        let size = if self.compact {
            ButtonSize::Small
        } else {
            ButtonSize::Medium
        };
        let id = format!("{}-send", self.id);
        let this = self.clone();
        let word = if sending { &l.sending } else { &l.send };
        let send = ButtonBuilder::with_key(id.clone(), word.clone())
            .variant(ButtonVariant::Primary)
            .size(size)
            .disabled(sending)
            .on_click(move |_| {
                this.send();
            })
            .build_component()
            .id(&id);
        let mut row = div().class("fs-mc-buttons");
        if self.attach {
            let id = format!("{}-attach", self.id);
            let this = self.clone();
            row = row.child(
                ButtonBuilder::with_key(id.clone(), l.attach.clone())
                    .variant(ButtonVariant::Ghost)
                    .size(size)
                    .on_click(move |_| {
                        this.request_attachment();
                    })
                    .build_component()
                    .id(&id),
            );
        }
        row.child(send)
    }
}
