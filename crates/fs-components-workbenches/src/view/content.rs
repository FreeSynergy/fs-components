//! The content area: the bar (back, path, actions), a notice, and the document or its editor.
//!
//! Pattern: Strategy by state ([`Shown`], editing or not) — page, editor, failure or nothing.

use blinc_cn::cn;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::{LinkHandler, TextView};

use super::actions::Actions;
use super::editor::editor;
use crate::reader::{Reader, Shown};

/// The area right of the sidebar.
pub(crate) fn content(reader: &Reader, actions: &Actions) -> Div {
    let mut column = div()
        .flex_col()
        .w_full()
        .flex_1()
        .child(bar(reader, actions));
    if let Some(notice) = reader.notice() {
        column = column.child(div().class("fs-rd-notice").child(text(notice)));
    }
    if let Some(editing) = reader.editing() {
        return column.child(editor(editing, actions));
    }
    let page = match reader.shown() {
        Shown::Nothing => div()
            .class("fs-rd-page")
            .w_full()
            .child(text("Nothing to show.")),
        Shown::Failed { reason, .. } => div().class("fs-rd-page").w_full().child(
            div()
                .class("fs-rd-failed")
                .child(text(format!("This page could not be read: {reason}"))),
        ),
        Shown::Page { document, .. } => {
            let follow = actions.follow.clone();
            let handler: LinkHandler = std::sync::Arc::new(move |href| follow(href));
            div().class("fs-rd-page").w_full().child(
                TextView::new(document.blocks.clone())
                    .on_link(handler)
                    .build(),
            )
        }
    };
    column.child(
        div()
            .flex_1()
            .w_full()
            .child(scroll().w_full().h_full().content(page)),
    )
}

/// Back, the path, and the buttons Edit and Export where the source offers them.
fn bar(reader: &Reader, actions: &Actions) -> Div {
    let back = actions.back.clone();
    let mut bar = div().class("fs-rd-bar").child(
        cn::button("Back")
            .disabled(!reader.can_go_back())
            .on_click(move |_| back()),
    );
    bar = bar.child(
        div()
            .flex_grow()
            .child(text(reader.current_path().unwrap_or("")).class("fs-rd-path")),
    );
    if reader.can_edit() && reader.editing().is_none() && reader.current_path().is_some() {
        let edit = actions.edit.clone();
        bar = bar.child(cn::button("Edit").on_click(move |_| edit()));
    }
    if reader.can_export() && reader.editing().is_none() && reader.current_path().is_some() {
        let export = actions.export.clone();
        bar = bar.child(cn::button("Export").on_click(move |_| export()));
    }
    bar
}
