//! The Notion-like layout: a calm sidebar, thin top line, a big title, a reading column of blocks, ticks at the edge.
//!
//! Pattern: Strategy (one of the layouts) — same three functions as the other layout.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use blinc_layout::text;
use blinc_layout::widgets::scroll::{Scroll, SharedScrollPhysics, scroll};
use fs_components_widgets::icon::{Glyph, Tone, icon};
use fs_components_widgets::{Block, LinkHandler, TextView};

use super::blocks::BlockEditor;
use super::confluence::canonical;
use super::head::{crumbs, lang_switch, notice_band, state_chip};
use super::outline::{body, ticks};
use super::rows::{Look, tree_scroll};
use super::tree::PageTree;
use crate::page_info::WikiLabels;
use crate::reader::{Reader, Shown};
use crate::view::{Actions, results_list};

/// The window: sidebar left, the page right.
pub(crate) fn notion(
    search: impl ElementBuilder + 'static,
    side: impl ElementBuilder + 'static,
    main: impl ElementBuilder + 'static,
    width: f32,
    height: f32,
) -> Div {
    div()
        .class("fs-wk-root")
        .w(width)
        .h(height)
        .flex_row()
        .child(
            div()
                .class("fs-wk-b-side")
                .w(264.0)
                .flex_shrink_0()
                .flex_col()
                .h_full()
                .min_h(0.0)
                .child(div().class("fs-wk-b-head").flex_col().child(search))
                .child(side),
        )
        .child(main)
}

/// The live part of the sidebar: a section heading and the tree (the search above it is the widget's).
pub(crate) fn side(
    reader: &Reader,
    tree: &PageTree,
    labels: &WikiLabels,
    actions: &Actions,
) -> Div {
    let list = if reader.term().is_empty() && reader.problem().is_none() {
        let selected = reader.current_path().map(|p| canonical(reader, p));
        tree_scroll(tree, selected.as_deref(), actions, Look::Quiet)
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
        .child(
            div().class("fs-wk-b-section").child(
                text(&labels.pages)
                    .class("fs-wk-b-section-text")
                    .no_cursor(),
            ),
        )
        .child(list)
}

/// The page: top line with the way and the buttons, the title, the language switch, the blocks.
pub(crate) fn main(
    reader: &Reader,
    labels: &WikiLabels,
    actions: &Actions,
    physics: Option<&SharedScrollPhysics>,
    blocks: Option<&BlockEditor>,
    report: &std::sync::Arc<dyn Fn(&str) + Send + Sync>,
) -> Div {
    let info = reader.page_info();
    let (path, document) = match reader.shown() {
        Shown::Document { path, document } => (canonical(reader, path), Some(document)),
        Shown::Failed { path, .. } => (path.clone(), None),
        _ => (String::new(), None),
    };
    let title = document.map_or(String::new(), |d| d.title.clone());
    let mut top = div()
        .class("fs-wk-b-top")
        .flex_row()
        .items_center()
        .w_full()
        .child(div().flex_grow().child(crumbs(&path, &title)))
        .child(lang_switch(&info, labels, actions));
    if reader.can_edit() {
        let (act, word, glyph) = if reader.editing().is_some() {
            (
                actions.save.clone(),
                reader.labels().save.clone(),
                Glyph::Check,
            )
        } else {
            (actions.edit.clone(), labels.edit.clone(), Glyph::Edit)
        };
        top = top.child(
            div()
                .class("fs-wk-btn")
                .flex_row()
                .items_center()
                .on_click(move |_| act())
                .child(icon(glyph, 16.0, Tone::Muted))
                .child(text(word).no_cursor()),
        );
    }
    top = top
        .child(icon(Glyph::History, 18.0, Tone::Muted))
        .child(icon(Glyph::Star, 18.0, Tone::Muted));
    let mut page = div().class("fs-wk-b-page").flex_col().w_full();
    if let Some(document) = document {
        page = page.child(page_icon(info.emoji.as_deref()));
        // While editing, the title is a big field above the blocks (it holds the leading heading); else plain text.
        match (reader.editing(), blocks) {
            (Some(_), Some(b)) => page = page.child(b.title_view(labels, report)),
            _ => page = page.child(text(&title).class("fs-wk-b-title").no_cursor()),
        }
        let meta = div()
            .class("fs-wk-b-meta")
            .flex_row()
            .items_center()
            .child(state_chip(&info, labels));
        page = page.child(meta);
        if let Some(band) = notice_band(&info) {
            page = page.child(band);
        }
        if let (Some(_), Some(blocks)) = (reader.editing(), blocks) {
            page = page.child(blocks.view(labels, report));
        } else {
            page = page.child(block_list(&body(document), actions));
        }
    } else if let Shown::Failed { reason, .. } = reader.shown() {
        page = page.child(div().class("fs-rd-failed").child(text(reason.clone())));
    } else {
        page = page.child(text(&reader.labels().nothing_open));
    }
    let scroller = physics.map_or_else(blinc_layout::widgets::scroll::scroll, |p| {
        Scroll::with_physics(p.clone())
    });
    let centred = div()
        .flex_row()
        .justify_center()
        .w_full()
        .child(div().w_full().max_w(720.0).child(page));
    let edge = document.map_or_else(div, ticks);
    let middle = div()
        .flex_col()
        .flex_1()
        .min_w(0.0)
        .min_h(0.0)
        .child(top)
        .child(
            div()
                .flex_1()
                .min_h(0.0)
                .w_full()
                .child(scroller.w_full().h_full().content(centred)),
        );
    div()
        .flex_row()
        .flex_1()
        .min_w(0.0)
        .h_full()
        .child(middle)
        .child(edge)
}

/// The big icon above the title: the emoji the page chose, else the icon of a page.
fn page_icon(emoji: Option<&str>) -> Div {
    let holder = div().class("fs-wk-b-icon").flex_row().items_center();
    match emoji {
        Some(e) => holder.child(text(e).size(64.0).no_cursor()),
        None => holder.child(icon(Glyph::FileText, 64.0, Tone::Kind)),
    }
}

/// The blocks of a page, one element each; a click on a block starts editing there.
fn block_list(blocks: &[Block], actions: &Actions) -> Div {
    let mut column = div().flex_col().w_full().gap(1.0);
    for (i, block) in blocks.iter().enumerate() {
        let edit = actions.block.clone();
        let follow = actions.follow.clone();
        let handler: LinkHandler = std::sync::Arc::new(move |href| follow(href));
        column = column.child(
            div()
                .class("fs-wk-block")
                .w_full()
                .on_click(move |_| edit(i))
                .child(
                    TextView::new(vec![block.clone()])
                        .reading()
                        .on_link(handler)
                        .build(),
                ),
        );
    }
    column
}
