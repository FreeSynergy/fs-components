//! The calls of one place, kept by id: an update replaces, it never appends a second entry.
//!
//! Pattern: Repository (in memory) + State. The rules mirror what a host promises: a lower
//! revision than held is stale; a settled call is final.

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
    calls: Vec<ToolCall>,
    index: HashMap<String, usize>,
}

impl ToolCalls {
    /// No calls.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or replace by id.
    pub fn apply(&mut self, call: ToolCall) -> Applied {
        let Some(&at) = self.index.get(&call.id) else {
            self.index.insert(call.id.clone(), self.calls.len());
            self.calls.push(call);
            return Applied::Added;
        };
        let held = &self.calls[at];
        if held.state == ToolState::Settled {
            return Applied::AlreadySettled;
        }
        if call.revision < held.revision {
            return Applied::Stale;
        }
        self.calls[at] = call;
        Applied::Updated
    }

    /// The call with this id.
    pub fn get(&self, id: &str) -> Option<&ToolCall> {
        self.index.get(id).map(|&i| &self.calls[i])
    }

    /// All calls in first-seen order.
    pub fn all(&self) -> &[ToolCall] {
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
