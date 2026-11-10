//! Tests for the findings of the fifth review (2026-11-10): the list says when its records are old.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_admin_frontend::view::AdminSurface;
use fs_admin_frontend::{ManualClock, Session};
use fs_components_app::Surface;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use memory::{Memory, record};
use ui::{layout, serial};

#[test]
fn fifth_11_the_list_carries_a_mark_while_its_records_are_old_and_only_then() {
    let _g = serial();
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    let session = Session::new(
        mem.description.clone(),
        mem.clone(),
        Arc::new(ManualClock::default()),
    );
    let s = AdminSurface::new(
        session,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        None,
    );
    let tree = layout(s.build(900.0, 600.0), 900.0, 600.0);
    assert!(tree.query_by_id("list-stale").is_none());
    *mem.down.lock().unwrap() = true;
    s.session().lock().unwrap().reload();
    let tree = layout(s.build(900.0, 600.0), 900.0, 600.0);
    assert!(
        tree.query_by_id("list-stale").is_some(),
        "old rows are marked as old"
    );
    *mem.down.lock().unwrap() = false;
    s.session().lock().unwrap().reload();
    let tree = layout(s.build(900.0, 600.0), 900.0, 600.0);
    assert!(tree.query_by_id("list-stale").is_none());
}

#[test]
fn sixth_a_command_that_is_not_supported_is_shown_locked_with_coming_soon_and_unlocks() {
    use ui::Input;
    let _g = serial();
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    mem.unsupported.lock().unwrap().insert("create".into());
    let session = Session::new(
        mem.description.clone(),
        mem.clone(),
        Arc::new(ManualClock::default()),
    );
    let s = AdminSurface::new(
        session,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        None,
    );
    let mut tree = layout(s.build(900.0, 600.0), 900.0, 600.0);
    assert!(
        tree.query_by_id("cmd-create").is_some(),
        "the control is there"
    );
    assert!(
        tree.query_by_id("soon-create").is_some(),
        "with the words 'coming soon'"
    );
    assert!(
        tree.query_by_id("soon-find").is_none(),
        "supported commands have none"
    );
    Input::new().click(&mut tree, "cmd-create");
    assert_eq!(
        *s.session().lock().unwrap().screen(),
        fs_admin_frontend::Screen::List,
        "pressing does nothing"
    );
    // the core reports the command: no change in the frontend, the control is free
    mem.unsupported.lock().unwrap().clear();
    s.session().lock().unwrap().reload();
    let mut tree = layout(s.build(900.0, 600.0), 900.0, 600.0);
    assert!(tree.query_by_id("soon-create").is_none());
    Input::new().click(&mut tree, "cmd-create");
    assert!(matches!(
        *s.session().lock().unwrap().screen(),
        fs_admin_frontend::Screen::Command { .. }
    ));
}
