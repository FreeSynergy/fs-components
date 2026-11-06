//! Where the detail page of a workbench is scrolled to.
//!
//! Pattern: Memento — a handle the widget keeps. What is shown on the page has a name; when that changes
//! the page starts at the top. Blinc keys a plain scroll by its call site, so without this a new item
//! would open wherever the last one was left.

use std::sync::{Arc, Mutex, PoisonError};

use blinc_layout::selector::ScrollRef;
use blinc_layout::widgets::scroll::{Scroll, ScrollPhysics, SharedScrollPhysics};

/// The scroll position of a page.
#[derive(Clone)]
pub(crate) struct PageScroll {
    physics: SharedScrollPhysics,
    scroll: ScrollRef,
    shown: Arc<Mutex<String>>,
}

impl Default for PageScroll {
    fn default() -> Self {
        Self {
            physics: Arc::new(Mutex::new(ScrollPhysics::default())),
            scroll: ScrollRef::new(),
            shown: Arc::new(Mutex::new(String::new())),
        }
    }
}

impl PageScroll {
    /// A scroller for the page showing `what`: at the top if that is something else than before.
    pub(crate) fn scroller(&self, what: &str) -> Scroll {
        let mut shown = self.shown.lock().unwrap_or_else(PoisonError::into_inner);
        if *shown != what {
            what.clone_into(&mut shown);
            self.physics
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .offset_y = 0.0;
            self.scroll.set_scroll_offset(0.0, 0.0);
        }
        Scroll::with_physics(Arc::clone(&self.physics)).bind(&self.scroll)
    }
}
