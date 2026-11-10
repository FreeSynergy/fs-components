//! Changing the start of whole lines: headings, list points, quotes.
//!
//! Pattern: none — pure functions from text and selection to an edit. The lines touched are the
//! ones the selection reaches (a selection that ends right after a line break does not reach the
//! next line).

use super::edit::Edit;
use super::selection::Selection;

/// What one line becomes: the length of its old prefix and the new prefix.
pub(super) type Rewrite = (usize, String);

/// The lines of the block the selection touches: where it starts and its lines.
fn block(chars: &[char], sel: Selection) -> (usize, Vec<String>) {
    let start = chars[..sel.start]
        .iter()
        .rposition(|c| *c == '\n')
        .map_or(0, |i| i + 1);
    let last = if !sel.is_empty() && chars[sel.end - 1] == '\n' {
        sel.end - 1
    } else {
        sel.end
    };
    let end = chars[last..]
        .iter()
        .position(|c| *c == '\n')
        .map_or(chars.len(), |i| last + i);
    let text: String = chars[start..end.max(start)].iter().collect();
    (start, text.split('\n').map(str::to_owned).collect())
}

/// The new position of `at` once line `i` has changed by its rewrite.
fn moved(at: usize, start: usize, lines: &[String], plan: &[Rewrite]) -> usize {
    let (mut old, mut new) = (start, start);
    for (line, (cut, put)) in lines.iter().zip(plan) {
        let size = line.chars().count();
        if at <= old + size {
            let inside = at - old;
            let put_len = put.chars().count();
            let shifted = if inside >= *cut {
                inside - cut + put_len
            } else {
                inside.min(put_len)
            };
            return new + shifted;
        }
        old += size + 1;
        new += size - cut + put.chars().count() + 1;
    }
    new
}

/// Rewrite the prefix of every line the selection touches; `decide` sees the lines and answers one
/// [`Rewrite`] per line. `None` if nothing changes.
pub(super) fn rewrite(
    chars: &[char],
    sel: Selection,
    decide: impl Fn(&[String]) -> Vec<Rewrite>,
) -> Option<Edit> {
    let sel = sel.clamp(chars.len());
    let (start, lines) = block(chars, sel);
    let plan = decide(&lines);
    let new: Vec<String> = lines
        .iter()
        .zip(&plan)
        .map(|(line, (cut, put))| format!("{put}{}", line.chars().skip(*cut).collect::<String>()))
        .collect();
    if new == lines {
        return None;
    }
    Some(Edit {
        at: start,
        removed: lines.join("\n"),
        inserted: new.join("\n"),
        before: sel,
        after: Selection::new(
            moved(sel.start, start, &lines, &plan),
            moved(sel.end, start, &lines, &plan),
        ),
    })
}

/// The lines that count for a toggle: the non-empty ones (all of them if none is).
fn counted(lines: &[String]) -> impl Fn(usize) -> bool + use<> {
    let any = lines.iter().any(|l| !l.is_empty());
    let empty: Vec<bool> = lines.iter().map(String::is_empty).collect();
    move |i| !any || !empty[i]
}

/// Heading of `level` (0 = none): on every counted line; the same level everywhere takes it off.
pub(super) fn heading_plan(level: u8) -> impl Fn(&[String]) -> Vec<Rewrite> {
    let level = usize::from(level.min(6));
    move |lines| {
        let now: Vec<usize> = lines.iter().map(|l| heading_of(l)).collect();
        let counts = counted(lines);
        let off = level == 0
            || (0..lines.len())
                .filter(|i| counts(*i))
                .all(|i| now[i] == level);
        (0..lines.len())
            .map(|i| {
                let cut = if now[i] == 0 { 0 } else { now[i] + 1 };
                if !counts(i) {
                    (0, String::new())
                } else if off {
                    (cut, String::new())
                } else {
                    (cut, format!("{} ", "#".repeat(level)))
                }
            })
            .collect()
    }
}

/// The heading level of a line (0 = not a heading).
fn heading_of(line: &str) -> usize {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&hashes) && line.chars().nth(hashes) == Some(' ') {
        hashes
    } else {
        0
    }
}

/// A prefix such as `- ` or `> ` on every counted line, or off again when all have it. `known` are
/// other spellings of the same prefix that count as present.
pub(super) fn prefix_plan(
    prefix: &'static str,
    known: &'static [&'static str],
) -> impl Fn(&[String]) -> Vec<Rewrite> {
    move |lines| {
        let has = |l: &str| -> Option<usize> {
            std::iter::once(&prefix)
                .chain(known)
                .find(|p| l.starts_with(**p))
                .map(|p| p.chars().count())
        };
        let counts = counted(lines);
        let off = (0..lines.len())
            .filter(|i| counts(*i))
            .all(|i| has(&lines[i]).is_some());
        (0..lines.len())
            .map(|i| match (counts(i), has(&lines[i])) {
                (false, _) => (0, String::new()),
                (true, Some(n)) if off => (n, String::new()),
                (true, Some(n)) => (n, prefix.to_owned()),
                (true, None) => (0, prefix.to_owned()),
            })
            .collect()
    }
}
