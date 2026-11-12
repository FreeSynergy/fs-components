//! The reader as a picture, drawn without a window.
//!
//! Pattern: Facade over Blinc's offscreen renderer. The same view functions as the window draw
//! the same `Reader` state; the steps below change the state the way a person's actions would
//! (but no click is simulated: the handlers of the elements are not exercised here).

use std::sync::Mutex;

use blinc_layout::widgets::scroll::Scroll;
use fs_components_primitives::{FieldSize, SearchField};
use fs_components_render::{Canvas, draw_to_png};

use super::error::ShellError;
use super::picture::{Picture, ScrollInfo, Snapshot, Step};
use super::setup::styles;
use crate::reader::Reader;
use crate::view::{
    Actions, BlockEditor, Layout, PageTree, confluence, content, editor_rows, frame, new_area,
    notion, search_box, side_body, tree_nodes,
};

/// Apply the steps to a reader and write a PNG to `path`.
///
/// # Errors
/// [`ShellError`] if the size is 0, there is no graphics adapter, or the picture cannot be written.
pub fn snapshot(
    picture: Picture,
    path: &std::path::Path,
) -> std::result::Result<Snapshot, ShellError> {
    let (width, height) = (picture.width, picture.height);
    let mut reader = Reader::new(picture.source, picture.labels);
    reader.set_export_target(picture.export_target);
    let mut scroll_px = 0_u32;
    let mut tree = PageTree::new(reader.entries());
    for step in &picture.steps {
        match step {
            Step::Open(p) => reader.open(p),
            Step::Follow(href) => {
                reader.follow(href);
            }
            Step::Back => reader.back(),
            Step::Search(term) => reader.search(term),
            Step::Edit => reader.begin_edit(),
            Step::Type(text) => reader.set_draft(text),
            Step::Save => reader.save(),
            Step::Cancel => reader.cancel_edit(),
            Step::Export => reader.export(),
            Step::Scroll(px) => scroll_px += px,
            Step::Expand(key) => tree.toggle(key),
        }
    }
    if let Some(path) = reader.current_path() {
        let shown = reader
            .page_info()
            .canonical
            .unwrap_or_else(|| path.to_string());
        tree.reveal(&shown);
    }
    let (layout, labels) = (picture.layout, picture.wiki_labels.clone());
    let physics = Scroll::new().physics();
    let info = Mutex::new(ScrollInfo::default());
    let canvas = Canvas::new(width, height, styles());
    {
        let (reader, physics) = (&reader, &physics);
        draw_to_png(
            &canvas,
            |w, h| {
                let field = SearchField::new(&reader.labels().search).size(FieldSize::Small);
                let actions = Actions::inert();
                if layout != Layout::Classic {
                    return wiki_frame(
                        layout,
                        reader,
                        &tree,
                        &labels,
                        physics,
                        field.build(),
                        w,
                        h,
                    );
                }
                let tree = tree_nodes(reader.entries());
                frame(
                    search_box(&field),
                    side_body(reader, &tree, &actions),
                    content(reader, &actions, Some(physics), None, editor_rows(h)),
                    w,
                    h,
                )
            },
            || {
                let mut p = physics
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if scroll_px > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    p.apply_scroll_delta(0.0, -(scroll_px as f32));
                }
                *info
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    ScrollInfo::new(p.content_height, p.viewport_height, -p.offset_y);
            },
            path,
        )
        .map_err(|e| ShellError(e.to_string()))?;
    }
    let scroll = *info
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(Snapshot { reader, scroll })
}

/// The picture of a wiki layout: the same functions the window's live areas call, with callbacks that do nothing.
#[allow(clippy::too_many_arguments)] // the pieces of one frame, all different
fn wiki_frame(
    layout: Layout,
    reader: &Reader,
    tree: &PageTree,
    labels: &crate::WikiLabels,
    physics: &blinc_layout::widgets::scroll::SharedScrollPhysics,
    search: impl blinc_layout::prelude::ElementBuilder + 'static,
    w: f32,
    h: f32,
) -> blinc_layout::div::Div {
    let actions = Actions::inert();
    if layout == Layout::Confluence {
        let area = reader
            .editing()
            .map(|e| new_area(&e.draft, &fs_components_primitives::FieldData::new()));
        return confluence::confluence(
            search,
            confluence::side(reader, tree, labels, &actions),
            confluence::main(reader, labels, &actions, Some(physics), area.as_ref(), h),
            labels,
            w,
            h,
        );
    }
    let blocks = reader.editing().map(|e| BlockEditor::new(e.id, &e.draft));
    notion::notion(
        search,
        notion::side(reader, tree, labels, &actions),
        notion::main(
            reader,
            labels,
            &actions,
            Some(physics),
            blocks.as_ref(),
            &actions.draft,
        ),
        w,
        h,
    )
}
