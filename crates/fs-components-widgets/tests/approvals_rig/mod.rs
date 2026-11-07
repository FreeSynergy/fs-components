//! Test support for the approval tests: one question, a clock by hand, a view that counts answers.
#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::chat_support::Manual;
use crate::common::layout;
use fs_components_widgets::ToolCallView;
use fs_components_widgets::tool_call::{
    Approvals, ToolCall, ToolChoice, ToolKind, ToolOption, ToolRisk,
};

pub fn q(id: &str, command: &str, digest: &[u8]) -> ToolCall {
    ToolCall::new(id, "shell")
        .kind(ToolKind::RunsCommand)
        .risk(ToolRisk::Destroys)
        .summary("Clean up")
        .detail(command)
        .asking(vec![
            ToolOption::new("no", "Refuse", ToolChoice::Refuse),
            ToolOption::new("once", "Allow once", ToolChoice::AllowOnce),
        ])
        .revision(1)
        .digest(digest.to_vec())
}

pub struct Rig {
    pub clock: Arc<Manual>,
    pub approvals: Approvals,
    pub view: ToolCallView,
    pub count: Arc<AtomicUsize>,
}

pub fn rig(id: &str) -> Rig {
    let clock = Arc::new(Manual::default());
    let approvals = Approvals::with_clock(clock.clone());
    let count = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&count);
    let view = ToolCallView::new()
        .id(id)
        .approvals(approvals.clone())
        .on_outcome(Arc::new(move |_| {
            c.fetch_add(1, Ordering::Relaxed);
        }));
    Rig {
        clock,
        approvals,
        view,
        count,
    }
}

impl Rig {
    pub fn draw(&self) -> blinc_layout::prelude::RenderTree {
        layout(self.view.build(), 520.0, 900.0)
    }
}
