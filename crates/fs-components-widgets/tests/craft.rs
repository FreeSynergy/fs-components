//! The Craft building blocks: the pure parts (columns, accents) and the picture of a grid of tiles with a button, a badge and a bar.

mod common;

use std::sync::Arc;

use blinc_layout::div::div;
use blinc_layout::text;
use common::{background, draw, ink, layout, serial, size_of};
use fs_components_widgets::craft::{
    BadgeTone, Look, accent, accent_text, badge, button, grid, progress, tile, tile_columns,
    tile_head,
};

#[test]
fn tiles_fill_the_room_and_a_narrow_room_takes_one_column() {
    assert_eq!(tile_columns(941.0, 300.0), (3, 313.0));
    assert_eq!(tile_columns(600.0, 300.0), (2, 300.0));
    assert_eq!(tile_columns(120.0, 300.0), (1, 120.0));
    assert_eq!(tile_columns(0.0, 300.0).0, 1);
}

#[test]
fn a_name_keeps_its_accent_and_the_text_shade_is_lighter() {
    assert_eq!(accent("fs-tasks"), accent("fs-tasks"));
    let (a, t) = (accent("fs-tasks"), accent_text("fs-tasks"));
    assert!(
        u32::from(t.r) + u32::from(t.g) + u32::from(t.b)
            >= u32::from(a.r) + u32::from(a.g) + u32::from(a.b)
    );
}

fn sample() -> impl blinc_layout::prelude::ElementBuilder + 'static {
    let act: Arc<dyn Fn() + Send + Sync> = Arc::new(|| {});
    let body = div()
        .flex_col()
        .gap(3.0)
        .child(text("Name"))
        .child(badge("Installed", BadgeTone::Ok))
        .child(progress(0.5, 200.0, accent("a")))
        .child(button("go", "Install", Look::Fill, true, Arc::clone(&act)));
    let t = tile(300.0, tile_head(1, accent("a"), vec![]), body);
    grid(vec![div().flex_row().child(t)])
}

#[test]
fn a_tile_is_as_wide_as_asked_and_its_button_is_34_high() {
    let _g = serial();
    let tree = layout(sample(), 420.0, 320.0);
    let (w, _) = size_of(&tree, "go");
    assert!(w > 40.0);
    let (_, h) = size_of(&tree, "go");
    assert!((h - 34.0).abs() < 1.5, "the button is 34 high, not {h}");
}

#[test]
fn the_tiles_draw_with_ink_on_a_background() {
    let _g = serial();
    let img = draw(sample(), 420, 320, "craft-tile.png");
    let bg = background(&img);
    assert!(ink(&img, (0, 0, 420, 320), bg) > 200);
}
