//! Uploads, downloads, unfinished uploads, and the rescan.
//!
//! Pattern: State — the long calls run on a worker (`jobs.rs`), so a row shows a transfer running and then what
//! came of it. The rows are this session's, plus what the
//! source says is unfinished. Dismissing an unfinished upload would throw bytes away, so it only asks.

use std::sync::Arc;

use fs_components_widgets::transfer_list::{Action, Outcome, State as TransferState, Transfer};

use super::ask::{Ask, Confirm};
use super::command::Command;
use super::error::{FileError, FileProblem};
use super::labels::FileLabels;
use super::reports::RescanSummary;
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

    pub(super) fn next_transfer(&mut self) -> String {
        self.transfer_counter += 1;
        format!("t{}", self.transfer_counter)
    }

    pub(super) fn push_row(&mut self, row: TransferRow) {
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

    pub(super) fn transfer_done(&mut self, command: &Command, said: &str) {
        if let Command::Upload(path, _) = command {
            let key = self.next_transfer();
            self.push_row(TransferRow::upload(key, said, path, TransferState::Done));
            self.read_pending();
        }
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
                self.ask = Ask::Confirm(Confirm::DropUpload {
                    id: id.clone(),
                    name: row.shown.name.clone(),
                });
            }
            (Action::Retry, None) => {
                if let Some(path) = &row.path {
                    self.begin_upload_at(path);
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

    /// Bring in what changed outside, below the current folder. It runs on a worker: the window stays usable and
    /// shows "running"; a second start meanwhile is refused. It deletes nothing: what is missing is reported, and
    /// its record stays.
    pub fn rescan(&mut self) {
        if !self.capabilities.rescan {
            return;
        }
        let here = self.folder().map(str::to_string);
        self.start_rescan(here);
    }

    /// The answer of a rescan.
    pub(super) fn rescan_report(&mut self, summary: RescanSummary) {
        self.report = Some(Arc::new(summary));
        self.side = Side::Report;
        self.detail_open = true;
        self.after_change();
        self.notice = Some(Notice::Info(self.labels.rescan_done.clone()));
    }

    /// What the last rescan found.
    pub fn report(&self) -> Option<&RescanSummary> {
        self.report.as_deref()
    }
}
