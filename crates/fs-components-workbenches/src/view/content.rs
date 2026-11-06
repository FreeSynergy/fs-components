//! The content area: the bar (back, path), a notice, and the document.
//!
//! Pattern: Strategy by state ([`Shown`]) — page, failure or nothing.

use blinc_cn::cn;
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::scroll::scroll;
use fs_components_widgets::{LinkHandler, TextView};

use super::actions::Actions;
use crate::reader::{Reader, Shown};

/// The area right of the sidebar.
pub(crate) fn content(reader: &Reader, actions: &Actions) -> Div {
    let back = actions.back.clone();
    let button = cn::button("Back")
        .disabled(!reader.can_go_back())
        .on_click(move |_| back());
    let path = reader.current_path().unwrap_or("");
    let mut bar = div().class("fs-rd-bar").child(button);
    bar = bar.child(text(path).class("fs-rd-path"));
    let mut column = div().flex_col().w_full().flex_1().child(bar);
    if let Some(notice) = reader.notice() {
        column = column.child(div().class("fs-rd-notice").child(text(notice)));
    }
    let page = match reader.shown() {
        Shown::Nothing => div()
            .class("fs-rd-page")
            .w_full()
            .child(text("Nothing to show.")),
        Shown::Failed { reason, .. } => div().class("fs-rd-page").w_full().child(
            div()
                .class("fs-rd-failed")
                .child(text(format!("This page could not be read: {reason}"))),
        ),
        Shown::Page { document, .. } => {
            let follow = actions.follow.clone();
            let handler: LinkHandler = std::sync::Arc::new(move |href| follow(href));
            div().class("fs-rd-page").w_full().child(
                TextView::new(document.blocks.clone())
                    .on_link(handler)
                    .build(),
            )
        }
    };
    column.child(
        div()
            .flex_1()
            .w_full()
            .child(scroll().w_full().h_full().content(page)),
    )
}
