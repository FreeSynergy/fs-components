//! A command to a server that hangs does not freeze the window (v0.0.10): the call runs on a worker, the frames go on,
//! typing is taken, "running" is shown, a second press sends nothing, and the answer comes in when the server answers.
//!
//! Pattern: Test double (a backend whose `run` waits on a gate). No display is needed (layout and frames, no drawing).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;
#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod ui;

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use fs_admin_frontend::view::AdminSurface;
use fs_admin_frontend::{Backend, BackendError, Outcome, Record, Screen, Session, Values};
use fs_components_app::Surface;
use fs_components_fields::{KeysAsText, Look};
use fs_components_styles::StyleId;
use memory::{Memory, clock, contact, record};
use ui::{Input, layout, serial};

const W: f32 = 900.0;
const H: f32 = 600.0;

/// A program whose `run` does not answer until the gate is opened (a server that hangs).
struct Hanging {
    inner: Arc<Memory>,
    open: (Mutex<bool>, Condvar),
    runs: AtomicUsize,
}

impl Hanging {
    fn release(&self) {
        *self.open.0.lock().unwrap() = true;
        self.open.1.notify_all();
    }
}

impl Backend for Hanging {
    fn list(&self, q: &Values) -> Result<Vec<Record>, BackendError> {
        self.inner.list(q)
    }
    fn supported(&self, c: &str) -> bool {
        self.inner.supported(c)
    }
    fn change(&self, r: &str, e: &fs_types::Edit) -> Result<Outcome, BackendError> {
        self.inner.change(r, e)
    }
    fn run(&self, n: &str, r: Option<&str>, i: &Values) -> Result<Outcome, BackendError> {
        self.runs.fetch_add(1, SeqCst);
        let mut open = self.open.0.lock().unwrap();
        while !*open {
            open = self.open.1.wait(open).unwrap();
        }
        drop(open);
        self.inner.run(n, r, i)
    }
}

#[test]
fn a_hanging_server_does_not_freeze_the_window() {
    let _g = serial();
    let hang = Arc::new(Hanging {
        inner: Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]),
        open: (Mutex::new(false), Condvar::new()),
        runs: AtomicUsize::new(0),
    });
    // A guard: if the press blocks (the fault this test is about), the server answers after 15 s so that the test ends
    // red with a message and does not hang for ever.
    let watchdog = Arc::clone(&hang);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(15));
        watchdog.release();
    });
    let session = Session::new(contact(), hang.clone(), clock());
    let s = AdminSurface::new(
        session,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
        None,
    );
    let mut input = Input::new();
    let mut tree = layout(s.build(W, H), W, H);
    input.click(&mut tree, "cmd-create");
    let mut tree = layout(s.build(W, H), W, H);
    input.click(&mut tree, "name-0");
    for ch in "Grace".chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        ui::frame(&mut tree, W, H);
    }

    // Save: the server never answers.
    let pressed = Instant::now();
    input.click(&mut tree, "save");
    assert!(
        pressed.elapsed() < Duration::from_secs(3),
        "the press waited {:?} for the server: the window is frozen",
        pressed.elapsed()
    );
    let mut tree = layout(s.build(W, H), W, H);
    assert!(
        tree.query_by_id("running").is_some(),
        "the page does not say that the command runs"
    );
    assert!(s.session().lock().unwrap().is_busy());

    // Frames go on and typing is taken while the server hangs.
    let started = Instant::now();
    for _ in 0..50 {
        ui::frame(&mut tree, W, H);
    }
    tree.broadcast_text_input_event('!', false, false, false, false);
    ui::frame(&mut tree, W, H);
    assert!(started.elapsed() < Duration::from_secs(3));
    assert_eq!(
        s.session().lock().unwrap().form().unwrap().values("name"),
        ["Grace!"],
        "typing is taken while the command runs"
    );

    // A second press sends nothing (no write twice), and "back" does not leave the screen the answer belongs to.
    input.click(&mut tree, "save");
    input.click(&mut tree, "cancel");
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(hang.runs.load(SeqCst), 1, "the command was sent twice");
    assert!(matches!(
        s.session().lock().unwrap().screen(),
        Screen::Command { .. }
    ));

    // The server answers: the answer is put in, the window leaves "running".
    hang.release();
    let limit = Instant::now() + Duration::from_secs(10);
    while s.session().lock().unwrap().is_busy() && Instant::now() < limit {
        std::thread::sleep(Duration::from_millis(10));
    }
    let session = s.session().lock().unwrap();
    assert!(!session.is_busy());
    assert!(matches!(session.screen(), Screen::Detail(_)));
    assert_eq!(hang.runs.load(SeqCst), 1);
    assert_eq!(hang.inner.records.lock().unwrap().len(), 2);
}
