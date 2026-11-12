//! The tabs above the bar of the way: the folder that is open, and a plus for another one.
//!
//! Pattern: Adapter (state to elements) — one tab is shown (the place the person is in); more tabs are not built
//! yet, so the plus is shown dimmed and says so (ADR 0066: controls are visible, locked, never hidden).

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_widgets::icon::{Glyph, Tone, icon};

use super::actions::icon_button;
use crate::file_manager::side::Place;
use crate::file_manager::state::FilesState;

/// How high the row of tabs is.
pub(in crate::file_manager) const TABS_HEIGHT: f32 = 38.0;

/// The row of tabs.
pub(in crate::file_manager) fn tabs_row(state: &FilesState, name: &str, width: f32) -> Div {
    let labels = state.labels();
    let word = match state.place() {
        Place::Trash => labels.trash_place.clone(),
        _ => state
            .path()
            .last()
            .map_or_else(|| labels.root.clone(), |f| f.name.clone()),
    };
    let tab = div()
        .id(format!("{name}-tab-0"))
        .class("fs-files-tab")
        .flex_row()
        .items_center()
        .gap(2.0)
        .child(icon(
            if state.in_trash() {
                Glyph::Trash
            } else {
                Glyph::Folder
            },
            18.0,
            Tone::Kind,
        ))
        .child(text(shorten(&word)).no_cursor())
        .child(
            div()
                .class("fs-files-tab-close")
                .child(icon(Glyph::Close, 12.0, Tone::Muted)),
        );
    let noop: super::actions::Act = std::sync::Arc::new(|| {});
    let tip = format!("{} ({})", labels.chrome.new_tab, labels.chrome.coming_soon);
    div()
        .class("fs-files-tabs")
        .flex_row()
        .items_center()
        .w(width)
        .h(TABS_HEIGHT)
        .child(tab)
        .child(icon_button(
            format!("{name}-new-tab"),
            Glyph::Plus,
            &tip,
            (false, false),
            &noop,
        ))
}

/// A name that fits a tab: at most 24 characters, cut with an ellipsis.
fn shorten(word: &str) -> String {
    if word.chars().count() <= 24 {
        word.to_string()
    } else {
        let cut: String = word.chars().take(23).collect();
        format!("{cut}\u{2026}")
    }
}
