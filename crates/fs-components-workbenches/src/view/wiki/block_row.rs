//! One block of the editor as an element: a grip, the text area at the width of its place, the menu under it.
//!
//! Pattern: Observer — the block depends on a signal of its own that is counted up when the width of its place or its
//! number of lines changes, so it is built again with the room its text needs (the area keeps its state).

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use std::sync::{Arc, PoisonError};

use blinc_cn::components::textarea::textarea;
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::NoState;
use blinc_layout::stateful::stateful;
use blinc_layout::widgets::text_area::{SharedTextAreaState, refresh_text_area};
use fs_components_widgets::icon::{Glyph, Tone, icon};

use super::block_view::{Blocks, MOST_ROWS, Report, Slot};

/// One block: a row with a grip and the area. The area is as wide as its place (measured by the layout) and as tall as
/// its text takes at that width; it is built again when the width or the number of lines changes.
pub(super) fn block(
    blocks: &Arc<Blocks>,
    i: usize,
    slot: &Slot,
    hint: &str,
    report: &Report,
    assemble: &Arc<dyn Fn() -> String + Send + Sync>,
) -> Div {
    let menu = blocks.menu_view(&slot.state);
    let (state, grow) = (slot.state.clone(), slot.grow);
    let built = Arc::new(AtomicUsize::new(0));
    let width = Arc::new(AtomicU32::new(0));
    let (blocks, hint, report, assemble) = (
        Arc::clone(blocks),
        hint.to_owned(),
        Arc::clone(report),
        Arc::clone(assemble),
    );
    let (built_in, width_in, width_out) = (Arc::clone(&built), Arc::clone(&width), width);
    let place = stateful::<NoState>()
        .deps([grow.id()])
        .on_state(move |_| {
            let w = f32::from_bits(width_in.load(Ordering::Relaxed));
            let lines = lines_at(&state, w);
            built_in.store(lines, Ordering::Relaxed);
            let (blocks, st, report, assemble, built) = (
                Arc::clone(&blocks),
                state.clone(),
                Arc::clone(&report),
                Arc::clone(&assemble),
                Arc::clone(&built_in),
            );
            let area = textarea(&state)
                .rows(lines)
                .placeholder(&hint)
                .on_change(move |_| {
                    blocks.on_changed(&st);
                    report(&assemble());
                    let lines = st
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .visual_line_count()
                        .clamp(1, MOST_ROWS);
                    // More or fewer lines than the area was built for: build it again, with the room it needs.
                    if built.load(Ordering::Relaxed) != lines {
                        grow.update(|v| v.wrapping_add(1));
                    }
                });
            // Until the layout has said how wide the place is, the area fills it; then it is that wide, so the
            // lines are cut at the real edge.
            let area = if w > 0.0 { area.w(w) } else { area.w_full() };
            div().class("fs-body-field").flex_1().min_w(0.0).child(area)
        })
        .on_layout(move |b| {
            let old = f32::from_bits(width_out.swap(b.width.to_bits(), Ordering::Relaxed));
            if (old - b.width).abs() > 0.5 {
                grow.update(|v| v.wrapping_add(1));
            }
        });
    div()
        .flex_col()
        .w_full()
        .child(
            div()
                .class("fs-wk-block")
                .class("fs-wk-block-editing")
                .id(format!("fs-wk-blk-{i}"))
                .flex_row()
                .items_start()
                .child(
                    div()
                        .class("fs-wk-grip")
                        .child(icon(Glyph::More, 16.0, Tone::Muted)),
                )
                .child(place),
        )
        .child(menu)
}

/// The lines the text of `state` takes in an area `width` px wide (the number the state counts now while the width is
/// not known). The area is made once to take that width and lay its lines out, so the answer does not wait for a
/// layout pass.
fn lines_at(state: &SharedTextAreaState, width: f32) -> usize {
    if width > 0.0 {
        drop(textarea(state).w(width).rows(1).build_component());
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
