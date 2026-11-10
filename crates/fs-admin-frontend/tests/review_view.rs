//! Tests for the findings of the independent reviews (2026-11-10): the views and the window.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::Arc;
use std::time::{Duration, Instant};

use fs_admin_frontend::view::{AboutInfo, AdminSurface, list_rows};
use fs_admin_frontend::window::{QUIT, TypeView, admin_window};
use fs_admin_frontend::{ListModel, ManualClock, MessageKind, Session};
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
fn review_7_a_message_raised_while_the_session_is_locked_neither_blocks_nor_is_lost_and_a_success_runs_out()
 {
    let _g = serial();
    let mem = people();
    let clock = Arc::new(ManualClock::default());
    let s = surface_over(&mem, clock.clone());
    let mut tree = layout(s.build(W, H), W, H);
    let board = Arc::clone(s.session().lock().unwrap().messages());
    // the session is locked (a transition is running) while another thread raises
    let guard = s.session().lock().unwrap();
    let b = Arc::clone(&board);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let id = b.raise(MessageKind::Success, "done");
        tx.send(id).unwrap();
    });
    let id = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("raising must not wait for the session");
    drop(guard);
    let name = format!("message-{id}");
    assert!(
        until(&mut tree, 5, |t| t.query_by_id(&name).is_some()),
        "the page was drawn again with the message"
    );
    clock.advance(5000);
    assert!(
        until(&mut tree, 8, |t| t.query_by_id(&name).is_none()),
        "and the success went by itself, nobody clicked"
    );
}

#[test]
fn review_9_and_f_quit_with_text_shows_the_surface_that_has_it_once_and_closes_about() {
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
    let second = Arc::clone(&views[1].surface);
    let app = admin_window("Test", &say, views, about(), "https://example.org".into()).unwrap();
    {
        let mut s = second.session().lock().unwrap();
        s.begin("create");
        s.form_mut().unwrap().set("name", 0, "unsaved text");
    }
    second.set_about(Some(about()));
    app.select("Contact"); // the first surface is the one that is shown
    let quit = app.with_shell(|s| s.handler_for(QUIT)).unwrap();
    quit(QUIT);
    quit(QUIT);
    assert_eq!(
        app.with_shell(|s| s.current().map(|c| c.id.clone()))
            .as_deref(),
        Some("Group"),
        "the surface with the text is shown"
    );
    let unsaved: Vec<_> = second
        .session()
        .lock()
        .unwrap()
        .messages()
        .current()
        .into_iter()
        .filter(|m| m.code == "unsaved")
        .collect();
    assert_eq!(unsaved.len(), 1, "asked twice, said once");
    let tree = layout(second.build(W, H), W, H);
    assert!(
        tree.query_by_id("about-back").is_none(),
        "About is closed, the form is shown"
    );
    assert!(
        tree.query_by_id(&format!("message-{}", unsaved[0].id))
            .is_some(),
        "and the warning is on it"
    );
}

#[test]
fn review_12_findings_are_two_separate_texts_and_only_required_fields_have_the_mark() {
    let _g = serial();
    let s = surface_over(&people(), Arc::new(ManualClock::default()));
    let mut tree = layout(s.build(W, H), W, H);
    let mut input = Input::new();
    input.click(&mut tree, "cmd-create");
    ui::frame(&mut tree, W, H);
    input.click(&mut tree, "save");
    assert!(until(&mut tree, 3, |t| t
        .query_by_id("finding-name-0-problem.missing-problem")
        .is_some()));
    assert!(
        tree.query_by_id("finding-name-0-problem.missing-label")
            .is_some(),
        "label and problem are not joined"
    );
    assert!(tree.query_by_id("required-name").is_some());
    assert!(tree.query_by_id("required-emails").is_none());
}

#[test]
fn review_a_a_record_that_could_not_be_read_is_said_so_and_has_no_record_commands() {
    let _g = serial();
    let mem = people();
    let s = surface_over(&mem, Arc::new(ManualClock::default()));
    let mut tree = layout(s.build(W, H), W, H);
    let mut input = Input::new();
    *mem.down_after_run.lock().unwrap() = true;
    input.click(&mut tree, "cmd-create");
    ui::frame(&mut tree, W, H);
    input.click(&mut tree, "name-0");
    for ch in "Grace".chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        ui::frame(&mut tree, W, H);
    }
    input.click(&mut tree, "save");
    assert!(until(&mut tree, 3, |t| t
        .query_by_id("not-loaded")
        .is_some()));
    assert!(tree.query_by_id("cmd-change").is_none() && tree.query_by_id("cmd-delete").is_none());
}

#[test]
fn review_i_several_values_are_joined_with_the_word_of_the_language() {
    let mem = people();
    let model = ListModel::new(&mem.description, mem.records.lock().unwrap().clone());
    let rows = list_rows(&model, &KeysAsText);
    let emails = model
        .columns()
        .iter()
        .position(|c| c.field.name == "emails")
        .unwrap();
    assert_eq!(
        rows[0].cells[emails],
        "ada@example.orglist.separatorada@analytical.test"
    );
}
