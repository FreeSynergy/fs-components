//! The other things the side area can show: transfers, the filter, the report of a rescan.
//!
//! Pattern: Adapter — `transfer-list` over the rows of the state; the buttons report to the state. Long
//! lists of names stand in a scroller of their own and are bounded: a status line could not hold them.

use std::sync::{Arc, Mutex};

use blinc_cn::components::button::ButtonVariant;
use blinc_cn::components::input::{InputSize, input};
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::{Scroll, ScrollPhysics};
use fs_components_widgets::list_view::fit_cell;
use fs_components_widgets::transfer_list::{Labels as TransferLabels, TransferList};

use super::actions::{FileActions, button};
use crate::file_manager::labels::FileLabels;
use crate::file_manager::state::FilesState;
use crate::input_slots::InputSlots;

/// Names shown per group of a report.
const REPORT_NAMES: usize = 100;

fn fresh_scroller() -> Scroll {
    Scroll::with_physics(Arc::new(Mutex::new(ScrollPhysics::default())))
}

fn title(words: &str) -> Div {
    div().class("fs-files-section-title").child(text(words))
}

pub(super) fn transfers_panel(
    state: &FilesState,
    actions: &FileActions,
    width: f32,
    height: f32,
) -> Div {
    let labels = state.labels();
    let shown = state.transfers().iter().map(|r| r.shown.clone()).collect();
    let apply = Arc::clone(&actions.apply);
    let list = TransferList::new(shown)
        .labels(TransferLabels::default())
        .on_outcome(Arc::new(move |outcome| {
            let outcome = outcome.clone();
            apply(&move |s: &mut FilesState| s.transfer_outcome(&outcome));
        }));
    div()
        .class("fs-files-panel")
        .flex_col()
        .w(width)
        .h(height)
        .child(title(&labels.transfers_title))
        .child(
            div()
                .class("fs-files-note")
                .w(width - 24.0)
                .child(text(&labels.transfers_note)),
        )
        .child(
            div().flex_1().min_h(0.0).w(width - 24.0).child(
                fresh_scroller()
                    .w(width - 24.0)
                    .h_full()
                    .content(div().w(width - 24.0).child(list.build())),
            ),
        )
}

pub(super) fn filter_panel(
    state: &FilesState,
    name: &str,
    actions: &FileActions,
    slots: &InputSlots,
    width: f32,
) -> Div {
    let labels = state.labels();
    let draft = state.filter_draft();
    let mut panel = div()
        .class("fs-files-panel")
        .flex_col()
        .w(width)
        .child(title(&labels.filter_title));
    for (key, word, value) in [
        ("min", &labels.filter_min, &draft.min_size),
        ("max", &labels.filter_max, &draft.max_size),
        ("from", &labels.filter_from, &draft.modified_from),
        ("before", &labels.filter_before, &draft.modified_before),
    ] {
        let data = slots.line(0, key, value);
        let (typed, key) = (Arc::clone(&actions.typed_filter), key.to_string());
        panel = panel.child(
            div()
                .id(format!("{name}-filter-{key}"))
                .class("fs-files-input")
                .w(width - 24.0)
                .child(
                    input(&data)
                        .size(InputSize::Small)
                        .label(word.clone())
                        .w_full()
                        .on_change(move |value| typed(&key, value)),
                ),
        );
    }
    panel.child(
        div()
            .class("fs-files-band-row")
            .flex_row()
            .flex_wrap()
            .child(button(
                format!("{name}-filter-clear"),
                &labels.clear,
                ButtonVariant::Secondary,
                &actions.plain(FilesState::clear_filter),
            ))
            .child(button(
                format!("{name}-filter-apply"),
                &labels.apply,
                ButtonVariant::Primary,
                &actions.plain(FilesState::apply_filter),
            )),
    )
}

fn group(words: &str, names: &[String], more: &str, width: f32, mut page: Div) -> Div {
    if names.is_empty() {
        return page;
    }
    page = page.child(title(&format!("{words} ({})", names.len())));
    for each in names.iter().take(REPORT_NAMES) {
        page = page.child(text(fit_cell(each, width)).no_wrap());
    }
    if names.len() > REPORT_NAMES {
        let left = (names.len() - REPORT_NAMES).to_string();
        page = page.child(text(FileLabels::fill(more, &left)));
    }
    page
}

pub(super) fn report_panel(state: &FilesState, name: &str, width: f32, height: f32) -> Div {
    let labels = state.labels();
    let mut page = div().flex_col().w(width - 24.0);
    if let Some(report) = state.report() {
        let room = width - 24.0;
        page = group(
            &labels.rescan_added,
            &report.added,
            &labels.rescan_more,
            room,
            page,
        );
        page = group(
            &labels.rescan_updated,
            &report.updated,
            &labels.rescan_more,
            room,
            page,
        );
        page = group(
            &labels.rescan_missing,
            &report.missing,
            &labels.rescan_more,
            room,
            page,
        );
        page = group(
            &labels.rescan_skipped,
            &report.skipped,
            &labels.rescan_more,
            room,
            page,
        );
    }
    div()
        .id(format!("{name}-report"))
        .class("fs-files-panel")
        .flex_col()
        .w(width)
        .h(height)
        .child(title(&labels.report_title))
        .child(
            div()
                .class("fs-files-note")
                .w(width - 24.0)
                .child(text(&labels.rescan_blocks)),
        )
        .child(
            div()
                .flex_1()
                .min_h(0.0)
                .w(width - 24.0)
                .child(fresh_scroller().w(width - 24.0).h_full().content(page)),
        )
}
