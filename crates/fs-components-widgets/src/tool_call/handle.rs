//! The handle of a tool-call view.
//!
//! Pattern: Facade + Observer — a clonable handle over the calls (kept by id, updated in place);
//! no method returns a lock guard. A question is answered only by a click on an offered option,
//! and the answer goes to the caller as an [`Outcome`]: this widget never decides or executes.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::div::div;

use super::card::{Sinks, card};
use super::labels::Labels;
use super::model::{ToolCall, ToolChoice, ToolState};
use super::store::{Applied, ToolCalls};
use crate::Element;
use crate::live::Live;

/// What a person's click on a question reported.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The person chose an option. The host decides what that means; nothing has happened yet.
    Chose {
        /// The call's id.
        call: String,
        /// The option's id, as given.
        option: String,
        /// What the option said it meant.
        choice: ToolChoice,
    },
    /// Nothing reported: no such call, not a question now, or no such option.
    NotAsked,
}

/// Called with every answer.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

struct Inner {
    calls: ToolCalls,
    expanded: HashSet<String>,
    on_outcome: Option<OnOutcome>,
}

/// The calls of one place, drawn as cards. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct ToolCallView {
    shared: Arc<Mutex<Inner>>,
    live: Live,
    labels: Labels,
    id: String,
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
            })),
            live: Live::new(),
            labels: Labels::default(),
            id: "fs-toolcall".into(),
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

    /// A copy of the calls in first-seen order.
    pub fn calls(&self) -> Vec<ToolCall> {
        self.lock().calls.all().to_vec()
    }

    /// Do any calls want attention (a question or a failure)?
    pub fn needs_attention(&self) -> bool {
        self.lock()
            .calls
            .all()
            .iter()
            .any(ToolCall::needs_attention)
    }

    /// Show or hide the details of a call.
    pub fn toggle(&self, call: &str) {
        {
            let mut inner = self.lock();
            if !inner.expanded.remove(call) {
                inner.expanded.insert(call.to_string());
            }
        }
        self.live.refresh();
    }

    /// Are the details of this call shown?
    pub fn is_expanded(&self, call: &str) -> bool {
        self.lock().expanded.contains(call)
    }

    /// The answer a click on an option gives. Reports to the callback; changes no call.
    pub fn choose(&self, call: &str, option: &str) -> Outcome {
        let (outcome, handler) = {
            let inner = self.lock();
            let found = inner
                .calls
                .get(call)
                .filter(|c| c.state == ToolState::Question)
                .and_then(|c| c.options.iter().find(|o| o.id == option));
            let outcome = found.map_or(Outcome::NotAsked, |o| Outcome::Chose {
                call: call.to_string(),
                option: option.to_string(),
                choice: o.choice,
            });
            (outcome, inner.on_outcome.clone())
        };
        if let (Outcome::Chose { .. }, Some(handler)) = (&outcome, handler) {
            handler(&outcome);
        }
        outcome
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        self.live.view(move || this.render())
    }

    fn render(&self) -> blinc_layout::div::Div {
        let (toggle_view, choose_view) = (self.clone(), self.clone());
        let sinks = Sinks {
            toggle: Arc::new(move |id| toggle_view.toggle(id)),
            choose: Arc::new(move |call, option| {
                choose_view.choose(call, option);
            }),
        };
        let inner = self.lock();
        let mut list = div().class("fs-tcl");
        for call in inner.calls.all() {
            let open = inner.expanded.contains(&call.id);
            list = list.child(card(call, open, &self.labels, &self.id, &sinks));
        }
        list
    }
}
