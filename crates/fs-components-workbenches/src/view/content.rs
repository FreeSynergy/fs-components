//! The content area: the bar (back, path, actions), a notice, and the document or its editor.
//!
//! Pattern: Strategy by state ([`Shown`], editing or not) — document, editor, failure or nothing.
//! Every column on the way down has `min_h(0)`, so the document gets a bounded height and the
//! scroll area can scroll and clip (without it a long document grows the layout without limit).

use blinc_cn::cn;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::{Scroll, SharedScrollPhysics, scroll};
use blinc_layout::widgets::text_area::SharedTextAreaState;
use fs_components_widgets::{LinkHandler, TextView};

use super::actions::Actions;
use super::editor::editor;
use crate::reader::{Reader, Shown};

/// The area right of the sidebar.
pub(crate) fn content(
    reader: &Reader,
    actions: &Actions,
    physics: Option<&SharedScrollPhysics>,
    area: Option<&SharedTextAreaState>,
    rows: usize,
) -> Div {
    let mut column = div()
        .flex_col()
        .w_full()
        .flex_1()
        .min_h(0.0)
        .child(bar(reader, actions));
    if let Some(notice) = reader.notice() {
        column = column.child(div().class("fs-rd-notice").child(text(notice)));
    }
    if let Some(editing) = reader.editing() {
        return column.child(editor(editing, reader.labels(), actions, area, rows));
    }
    let page = div().class("fs-rd-page").w_full();
    let page = match reader.shown() {
        Shown::Document { document, .. } => {
            let follow = actions.follow.clone();
            let handler: LinkHandler = std::sync::Arc::new(move |href| follow(href));
            page.child(
                TextView::new(document.blocks.clone())
                    .on_link(handler)
                    .build(),
            )
        }
        Shown::Failed { reason, .. } => {
            page.child(div().class("fs-rd-failed").child(text(reason.clone())))
        }
        _ => page.child(text(&reader.labels().nothing_open)),
    };
    column.child(
        div()
            .flex_1()
            .min_h(0.0)
            .w_full()
            .child(scroller(physics).w_full().h_full().content(page)),
    )
}

/// Back, the path, and the buttons Edit and Export where the source offers them.
fn bar(reader: &Reader, actions: &Actions) -> Div {
    let labels = reader.labels();
    let back = actions.back.clone();
    let mut bar = div().class("fs-rd-bar").child(
        cn::button(labels.back.clone())
            .disabled(!reader.can_go_back())
            .on_click(move |_| back()),
    );
    bar = bar.child(
        div()
            .flex_grow()
            .child(text(reader.current_path().unwrap_or("")).class("fs-rd-path")),
    );
    if reader.editing().is_none() && reader.can_edit() {
        let edit = actions.edit.clone();
        bar = bar.child(cn::button(labels.edit.clone()).on_click(move |_| edit()));
    }
    if reader.editing().is_none() && reader.can_export() {
        let export = actions.export.clone();
        bar = bar.child(cn::button(labels.export.clone()).on_click(move |_| export()));
    }
    bar
}

/// The scroll area; a picture shares its physics to scroll and to read the bounds.
fn scroller(physics: Option<&SharedScrollPhysics>) -> Scroll {
    physics.map_or_else(scroll, |p| Scroll::with_physics(p.clone()))
}
