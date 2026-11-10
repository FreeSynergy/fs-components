//! Review of 2026-11-10 (Opus), admin frontend: the handshake message is drawn with its words and "Try again", and
//! Quit closes the window instead of ending the process.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fs_admin_frontend::view::{AboutInfo, AdminSurface};
use fs_admin_frontend::window::{QUIT, TypeView, admin_window};
use fs_admin_frontend::{BackendError, ManualClock, Session};
use fs_components_app::Surface;
use fs_components_fields::{KeysAsText, Look, Say};
use fs_components_styles::StyleId;
use memory::{Memory, record};
use ui::{layout, serial};

fn surface(mem: &Arc<Memory>) -> AdminSurface {
    let session = Session::new(
        mem.description.clone(),
        mem.clone(),
        Arc::new(ManualClock::default()),
    );
    AdminSurface::new(
        session,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        None,
    )
}

#[test]
fn the_handshake_message_shows_the_words_of_the_error_and_a_try_again_button() {
    let _g = serial();
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    *mem.list_error.lock().unwrap() = Some(BackendError::Handshake(
        "Close the program completely.".into(),
    ));
    let s = surface(&mem);
    let id = s.session().lock().unwrap().messages().current()[0].id;
    let tree = layout(s.build(900.0, 600.0), 900.0, 600.0);
    for element in [
        format!("message-{id}"),
        format!("message-{id}-detail"),
        format!("retry-{id}"),
    ] {
        let node = tree
            .query_by_id(&element)
            .unwrap_or_else(|| panic!("no {element}"));
        let b = tree.get_absolute_bounds(node).unwrap();
        assert!(
            b.x >= 0.0 && b.x + b.width <= 900.5,
            "{element} is inside the window"
        );
    }
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

#[test]
fn quit_closes_the_window_and_does_not_end_the_process() {
    let _g = serial();
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    let say: Arc<dyn Say> = Arc::new(KeysAsText);
    let views = vec![TypeView {
        id: "Contact".to_owned(),
        label: "type.Contact".to_owned(),
        surface: Arc::new(surface(&mem)),
    }];
    let app = admin_window("Test", &say, views, about(), "https://example.org".into()).unwrap();
    let closed = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&closed);
    app.set_quit(move || {
        seen.fetch_add(1, Ordering::SeqCst);
    });
    let quit = app.with_shell(|s| s.handler_for(QUIT)).unwrap();
    quit(QUIT);
    assert_eq!(
        closed.load(Ordering::SeqCst),
        1,
        "the window was asked to close"
    );
}
