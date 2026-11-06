//! The calls of one place, kept by (message, id): an update replaces, it never appends a second
//! entry; the same id in two messages is two calls.
//!
//! Pattern: Repository (in memory) + State. The rules mirror what a host promises: a lower
//! revision than held is stale; a settled call is final (not even a whole new list revives it).

use std::collections::HashMap;

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

    /// Replace everything by the host's list as it is now. Settled calls stay as they are.
    pub fn replace(&mut self, calls: Vec<(String, ToolCall)>) {
        let settled: Vec<(String, ToolCall)> = self
            .calls
            .iter()
            .filter(|(_, c)| c.state == ToolState::Settled)
            .cloned()
            .collect();
        let mut fresh = Self::new();
        for (message, call) in calls {
            match settled
                .iter()
                .find(|(m, c)| *m == message && c.id == call.id)
            {
                Some((_, done)) => fresh.apply_in(&message, done.clone()),
                None => fresh.apply_in(&message, call),
            };
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
