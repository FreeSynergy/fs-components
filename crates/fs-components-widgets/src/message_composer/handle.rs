//! The handle of a message composer.
//!
//! Pattern: Facade + Observer — a clonable handle over the state, the text area and the optional
//! draft writer; no method returns a lock guard. Sending is an outcome; the host answers it with
//! `sent()` or `failed(reason)`. The text area sits outside the part that is drawn again, so
//! typing never loses focus.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area_state};

use super::drafts::Job;
use super::labels::Labels;
use super::model::{Attachment, ComposerDrafts, Outcome};
use super::state::Composing;
use crate::keys::{Key, KeyEvent};
use crate::live::Live;

/// Called with every outcome.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

pub(super) struct Inner {
    pub state: Composing,
    pub attachments: Vec<Attachment>,
    pub on_outcome: Option<OnOutcome>,
    pub job: Option<Arc<Job>>,
    pub note: Option<String>,
}

/// A composer. Cheap to clone; clones share everything.
#[derive(Clone)]
pub struct MessageComposer {
    pub(super) shared: Arc<Mutex<Inner>>,
    pub(super) live: Live,
    pub(super) area: SharedTextAreaState,
    pub(super) labels: Labels,
    pub(super) id: String,
    pub(super) key: String,
    pub(super) compact: bool,
    pub(super) attach: bool,
    pub(super) send_key: Option<Key>,
}

impl MessageComposer {
    /// A composer for the conversation `key` (the key names its draft).
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            shared: Arc::new(Mutex::new(Inner {
                state: Composing::new(),
                attachments: Vec::new(),
                on_outcome: None,
                job: None,
                note: None,
            })),
            live: Live::new(),
            area: text_area_state(),
            labels: Labels::default(),
            id: "fs-mc".into(),
            key: key.into(),
            compact: false,
            attach: false,
            send_key: None,
        }
    }

    pub(super) fn lock(&self) -> MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Element id prefix (`<id>-send`, `<id>-attach`, `<id>-remove-<key>`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// The small form: one line of text and a small button, for embedding in another program.
    #[must_use]
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Show an Attach button (it only reports `AttachRequested`).
    #[must_use]
    pub fn attach_button(mut self, on: bool) -> Self {
        self.attach = on;
        self
    }

    /// Send with Ctrl+`key` (usually Enter) when the frontend passes key presses to `press`.
    #[must_use]
    pub fn send_key(mut self, key: Key) -> Self {
        self.send_key = Some(key);
        self
    }

    /// Hear of every outcome.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.lock().on_outcome = Some(handler);
        self
    }

    /// Keep the draft in `store` and put back a saved one (only when nothing is typed yet).
    #[must_use]
    #[allow(clippy::needless_pass_by_value)] // an Arc is handed over, the job keeps a clone
    pub fn drafts(self, store: Arc<dyn ComposerDrafts>) -> Self {
        let job = Job::new(store.clone(), self.key.clone());
        match store.load(&self.key) {
            Ok(Some(text)) if !text.is_empty() && self.text().is_empty() => {
                self.set_text(&text);
                self.lock().note = Some(self.labels.restored.clone());
            }
            Err(why) => {
                self.lock().note = Some(self.labels.draft_failed.replacen("{}", &why, 1));
            }
            _ => {}
        }
        self.lock().job = Some(job);
        self
    }

    /// The text, exactly as held.
    pub fn text(&self) -> String {
        self.lock().state.text().to_owned()
    }

    /// Replace the text (also in the text area).
    pub fn set_text(&self, text: &str) {
        self.lock().state.set_text(text);
        self.area
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .set_value(text);
        self.live.refresh();
    }

    /// What was typed: the text area reports it here, and a frontend with its own input may too.
    pub fn typed(&self, text: &str) {
        let job = {
            let mut inner = self.lock();
            inner.state.set_text(text);
            inner.note = None;
            inner.job.clone()
        };
        if let Some(job) = job {
            let live = self.live;
            job.touch(text, move || live.refresh());
        }
    }

    /// Write the draft now (it is otherwise written shortly after typing stops).
    pub fn save_draft_now(&self) {
        if let Some(job) = self.lock().job.clone() {
            job.flush();
        }
        self.live.refresh();
    }

    /// Set the attachments listed (the host's list, as it is now).
    pub fn set_attachments(&self, list: Vec<Attachment>) {
        self.lock().attachments = list;
        self.live.refresh();
    }

    /// A copy of the attachments listed.
    pub fn attachments(&self) -> Vec<Attachment> {
        self.lock().attachments.clone()
    }

    fn report(&self, outcome: Outcome) -> Outcome {
        let handler = self.lock().on_outcome.clone();
        if let Some(handler) = handler {
            handler(&outcome);
        }
        self.live.refresh();
        outcome
    }

    /// Ask to send. Nothing is cleared: the host answers with `sent()` or `failed()`.
    pub fn send(&self) -> Outcome {
        let outcome = {
            let mut inner = self.lock();
            let keys = inner.attachments.iter().map(|a| a.key.clone()).collect();
            inner.state.send(keys)
        };
        if let Some(job) = self.lock().job.clone() {
            job.flush();
        }
        self.report(outcome)
    }

    /// The host sent the message (the attempt waiting): the text is cleared if it is unchanged.
    pub fn sent(&self) {
        let waiting = self.lock().state.waiting();
        if let Some(n) = waiting {
            self.sent_for(n);
        }
    }

    /// The host sent attempt `attempt`. A late answer for an older attempt changes nothing.
    pub fn sent_for(&self, attempt: u64) {
        let (cleared, job) = {
            let mut inner = self.lock();
            (inner.state.sent(attempt), inner.job.clone())
        };
        if cleared {
            self.area
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .set_value("");
            if let Some(job) = job {
                job.clear();
            }
        }
        self.live.refresh();
    }

    /// The host could not send the message (the attempt waiting): the text stays, the reason is shown.
    pub fn failed(&self, reason: &str) {
        let waiting = self.lock().state.waiting();
        if let Some(n) = waiting {
            self.failed_for(n, reason);
        }
    }

    /// The host could not send attempt `attempt`. A late answer for an older attempt changes nothing.
    pub fn failed_for(&self, attempt: u64, reason: &str) {
        self.lock().state.failed(attempt, reason);
        self.live.refresh();
    }

    /// Is a send waiting for its answer?
    pub fn is_sending(&self) -> bool {
        self.lock().state.is_sending()
    }

    /// The person asked to remove an attachment.
    pub fn remove_attachment(&self, key: &str) -> Outcome {
        self.report(Outcome::RemoveAttachment {
            key: key.to_owned(),
        })
    }

    /// The person asked to attach something.
    pub fn request_attachment(&self) -> Outcome {
        self.report(Outcome::AttachRequested)
    }

    /// A key press from the frontend: Ctrl + the send key sends; `None` when it means nothing here.
    pub fn press(&self, event: &KeyEvent) -> Option<Outcome> {
        (event.ctrl && Some(event.key) == self.send_key).then(|| self.send())
    }
}
