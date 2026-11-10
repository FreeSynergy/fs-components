//! Scrolling of the timeline: where the view is, where it was asked to be, and how a zoom keeps it.
//!
//! Pattern: State, part of `Timeline` (the handle is in `handle.rs`). A scroll asked for counts at
//! once, before the view has drawn it, but only for a moment: after that the view is the one to ask,
//! so a reader who scrolled is not answered with an old request.

use std::sync::PoisonError;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use crate::handle::Timeline;
use crate::model::Zoom;

/// How long a scroll asked for counts before the view has drawn it: a frame is much shorter.
const ASKED_FOR: Duration = Duration::from_millis(100);

impl Timeline {
    /// The axis keeps its first day whatever the zoom, so the same day stays at the left of the
    /// view: the offset scales with the pixels per day and stays inside the content. Left alone it
    /// would stay as it was and, when the content shrinks, point past its end.
    pub(crate) fn rescale(&self, old: Zoom, new: Zoom) {
        let scaled = self.scroll_x() * new.px_per_day() / old.px_per_day();
        self.put_x(scaled.min(self.max_scroll_x_at(new)));
    }

    /// Ask the view to scroll to `x`, and remember it until the view has done so.
    pub(crate) fn put_x(&self, x: f32) {
        let (base, y) = self.scroll.offset();
        *self.target.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((x, base, Instant::now()));
        self.scroll.set_scroll_offset(x, y);
    }

    /// The width of the view is known now: a scroll asked for before (with the width taken as zero)
    /// or at another width is brought inside the content.
    pub(crate) fn reclamp(&self) {
        let asked = *self.target.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((x, _, _)) = asked {
            let max = self.max_scroll_x();
            if x > max {
                self.put_x(max);
            }
        }
    }

    /// How far the view is scrolled to the right, in pixels. A scroll asked for with
    /// `set_scroll_x` or by a zoom change counts at once, before the view has drawn it.
    pub fn scroll_x(&self) -> f32 {
        let shown = self.scroll.scroll_x();
        let mut target = self.target.lock().unwrap_or_else(PoisonError::into_inner);
        match *target {
            // Still where it was when asked, and asked a moment ago: the view has not drawn it yet.
            // After that the view is the one to ask: a reader who scrolled back to where the view
            // was at the time must not be answered with an old request.
            Some((x, base, at)) if (shown - base).abs() < 0.5 && at.elapsed() < ASKED_FOR => x,
            _ => {
                *target = None;
                shown
            }
        }
    }

    /// Scroll the view to `x` pixels from the left of the axis, kept between 0 and `max_scroll_x`.
    pub fn set_scroll_x(&self, x: f32) {
        // NaN and infinity are no position: they mean the start.
        let x = if x.is_finite() { x } else { 0.0 };
        self.put_x(x.clamp(0.0, self.max_scroll_x()));
    }

    /// The largest `scroll_x` the content allows at the current zoom, from the items and the width
    /// of the view as built last. Before the first build the width of the view counts as zero, so it is
    /// the width of the whole axis.
    pub fn max_scroll_x(&self) -> f32 {
        let zoom = self.lock().zoom;
        self.max_scroll_x_at(zoom)
    }

    pub(crate) fn max_scroll_x_at(&self, zoom: Zoom) -> f32 {
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
}
