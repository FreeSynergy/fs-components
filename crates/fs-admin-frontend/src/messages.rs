//! Messages with a lifetime (ADR 0065 point 4).
//!
//! Pattern: Observer with a lifetime per message. A message belongs to its occasion (the screen it was raised on):
//! success goes away by itself after a short time or when the screen changes, an error stays until it is closed or
//! solved, and neither is ever shown on another screen. Listeners are told after every change.

use std::sync::{Arc, Mutex, PoisonError};

use crate::clock::Clock;

/// How long a success stays, in milliseconds.
pub const SUCCESS_MS: u64 = 4000;

/// Success or failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    /// Gone after [`SUCCESS_MS`] or when the screen changes.
    Success,
    /// Stays until dismissed or solved; never silent (ADR 0065 point 5).
    Error,
}

/// One message: a code (translation key suffix, ADR 0057), never a sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// Identity, to dismiss it.
    pub id: u64,
    /// Success or error.
    pub kind: MessageKind,
    /// `message.<code>` is the translation key.
    pub code: String,
    /// The screen it was raised on.
    pub scope: String,
    /// When it was raised (clock milliseconds).
    pub born_ms: u64,
}

type Listener = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct State {
    next: u64,
    items: Vec<Message>,
    scope: String,
}

/// The messages of the window.
pub struct Messages {
    clock: Arc<dyn Clock>,
    state: Mutex<State>,
    listeners: Mutex<Vec<Listener>>,
}

impl Messages {
    /// No messages, on no screen yet.
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self {
            clock,
            state: Mutex::new(State::default()),
            listeners: Mutex::new(Vec::new()),
        }
    }

    /// Call `f` after every change.
    pub fn subscribe(&self, f: impl Fn() + Send + Sync + 'static) {
        self.listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Arc::new(f));
    }

    fn changed(&self) {
        // Copy the list and call outside the lock: a listener may ask the messages, or subscribe, without deadlock.
        let listeners: Vec<Listener> = self
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        for f in &listeners {
            f();
        }
    }

    /// The window shows another screen: everything raised elsewhere goes.
    pub fn enter(&self, scope: &str) {
        {
            let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            if s.scope == scope {
                return;
            }
            scope.clone_into(&mut s.scope);
            s.items.retain(|m| m.scope == scope);
        }
        self.changed();
    }

    /// Raise a message on the current screen.
    pub fn raise(&self, kind: MessageKind, code: &str) -> u64 {
        let id = {
            let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            s.next += 1;
            let id = s.next;
            let scope = s.scope.clone();
            s.items.push(Message {
                id,
                kind,
                code: code.to_owned(),
                scope,
                born_ms: self.clock.now_ms(),
            });
            id
        };
        self.changed();
        id
    }

    /// Close one message. Returns whether there was one.
    pub fn dismiss(&self, id: u64) -> bool {
        let found = {
            let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let before = s.items.len();
            s.items.retain(|m| m.id != id);
            s.items.len() != before
        };
        if found {
            self.changed();
        }
        found
    }

    /// The problem is solved: close the errors with this code, on whatever screen they were raised.
    pub fn solve(&self, code: &str) {
        let found = {
            let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let before = s.items.len();
            s.items
                .retain(|m| !(m.kind == MessageKind::Error && m.code == code));
            s.items.len() != before
        };
        if found {
            self.changed();
        }
    }

    /// What is to be shown now: old successes are dropped first.
    pub fn current(&self) -> Vec<Message> {
        let now = self.clock.now_ms();
        let (out, expired) = {
            let mut s = self.state.lock().unwrap_or_else(PoisonError::into_inner);
            let before = s.items.len();
            s.items.retain(|m| {
                m.kind == MessageKind::Error || now.saturating_sub(m.born_ms) < SUCCESS_MS
            });
            (s.items.clone(), s.items.len() != before)
        };
        if expired {
            self.changed();
        }
        out
    }
}
