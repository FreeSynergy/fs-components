//! Uploads, downloads, unfinished uploads, and the rescan.
//!
//! Pattern: State — the source's calls block until they are done (provisional, DESIGN.md), so a row
//! never shows a transfer running: it shows what came of it. The rows are this session's, plus what the
//! source says is unfinished. Dismissing an unfinished upload would throw bytes away, so it only asks.

use std::sync::Arc;

use fs_components_widgets::transfer_list::{Action, Outcome, State as TransferState, Transfer};

use super::ask::{Ask, Confirm, InputAsk};
use super::command::Command;
use super::error::{FileError, FileProblem};
use super::labels::FileLabels;
use super::reports::{RescanSummary, Transferred};
use super::side::{Side, TransferRow};
use super::state::{FilesState, Notice};

impl FilesState {
    /// Show the transfers beside the list, with the unfinished uploads read again.
    pub fn open_transfers(&mut self) {
        self.side = Side::Transfers;
        self.detail_open = true;
        self.read_pending();
    }

    /// The rows of the transfers.
    pub fn transfers(&self) -> &Arc<Vec<TransferRow>> {
        &self.transfers
    }

    fn next_transfer(&mut self) -> String {
        self.transfer_counter += 1;
        format!("t{}", self.transfer_counter)
    }

    fn push_row(&mut self, row: TransferRow) {
        let mut rows = self.transfers.as_ref().clone();
        rows.push(row);
        self.transfers = Arc::new(rows);
    }

    /// The unfinished uploads as rows; the others stay as they are. A failure says why.
    pub(super) fn read_pending(&mut self) {
        if !self.capabilities.upload {
            return;
        }
        match self.source.pending_uploads() {
            Ok(pending) => {
                let mut rows: Vec<TransferRow> = self
                    .transfers
                    .iter()
                    .filter(|row| row.pending.is_none())
                    .cloned()
                    .collect();
                for upload in pending {
                    let shown = Transfer::new(
                        format!("pending-{}", upload.id),
                        upload.name,
                        fs_components_widgets::transfer_list::Direction::Upload,
                    )
                    .with_state(TransferState::Failed(self.labels.unfinished.clone()))
                    .with_progress(upload.received, None);
                    rows.push(TransferRow {
                        shown,
                        pending: Some(upload.id),
                        path: None,
                        file: None,
                    });
                }
                self.transfers = Arc::new(rows);
            }
            Err(e) => {
                let text = self.problem_text(&e);
                self.notice = Some(Notice::Problem(e.problem, text));
            }
        }
    }

    pub(super) fn upload_done(&mut self, path: &str, done: &Transferred) {
        let key = self.next_transfer();
        let mut row = TransferRow::upload(key, &done.name, path, TransferState::Done);
        row.shown = row.shown.with_progress(done.bytes, Some(done.bytes));
        self.push_row(row);
        self.read_pending();
    }

    pub(super) fn transfer_failed(&mut self, command: &Command, e: &FileError) {
        if let Command::Upload(path, _) = command
            && e.problem != FileProblem::NameConflict
        {
            let key = self.next_transfer();
            let reason = self.problem_text(e);
            self.push_row(TransferRow::upload(
                key,
                path,
                path,
                TransferState::Failed(reason),
            ));
            self.read_pending();
        }
    }

    /// Write a file out; `release_marked` only after the person's explicit act. Tells whether it went.
    pub(super) fn download_now(
        &mut self,
        key: &str,
        name: &str,
        path: &str,
        release_marked: bool,
    ) -> bool {
        let row = self.next_transfer();
        match self.source.download(key, path, release_marked) {
            Ok(done) => {
                self.ask = Ask::Nothing;
                let mut line = TransferRow::download(row, name, key, TransferState::Done);
                line.shown = line.shown.with_progress(done.bytes, Some(done.bytes));
                self.push_row(line);
                self.notice = Some(Notice::Info(FileLabels::fill(
                    &self.labels.downloaded,
                    &done.name,
                )));
                true
            }
            Err(e) if e.problem == FileProblem::MarkedNeedsRelease && !release_marked => {
                self.ask_download_marked(key, name.to_string(), path);
                false
            }
            Err(e) => {
                let text = self.problem_text(&e);
                self.push_row(TransferRow::download(
                    row,
                    name,
                    key,
                    TransferState::Failed(text.clone()),
                ));
                self.notice = Some(Notice::Problem(e.problem, text));
                false
            }
        }
    }

    pub(super) fn resume_now(&mut self, input: InputAsk, id: &str, path: &str) {
        match self.source.resume_upload(id, path) {
            Ok(done) => {
                self.ask = Ask::Nothing;
                let said = FileLabels::fill(&self.labels.uploaded, &done.name);
                self.upload_done(path, &done);
                self.after_change();
                self.notice = Some(Notice::Info(said));
            }
            Err(e) => {
                self.read_pending();
                self.input_failed(input, &e);
            }
        }
    }

    pub(super) fn drop_now(&mut self, id: &str, name: &str) {
        match self.source.drop_upload(id) {
            Ok(()) => {
                self.read_pending();
                self.notice = Some(Notice::Info(FileLabels::fill(&self.labels.dropped, name)));
            }
            Err(e) => {
                self.read_pending();
                self.failed(&e);
            }
        }
    }

    /// A button of a transfer was pressed.
    pub fn transfer_outcome(&mut self, outcome: &Outcome) {
        let Some(row) = self
            .transfers
            .iter()
            .find(|r| r.shown.key == outcome.key)
            .cloned()
        else {
            return;
        };
        match (outcome.action, &row.pending) {
            (Action::Retry, Some(id)) => self.begin_resume(id.clone(), &row.shown.name),
            (Action::Dismiss, Some(id)) => {
                if self.typing() {
                    return;
                }
                self.ask = Ask::Confirm(Confirm::DropUpload {
                    id: id.clone(),
                    name: row.shown.name.clone(),
                });
            }
            (Action::Retry, None) => {
                if let Some(path) = &row.path {
                    self.begin_upload();
                    self.set_input(path);
                } else if let Some(file) = &row.file {
                    self.begin_download_of(file.clone(), row.shown.name.clone());
                }
            }
            _ => {
                let rows = self
                    .transfers
                    .iter()
                    .filter(|r| r.shown.key != outcome.key)
                    .cloned()
                    .collect();
                self.transfers = Arc::new(rows);
            }
        }
    }

    /// Bring in what changed outside, below the current folder. Holds the window until it is done. It
    /// deletes nothing: what is missing is reported, and its record stays.
    pub fn rescan(&mut self) {
        if !self.capabilities.rescan {
            return;
        }
        let here = self.folder().map(str::to_string);
        match self.source.rescan(here.as_deref()) {
            Ok(summary) => {
                self.report = Some(Arc::new(summary));
                self.side = Side::Report;
                self.detail_open = true;
                self.after_folder_change();
                self.notice = Some(Notice::Info(self.labels.rescan_done.clone()));
            }
            Err(e) => self.failed(&e),
        }
    }

    /// What the last rescan found.
    pub fn report(&self) -> Option<&RescanSummary> {
        self.report.as_deref()
    }
}
