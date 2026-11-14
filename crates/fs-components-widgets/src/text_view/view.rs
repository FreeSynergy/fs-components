//! The text view itself.
//!
//! Pattern: Adapter — [`TextView`] turns the neutral model into Blinc elements; links are only
//! reported through a [`LinkHandler`].

use std::sync::Arc;

use crate::Element;

use super::task::TaskHandler;
use super::{blocks, look};
use crate::text::Block;

/// Called with the `href` of a clicked link, exactly as written in the text.
pub type LinkHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// Opens an address in the system's default program (web page, mail).
pub fn open_external(url: &str) {
    blinc_layout::widgets::link::open_url(url);
}

/// Draws blocks.
#[derive(Clone, Default)]
pub struct TextView {
    blocks: Vec<Block>,
    on_link: Option<LinkHandler>,
    reading: bool,
    tasks: Option<(usize, Option<TaskHandler>)>,
}

impl TextView {
    /// A view of `blocks`.
    pub fn new(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            on_link: None,
            reading: false,
            tasks: None,
        }
    }

    /// Report clicks on links to `handler`.
    #[must_use]
    pub fn on_link(mut self, handler: LinkHandler) -> Self {
        self.on_link = Some(handler);
        self
    }

    /// Make the checkboxes clickable: a click calls `handler` with the number of the box (document order, counting from
    /// `first`, for a view that shows only some blocks of a page) and the state it should get.
    #[must_use]
    pub fn on_task(mut self, handler: TaskHandler, first: usize) -> Self {
        self.tasks = Some((first, Some(handler)));
        self
    }

    /// Show the checkboxes locked: their state is drawn, a click does nothing (no right to change the page).
    #[must_use]
    pub fn tasks_locked(mut self, first: usize) -> Self {
        self.tasks = Some((first, None));
        self
    }

    /// The reading profile (wiki pages): 16 px text at line height 1.5, larger headings.
    #[must_use]
    pub fn reading(mut self) -> Self {
        self.reading = true;
        self
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let mut look = if self.reading {
            look::Look::reading()
        } else {
            look::Look::current()
        };
        look.tasks = self.tasks.as_ref().map(|(first, handler)| {
            std::sync::Arc::new(super::task::Tasks::new(*first, handler.clone()))
        });
        blocks::blocks(&self.blocks, &look, self.on_link.as_ref(), 0).class("fs-tv")
    }
}
