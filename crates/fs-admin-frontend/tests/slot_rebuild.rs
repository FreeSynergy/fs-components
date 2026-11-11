//! Fall C ("the input is dead"): the data of the fields of a form is made once and kept, the page is built again
//! and again; the signals of the data must outlive the build that made them, and a rebuild must not add any.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;

use blinc_core::owner::BuildScope;
use fs_admin_frontend::Session;
use fs_admin_frontend::view::AdminSurface;
use fs_components_app::Surface;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use memory::{Memory, clock, contact, record};
use ui::{Input, layout, serial};

const W: f32 = 900.0;
const H: f32 = 600.0;

#[test]
fn a_form_page_rebuilt_a_thousand_times_keeps_its_inputs_alive() {
    let _g = serial();
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    let session = Session::new(contact(), mem, clock());
    let s = AdminSurface::new(
        session,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        None,
    );
    let mut scope = BuildScope::new();
    // The window builds under a scope and disposes the one before it at every fresh tree.
    let mut build = |s: &AdminSurface| {
        let pass = scope.begin();
        let tree = layout(s.build(W, H), W, H);
        scope.finish(pass);
        tree
    };
    let mut tree = build(&s);
    let mut input = Input::new();
    input.click(&mut tree, "cmd-create");
    ui::frame(&mut tree, W, H);
    let _ = build(&s);
    // Not counted here: Blinc's `cn::button` leaves 3 signals per build (found by probe, 0 for a text input),
    // a defect of its own; the leak of the field data is counted in `fs-components-fields/tests/rebuild.rs`.
    for _ in 0..1000 {
        drop(build(&s));
    }
    // The inputs are alive after all those rebuilds: typing shows in the model.
    let mut tree = build(&s);
    input.click(&mut tree, "name-0");
    for ch in "Grace".chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        ui::frame(&mut tree, W, H);
    }
    assert_eq!(
        s.session().lock().unwrap().form().unwrap().values("name"),
        ["Grace"]
    );
}
