//! The page head: the way to the page, the title, the language switch, the state.
//!
//! Pattern: pure functions from the [`PageInfo`] of a page to elements; what is clicked is reported through `Actions`.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::icon::{Glyph, Tone, icon};

use crate::page_info::{PageInfo, PageState, WikiLabels};
use crate::view::Actions;

/// The language switch: the languages as chips, the shown one filled. Empty (a spacer-free `div`) with fewer than two.
pub(super) fn lang_switch(info: &PageInfo, labels: &WikiLabels, actions: &Actions) -> Div {
    let mut row = div().class("fs-wk-langs").flex_row().items_center();
    if info.langs.len() < 2 {
        return row;
    }
    row = row.child(icon(Glyph::Language, 16.0, Tone::Muted));
    for lang in &info.langs {
        let follow = actions.follow.clone();
        let href = lang.href.clone();
        let chip = div()
            .id(format!("fs-wk-lang-{}", lang.code))
            .class("fs-wk-lang")
            .attr("title", &labels.language)
            .on_click(move |_| follow(&href))
            .child(text(&lang.name).no_cursor());
        row = row.child(if lang.current {
            chip.class("fs-wk-lang-on")
        } else {
            chip
        });
    }
    row
}

/// The state: a dot and a word (colour plus word, never colour alone).
pub(super) fn state_chip(info: &PageInfo, labels: &WikiLabels) -> Div {
    let (class, word) = match info.state {
        PageState::Current => ("fs-wk-dot-ok", &labels.state_current),
        PageState::Outdated => ("fs-wk-dot-old", &labels.state_outdated),
        PageState::Fallback => ("fs-wk-dot-other", &labels.state_fallback),
    };
    div()
        .class("fs-wk-state")
        .flex_row()
        .items_center()
        .child(div().class("fs-wk-dot").class(class))
        .child(text(word).no_cursor())
}

/// The sentence under the head when the page is outdated or in another language.
pub(super) fn notice_band(info: &PageInfo) -> Option<Div> {
    let note = info.notice.as_deref()?;
    Some(
        div()
            .class("fs-wk-band")
            .flex_row()
            .items_center()
            .child(icon(Glyph::Info, 18.0, Tone::Accent))
            .child(text(note).no_cursor()),
    )
}

/// The way to the page: the folders of its path, then the page.
pub(super) fn crumbs(path: &str, title: &str) -> Div {
    let mut row = div().class("fs-wk-crumbs").flex_row().items_center();
    let parts: Vec<&str> = path.split('/').collect();
    for part in &parts[..parts.len().saturating_sub(1)] {
        row = row
            .child(text(*part).class("fs-wk-crumb").no_cursor())
            .child(text("/").class("fs-wk-crumb-sep").no_cursor());
    }
    row.child(text(title).class("fs-wk-crumb-last").no_cursor())
}
