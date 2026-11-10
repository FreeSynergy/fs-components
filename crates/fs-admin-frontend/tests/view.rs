//! Tests of the views: the generated pages drawn offscreen in the Admin style, driven with the window's own router.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner) and, for the pixel checks,
//! system fonts.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_admin_frontend::view::AdminSurface;
use fs_admin_frontend::{Screen, Session};
use fs_components_app::Surface;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use memory::{Memory, clock, contact, record};
use ui::{Input, background, draw_tree, has_fonts, ink, layout, serial, size_of};

const W: f32 = 900.0;
const H: f32 = 600.0;

fn surface() -> (AdminSurface, Arc<Memory>) {
    let mem = Memory::new(vec![
        record(
            "urn:1",
            &[
                ("name", &["Ada Lovelace"]),
                ("emails", &["ada@example.org", "ada@analytical.test"]),
            ],
        ),
        record(
            "urn:2",
            &[
                ("name", &["Charles Babbage"]),
                ("phone", &["+442071234567"]),
            ],
        ),
        record("urn:3", &[("name", &["Alan Turing"])]),
    ]);
    let session = Session::new(contact(), mem.clone(), clock());
    let look = Look::new(StyleId::Admin, false);
    (
        AdminSurface::new(session, look, Arc::new(KeysAsText), None),
        mem,
    )
}

fn screen(s: &AdminSurface) -> Screen {
    s.session().lock().unwrap().screen().clone()
}

#[test]
fn the_list_shows_a_head_per_column_the_rows_and_a_button_per_list_command() {
    let _g = serial();
    let (s, _) = surface();
    let tree = layout(s.build(W, H), W, H);
    for id in [
        "head-name",
        "head-emails",
        "head-phone",
        "records",
        "cmd-create",
        "cmd-find",
        "cmd-domains",
    ] {
        assert!(tree.query_by_id(id).is_some(), "no element {id}");
    }
    assert!(
        tree.query_by_id("cmd-change").is_none(),
        "record commands wait for a record"
    );
    assert!(
        tree.query_by_id("head-note").is_none(),
        "multi-line text has no column"
    );
    let (w, _) = size_of(&tree, "head-name");
    assert!(
        (w - 300.0).abs() < 40.0,
        "three columns share the width: {w}"
    );
}

#[test]
fn the_list_is_drawn_with_text_when_fonts_exist() {
    let _g = serial();
    if !has_fonts() {
        return;
    }
    let (s, _) = surface();
    let tree = layout(s.build(W, H), W, H);
    let img = draw_tree(&tree, W as u32, H as u32, "admin-list.png");
    let bg = background(&img);
    assert!(
        ink(&img, (0, 0, W as u32, 80), bg) > 200,
        "title and buttons"
    );
    assert!(
        ink(&img, (0, 120, W as u32, 200), bg) > 300,
        "head and rows"
    );
}

#[test]
fn a_click_on_a_head_sorts_and_a_click_on_a_row_opens_the_detail() {
    let _g = serial();
    let (s, _) = surface();
    let mut tree = layout(s.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "head-name");
    assert!(s.session().lock().unwrap().list().sort().is_some());
    ui::frame(&mut tree, W, H);
    input.click(&mut tree, "records-urn:2");
    assert_eq!(screen(&s), Screen::Detail("urn:2".into()));
    ui::frame(&mut tree, W, H);
    assert!(
        tree.query_by_id("cmd-change").is_some(),
        "now the record commands are there"
    );
    assert!(tree.query_by_id("cmd-create").is_none());
    assert!(tree.query_by_id("back").is_some());
    input.click(&mut tree, "back");
    assert_eq!(screen(&s), Screen::List);
}

#[test]
fn the_create_form_takes_typed_text_into_the_model_and_sends_it() {
    let _g = serial();
    let (s, mem) = surface();
    let mut tree = layout(s.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "cmd-create");
    ui::frame(&mut tree, W, H);
    assert!(matches!(screen(&s), Screen::Command { .. }));
    for id in ["name-0", "emails-0-local", "add-emails", "save", "cancel"] {
        assert!(tree.query_by_id(id).is_some(), "no element {id}");
    }
    input.click(&mut tree, "name-0");
    for ch in "Grace Hopper".chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        ui::frame(&mut tree, W, H);
    }
    assert_eq!(
        s.session().lock().unwrap().form().unwrap().values("name"),
        ["Grace Hopper"]
    );
    input.click(&mut tree, "add-emails");
    ui::frame(&mut tree, W, H);
    assert!(
        tree.query_by_id("emails-1-local").is_some(),
        "a second slot after Add"
    );
    assert_eq!(
        s.session().lock().unwrap().form().unwrap().values("name"),
        ["Grace Hopper"],
        "the text survives"
    );
    input.click(&mut tree, "save");
    assert!(matches!(screen(&s), Screen::Detail(_)));
    assert_eq!(mem.records.lock().unwrap().len(), 4);
}

#[test]
fn saving_an_empty_required_form_stays_and_marks_the_field() {
    let _g = serial();
    let (s, mem) = surface();
    let mut tree = layout(s.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "cmd-create");
    ui::frame(&mut tree, W, H);
    input.click(&mut tree, "save");
    assert!(matches!(screen(&s), Screen::Command { .. }));
    assert_eq!(mem.records.lock().unwrap().len(), 3);
    assert_eq!(
        s.session()
            .lock()
            .unwrap()
            .form()
            .unwrap()
            .visible_issues()
            .len(),
        1
    );
}
