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
