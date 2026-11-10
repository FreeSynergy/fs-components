//! The toolbar: one button per command, plus undo and redo.
//!
//! Pattern: View function — state to elements; a click only calls the handle, no gesture logic.

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;

use super::handle::EditorView;
use crate::editor::TextEditor;
use crate::model::EditCommand;

impl EditorView {
    fn button<F: Fn(&mut Self) + Send + Sync + 'static>(
        &self,
        slot: &str,
        label: &str,
        enabled: bool,
        act: F,
    ) -> impl ElementBuilder + use<F> {
        let id = format!("{}-{slot}", self.id);
        let this = self.clone();
        ButtonBuilder::with_key(id.clone(), label.to_owned())
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Small)
            .disabled(!enabled)
            .on_click(move |_| act(&mut this.clone()))
            .build_component()
            .id(&id)
    }

    fn command(
        &self,
        slot: &str,
        label: &str,
        command: EditCommand,
    ) -> impl ElementBuilder + use<> {
        self.button(slot, label, true, move |editor| {
            editor.apply(&command);
        })
    }

    /// The row of buttons.
    pub(super) fn toolbar(&self) -> Div {
        let l = &self.labels;
        let heading = |n: u8| l.heading.replacen("{level}", &n.to_string(), 1);
        let (can_undo, can_redo) = (self.can_undo(), self.can_redo());
        div()
            .flex_row()
            .flex_wrap()
            .gap(4.0)
            .w_full()
            .child(self.command("bold", &l.bold, EditCommand::Bold))
            .child(self.command("italic", &l.italic, EditCommand::Italic))
            .child(self.command("h1", &heading(1), EditCommand::Heading(1)))
            .child(self.command("h2", &heading(2), EditCommand::Heading(2)))
            .child(self.command("h3", &heading(3), EditCommand::Heading(3)))
            .child(self.command("list", &l.list, EditCommand::ListItem))
            .child(self.command("link", &l.link, EditCommand::Link(String::new())))
            .child(self.command("code", &l.code, EditCommand::Code))
            .child(self.command("quote", &l.quote, EditCommand::Quote))
            .child(self.button("undo", &l.undo, can_undo, |e| {
                e.undo();
            }))
            .child(self.button("redo", &l.redo, can_redo, |e| {
                e.redo();
            }))
    }
}
