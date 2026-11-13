//! The rows of the page tree as elements.
//!
//! Pattern: Adapter (arena rows to Blinc elements); a click on a folder toggles it, a click on a page opens it.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::icon::{Glyph, Tone, icon};

use super::tree::PageTree;
use crate::view::Actions;

/// Most rows built at once: a guard, the open folders of a real tree stay far below.
const MOST_ROWS: usize = 1500;

/// How the rows look.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Look {
    /// Folder and page icons, a bar at the selected page (Confluence-like).
    Icons,
    /// Text only, soft surface at the selected page (Notion-like).
    Quiet,
}

/// The tree in a scroll area filling its room.
pub(super) fn tree_scroll(
    tree: &PageTree,
    selected: Option<&str>,
    actions: &Actions,
    look: Look,
) -> Div {
    let mut column = div().flex_col().w_full().class("fs-wk-tree");
    for row in tree.rows(MOST_ROWS) {
        let node = row.node;
        let here = node.page && selected == Some(node.key.as_str());
        let key = node.key.clone();
        // A plain folder toggles on any click; a page opens by its title (and a page with sub-pages expands by its arrow).
        let toggle = std::sync::Arc::new({
            let (toggle, key) = (actions.toggle.clone(), key.clone());
            move || toggle(&key)
        });
        let open = std::sync::Arc::new({
            let (open, key) = (actions.open.clone(), key);
            move || open(&key)
        });
        let mut line = div()
            .class("fs-wk-row")
            .flex_row()
            .items_center()
            .w_full()
            .pl(f32::from(node.depth) * 4.0 + 2.0);
        if look == Look::Quiet {
            line = line.class("fs-wk-row-quiet");
        }
        if here {
            line = line.class("fs-wk-row-here");
        }
        let plain_folder = node.folder && !node.page;
        if plain_folder {
            let toggle = toggle.clone();
            line = line.on_click(move |_| toggle());
        } else if !node.folder {
            let open = open.clone();
            line = line.on_click(move |_| open());
        }
        if node.folder {
            let chevron = if row.open {
                Glyph::Down
            } else {
                Glyph::ChevronRight
            };
            let arrow = div().child(icon(chevron, 14.0, Tone::Muted));
            line = line.child(if plain_folder {
                arrow
            } else {
                arrow.on_click(move |_| toggle())
            });
        } else {
            line = line.child(div().w(14.0));
        }
        let glyph = if plain_folder {
            Glyph::Folder
        } else {
            Glyph::FileText
        };
        let kind = match look {
            Look::Icons if plain_folder => icon(glyph, 17.0, Tone::Kind),
            Look::Icons => icon(glyph, 17.0, Tone::Muted),
            // The quiet tree: a small icon before every page and folder (the place where the emoji of a page goes).
            Look::Quiet => icon(glyph, 16.0, Tone::Kind),
        };
        let label = text(&node.label).no_cursor();
        let label = if plain_folder {
            label.class("fs-wk-folder")
        } else {
            label
        };
        line = if node.folder && node.page {
            // The title part opens the page; the arrow before it expands.
            line.child(
                div()
                    .flex_row()
                    .items_center()
                    .on_click(move |_| open())
                    .child(kind)
                    .child(label),
            )
        } else {
            line.child(kind).child(label)
        };
        column = column.child(line);
    }
    div()
        .flex_1()
        .min_h(0.0)
        .w_full()
        .child(scroll().w_full().h_full().content(column))
}
