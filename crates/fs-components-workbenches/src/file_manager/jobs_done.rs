//! What the answer of a long call does to the window, one method per kind of call.
//!
//! Pattern: State — the second half of `jobs.rs` (`finish` hands each answer to one of these); each puts the answer in
//! through the paths that used to follow the blocking call, and only into a free place (never over what is open).

use fs_components_widgets::transfer_list::State as TransferState;

use super::ask::{Ask, InputAsk};
use super::error::{FileError, FileProblem};
use super::jobs::Reply;
use super::labels::FileLabels;
use super::side::TransferRow;
use super::state::{FilesState, Notice};

impl FilesState {
    pub(super) fn rescan_done(&mut self, answer: Result<Reply, FileError>) {
        match answer {
            Ok(Reply::Summary(summary)) => self.rescan_report(summary),
            Ok(Reply::Said(_)) => self.notice = None,
            Err(e) => self.failed(&e),
        }
    }

    pub(super) fn download_done(
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
                if matches!(self.ask, Ask::Nothing) {
                    self.ask_download_marked(key, name.to_string(), path);
                } else {
                    self.notice = Some(Notice::Problem(e.problem, self.problem_text(&e)));
                }
            }
            Err(e) => {
                let text = self.problem_text(&e);
                let failed = TransferState::Failed(text.clone());
                self.push_row(TransferRow::download(row, name, key, failed));
                self.notice = Some(Notice::Problem(e.problem, text));
                // Only into a free place: what the person opened meanwhile is not thrown away.
                if let Some(input) = input.filter(|_| matches!(self.ask, Ask::Nothing)) {
                    self.ask = Ask::Input(self.input_with_transfer_error(input));
                }
            }
            Ok(Reply::Summary(_)) => {}
        }
    }

    pub(super) fn resume_done(
        &mut self,
        input: InputAsk,
        path: &str,
        answer: Result<Reply, FileError>,
    ) {
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
                if matches!(self.ask, Ask::Nothing) {
                    self.input_failed(input, &e);
                } else {
                    self.after_change();
                    self.notice = Some(Notice::Problem(e.problem, self.problem_text(&e)));
                }
            }
            Ok(Reply::Summary(_)) => {}
        }
    }
}
