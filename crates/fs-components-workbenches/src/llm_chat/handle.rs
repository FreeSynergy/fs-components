//! The handle of the chat.
//!
//! Pattern: Facade + Observer — a clonable handle that joins the message list, the text field,
//! the agent tree and the question dock over one [`ChatSource`]. No method returns a lock guard.
//! The parts' outcomes are flattened into one [`Outcome`] for the host. Nothing is decided here:
//! the stop and every answer are reported; the engine carries them out.

use std::sync::{Arc, Mutex, PoisonError};

use fs_components_widgets::agent_tree::{self, AgentTree};
use fs_components_widgets::live::Live;
use fs_components_widgets::message_composer::{self, MessageComposer};
use fs_components_widgets::message_list::{self, MessageList, MessageSource, SourceError};
use fs_components_widgets::tool_call::{Approvals, ToolCallView};

use super::labels::Labels;
use super::outcome::Outcome;
use super::source::ChatSource;

/// Called with every outcome.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;
type Sink = Arc<Mutex<Option<OnOutcome>>>;

/// The source's messages for the list.
struct Messages(Arc<dyn ChatSource>);

impl MessageSource for Messages {
    fn count(&self) -> usize {
        self.0.count()
    }

    fn messages(
        &self,
        first: usize,
        limit: usize,
    ) -> Result<Vec<message_list::ChatMessage>, SourceError> {
        self.0.messages(first, limit)
    }
}

/// The chat. Cheap to clone; clones share everything.
#[derive(Clone)]
pub struct LlmChat {
    pub(super) source: Arc<dyn ChatSource>,
    pub(super) list: MessageList,
    pub(super) composer: MessageComposer,
    pub(super) tree: AgentTree,
    pub(super) dock: ToolCallView,
    pub(super) layout: Live,
    pub(super) parts: Live,
    pub(super) labels: Labels,
    pub(super) id: String,
    pub(super) compact: bool,
    sink: Sink,
    had_questions: Arc<Mutex<usize>>,
    approvals: Approvals,
    pub(super) waiting: Arc<std::sync::atomic::AtomicUsize>,
}

impl LlmChat {
    /// The full chat over `source`: messages, agents on the side, the text field.
    pub fn new(source: Arc<dyn ChatSource>) -> Self {
        let sink: Sink = Arc::default();
        let id = "llm".to_string();
        let to = |sink: &Sink| {
            let sink = Arc::clone(sink);
            move |o: Outcome| {
                let handler = sink.lock().unwrap_or_else(PoisonError::into_inner).clone();
                if let Some(handler) = handler {
                    handler(&o);
                }
            }
        };
        let emit = to(&sink);
        let approvals = Approvals::new();
        let list = MessageList::new(Arc::new(Messages(Arc::clone(&source))))
            .id(format!("{id}-list"))
            .approvals(approvals.clone())
            .on_outcome(Arc::new(move |o| match o.clone() {
                message_list::Outcome::ToolChose(decision) => emit(Outcome::ToolChose(decision)),
                message_list::Outcome::Retry { message } => emit(Outcome::Retry { message }),
                message_list::Outcome::Reference { message, reference } => {
                    emit(Outcome::Reference { message, reference });
                }
                _ => {}
            }));
        let emit = to(&sink);
        let composer = MessageComposer::new(source.key())
            .id(format!("{id}-composer"))
            .attach_button(true)
            .on_outcome(Arc::new(move |o| match o.clone() {
                message_composer::Outcome::Send {
                    text,
                    attachments,
                    attempt,
                } => {
                    emit(Outcome::Send {
                        text,
                        attachments,
                        attempt,
                    });
                }
                message_composer::Outcome::RemoveAttachment { key } => {
                    emit(Outcome::RemoveAttachment { key });
                }
                message_composer::Outcome::AttachRequested => emit(Outcome::AttachRequested),
                _ => {}
            }));
        let emit = to(&sink);
        let tree = AgentTree::new()
            .id(format!("{id}-tree"))
            .on_outcome(Arc::new(move |o| {
                if let agent_tree::Outcome::Selected { key } = o {
                    emit(Outcome::AgentSelected { key: key.clone() });
                }
            }));
        let emit = to(&sink);
        let asked = Arc::clone(&source);
        let dock = ToolCallView::new()
            .id(format!("{id}-dock"))
            .approvals(approvals.clone())
            .verify(Arc::new(move |d| {
                // what the source holds now, not what was drawn
                asked.questions().iter().any(|p| {
                    p.message == d.message
                        && p.call.id == d.call
                        && p.call.state == fs_components_widgets::tool_call::ToolState::Question
                        && p.call.revision == d.revision
                        && p.call.digest == d.digest
                })
            }))
            .on_outcome(Arc::new(move |o| {
                if let fs_components_widgets::tool_call::Outcome::Chose(decision) = o.clone() {
                    emit(Outcome::ToolChose(decision));
                }
            }));
        let chat = Self {
            source,
            list,
            composer,
            tree,
            dock,
            layout: Live::new(),
            parts: Live::new(),
            labels: Labels::default(),
            id,
            compact: false,
            sink,
            had_questions: Arc::default(),
            approvals,
            waiting: Arc::default(),
        };
        chat.sync();
        chat
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.list = self.list.labels(labels.list.clone());
        self.composer = self.composer.labels(labels.composer.clone());
        self.tree = self.tree.labels(labels.tree.clone());
        self.dock = self.dock.labels(labels.tools.clone());
        self.labels = labels;
        self
    }

    /// The compact variant for embedding in another program: no side panel, a smaller text field,
    /// one line above that says how many agents need attention. Marks stay visible.
    #[must_use]
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self.composer = self.composer.compact(compact);
        self
    }

    /// Hear of every outcome.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        *self.sink.lock().unwrap_or_else(PoisonError::into_inner) = Some(handler);
        self
    }

    fn sync(&self) -> bool {
        self.tree.set_nodes(self.source.agents());
        let pending: Vec<_> = self
            .source
            .questions()
            .into_iter()
            .map(|p| (p.message, p.call))
            .collect();
        let now = pending.len();
        self.waiting
            .store(now, std::sync::atomic::Ordering::Relaxed);
        // only the first waiting question is the dock's; it has fixed room and its buttons never
        // scroll away, the others wait in their messages (and are counted in the heading)
        self.dock.replace(pending.into_iter().take(1).collect());
        let mut had = self
            .had_questions
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        std::mem::replace(&mut *had, now) != now
    }

    /// Use this guard (and its clock) for every place that can answer a question.
    #[must_use]
    pub fn approvals(mut self, approvals: Approvals) -> Self {
        self.list = self.list.approvals(approvals.clone());
        self.dock = self.dock.approvals(approvals.clone());
        self.approvals = approvals;
        self
    }

    /// The engine refused a decision (the digest or revision no longer fit, or the call is
    /// settled): the card shows the reason with the fresh question, and the person decides anew.
    pub fn decision_refused(&self, message: &str, call: &str, reason: &str) {
        self.list.decision_refused(message, call, reason);
        self.dock.refused_in(message, call, reason);
        self.layout.refresh_now();
    }

    /// The host's source changed (new text, agents, questions, stop state): draw what changed.
    /// The text field is not touched, so typing keeps its focus.
    pub fn refresh(&self) {
        self.list.refresh();
        if self.sync() {
            self.layout.refresh_now(); // a question appearing must not wait for a frame budget
        } else {
            self.parts.refresh();
        }
    }

    /// The host sent the message: the text field is cleared if it is unchanged.
    pub fn sent(&self) {
        self.composer.sent();
    }

    /// The host could not send it: the text stays, the reason is shown.
    pub fn failed(&self, reason: &str) {
        self.composer.failed(reason);
    }

    /// The text field (to set attachments, read the text, pass key presses).
    pub fn composer(&self) -> MessageComposer {
        self.composer.clone()
    }

    /// The message list (to jump to the end).
    pub fn message_list(&self) -> MessageList {
        self.list.clone()
    }

    /// The agent tree.
    pub fn agent_tree(&self) -> AgentTree {
        self.tree.clone()
    }

    pub(super) fn lock_sink_emit(&self, outcome: &Outcome) {
        let handler = self
            .sink
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(handler) = handler {
            handler(outcome);
        }
    }
}

impl LlmChat {
    /// The engine's refusal shown on the docked question, if any.
    pub(super) fn dock_refusal(&self) -> bool {
        self.dock.calls().first().is_some_and(|c| {
            self.approvals.refusal("", &c.id).is_some() || self.dock_message_refusal(&c.id)
        })
    }

    fn dock_message_refusal(&self, call: &str) -> bool {
        self.source
            .questions()
            .iter()
            .any(|p| p.call.id == call && self.approvals.refusal(&p.message, call).is_some())
    }
}
