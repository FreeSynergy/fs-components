//! The handle of a timeline: shared state, source, events, and the redraw.
//!
//! Pattern: Facade + Observer — one clonable handle over [`TimelineState`]; each call changes the
//! state, tells the host through [`TimelineEvents`], and asks the live area to draw again (only
//! the timeline is built again, never per frame).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use blinc_layout::selector::ScrollRef;
use fs_components_fields::{Look, Say};
use fs_components_widgets::Key;
use fs_components_widgets::live::Live;

use crate::model::Zoom;
use crate::source::{NoEvents, TimelineEvents, TimelineSource};
use crate::state::{Step, TimelineState};

/// A timeline. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct Timeline {
    state: Arc<Mutex<TimelineState>>,
    pub(crate) source: Arc<dyn TimelineSource>,
    events: Arc<dyn TimelineEvents>,
    pub(crate) say: Arc<dyn Say>,
    pub(crate) look: Look,
    live: Live,
    pub(crate) scroll: ScrollRef,
    /// Width of the view as built last, in pixels (as `f32` bits).
    width: Arc<AtomicU32>,
}

impl Timeline {
    /// A timeline over `source`, words from `say`, in `look`.
    pub fn new(
        state: TimelineState,
        source: Arc<dyn TimelineSource>,
        say: Arc<dyn Say>,
        look: Look,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(state)),
            source,
            events: Arc::new(NoEvents),
            say,
            look,
            live: Live::new(),
            scroll: ScrollRef::new(),
            width: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Tell `events` of the reader's choices.
    #[must_use]
    pub fn events(mut self, events: Arc<dyn TimelineEvents>) -> Self {
        self.events = events;
        self
    }

    /// A copy of the state.
    pub fn snapshot(&self) -> TimelineState {
        self.lock().clone()
    }

    fn lock(&self) -> MutexGuard<'_, TimelineState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn apply(&self, step: Step) -> Step {
        match &step {
            Step::Select(k) => self.events.select_item(k),
            Step::Zoomed(z) => self.events.zoomed(*z),
            Step::Ignored => return step,
        }
        self.live.refresh();
        step
    }

    /// Choose an item.
    pub fn select(&self, key: &str) -> Step {
        let step = self.lock().select(key);
        self.apply(step)
    }

    /// Set the zoom level.
    pub fn set_zoom(&self, zoom: Zoom) -> Step {
        let old = self.lock().zoom;
        let step = self.lock().set_zoom(zoom);
        if step != Step::Ignored {
            // The axis keeps its first day whatever the zoom, so the same day stays at the left of
            // the view: the offset scales with the pixels per day. Left alone it would stay as it
            // was and, when the content shrinks, point past its end.
            let (x, y) = self.scroll.offset();
            let scaled = x * zoom.px_per_day() / old.px_per_day();
            self.scroll
                .set_scroll_offset(scaled.min(self.max_scroll_x_at(zoom)), y);
        }
        self.apply(step)
    }

    /// How far the view is scrolled to the right, in pixels.
    pub fn scroll_x(&self) -> f32 {
        self.scroll.scroll_x()
    }

    /// Scroll the view to `x` pixels from the left of the axis.
    pub fn set_scroll_x(&self, x: f32) {
        let y = self.scroll.scroll_y();
        self.scroll.set_scroll_offset(x.max(0.0), y);
    }

    /// The largest `scroll_x` the content allows at the current zoom, from the items and the width
    /// of the view as built last (zero before the first build).
    pub fn max_scroll_x(&self) -> f32 {
        let zoom = self.lock().zoom;
        self.max_scroll_x_at(zoom)
    }

    fn max_scroll_x_at(&self, zoom: Zoom) -> f32 {
        let today = self.lock().today;
        let (first, last) = crate::layout::range(
            &self.source.items(),
            7,
            today.unwrap_or(crate::model::Day(0)),
        );
        let scale = crate::layout::Scale {
            origin: first,
            px: zoom.px_per_day(),
        };
        (scale.x_of(last.add_days(1)) - f32::from_bits(self.width.load(Ordering::Relaxed))).max(0.0)
    }

    /// A key press; true when the timeline used it.
    pub fn key(&self, key: Key) -> bool {
        let items = self.source.items();
        let step = self.lock().key(&items, key);
        self.apply(step) != Step::Ignored
    }

    /// Read the source again and draw: the host changed its data.
    pub fn refresh(&self) {
        self.live.refresh();
    }

    /// The element of `width` x `height` pixels. The keyboard binding sits on a stable outer box;
    /// inside it a live area is built again after each change, nothing else.
    pub fn build(
        &self,
        width: f32,
        height: f32,
    ) -> impl blinc_layout::prelude::ElementBuilder + use<> {
        self.width.store(width.to_bits(), Ordering::Relaxed);
        let (me, keys) = (self.clone(), self.clone());
        let inner = self.live.region("timeline", move || {
            crate::view::timeline(&me, width, height)
        });
        crate::view::bind_keys(
            blinc_layout::div::div().id("tl-root").w(width).h(height),
            Arc::new(move |k| keys.key(k)),
        )
        .child(inner)
    }
}
