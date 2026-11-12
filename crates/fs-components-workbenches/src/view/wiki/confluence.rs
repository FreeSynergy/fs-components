//! The Confluence-like layout: top bar, page tree left, page head and text in the middle, outline and history right.
//!
//! Pattern: Strategy (one of the layouts) — three pure functions: the sidebar, the main area, the frame around them.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use blinc_layout::text;
use blinc_layout::widgets::scroll::{Scroll, SharedScrollPhysics, scroll};
use blinc_layout::widgets::text_area::SharedTextAreaState;
use fs_components_widgets::icon::{Glyph, Tone, icon};
use fs_components_widgets::{LinkHandler, TextView};

use super::head::{crumbs, lang_switch, notice_band, state_chip};
use super::outline::{body, history_panel, outline_panel};
use super::rows::{Look, tree_scroll};
use super::tree::PageTree;
use crate::page_info::WikiLabels;
use crate::reader::{Reader, Shown};
use crate::view::{Actions, editor, editor_rows, results_list};

/// The whole window: bar on top, the two areas below.
pub(crate) fn confluence(
    search: impl ElementBuilder + 'static,
    side: impl ElementBuilder + 'static,
    main: impl ElementBuilder + 'static,
    labels: &WikiLabels,
    width: f32,
    height: f32,
) -> Div {
    let bar = div()
        .class("fs-wk-a-top")
        .flex_row()
        .items_center()
        .w_full()
        .child(
            div()
                .class("fs-wk-a-brand")
                .flex_row()
                .items_center()
                .child(icon(Glyph::FileText, 22.0, Tone::Accent))
                .child(text(&labels.program).no_cursor()),
        )
        .child(div().w(460.0).child(search));
    let below = div()
        .flex_row()
        .w_full()
        .flex_1()
        .min_h(0.0)
        .child(
            div()
                .class("fs-wk-a-side")
                .w(300.0)
                .flex_shrink_0()
                .flex_col()
                .h_full()
                .min_h(0.0)
                .child(side),
        )
        .child(main);
    div()
        .class("fs-wk-root")
        .w(width)
        .h(height)
        .flex_col()
        .child(bar)
        .child(below)
}

/// The left column: the name of the space, the tree (or the hits of a search).
pub(crate) fn side(
    reader: &Reader,
    tree: &PageTree,
    labels: &WikiLabels,
    actions: &Actions,
) -> Div {
    let count = labels
        .page_count
        .replacen("{}", &tree.pages().to_string(), 1);
    let head = div()
        .class("fs-wk-a-space")
        .flex_col()
        .child(text(&labels.pages).class("fs-wk-a-space-name").no_cursor())
        .child(text(count).class("fs-wk-quiet").no_cursor());
    let body = if reader.term().is_empty() && reader.problem().is_none() {
        let selected = reader.current_path().map(|p| canonical(reader, p));
        tree_scroll(tree, selected.as_deref(), actions, Look::Icons)
    } else {
        div().flex_1().min_h(0.0).w_full().child(
            scroll()
                .w_full()
                .h_full()
                .content(results_list(reader, actions)),
        )
    };
    div()
        .flex_col()
        .flex_1()
        .min_h(0.0)
        .w_full()
        .child(head)
        .child(body)
}

/// The path the tree knows the shown page by.
pub(super) fn canonical(reader: &Reader, shown: &str) -> String {
    reader
        .page_info()
        .canonical
        .unwrap_or_else(|| shown.to_string())
}

/// The middle and right: way to the page, head, text; outline and history right of it.
pub(crate) fn main(
    reader: &Reader,
    labels: &WikiLabels,
    actions: &Actions,
    physics: Option<&SharedScrollPhysics>,
    area: Option<&SharedTextAreaState>,
    height: f32,
) -> Div {
    let info = reader.page_info();
    let (path, document) = match reader.shown() {
        Shown::Document { path, document } => (path.clone(), Some(document)),
        Shown::Failed { path, .. } => (path.clone(), None),
        _ => (String::new(), None),
    };
    let title = document.map_or(String::new(), |d| d.title.clone());
    let mut bar = div()
        .class("fs-wk-a-crumbs")
        .flex_row()
        .items_center()
        .w_full()
        .child(
            div()
                .flex_grow()
                .child(crumbs(&canonical(reader, &path), &title)),
        );
    if reader.editing().is_none() && reader.can_edit() {
        let edit = actions.edit.clone();
        bar = bar.child(
            div()
                .class("fs-wk-btn")
                .flex_row()
                .items_center()
                .on_click(move |_| edit())
                .child(icon(Glyph::Edit, 16.0, Tone::Muted))
                .child(text(&labels.edit).no_cursor()),
        );
    }
    let mut page = div().class("fs-wk-a-page").flex_col().w_full();
    if let Some(editing) = reader.editing() {
        page = page.child(editor(
            editing,
            reader.labels(),
            actions,
            area,
            editor_rows(height),
        ));
    } else if let Some(document) = document {
        let follow = actions.follow.clone();
        let handler: LinkHandler = std::sync::Arc::new(move |href| follow(href));
        let meta = div()
            .class("fs-wk-a-meta")
            .flex_row()
            .items_center()
            .child(lang_switch(&info, labels, actions))
            .child(state_chip(&info, labels));
        page = page
            .child(text(&title).class("fs-wk-a-title").no_cursor())
            .child(meta);
        if let Some(band) = notice_band(&info) {
            page = page.child(band);
        }
        page = page.child(TextView::new(body(document)).on_link(handler).build());
    } else if let Shown::Failed { reason, .. } = reader.shown() {
        page = page.child(div().class("fs-rd-failed").child(text(reason.clone())));
    } else {
        page = page.child(text(&reader.labels().nothing_open));
    }
    let scroller = physics.map_or_else(blinc_layout::widgets::scroll::scroll, |p| {
        Scroll::with_physics(p.clone())
    });
    let middle = div()
        .flex_col()
        .flex_1()
        .min_w(0.0)
        .min_h(0.0)
        .child(bar)
        .child(
            div()
                .flex_1()
                .min_h(0.0)
                .w_full()
                .child(scroller.w_full().h_full().content(page)),
        );
    let mut right = div()
        .class("fs-wk-a-right")
        .w(270.0)
        .flex_shrink_0()
        .flex_col()
        .h_full()
        .child(document.map_or_else(div, |d| outline_panel(d, labels)))
        .child(div().class("fs-wk-a-sep").w_full());
    right = right.child(history_panel(&info, labels));
    div()
        .flex_row()
        .flex_1()
        .min_w(0.0)
        .h_full()
        .child(middle)
        .child(right)
}
