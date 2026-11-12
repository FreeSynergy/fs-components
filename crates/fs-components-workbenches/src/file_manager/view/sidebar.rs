//! The sidebar: Home alone at the top, the pinned user folders, the storage with its folders, the network, and at
//! the bottom the trash and the settings with no heading of their own.
//!
//! Pattern: Adapter (state to elements) — groups fold; a group that has nothing says so plainly (the network: "nothing
//! connected yet"); what is not built (settings) is shown dimmed (ADR 0066). The pinned folders are the folders at the
//! top of the storage that the host knows by the names of the user folders.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::TreeView;
use fs_components_widgets::icon::{Glyph, Tone, icon};
use fs_components_widgets::pinned_icon;

use super::actions::{Act, FileActions};
use super::places::{group, place_row};
use crate::file_manager::context::Variant;
use crate::file_manager::state::FilesState;
use crate::file_manager::tree::TOP;

/// The sidebar.
#[allow(clippy::too_many_lines)] // one column of groups in the order they stand: splitting scatters the order
pub(in crate::file_manager) fn tree_pane(state: &FilesState, actions: &FileActions) -> Div {
    let mut pane = div().flex_col().w_full().h_full();
    if let Some(problem) = state.tree_problem() {
        pane = pane.child(div().class("fs-files-failed-line").child(text(problem)));
    }
    if state.variant() == Variant::Picker {
        return pane;
    }
    let (chrome, labels) = (&state.labels().chrome, state.labels());
    let in_trash = state.in_trash();
    let top = !in_trash && state.folder().is_none();
    // Home, alone: no group, no second "Home" under a heading "Home".
    pane = pane.child(place_row(
        "fs-files-place-top",
        Glyph::Home,
        &chrome.group_home,
        top,
        actions.with(|s| s.open_folder(None)),
    ));
    let pinned = state.pinned();
    if !pinned.is_empty() {
        pane = pane.child(group(
            div().flex_col().w_full().flex_shrink_0(),
            state,
            actions,
            ("pinned", &chrome.group_pinned),
            |mut g| {
                for (kind, folder) in &pinned {
                    let key = folder.key.clone();
                    let here = state.folder() == Some(folder.key.as_str());
                    g = g.child(pinned_row(
                        &folder.name,
                        *kind,
                        here,
                        actions.with(move |s| s.open_folder(Some(&key))),
                    ));
                }
                g
            },
        ));
    }
    pane = pane.child(group(
        div().flex_col().w_full().flex_grow().min_h(0.0),
        state,
        actions,
        ("storage", &chrome.group_storage),
        |g| {
            let choose = actions.on_key(FilesState::choose_in_tree);
            let current = state.folder().unwrap_or(TOP).to_string();
            let tree = TreeView::new(state.tree_nodes())
                .selected(current)
                .on_select(move |key| choose(key))
                .build();
            g.child(
                div()
                    .w_full()
                    .flex_grow()
                    .min_h(0.0)
                    .child(scroll().w_full().h_full().content(tree)),
            )
        },
    ));
    pane = pane.child(group(
        div().flex_col().w_full().flex_shrink_0(),
        state,
        actions,
        ("network", &chrome.group_network),
        |g| {
            g.child(
                div()
                    .class("fs-files-note")
                    .child(text(&chrome.network_empty).no_cursor()),
            )
        },
    ));
    // The trash and the settings, at the bottom, with no heading.
    if state.capabilities().trash {
        pane = pane.child(place_row(
            "fs-files-place-trash",
            Glyph::Trash,
            &labels.trash,
            in_trash,
            actions.plain(FilesState::open_trash),
        ));
    }
    let noop: Act = std::sync::Arc::new(|| {});
    pane.child(
        div()
            .id("fs-files-place-settings")
            .class("fs-files-place fs-files-place-off")
            .flex_shrink_0()
            .flex_row()
            .items_center()
            .gap(2.0)
            .w_full()
            .h(ROW)
            .attr(
                "title",
                format!("{} ({})", chrome.settings, chrome.coming_soon),
            )
            .on_click(move |_| noop())
            .child(icon(Glyph::Settings, 18.0, Tone::Muted))
            .child(text(&chrome.settings).no_cursor()),
    )
}

/// The height of a row of the sidebar (the model has 32).
pub(super) const ROW: f32 = 32.0;

/// A pinned folder: its coloured icon and its name.
fn pinned_row(name: &str, kind: fs_components_widgets::PinnedKind, current: bool, act: Act) -> Div {
    let row = div()
        .id(format!("fs-files-pinned-{name}"))
        .class("fs-files-place");
    let row = if current {
        row.class("fs-files-place-current")
    } else {
        row
    };
    row.w_full()
        .h(ROW)
        .flex_row()
        .justify_start()
        .items_center()
        .gap(2.0)
        .attr("tabindex", "0")
        .on_click(move |_| act())
        .child(pinned_icon(kind, 20.0))
        .child(text(name).no_cursor())
}
