//! The bar of the way: back, forward, up, reload, the path in a field of its own, and the search.
//!
//! Pattern: Adapter (state to elements) — what the source or the variant cannot do is not offered. The path is a
//! breadcrumb inside a box with the home icon; the search is a field with a magnifier before it.

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::ElementBuilder;
use fs_components_widgets::icon::{Glyph, Tone, icon};

use super::actions::{FileActions, ICON_BUTTON, icon_button};
use super::commands::PADDING;
use crate::file_manager::context::Variant;
use crate::file_manager::state::FilesState;

/// How high the bar of the way is.
pub(in crate::file_manager) const NAV_HEIGHT: f32 = 52.0;
/// The room of the search field at the right end, and of the magnifier before it.
const SEARCH_WIDTH: f32 = 220.0;
const LUPE: f32 = 26.0;
/// Below this width the path is left out of the bar.
const NARROW: f32 = 480.0;

/// The bar of the way, `field` being the search field.
pub(in crate::file_manager) fn nav_row(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    field: impl ElementBuilder + 'static,
    width: f32,
) -> Div {
    let id = |what: &str| format!("{name}-{what}");
    let labels = state.labels();
    let mut bar = div()
        .class("fs-files-bar-row")
        .flex_row()
        .items_center()
        .gap(1.0)
        .w(width - PADDING)
        .h(NAV_HEIGHT - 8.0);
    let mut used = 0.0;
    if state.variant() == Variant::Full {
        for (what, glyph, tip, enabled, act) in [
            (
                "go-back",
                Glyph::Back,
                &labels.chrome.nav_back,
                state.can_go_back(),
                actions.plain(FilesState::go_back),
            ),
            (
                "go-forward",
                Glyph::Forward,
                &labels.chrome.nav_forward,
                state.can_go_forward(),
                actions.plain(FilesState::go_forward),
            ),
            (
                "go-up",
                Glyph::Up,
                &labels.chrome.nav_up,
                state.can_go_up(),
                actions.plain(FilesState::go_up),
            ),
            (
                "reload",
                Glyph::Reload,
                &labels.chrome.nav_reload,
                true,
                actions.plain(FilesState::retry),
            ),
        ] {
            bar = bar.child(icon_button(id(what), glyph, tip, (false, enabled), &act));
            used += ICON_BUTTON;
        }
    }
    let search = state.capabilities().search;
    // In a narrow window the path gives way: the way is also the name of the tab, the search keeps the room.
    let narrow = width < NARROW;
    let search_wide = if !search {
        0.0
    } else if narrow {
        (width - PADDING - used).max(60.0)
    } else {
        SEARCH_WIDTH.min((width - PADDING - used) * 0.4)
    };
    let lupe = if search && !narrow { LUPE } else { 0.0 };
    let room = (width - PADDING - used - search_wide - lupe - 24.0).max(40.0);
    let path = div()
        .class("fs-files-pathbox")
        .flex_row()
        .items_center()
        .gap(1.0)
        .w(room)
        .h(36.0)
        .overflow_clip()
        .child(icon(Glyph::Home, 18.0, Tone::Kind))
        .child(icon(Glyph::ChevronRight, 14.0, Tone::Muted))
        .child(super::places::crumbs(state, name, actions, room - 60.0));
    if !narrow {
        bar = bar.child(path);
    }
    if search {
        if !narrow {
            bar = bar.child(div().flex_shrink_0().w(lupe).child(icon(
                Glyph::Search,
                18.0,
                Tone::Muted,
            )));
        }
        bar = bar.child(
            div()
                .id(id("search"))
                .class("fs-files-search-field")
                .w(search_wide)
                .flex_shrink_0()
                .child(field),
        );
    }
    bar
}
