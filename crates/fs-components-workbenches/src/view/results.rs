//! The list of search results.
//!
//! Pattern: Adapter — one clickable row per hit.

use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::Actions;
use crate::reader::Reader;

/// Rows of the hits of the current search.
pub(crate) fn results(reader: &Reader, actions: &Actions) -> Div {
    let mut column = div().flex_col().w_full();
    if let Some(problem) = reader.problem() {
        return column.child(div().class("fs-rd-empty").child(text(problem)));
    }
    if reader.hits().is_empty() {
        return column.child(
            div()
                .class("fs-rd-empty")
                .child(text(format!("Nothing found for \"{}\".", reader.term()))),
        );
    }
    for hit in reader.hits() {
        let open = actions.open.clone();
        let path = hit.path.clone();
        column = column.child(
            div()
                .class("fs-rd-hit")
                .on_click(move |_| open(&path))
                .child(text(&hit.title))
                .child(text(&hit.path).class("fs-rd-hit-path")),
        );
    }
    column
}
