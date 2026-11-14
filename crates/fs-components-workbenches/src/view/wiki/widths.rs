//! How wide the side columns of the wiki layouts are, so the page in the middle never gets too narrow to read.
//!
//! Pattern: none — pure functions of the window width. The middle column keeps at least `MIDDLE_MIN` pixels: first the
//! right panel (outline and history) goes, then the left column shrinks. A title never breaks letter by letter (finding E5 of
//! 2026-11-14: a 130 px column at a window width of 700 px).

/// The least room the page in the middle gets, in pixels.
pub(super) const MIDDLE_MIN: f32 = 360.0;
/// The widest the left column is (the page tree).
pub(super) const LEFT: f32 = 300.0;
/// The Notion sidebar.
pub(super) const NOTION_SIDE: f32 = 264.0;
/// The right panel of the Confluence layout.
pub(super) const RIGHT: f32 = 270.0;
/// The left column never gets narrower than this while the window allows it.
const LEFT_LEAST: f32 = 120.0;

/// Left column and right panel of the Confluence layout in a window `width` wide: the right panel is `None` when the middle
/// would be narrower than `MIDDLE_MIN` with it.
pub(super) fn confluence(width: f32) -> (f32, Option<f32>) {
    if width >= LEFT + RIGHT + MIDDLE_MIN {
        (LEFT, Some(RIGHT))
    } else {
        (shrunk(width, LEFT), None)
    }
}

/// The Notion sidebar in a window `width` wide.
pub(super) fn notion(width: f32) -> f32 {
    shrunk(width, NOTION_SIDE)
}

fn shrunk(width: f32, wanted: f32) -> f32 {
    wanted.min((width - MIDDLE_MIN).max(LEFT_LEAST))
}
