//! The handle of a message list.
//!
//! Pattern: Facade + Observer + Memento — a clonable handle over the shared state (heights,
//! follow flag, what is open) and the scroll position; no method returns a lock guard. The host
//! changes its source and calls [`MessageList::refresh`]; what a person does comes back as an
//! [`Outcome`]. Nothing is decided here.

use std::collections::HashSet;
use std::ops::Range;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::heights::Heights;
use super::labels::Labels;
use super::model::MessageSource;
use super::window::covered;
use crate::list_view::ListPosition;
use crate::live::Live;
use crate::tool_call::ToolChoice;

/// What a person did that the host must answer.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A tool-call question was answered: nothing has happened yet, the host decides.
    ToolChose {
        /// The message holding the call.
        message: String,
        /// The call's id.
        call: String,
        /// The option's id.
        option: String,
        /// What the option said it meant.
        choice: ToolChoice,
    },
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
            live: Live::new(),
            position: ListPosition::new(),
            labels: Labels::default(),
            id: "fs-ml".into(),
            watch: Arc::default(),
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

    /// Scroll to the newest message and follow it from now on.
    pub fn jump_to_end(&self) {
        {
            let mut inner = self.lock();
            inner.follow = true;
            inner.unseen = 0;
        }
        self.live.refresh();
    }

    /// Does the list follow the newest message (the reader is at the bottom)?
    pub fn is_following(&self) -> bool {
        self.lock().follow
    }

    /// Messages that arrived while the reader was away from the bottom.
    pub fn unseen(&self) -> usize {
        self.lock().unseen
    }

    /// Pixels scrolled from the top.
    pub fn offset(&self) -> f32 {
        self.position.offset()
    }

    /// Total height of the conversation as known now (estimates and measurements).
    pub fn total_height(&self) -> f32 {
        self.lock().heights.total()
    }

    /// The messages built now (indexes).
    pub fn built(&self) -> Range<usize> {
        self.lock().built.clone()
    }

    /// Scroll by `pixels` (positive = towards newer messages), as a wheel would.
    pub fn scroll_by(&self, pixels: f32) {
        let y = (self.position.offset() + pixels).max(0.0);
        self.position.apply_offset(y);
        self.on_scroll(y);
    }

    /// The scroll position changed (called from the scroll watcher; also by tests).
    pub fn on_scroll(&self, y: f32) {
        let rebuild = {
            let mut inner = self.lock();
            if inner.built.is_empty() {
                return; // nothing drawn yet: the first drawing decides where the list stands
            }
            let view = inner.size.1;
            let at_end = y + view + 4.0 >= inner.heights.total();
            if at_end {
                inner.unseen = 0;
            }
            let changed = inner.follow != at_end;
            inner.follow = at_end;
            let built = inner.built.clone();
            changed || !covered(&inner.heights, &built, y, view)
        };
        if rebuild {
            // Called from inside the reactive graph (the scroll watcher): drawing again from here
            // would write a signal while the graph is locked, so it is done from outside.
            let live = self.live;
            std::thread::spawn(move || live.refresh());
        }
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
