//! The surface as an element: one text area without a frame, as wide as its place and as tall as its text, the block menu under
//! the cursor.
//!
//! Pattern: Observer — the area depends on a signal that is counted up when the width of its place or its number of lines
//! changes, so it is built again with the room its text needs (the state of the area, text and cursor and focus, is kept by the
//! surface and survives every build).

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, PoisonError};

use blinc_layout::div::{Div, div};
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use blinc_layout::widgets::text_area::{SharedTextAreaState, refresh_text_area, text_area};
use fs_components_widgets::SlashLabels;

use super::surface::{Report, Surface};

/// Most rows the surface asks for.
const MOST_ROWS: usize = 4000;

/// The lines the text of `state` takes in an area `width` px wide (the number the state counts now while the width is not
/// known). The area is made once to take that width and lay its lines out, so the answer does not wait for a layout pass.
fn lines_at(state: &SharedTextAreaState, width: f32) -> usize {
    if width > 0.0 {
        drop(text_area(state).w(width).rows(1));
        refresh_text_area(state);
    } else {
        crate::view::measure_again(state);
    }
    state
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .visual_line_count()
        .clamp(1, MOST_ROWS)
}

impl Surface {
    /// The element of the surface. `report` gets the whole text after each change (`whole` makes it).
    pub fn view(
        self: &Arc<Self>,
        hint: &str,
        words: &SlashLabels,
        report: &Report,
        whole: Arc<dyn Fn() -> String + Send + Sync>,
    ) -> Div {
        *self.words.lock().unwrap_or_else(PoisonError::into_inner) = words.clone();
        {
            let (report, whole) = (Arc::clone(report), Arc::clone(&whole));
            self.announce_to(Arc::new(move || report(&whole())));
        }
        let (me, hint, report) = (Arc::clone(self), hint.to_owned(), Arc::clone(report));
        let (built, width) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicU32::new(0)));
        let (built_in, width_in, width_out) =
            (Arc::clone(&built), Arc::clone(&width), Arc::clone(&width));
        let grow = self.grow;
        let place = stateful::<NoState>()
            .deps([grow.id()])
            .on_state(move |_| {
                let w = f32::from_bits(width_in.load(Ordering::Relaxed));
                let lines = lines_at(&me.state, w);
                built_in.store(lines, Ordering::Relaxed);
                let (surface, report, whole, built) = (
                    Arc::clone(&me),
                    Arc::clone(&report),
                    Arc::clone(&whole),
                    Arc::clone(&built_in),
                );
                // The area of the layout, not the framed one of the theme: the surface has no box (its class takes the frame, the
                // shadow and the fill away).
                let area = text_area(&me.state)
                    .rows(lines)
                    .border_width(0.0)
                    .class("fs-wk-surface-area")
                    .placeholder(&hint)
                    .on_change(move |_| {
                        surface.on_changed();
                        report(&whole());
                        let now = surface
                            .state
                            .lock()
                            .unwrap_or_else(PoisonError::into_inner)
                            .visual_line_count()
                            .clamp(1, MOST_ROWS);
                        // More or fewer lines than the area was built for: build it again, with the room it needs.
                        if built.load(Ordering::Relaxed) != now {
                            surface.grow.update(|v| v.wrapping_add(1));
                        }
                    });
                // Until the layout has said how wide the place is, the area fills it; then it is that wide, so the lines
                // are cut at the real edge.
                let area = if w > 0.0 { area.w(w) } else { area.w_full() };
                div().class("fs-wk-surface").w_full().child(area)
            })
            .on_layout(move |b| {
                let old = f32::from_bits(width_out.swap(b.width.to_bits(), Ordering::Relaxed));
                if (old - b.width).abs() > 0.5 {
                    grow.update(|v| v.wrapping_add(1));
                }
            });
        let keys = Arc::clone(self);
        div()
            .class("fs-wk-blocks")
            .w_full()
            .relative()
            .on_key_down(move |e| keys.menu_key(e.key_code))
            .child(place)
            .child(self.menu_view())
    }
}
