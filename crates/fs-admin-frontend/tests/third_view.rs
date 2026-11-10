//! Tests for the findings of the third review (2026-11-10): the views and the window.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;
use std::time::{Duration, Instant};

use fs_admin_frontend::view::{AboutInfo, AdminSurface};
use fs_admin_frontend::window::{QUIT, TypeView, admin_window};
use fs_admin_frontend::{ManualClock, Session};
use fs_components_app::Surface;
use fs_components_fields::{KeysAsText, Look, Say};
use fs_components_styles::StyleId;
use memory::{Memory, record};
use ui::{Input, layout, serial};

const W: f32 = 900.0;
const H: f32 = 600.0;

fn surface_over(mem: &Arc<Memory>, clock: Arc<ManualClock>) -> AdminSurface {
    let session = Session::new(mem.description.clone(), mem.clone(), clock);
    AdminSurface::new(
        session,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        None,
    )
}

fn people() -> Arc<Memory> {
    Memory::new(vec![
        record(
            "urn:1",
            &[
                ("name", &["Ada Lovelace"]),
                ("emails", &["ada@example.org", "ada@analytical.test"]),
            ],
        ),
        record("urn:2", &[("name", &["Charles Babbage"])]),
    ])
}

fn about() -> AboutInfo {
    AboutInfo {
        name: "FreeSynergy.Test".into(),
        version: "v0.0.1".into(),
        origin: "x".into(),
        licence: "x".into(),
        repository: "x".into(),
    }
}

/// Draw again (frames) until `ready` holds or `secs` have passed.
fn until(
    tree: &mut blinc_layout::prelude::RenderTree,
    secs: u64,
    ready: impl Fn(&blinc_layout::prelude::RenderTree) -> bool,
) -> bool {
    let end = Instant::now() + Duration::from_secs(secs);
    while Instant::now() < end {
        ui::frame(tree, W, H);
        if ready(tree) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    ready(tree)
}

#[test]
fn third_3_and_4_each_finding_has_its_own_ids_and_says_which_slot() {
    let _g = serial();
    let s = surface_over(&people(), Arc::new(ManualClock::default()));
    let mut tree = layout(s.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "cmd-create");
    ui::frame(&mut tree, W, H);
    {
        let mut g = s.session().lock().unwrap();
        let f = g.form_mut().unwrap();
        f.set("name", 0, "x");
        f.add_value("emails");
        f.set("emails", 0, "bad one");
        f.set("emails", 1, "bad two");
    }
    s.refresh();
    ui::frame(&mut tree, W, H);
    input.click(&mut tree, "save");
    let ids: Vec<String> = ["0", "1"]
        .iter()
        .flat_map(|i| {
            vec![
                format!("finding-emails-{i}-"),
                format!("finding-emails-{i}-"),
            ]
        })
        .collect();
    assert!(
        until(&mut tree, 3, |t| t
            .query_by_id("finding-emails-1-slot")
            .is_some()),
        "{ids:?}"
    );
    assert!(
        tree.query_by_id("finding-emails-0-slot").is_some(),
        "slot numbers on every finding of a multi field"
    );
}

#[test]
fn third_12_quit_with_text_in_two_surfaces_keeps_the_one_that_is_shown() {
    let _g = serial();
    let say: Arc<dyn Say> = Arc::new(KeysAsText);
    let views: Vec<TypeView> = ["Contact", "Group"]
        .iter()
        .map(|n| TypeView {
            id: (*n).to_owned(),
            label: format!("type.{n}"),
            surface: Arc::new(surface_over(&people(), Arc::new(ManualClock::default()))),
        })
        .collect();
    for v in &views {
        let mut s = v.surface.session().lock().unwrap();
        s.begin("create");
        s.form_mut().unwrap().set("name", 0, "text");
    }
    let app = admin_window("Test", &say, views, about(), "https://example.org".into()).unwrap();
    app.select("Group");
    let quit = app.with_shell(|s| s.handler_for(QUIT)).unwrap();
    quit(QUIT);
    assert_eq!(
        app.with_shell(|s| s.current().map(|c| c.id.clone()))
            .as_deref(),
        Some("Group")
    );
}
