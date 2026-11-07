//! The handle of a message list.
//!
//! Pattern: Facade + Observer + State — a clonable handle over the shared state (heights,
//! follow flag, what is open) and the scroll position; no method returns a lock guard. The host
//! changes its source and calls [`MessageList::refresh`]; what a person does comes back as an
//! [`Outcome`]. Nothing is decided here.

use std::collections::HashSet;
use std::ops::Range;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::heights::Heights;
use super::labels::Labels;
use super::model::MessageSource;
use crate::list_view::ListPosition;
use crate::live::Live;
use crate::tool_call::{Approvals, Decision, ToolCall, ToolState};

/// What a person did that the host must answer.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A tool-call question was answered: nothing has happened yet, the host decides.
    ToolChose(Decision),
    /// "Try again" on a message that failed.
    Retry {
        /// The message.
        message: String,
    },
    /// A reference (a task) was clicked: the host opens it.
    Reference {
        /// The message.
        message: String,
        /// The reference's key.
        reference: String,
    },
}

/// Called with every outcome.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;
/// Reads the laid-out height of an element by id (the default asks Blinc).
pub type Probe = Arc<dyn Fn(&str) -> Option<f32> + Send + Sync>;

pub(super) struct Inner {
    pub source: Arc<dyn MessageSource>,
    pub heights: Heights,
    pub revs: Vec<u64>,
    pub built: Range<usize>,
    pub shown: Vec<(usize, String)>,
    pub follow: bool,
    pub unseen: usize,
    pub open: HashSet<String>,
    pub size: (f32, f32),
    pub on_outcome: Option<OnOutcome>,
    pub probe: Probe,
}

/// A list of messages, virtualised. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct MessageList {
    pub(super) shared: Arc<Mutex<Inner>>,
    pub(super) live: Live,
    pub(super) position: ListPosition,
    pub(super) labels: Labels,
    pub(super) id: String,
    pub(super) watch: Arc<Mutex<Option<blinc_core::reactive::Effect>>>,
    pub(super) approvals: Approvals,
    pub(super) slots: Arc<std::sync::atomic::AtomicUsize>,
}

fn default_probe(id: &str) -> Option<f32> {
    blinc_layout::selector::query(id)?
        .bounds()
        .map(|b| b.height)
}

impl MessageList {
    /// A list over `source`, at the newest message.
    pub fn new(source: Arc<dyn MessageSource>) -> Self {
        let default = 96.0;
        let count = source.count();
        let inner = Inner {
            source,
            heights: Heights::new(count, default),
            revs: vec![0; count],
            built: 0..0,
            shown: Vec::new(),
            follow: true,
            unseen: 0,
            open: HashSet::new(),
            size: (600.0, 400.0),
            on_outcome: None,
            probe: Arc::new(default_probe),
        };
        Self {
            shared: Arc::new(Mutex::new(inner)),
            // the rows are a costly draw: at most four a second however fast the feed
            live: Live::new().gap(250),
            position: ListPosition::new(),
            labels: Labels::default(),
            id: "fs-ml".into(),
            watch: Arc::default(),
            approvals: Approvals::new(),
            slots: Arc::default(),
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

    /// Element id prefix (rows are `<id>-msg-<key>`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Share the guard with other places that answer the same questions (the first answer wins).
    #[must_use]
    pub fn approvals(mut self, approvals: Approvals) -> Self {
        self.approvals = approvals;
        self
    }

    /// The engine refused a decision for this call: shown on its card with the fresh question.
    pub fn decision_refused(&self, message: &str, call: &str, reason: &str) {
        self.approvals.refused(message, call, reason);
        crate::tool_call::refresh_later(self.live, self.approvals.delay());
        self.live.refresh_now();
    }

    /// The call as the source has it now (not as it was drawn).
    fn current_call(&self, message: &str, call: &str) -> Option<ToolCall> {
        let (source, index) = {
            let inner = self.lock();
            let index = inner
                .shown
                .iter()
                .find(|(_, k)| k == message)
                .map(|(i, _)| *i)?;
            (inner.source.clone(), index)
        };
        let found = source
            .messages(index, 1)
            .ok()?
            .into_iter()
            .find(|m| m.key == message)?;
        found.tool_calls.into_iter().find(|c| c.id == call)
    }

    /// Report an answer if the call is still the question that was drawn (a question, the same
    /// revision and digest, the option offered). Otherwise nothing is reported.
    pub fn answer(&self, decision: Decision) -> bool {
        let fits = self
            .current_call(&decision.message, &decision.call)
            .is_some_and(|c| {
                c.state == ToolState::Question
                    && c.revision == decision.revision
                    && c.digest == decision.digest
                    && (!c.digest.is_empty()
                        || decision.choice == crate::tool_call::ToolChoice::Refuse)
                    && c.options.iter().any(|o| o.id == decision.option)
            });
        if fits {
            self.approvals
                .commit(&decision.message, &decision.call, decision.revision);
            self.emit(&Outcome::ToolChose(decision));
        }
        fits
    }

    /// Hear of every outcome.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.lock().on_outcome = Some(handler);
        self
    }

    /// Where laid-out heights come from (the default asks Blinc; a test may ask a layout tree).
    #[must_use]
    pub fn probe(self, probe: Probe) -> Self {
        self.lock().probe = probe;
        self
    }

    pub(super) fn emit(&self, outcome: &Outcome) {
        let handler = self.lock().on_outcome.clone();
        if let Some(handler) = handler {
            handler(outcome);
        }
    }

    /// The host changed its source: read the count again, keep the place, count what is new.
    pub fn refresh(&self) {
        {
            let mut inner = self.lock();
            let count = inner.source.count();
            let old = inner.heights.len();
            if count < old {
                inner.heights.reset(count);
                inner.revs = vec![0; count];
                inner.built = 0..0;
            } else {
                inner.heights.grow(count);
                inner.revs.resize(count, 0);
                if count > old && !inner.follow {
                    inner.unseen += count - old;
                }
            }
        }
        self.live.refresh();
    }

    /// Is this tool call or reasoning open?
    pub fn is_open(&self, tag: &str) -> bool {
        self.lock().open.contains(tag)
    }

    /// Open or close a tool call (`t:<message>:<call>`) or the reasoning of a message (`r:<message>`).
    pub fn toggle_open(&self, tag: impl Into<String>) {
        let tag = tag.into();
        {
            let mut inner = self.lock();
            if !inner.open.remove(&tag) {
                inner.open.insert(tag);
            }
        }
        self.live.refresh();
    }
}
