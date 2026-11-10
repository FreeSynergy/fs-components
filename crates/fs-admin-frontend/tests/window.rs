//! Tests of the window: the standard menus of ADR 0065 and what their entries do.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use fs_admin_frontend::Session;
use fs_admin_frontend::view::{AboutInfo, AdminSurface};
use fs_admin_frontend::window::{ABOUT, TypeView, admin_window};
use fs_components_fields::{KeysAsText, Look, Say};
use fs_components_styles::StyleId;
use memory::{Memory, clock, contact};
use ui::{layout, serial};

fn views() -> Vec<TypeView> {
    ["Contact", "Group"]
        .iter()
        .map(|n| {
            let session = Session::new(contact(), Memory::new(vec![]), clock());
            let say: Arc<dyn Say> = Arc::new(KeysAsText);
            TypeView {
                id: (*n).to_owned(),
                label: format!("type.{n}"),
                surface: Arc::new(AdminSurface::new(
                    session,
                    Look::new(StyleId::Admin, false),
                    say,
                    None,
                )),
            }
        })
        .collect()
}

fn about() -> AboutInfo {
    AboutInfo {
        name: "FreeSynergy.Test".into(),
        version: "v0.0.7".into(),
        origin: "catalog".into(),
        licence: "MIT OR Apache-2.0".into(),
        repository: "https://github.com/FreeSynergy/fs-test".into(),
    }
}

#[test]
fn the_window_has_the_three_standard_menus_and_a_view_entry_per_type() {
    let _g = serial();
    let say: Arc<dyn Say> = Arc::new(KeysAsText);
    let app = admin_window(
        "Test",
        &say,
        views(),
        about(),
        "https://example.org/help".into(),
    )
    .unwrap();
    app.with_shell(|s| {
        let titles: Vec<&str> = s.menus().iter().map(|m| m.label.as_str()).collect();
        assert_eq!(titles, ["menu.program", "menu.view", "menu.help"]);
        let ids: Vec<&str> = s
            .menus()
            .iter()
            .flat_map(|m| m.entries.iter().map(|e| e.id.as_str()))
            .collect();
        assert_eq!(
            ids,
            [
                "program-quit",
                "view-Contact",
                "view-Group",
                "help-help",
                "help-about"
            ]
        );
    });
}

#[test]
fn choosing_a_view_selects_it_and_about_shows_the_version() {
    let _g = serial();
    let say: Arc<dyn Say> = Arc::new(KeysAsText);
    let v = views();
    let first = Arc::clone(&v[0].surface);
    let app = admin_window("Test", &say, v, about(), "https://example.org/help".into()).unwrap();
    let run = |id: &str| {
        let h = app
            .with_shell(|s| s.handler_for(id))
            .expect("entry is enabled");
        h(id);
    };
    run("view-Group");
    assert_eq!(
        app.with_shell(|s| s.current().map(|c| c.id.clone()))
            .as_deref(),
        Some("Group")
    );
    run("view-Contact");
    run(ABOUT);
    use fs_components_app::Surface;
    let tree = layout(first.build(900.0, 600.0), 900.0, 600.0);
    assert!(
        tree.query_by_id("about-back").is_some(),
        "the About page is up, with a way back"
    );
}
