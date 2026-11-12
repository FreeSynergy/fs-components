//! `badge`: a dot for "new or changed", or a number for "how many", on top of something else.
//!
//! Pattern: Decorator — it is laid over the corner of another element (the caller makes that element `relative`).
//! The look is Blinc's `cn::badge`; the number stops at 99 and reads `99+`. A badge is a picture to the eye, so the
//! words for a screen reader are part of it: the caller gives what the badge says ("3 transfers running"), and it
//! is put on the element as its label (`aria-label`) and in its title.

use blinc_cn::components::badge::{BadgeVariant, badge};
use blinc_layout::div::{Div, div};

/// The style sheet of the badge.
pub const STYLES: &str = ".fs-badge-dot { background: var(--error); border-width: 2px; border-color: var(--surface); }\n";

/// The most a badge counts exactly; above it the text is `99+`.
pub const MOST: u32 = 99;

/// The text of a badge that counts: the number, or `99+`.
pub fn count_text(count: u32) -> String {
    if count > MOST {
        format!("{MOST}+")
    } else {
        count.to_string()
    }
}

fn corner(inner: Div, spoken: &str) -> Div {
    div()
        .class("fs-badge")
        .absolute()
        .right(-4.0)
        .top(-4.0)
        .attr("aria-label", spoken)
        .attr("title", spoken)
        .pointer_events_none()
        .child(inner)
}

/// A dot: something is new or changed. `spoken` says what, for a screen reader.
pub fn badge_dot(spoken: &str) -> Div {
    corner(
        div().w(10.0).h(10.0).rounded(5.0).class("fs-badge-dot"),
        spoken,
    )
}

/// A number: how many. `None` for zero (nothing to show). `spoken` says what is counted, for a screen reader.
pub fn badge_count(count: u32, spoken: &str) -> Option<Div> {
    (count > 0).then(|| {
        corner(
            div().child(badge(count_text(count)).variant(BadgeVariant::Destructive)),
            spoken,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::count_text;

    #[test]
    fn a_number_stops_at_ninety_nine() {
        assert_eq!(count_text(1), "1");
        assert_eq!(count_text(99), "99");
        assert_eq!(count_text(100), "99+");
        assert_eq!(count_text(u32::MAX), "99+");
    }
}
