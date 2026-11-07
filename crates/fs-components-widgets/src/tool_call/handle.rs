//! The handle of a tool-call view.
//!
//! Pattern: Facade + Observer — a clonable handle over the calls (kept by id, updated in place);
//! no method returns a lock guard. A question is answered only by a click on an offered option,
//! and the answer goes to the caller as an [`Outcome`]: this widget never decides or executes.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::div;

use super::card::{Ctx, Sinks, card};
use super::guard::Approvals;
use super::labels::Labels;
use super::model::{Decision, ToolCall, ToolChoice, ToolState};
use super::store::{Applied, ToolCalls};
use crate::Element;
use crate::live::Live;

/// What a person's answer to a question reported.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The person chose an option, bound to the question as drawn. The host decides what that means;
    /// nothing has happened yet.
    Chose(Decision),
    /// Nothing reported: no such call, not a question now, no such option, or the call changed
    /// after what the person saw.
    NotAsked,
}

/// Asks the host, at answer time, whether the decision still fits what it holds now.
pub type Verify = Arc<dyn Fn(&Decision) -> bool + Send + Sync>;
/// Called with every answer.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

fn tag_of(message: &str, call: &str) -> String {
    format!("{message}\u{1}{call}")
}

struct Inner {
    calls: ToolCalls,
    expanded: HashSet<String>,
    on_outcome: Option<OnOutcome>,
    verify: Option<Verify>,
    body_h: Option<f32>,
}

/// The calls of one place, drawn as cards. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct ToolCallView {
    shared: Arc<Mutex<Inner>>,
    live: Live,
    labels: Labels,
    id: String,
    approvals: Approvals,
}

impl Default for ToolCallView {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolCallView {
    /// No calls yet.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Inner {
                calls: ToolCalls::new(),
                expanded: HashSet::new(),
                on_outcome: None,
                verify: None,
                body_h: None,
            })),
            live: Live::new(),
            labels: Labels::default(),
            id: "fs-toolcall".into(),
            approvals: Approvals::new(),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Element id prefix (cards are `<id>-card-<call>`, options `<id>-opt-<call>-<option>`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// The height of the window on the text of a question (the default fits the text, 100 to 150 px).
    #[must_use]
    pub fn question_height(self, px: f32) -> Self {
        self.lock().body_h = Some(px);
        self
    }

    /// Set the question window's height on a view that is already shared.
    pub fn set_question_height(&self, px: f32) {
        self.lock().body_h = Some(px);
    }

    /// Share the guard with other places that can answer the same questions (the first answer wins).
    #[must_use]
    pub fn approvals(mut self, approvals: Approvals) -> Self {
        self.approvals = approvals;
        self
    }

    /// The engine refused a decision for this call: shown on its card.
    pub fn refused(&self, call: &str, reason: &str) {
        self.approvals.refused("", call, reason);
        super::refresh_later(self.live, self.approvals.delay());
        self.live.refresh_now();
    }

    /// The engine refused a decision for this call in this message: shown on its card.
    pub fn refused_in(&self, message: &str, call: &str, reason: &str) {
        self.approvals.refused(message, call, reason);
        super::refresh_later(self.live, self.approvals.delay());
        self.live.refresh_now();
    }

    /// Ask the host at answer time whether the decision still fits what it holds now; an answer
    /// it denies is dropped.
    #[must_use]
    pub fn verify(self, check: Verify) -> Self {
        self.lock().verify = Some(check);
        self
    }

    /// Hear of every answer.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.lock().on_outcome = Some(handler);
        self
    }

    /// Add a call or replace the one with the same id, in place.
    pub fn apply(&self, call: ToolCall) -> Applied {
        let applied = self.lock().calls.apply(call);
        self.live.refresh();
        applied
    }

    /// Replace all calls by the host's list as it is now, each with the message it belongs to.
    /// Settled calls are final and are not revived by it.
    pub fn replace(&self, calls: Vec<(String, ToolCall)>) {
        {
            let mut inner = self.lock();
            if inner.calls.entries() == calls.as_slice() {
                return; // nothing changed: nothing to draw
            }
            inner.calls.replace(calls);
            let Inner {
                calls, expanded, ..
            } = &mut *inner;
            expanded.retain(|tag| {
                calls
                    .entries()
                    .iter()
                    .any(|(m, c)| &tag_of(m, &c.id) == tag)
            });
        }
        self.live.refresh();
    }

    /// Add or replace the call with this id in this message.
    pub fn apply_in(&self, message: &str, call: ToolCall) -> Applied {
        let applied = self.lock().calls.apply_in(message, call);
        self.live.refresh();
        applied
    }

    /// A copy of the calls in first-seen order.
    pub fn calls(&self) -> Vec<ToolCall> {
        self.lock().calls.all().into_iter().cloned().collect()
    }

    /// Do any calls want attention (a question or a failure)?
    pub fn needs_attention(&self) -> bool {
        self.lock()
            .calls
            .all()
            .into_iter()
            .any(ToolCall::needs_attention)
    }

    /// Show or hide the details of a call (in no message).
    pub fn toggle(&self, call: &str) {
        self.toggle_in("", call);
    }

    /// Show or hide the details of a call in a message.
    pub fn toggle_in(&self, message: &str, call: &str) {
        {
            let mut inner = self.lock();
            let tag = tag_of(message, call);
            if !inner.expanded.remove(&tag) {
                inner.expanded.insert(tag);
            }
        }
        self.live.refresh();
    }

    /// Are the details of this call shown?
    pub fn is_expanded(&self, call: &str) -> bool {
        self.lock().expanded.contains(&tag_of("", call))
    }

    /// Report an answer, if it still fits: the call must still be a question, of the revision and
    /// digest the person saw, and offer this option, meaning what the decision says it means. Otherwise nothing is reported.
    pub fn answer(&self, decision: Decision) -> Outcome {
        let (outcome, handler) = {
            let inner = self.lock();
            let fits = inner
                .calls
                .get_in(&decision.message, &decision.call)
                .is_some_and(|c| {
                    c.state == ToolState::Question
                        && c.revision == decision.revision
                        && c.digest == decision.digest
                        && (!c.digest.is_empty() || decision.choice == ToolChoice::Refuse)
                        && c.options
                            .iter()
                            .any(|o| o.id == decision.option && o.choice == decision.choice)
                });
            let fits = fits && inner.verify.as_ref().is_none_or(|v| v(&decision));
            let outcome = if fits {
                Outcome::Chose(decision)
            } else {
                Outcome::NotAsked
            };
            (outcome, inner.on_outcome.clone())
        };
        if let Outcome::Chose(d) = &outcome {
            self.approvals.commit(&d.message, &d.call, d.revision);
            if let Some(handler) = handler {
                handler(&outcome);
            }
        }
        outcome
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.view("cards", move || this.render())
    }

    fn render(&self) -> blinc_layout::div::Div {
        let (toggle_view, choose_view, live) = (self.clone(), self.clone(), self.live);
        let sinks = Sinks {
            toggle: Arc::new(move |message, id| toggle_view.toggle_in(message, id)),
            choose: Arc::new(move |decision| {
                choose_view.answer(decision);
            }),
            rearm: Arc::new(move |ms| crate::tool_call::refresh_later(live, ms)),
        };
        let inner = self.lock();
        let mut list = div().class("fs-tcl");
        self.approvals.begin_place(&self.id);
        let mut slot = 0;
        for (message, call) in inner.calls.entries() {
            let open = inner.expanded.contains(&tag_of(message, &call.id));
            let ctx = Ctx {
                approvals: self.approvals.clone(),
                message: message.clone(),
                place: self.id.clone(),
                slot,
                body_h: inner.body_h,
            };
            if call.state == ToolState::Question {
                slot += 1;
            }
            let prefix = if message.is_empty() {
                self.id.clone()
            } else {
                format!("{}-{message}", self.id)
            };
            list = list.child(card(call, open, &self.labels, &prefix, &sinks, &ctx));
        }
        self.approvals.end_place(&self.id);
        list
    }
}
