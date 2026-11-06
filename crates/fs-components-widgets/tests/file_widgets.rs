//! Breadcrumb, file-tile and file-preview: the pure rules, the elements, a press and the keys.
//!
//! Pattern: Model tests and an in-process mouse over the real hit test.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout, serial};
use fs_components_widgets::breadcrumb::{Breadcrumb, Crumb, plan};
use fs_components_widgets::file_preview::{
    FilePreview, MAX_LINE_CHARS, MAX_LINES, PreviewContent, picture_address, picture_fits,
    safe_text,
};
use fs_components_widgets::file_tile::{FileTile, TileGrid, grid_target};
use fs_components_widgets::keys::Key;
use fs_components_widgets::list_view::ListPosition;

fn way(count: usize) -> Vec<Crumb> {
    (0..count)
        .map(|i| Crumb::new(format!("k{i}"), format!("folder {i}")))
        .collect()
}

#[test]
fn a_long_way_keeps_the_first_and_the_last_and_says_something_is_left_out() {
    assert!(plan(&[], 800.0).is_empty());
    assert_eq!(plan(&way(1), 800.0), vec![Some(0)]);
    assert_eq!(plan(&way(3), 800.0), vec![Some(0), Some(1), Some(2)]);
    let narrow = plan(&way(30), 300.0);
    assert_eq!(narrow.first(), Some(&Some(0)));
    assert_eq!(narrow.last(), Some(&Some(29)));
    assert!(narrow.contains(&None));
    assert!(narrow.len() < 8);
}

#[test]
fn pressing_a_step_reports_its_key_and_the_place_itself_is_no_button() {
    let _g = serial();
    let got = Arc::new(Mutex::new(Vec::new()));
    let seen = got.clone();
    let crumb = Breadcrumb::new(way(30), "w").on_choose(Arc::new(move |key| {
        seen.lock().unwrap().push(key.to_string());
    }));
    let mut tree = layout(crumb.build(400.0), 400.0, 60.0);
    let mut input = Input::new();
    input.click(&mut tree, "w-crumb-0");
    input.click(&mut tree, "w-crumb-hidden");
    input.click(&mut tree, "w-crumb-29");
    let got = got.lock().unwrap();
    assert_eq!(got.first().map(String::as_str), Some("k0"));
    assert_eq!(
        got.len(),
        2,
        "the last step is where the person is: {got:?}"
    );
    assert!(got[1] != "k29");
}

#[test]
fn a_step_is_reached_by_tab_and_enter() {
    let _g = serial();
    let got = Arc::new(Mutex::new(Vec::new()));
    let seen = got.clone();
    let crumb = Breadcrumb::new(way(2), "w").on_choose(Arc::new(move |key| {
        seen.lock().unwrap().push(key.to_string());
    }));
    let mut tree = layout(crumb.build(600.0), 600.0, 60.0);
    let mut input = Input::new();
    assert!(input.tab_to(&mut tree, "w-crumb-0"));
    input.key(&mut tree, 13);
    assert_eq!(got.lock().unwrap().as_slice(), ["k0"]);
}

#[test]
fn keys_walk_the_grid_by_column_and_stop_at_its_edges() {
    assert_eq!(grid_target(Key::Right, Some(0), 10, 3, 2), Some(1));
    assert_eq!(grid_target(Key::Down, Some(1), 10, 3, 2), Some(4));
    assert_eq!(grid_target(Key::Down, Some(8), 10, 3, 2), Some(8));
    assert_eq!(grid_target(Key::Up, Some(1), 10, 3, 2), Some(1));
    assert_eq!(grid_target(Key::End, None, 10, 3, 2), Some(9));
    assert_eq!(grid_target(Key::Down, None, 0, 3, 2), None);
    assert_eq!(grid_target(Key::Escape, Some(1), 10, 3, 2), None);
}

#[test]
fn a_very_long_grid_has_only_the_tiles_in_view_and_a_press_chooses_one() {
    let _g = serial();
    let tiles: Vec<FileTile> = (0..100_000)
        .map(|i| FileTile::new(format!("t{i}"), format!("name {i}"), "txt"))
        .collect();
    let got = Arc::new(Mutex::new(Vec::new()));
    let seen = got.clone();
    let position = ListPosition::new();
    let grid = TileGrid::new(Arc::new(tiles), (160.0, 120.0), &position, "g")
        .on_select(Arc::new(move |key: &str| {
            seen.lock().unwrap().push(key.to_string())
        }))
        .build(800.0, 400.0);
    let mut tree = layout(grid, 800.0, 400.0);
    assert!(tree.query_by_id("g-t0").is_some());
    assert!(tree.query_by_id("g-t50000").is_none());
    let mut input = Input::new();
    input.click(&mut tree, "g-t1");
    assert_eq!(got.lock().unwrap().as_slice(), ["t1"]);
}

#[test]
fn only_text_is_text_and_what_could_mislead_is_replaced() {
    assert!(safe_text(b"a\0b", false).is_none());
    assert!(safe_text(&[0xFF, 0xFE, 0x41], false).is_none());
    let text = safe_text("a\u{202E}b\r\n\u{1b}[31mc\tz".as_bytes(), false).unwrap();
    assert_eq!(text.lines, ["a\u{FFFD}b", "\u{FFFD}[31mc z"]);
    assert!(!text.cut);
    // A character cut by the end of what was read is no reason to refuse the file.
    let cut = &"xé".as_bytes()[..2];
    assert!(safe_text(cut, false).is_none());
    let text = safe_text(cut, true).unwrap();
    assert_eq!(text.lines, ["x"]);
    assert!(text.cut);
}

#[test]
fn text_is_bounded_in_lines_and_length_and_says_so() {
    let long = "y".repeat(MAX_LINE_CHARS * 3);
    let text = safe_text(long.as_bytes(), false).unwrap();
    assert_eq!(text.lines[0].chars().count(), MAX_LINE_CHARS + 1);
    assert!(text.cut);
    let many = "l\n".repeat(MAX_LINES * 2);
    let text = safe_text(many.as_bytes(), false).unwrap();
    assert_eq!(text.lines.len(), MAX_LINES);
    assert!(text.cut);
}

#[test]
fn a_picture_passes_only_with_a_header_that_says_it_is_small_enough() {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    png.extend(100u32.to_be_bytes());
    png.extend(200u32.to_be_bytes());
    assert!(picture_fits(&png, 4096));
    assert!(!picture_fits(&png, 150), "too high");
    let mut bomb = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    bomb.extend(100_000u32.to_be_bytes());
    bomb.extend(100_000u32.to_be_bytes());
    assert!(!picture_fits(&bomb, 4096));
    assert!(!picture_fits(b"not a picture", 4096));
    assert!(!picture_fits(b"", 4096));
    let jpeg = [
        0xFF, 0xD8, 0xFF, 0xE0, 0, 4, 0, 0, 0xFF, 0xC0, 0, 17, 8, 0, 50, 0, 60, 3, 0, 0, 0, 0,
    ];
    assert!(picture_fits(&jpeg, 100));
    assert!(!picture_fits(&jpeg, 55));
    assert!(!picture_fits(&jpeg[..6], 100), "cut off before the size");
    assert_eq!(
        picture_address("image/png", b"Man"),
        "data:image/png;base64,TWFu"
    );
    assert_eq!(
        picture_address("image/png", b"Ma"),
        "data:image/png;base64,TWE="
    );
}

#[test]
fn the_preview_builds_with_marks_and_every_kind_of_content() {
    let _g = serial();
    for content in [
        PreviewContent::Nothing,
        PreviewContent::Note("no".into()),
        PreviewContent::Refused("changed".into()),
        PreviewContent::Text(safe_text(b"hello", true).unwrap(), "cut".into()),
    ] {
        let mut preview = FilePreview::new("a name");
        preview.marks.push("Sensitive".into());
        preview.content = content;
        let tree = layout(preview.build("p", 300.0, 200.0), 300.0, 200.0);
        assert!(tree.query_by_id("p-marks").is_some());
        assert!(tree.query_by_id("p-content").is_some());
    }
}
