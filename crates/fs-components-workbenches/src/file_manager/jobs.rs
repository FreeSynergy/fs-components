//! The long calls, off the interface thread: rescan, upload, download, and going on with an upload.
//!
//! Pattern: Command object + State, the split of the admin frontend's `Session::submit`. The state method that
//! begins a call (`start`) checks that nothing else is running, takes what the call needs into a [`FileJob`], shows
//! "running" and returns at once; [`FileJob::run`] makes the call on a worker with no lock held; [`FilesState::finish`]
//! puts the answer in under the lock, through the paths that used to follow the blocking call. While a job is on its
//! way a second start is refused (never queued: a write is not carried out twice), and the transfer row says
//! "running", not what has not happened yet.
//!
//! A source that cannot be asked while it works says so in its own call (the short calls stay synchronous, with
//! their own time limit — DESIGN.md); the workbench does not hide that behind a lie.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use fs_components_widgets::transfer_list::State as TransferState;

use super::ask::{Ask, InputAsk};
use super::command::Command;
use super::error::{FileError, FileProblem};
use super::labels::FileLabels;
use super::model::NameRule;
use super::reports::RescanSummary;
use super::side::TransferRow;
use super::source::FileSource;
use super::state::{FilesState, Notice};

/// What a worker is to call.
#[derive(Clone, Debug)]
enum Call {
    Rescan(Option<String>),
    Upload(String, Option<String>, NameRule),
    Download(String, String, bool),
    Resume(String, String),
}

/// What it gave back.
enum Reply {
    Summary(RescanSummary),
    Said(String),
}

/// What a call needs, taken from the state so that the work needs no lock.
#[derive(Clone)]
pub struct FileJob {
    source: Arc<dyn FileSource>,
    call: Call,
}

/// What a call found out, to be put into the state by [`FilesState::finish`].
pub struct JobDone {
    answer: Result<Reply, FileError>,
}

impl JobDone {
    /// A job that could not even be started (no thread to run it on).
    #[must_use]
    pub fn not_started(why: &str) -> Self {
        Self {
            answer: Err(FileError::failed(why)),
        }
    }
}

impl FileJob {
    /// Make the call. A source that panics is a failed call, never one that does not end: the window would stay
    /// "running" for good.
    #[must_use]
    pub fn run(self) -> JobDone {
        let s = &self.source;
        let answer = catch_unwind(AssertUnwindSafe(|| match &self.call {
            Call::Rescan(folder) => s.rescan(folder.as_deref()).map(Reply::Summary),
            Call::Upload(path, to, rule) => s.upload(path, to.as_deref(), *rule).map(Reply::Said),
            Call::Download(key, path, release) => s.download(key, path, *release).map(Reply::Said),
            Call::Resume(id, path) => s.resume_upload(id, path).map(Reply::Said),
        }))
        .unwrap_or_else(|_| Err(FileError::failed("the call panicked")));
        JobDone { answer }
    }
}

/// How a job is run: the widget hands each job to a worker. `Err` = no worker could be had.
pub type Launcher = Arc<dyn Fn(FileJob) -> Result<(), String> + Send + Sync>;

/// What is on its way, and what is needed to finish it.
#[derive(Clone)]
pub(super) struct Running {
    row: Option<String>,
    started: Started,
}

#[derive(Clone)]
enum Started {
    Rescan,
    Upload(Command, NameRule, Option<InputAsk>),
    Download {
        key: String,
        name: String,
        path: String,
        released: bool,
        input: Option<InputAsk>,
    },
    Resume(InputAsk, String),
}

impl FilesState {
    /// Whether a long call is on its way.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.is_some()
    }

    /// How jobs are run. Without a launcher a job waits to be taken with [`FilesState::take_job`].
    pub fn set_launcher(&mut self, launcher: Launcher) {
        self.launcher = Some(launcher);
    }

    /// The job that waits to be run (only when there is no launcher).
    pub fn take_job(&mut self) -> Option<FileJob> {
        self.parked.take()
    }

    fn start(&mut self, call: Call, started: Started, row: Option<TransferRow>) -> bool {
        if self.running.is_some() {
            self.notice = Some(Notice::Info(self.labels.chrome.busy.clone()));
            return false;
        }
        let key = row.as_ref().map(|r| r.shown.key.clone());
        if let Some(row) = row {
            self.push_row(row);
        }
        self.running = Some(Running { row: key, started });
        self.ask = Ask::Nothing;
        self.notice = Some(Notice::Info(self.labels.chrome.running.clone()));
        let job = FileJob {
            source: Arc::clone(&self.source),
            call,
        };
        match self.launcher.clone() {
            Some(launch) => {
                if let Err(why) = launch(job) {
                    self.finish(JobDone::not_started(&why));
                }
            }
            None => self.parked = Some(job),
        }
        true
    }

    /// Bring in what changed outside, below the current folder. The window stays usable.
    pub(super) fn start_rescan(&mut self, folder: Option<String>) {
        self.start(Call::Rescan(folder), Started::Rescan, None);
    }

    /// Upload, the person's input kept in case it does not go.
    pub(super) fn start_upload(
        &mut self,
        command: Command,
        rule: NameRule,
        from: Option<InputAsk>,
    ) {
        let Command::Upload(path, to) = &command else {
            return;
        };
        let row = self.running_upload_row(path);
        let call = Call::Upload(path.clone(), to.clone(), rule);
        self.start(call, Started::Upload(command, rule, from), Some(row));
    }

    pub(super) fn start_download(
        &mut self,
        (key, name, path): (&str, &str, &str),
        released: bool,
        input: Option<InputAsk>,
    ) {
        let id = self.next_transfer();
        let row = TransferRow::download(id, name, key, TransferState::Running);
        let started = Started::Download {
            key: key.to_string(),
            name: name.to_string(),
            path: path.to_string(),
            released,
            input,
        };
        let call = Call::Download(key.to_string(), path.to_string(), released);
        self.start(call, started, Some(row));
    }

    pub(super) fn start_resume(&mut self, input: InputAsk, id: &str, path: &str) {
        let call = Call::Resume(id.to_string(), path.to_string());
        self.start(call, Started::Resume(input, path.to_string()), None);
    }

    fn running_upload_row(&mut self, path: &str) -> TransferRow {
        let key = self.next_transfer();
        TransferRow::upload(key, path, path, TransferState::Running)
    }

    /// Put the answer of the job in. An answer that nobody waits for (a second one) is dropped.
    pub fn finish(&mut self, done: JobDone) {
        let Some(running) = self.running.take() else {
            return;
        };
        if let Some(row) = &running.row {
            let rows = self.transfers.iter().filter(|r| &r.shown.key != row);
            self.transfers = Arc::new(rows.cloned().collect());
        }
        match (running.started, done.answer) {
            (Started::Rescan, answer) => self.rescan_done(answer),
            (Started::Upload(command, rule, from), answer) => {
                let said = answer.map(|r| match r {
                    Reply::Said(n) => FileLabels::fill(&self.labels.uploaded, &n),
                    Reply::Summary(_) => String::new(),
                });
                self.settled(command, rule, from, said);
            }
            (
                Started::Download {
                    key,
                    name,
                    path,
                    released,
                    input,
                },
                answer,
            ) => self.download_done((&key, &name, &path), released, input, answer),
            (Started::Resume(input, path), answer) => self.resume_done(input, &path, answer),
        }
    }

    fn rescan_done(&mut self, answer: Result<Reply, FileError>) {
        match answer {
            Ok(Reply::Summary(summary)) => self.rescan_report(summary),
            Ok(Reply::Said(_)) => self.notice = None,
            Err(e) => self.failed(&e),
        }
    }

    fn download_done(
        &mut self,
        (key, name, path): (&str, &str, &str),
        released: bool,
        input: Option<InputAsk>,
        answer: Result<Reply, FileError>,
    ) {
        let row = self.next_transfer();
        match answer {
            Ok(Reply::Said(place)) => {
                self.push_row(TransferRow::download(row, name, key, TransferState::Done));
                self.notice = Some(Notice::Info(FileLabels::fill(
                    &self.labels.downloaded,
                    &place,
                )));
            }
            Err(e) if e.problem == FileProblem::MarkedNeedsRelease && !released => {
                self.ask_download_marked(key, name.to_string(), path);
            }
            Err(e) => {
                let text = self.problem_text(&e);
                let failed = TransferState::Failed(text.clone());
                self.push_row(TransferRow::download(row, name, key, failed));
                self.notice = Some(Notice::Problem(e.problem, text));
                if let Some(input) = input {
                    self.ask = Ask::Input(self.input_with_transfer_error(input));
                }
            }
            Ok(Reply::Summary(_)) => {}
        }
    }

    fn resume_done(&mut self, input: InputAsk, path: &str, answer: Result<Reply, FileError>) {
        match answer {
            Ok(Reply::Said(name)) => {
                let key = self.next_transfer();
                let said = FileLabels::fill(&self.labels.uploaded, &name);
                self.push_row(TransferRow::upload(key, &said, path, TransferState::Done));
                self.read_pending();
                self.after_change();
                self.notice = Some(Notice::Info(said));
            }
            Err(e) => {
                self.read_pending();
                self.input_failed(input, &e);
            }
            Ok(Reply::Summary(_)) => {}
        }
    }
}
