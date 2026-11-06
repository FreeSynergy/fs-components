//! The text view itself.
//!
//! Pattern: Adapter — [`TextView`] turns the neutral model into Blinc elements; links are only
//! reported through a [`LinkHandler`].

use std::sync::Arc;

use crate::Element;

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
}

impl TextView {
    /// A view of `blocks`.
    pub fn new(blocks: Vec<Block>) -> Self {
        Self {
            blocks,
            on_link: None,
        }
    }

    /// Report clicks on links to `handler`.
    #[must_use]
    pub fn on_link(mut self, handler: LinkHandler) -> Self {
        self.on_link = Some(handler);
        self
    }

    /// The element to place in a layout.
    pub fn build(&self) -> impl Element + use<> {
        let look = look::Look::current();
        blocks::blocks(&self.blocks, &look, self.on_link.as_ref(), 0).class("fs-tv")
    }
}
