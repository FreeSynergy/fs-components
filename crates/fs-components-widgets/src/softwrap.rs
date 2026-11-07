//! Long unbroken text, made breakable.
//!
//! Pattern: none. Blinc breaks lines at spaces; an unbroken run of thousands of characters
//! is slow to measure and runs out of the box. A zero-width space every few characters lets it wrap
//! and keeps the text readable (it is only for drawing; the data is never changed).

/// Longest run without a break point.
const RUN: usize = 24;

/// `text` with a zero-width space after every [`RUN`] characters that have no space or break.
pub(crate) fn soft_wrap(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / RUN);
    let mut run = 0;
    for c in text.chars() {
        if c.is_whitespace() || c == '\u{200b}' {
            run = 0;
        } else {
            run += 1;
            if run > RUN {
                out.push('\u{200b}');
                run = 1;
            }
        }
        out.push(c);
    }
    out
}
