//! The frames of the Craft look: tiles that share their lines, a grid of them, the head of a tile, a section title, the bar and the
//! footer. They hold what a program puts in; the look is the `craft-*` classes.
//!
//! Pattern: Template (a fixed skeleton, the content passed in). Measures are those of `CRAFT-MEASUREMENT.md` of `fs-components-styles`.

use blinc_layout::div::{Div, div};
use blinc_layout::text;
use fs_components_styles::Rgba;
use fs_components_styles::styles::craft_sheet::caps;

use super::parts::{Act, label_dim, number_box};

/// Height of the top bar and of its cells.
pub const BAR_HEIGHT: f32 = 47.0;
/// Height of the footer.
pub const FOOTER_HEIGHT: f32 = 33.0;
/// What a tile takes beside its content: the lines and the padding at both sides.
pub const TILE_PADDING: f32 = 41.0;

/// A link in the head of a tile or the end of a section: dim capitals and an arrow; a tab stop.
pub fn link(id: &str, words: &str, act: Act) -> Div {
    div()
        .id(id.to_owned())
        .class("craft-link")
        .attr("tabindex", "0")
        .child(
            text(format!("{} \u{2197}", caps(words)))
                .class("craft-caps-dim")
                .no_wrap(),
        )
        .on_click(move |_| act())
}

/// The head of a tile (39 px high, a line below): the numbered box on the left, `links` on the right.
pub fn tile_head(number: usize, accent: Rgba, links: Vec<Div>) -> Div {
    let mut head = div()
        .class("craft-tile-head")
        .flex_row()
        .items_center()
        .gap(3.5)
        .child(number_box(number, accent))
        .child(div().flex_grow());
    for l in links {
        head = head.child(l);
    }
    head
}

/// A tile `width` pixels wide: the head, then the body (20 px padding). Its right and bottom lines are shared with the next tile.
pub fn tile(width: f32, head: Div, body: Div) -> Div {
    div()
        .class("craft-tile")
        .w(width)
        .flex_shrink_0()
        .flex_col()
        .child(head)
        .child(body.class("craft-tile-body"))
}

/// Rows of tiles in one frame whose top and left lines close the shared ones.
pub fn grid(rows: Vec<Div>) -> Div {
    let mut grid = div().class("craft-grid").flex_col().flex_shrink_0();
    for row in rows {
        grid = grid.child(row);
    }
    grid
}

/// How many tiles of at least `least` pixels fit a row of `room` pixels (at least one), and how wide each one is.
pub fn tile_columns(room: f32, least: f32) -> (usize, f32) {
    let fit = (room / least).floor();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let cols = if fit < 1.0 { 1 } else { fit as usize };
    #[allow(clippy::cast_precision_loss)]
    let width = (room / cols as f32).floor();
    (cols, width)
}

/// `01 / PROGRAMS` in dim capitals on the left, `right` (a count, a link) on the right.
pub fn section_title(number: usize, title: &str, right: Div) -> Div {
    div()
        .flex_row()
        .items_center()
        .child(label_dim(&format!("{number:02} / {title}")))
        .child(div().flex_grow())
        .child(right)
}

/// A cell of the top bar: content with a line on its left.
pub fn bar_cell(child: Div) -> Div {
    div()
        .class("craft-cell")
        .flex_row()
        .items_center()
        .px(3.5)
        .child(child)
}

/// The top bar: `left` cells, then the room, then `right` cells; 47 px high with a line below.
pub fn bar(left: Vec<Div>, right: Vec<Div>) -> Div {
    let mut bar = div()
        .class("craft-bar")
        .flex_row()
        .items_center()
        .h(BAR_HEIGHT);
    for c in left {
        bar = bar.child(c);
    }
    bar = bar.child(div().flex_grow());
    for c in right {
        bar = bar.child(c);
    }
    bar
}

/// The footer: dim capitals, `left` first, the room, `right` last; 33 px high with a line above.
pub fn footer(left: &[String], right: &[String]) -> Div {
    let mut f = div()
        .class("craft-footer")
        .flex_row()
        .items_center()
        .gap(4.0);
    for l in left {
        f = f.child(
            div()
                .flex_shrink()
                .min_w(0.0)
                .overflow_clip()
                .child(label_dim(l)),
        );
    }
    f = f.child(div().flex_grow());
    for r in right {
        f = f.child(label_dim(r));
    }
    f
}
