//! The long calls (rescan, upload, download, going on with an upload) never hold the window.
//!
//! Tests: State tests over a source in memory whose long calls wait at a gate (a server that takes its time). The
//! counter-proof runs the same course with a launcher that makes the call where it is started, the way the window
//! worked before: the course must fail there, or the test proves nothing.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use files_common::Shelf;
use fs_components_widgets::transfer_list::State as TransferState;
use fs_components_workbenches::file_manager::{
    FileLabels, FilesState, JobDone, Launcher, Notice, Variant,
};

const FAST: Duration = Duration::from_millis(400);

/// Jobs on a thread of their own, answers to the test (which plays the window's lock).
fn on_a_worker() -> (Launcher, Receiver<JobDone>) {
    let (tx, rx) = channel();
    let launcher: Launcher = Arc::new(move |job| {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(job.run());
        });
        Ok(())
    });
    (launcher, rx)
}

/// The old way: the call is made where it is started.
fn right_here() -> (Launcher, Receiver<JobDone>) {
    let (tx, rx) = channel();
    let launcher: Launcher = Arc::new(move |job| {
        let _ = tx.send(job.run());
        Ok(())
    });
    (launcher, rx)
}

fn window(launcher: Launcher) -> (Arc<Shelf>, FilesState) {
    let shelf = Arc::new(Shelf::new().file("a"));
    let mut state = FilesState::new(shelf.clone(), FileLabels::default(), Variant::Full);
    state.set_launcher(launcher);
    (shelf, state)
}

fn upload_of(state: &mut FilesState, path: &str) -> Duration {
    state.begin_upload();
    state.set_input(path);
    let at = Instant::now();
    state.submit_input();
    at.elapsed()
}

/// The course: a long upload is started, the window is used meanwhile, the server answers.
fn course(
    answers: &Receiver<JobDone>,
    shelf: &Shelf,
    state: &mut FilesState,
) -> Result<(), String> {
    shelf.gate.shut();
    let took = upload_of(state, "/tmp/new.txt");
    if took > FAST {
        shelf.gate.open();
        return Err(format!("the start held the window for {took:?}"));
    }
    if !state.is_running() {
        return Err("not shown as running".into());
    }
    let row = state.transfers().first().map(|r| r.shown.state.clone());
    if row != Some(TransferState::Running) {
        return Err(format!("the row says {row:?}, not running"));
    }
    state.search("a"); // input during the call is taken
    if state.term() != "a" {
        return Err("input was lost".into());
    }
    let again = upload_of(state, "/tmp/other.txt");
    if again > FAST || shelf.writes() != 0 || state.transfers().len() != 1 {
        return Err("a second start was not refused".into());
    }
    shelf.gate.open();
    let done = answers
        .recv_timeout(Duration::from_secs(5))
        .map_err(|e| e.to_string())?;
    state.finish(done);
    Ok(())
}

#[test]
fn a_long_upload_leaves_the_window_free_and_ends_done() {
    let (launcher, answers) = on_a_worker();
    let (shelf, mut state) = window(launcher);
    course(&answers, &shelf, &mut state).unwrap();
    assert!(!state.is_running());
    assert_eq!(state.transfers().len(), 1);
    assert_eq!(state.transfers()[0].shown.state, TransferState::Done);
    assert_eq!(shelf.writes(), 1, "carried out once");
    assert!(matches!(state.notice(), Some(Notice::Info(_))));
}

#[test]
fn counter_proof_a_call_made_in_place_fails_the_course() {
    let (launcher, answers) = right_here();
    let (shelf, mut state) = window(launcher);
    // The server answers late; the in-place variant waits for it on the very thread that started the call.
    let door = Arc::clone(&shelf);
    let opener = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(900));
        door.gate.open();
    });
    let result = course(&answers, &shelf, &mut state);
    opener.join().unwrap();
    assert!(
        result.is_err(),
        "the course must be red when the call blocks"
    );
}

#[test]
fn a_failed_upload_ends_as_failed_and_keeps_the_input_path_for_a_retry() {
    let (launcher, answers) = on_a_worker();
    let (shelf, mut state) = window(launcher);
    *shelf.broken.lock().unwrap() = Some("server down".into());
    shelf.gate.shut();
    upload_of(&mut state, "/tmp/new.txt");
    assert_eq!(state.transfers()[0].shown.state, TransferState::Running);
    shelf.gate.open();
    state.finish(answers.recv_timeout(Duration::from_secs(5)).unwrap());
    assert!(matches!(
        state.transfers()[0].shown.state,
        TransferState::Failed(_)
    ));
    assert!(!state.is_running());
}

#[test]
fn a_rescan_runs_on_a_worker_and_a_second_start_is_refused() {
    let (launcher, answers) = on_a_worker();
    let (shelf, mut state) = window(launcher);
    shelf.gate.shut();
    let at = Instant::now();
    state.rescan();
    state.rescan();
    assert!(at.elapsed() < FAST);
    assert!(state.is_running());
    assert!(
        state.report().is_none(),
        "nothing is shown that has not happened"
    );
    shelf.gate.open();
    state.finish(answers.recv_timeout(Duration::from_secs(5)).unwrap());
    assert!(state.report().is_some());
    assert!(
        answers.recv_timeout(Duration::from_millis(200)).is_err(),
        "only one call went out"
    );
}

#[test]
fn a_download_shows_running_then_done() {
    let (launcher, answers) = on_a_worker();
    let (shelf, mut state) = window(launcher);
    state.select("a");
    shelf.gate.shut();
    state.begin_download();
    state.set_input("/tmp/out.txt");
    state.submit_input();
    assert_eq!(state.transfers()[0].shown.state, TransferState::Running);
    shelf.gate.open();
    state.finish(answers.recv_timeout(Duration::from_secs(5)).unwrap());
    assert_eq!(state.transfers()[0].shown.state, TransferState::Done);
}

#[test]
fn the_widget_runs_the_long_call_on_its_own_worker_and_puts_the_answer_in() {
    use fs_components_workbenches::file_manager::FileManagerWidget;
    let shelf = Arc::new(Shelf::new().file("a"));
    let widget = FileManagerWidget::new(shelf.clone(), FileLabels::default());
    shelf.gate.shut();
    let at = Instant::now();
    widget.with_state(|s| {
        s.begin_upload();
        s.set_input("/tmp/new.txt");
        s.submit_input();
    });
    assert!(at.elapsed() < FAST, "the window is free");
    assert!(widget.with_state(|s| s.is_running()));
    shelf.gate.open();
    let until = Instant::now() + Duration::from_secs(5);
    while widget.with_state(|s| s.is_running()) {
        assert!(Instant::now() < until, "the answer never came in");
        std::thread::sleep(Duration::from_millis(10));
    }
    let state = widget.with_state(|s| s.clone());
    assert_eq!(state.transfers()[0].shown.state, TransferState::Done);
    assert_eq!(shelf.writes(), 1);
}
