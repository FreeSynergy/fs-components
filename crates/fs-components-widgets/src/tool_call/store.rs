//! The calls of one place, kept by (message, id): an update replaces, it never appends a second
//! entry; the same id in two messages is two calls.
//!
//! Pattern: State (a keyed in-memory store; no store trait, so not a Repository). The rules mirror what a host promises: a lower
//! revision than held is stale; a settled call is final (not even a whole new list revives it).

use std::collections::{HashMap, HashSet};

use super::model::{ToolCall, ToolState};

/// What [`ToolCalls::apply`] did.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Applied {
    /// A call with a new id: added at the end.
    Added,
    /// A call with a known id: replaced where it stood.
    Updated,
    /// Not applied: its revision is lower than the one held.
    Stale,
    /// Not applied: the call held is settled, which is final.
    AlreadySettled,
}

/// Calls in the order they first appeared, each once.
#[derive(Clone, Debug, Default)]
pub struct ToolCalls {
    calls: Vec<(String, ToolCall)>,
    index: HashMap<(String, String), usize>,
    /// The highest revision seen per call: a lower one is never taken, not even after the call left.
    floor: HashMap<(String, String), u64>,
    /// Calls that settled: final for good, even if a list leaves them out for a while.
    done: HashSet<(String, String)>,
}

impl ToolCalls {
    /// No calls.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace by id (the call belongs to no message).
    pub fn apply(&mut self, call: ToolCall) -> Applied {
        self.apply_in("", call)
    }

    /// Add or replace by (message, id).
    pub fn apply_in(&mut self, message: &str, call: ToolCall) -> Applied {
        let k = (message.to_owned(), call.id.clone());
        if self.done.contains(&k) {
            return Applied::AlreadySettled;
        }
        if self.floor.get(&k).is_some_and(|&f| call.revision < f) {
            return Applied::Stale;
        }
        self.floor.insert(k.clone(), call.revision);
        if call.state == ToolState::Settled {
            self.done.insert(k.clone());
        }
        let Some(&at) = self.index.get(&k) else {
            self.index.insert(k, self.calls.len());
            self.calls.push((message.to_owned(), call));
            return Applied::Added;
        };
        let held = &self.calls[at].1;
        if held.state == ToolState::Settled {
            return Applied::AlreadySettled;
        }
        if call.revision < held.revision {
            return Applied::Stale;
        }
        self.calls[at].1 = call;
        Applied::Updated
    }

    /// Replace everything by the host's list as it is now. Settled calls stay settled and a lower
    /// revision than one seen before is refused, also for a call the list left out for a while.
    pub fn replace(&mut self, calls: Vec<(String, ToolCall)>) {
        let held: Vec<(String, ToolCall)> = self.calls.clone();
        let mut fresh = Self {
            floor: self.floor.clone(),
            done: self.done.clone(),
            ..Self::new()
        };
        for (message, call) in calls {
            let k = (message.clone(), call.id.clone());
            // keep what is held for a settled call; the list cannot revive it
            let kept = held
                .iter()
                .find(|(m, c)| *m == message && c.id == call.id && c.state == ToolState::Settled);
            if let Some((_, done)) = kept {
                // named twice in the list: one entry
                if !fresh.index.contains_key(&k) {
                    fresh.index.insert(k, fresh.calls.len());
                    fresh.calls.push((message, done.clone()));
                }
                continue;
            }
            let id = call.id.clone();
            let applied = fresh.apply_in(&message, call);
            if matches!(applied, Applied::Stale | Applied::AlreadySettled) {
                // a stale copy: what is held stays
                let known = fresh.index.contains_key(&(message.clone(), id.clone()));
                if let Some((_, old)) = held
                    .iter()
                    .find(|(m, c)| !known && *m == message && c.id == id)
                {
                    fresh.index.insert((message.clone(), id), fresh.calls.len());
                    fresh.calls.push((message, old.clone()));
                }
            }
        }
        *self = fresh;
    }

    /// The call with this id (in no message).
    pub fn get(&self, id: &str) -> Option<&ToolCall> {
        self.get_in("", id)
    }

    /// The call with this id in this message.
    pub fn get_in(&self, message: &str, id: &str) -> Option<&ToolCall> {
        self.index
            .get(&(message.to_owned(), id.to_owned()))
            .map(|&i| &self.calls[i].1)
    }

    /// All calls in first-seen order.
    pub fn all(&self) -> Vec<&ToolCall> {
        self.calls.iter().map(|(_, c)| c).collect()
    }

    /// All calls with the message each belongs to.
    pub fn entries(&self) -> &[(String, ToolCall)] {
        &self.calls
    }

    /// How many calls there are.
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// Are there none?
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }
}
