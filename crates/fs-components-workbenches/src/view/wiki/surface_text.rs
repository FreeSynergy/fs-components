//! The text of the writing surface: where the blocks are in it, and what Enter and Backspace make of it.
//!
//! Pattern: Strategy by the kind of block (`block_enter`) over pure functions of the text; the surface applies the result.
//!
//! The surface is one text. Its blocks are the runs of lines between blank lines (a fenced code block is one block whatever it
//! holds), so what the person sees is what the model stores; Enter in a paragraph makes the next block by putting a blank line
//! in (the gap between blocks), Enter in a list, a quote, code or a table goes on in the same block.

use super::block_enter::{Enter, enter};

/// The byte ranges of the blocks (not blank, the blank lines around them left out); the same cut as `Doc::parse`.
pub(super) fn spans(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let (mut start, mut last_end, mut pos) = (None, 0, 0);
    let mut fence: Option<&str> = None;
    for line in text.split('\n') {
        let line_end = pos + line.len();
        let tick = ["```", "~~~"]
            .into_iter()
            .find(|f| line.trim_start().starts_with(f));
        match (fence, tick) {
            (None, Some(f)) => fence = Some(f),
            (Some(f), Some(t)) if f == t => fence = None,
            _ => {}
        }
        if fence.is_none() && tick.is_none() && line.trim().is_empty() {
            if let Some(s) = start.take() {
                out.push((s, last_end));
            }
        } else {
            start.get_or_insert(pos);
            last_end = line_end;
        }
        pos = line_end + 1;
    }
    if let Some(s) = start {
        out.push((s, last_end));
    }
    out
}

/// One block of the surface as the person sees it: a block with text, or an empty one (a gap wide enough to hold a block that
/// was made by Enter and not written yet).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Entry {
    pub start: usize,
    pub end: usize,
    pub empty: bool,
}

/// The blocks in order, the empty ones between: two line breaks between blocks are the usual gap, every two more hold one
/// empty block (Enter in a paragraph puts two in); at the end any gap of two or more holds one, and an empty text one.
pub(super) fn entries(text: &str) -> Vec<Entry> {
    let found = spans(text);
    let mut out = Vec::new();
    let mut prev_end = None;
    for &(start, end) in &found {
        let gap = start - prev_end.unwrap_or(0);
        let empties = match prev_end {
            Some(_) => gap.saturating_sub(2) / 2,
            None => usize::from(gap >= 2),
        };
        for _ in 0..empties {
            let at = prev_end.map_or(0, |e| e + 2);
            out.push(Entry {
                start: at,
                end: at,
                empty: true,
            });
        }
        out.push(Entry {
            start,
            end,
            empty: false,
        });
        prev_end = Some(end);
    }
    match prev_end {
        None => out.push(Entry {
            start: 0,
            end: 0,
            empty: true,
        }),
        Some(end) if text.len() - end >= 2 => {
            out.push(Entry {
                start: end + 2,
                end: end + 2,
                empty: true,
            });
        }
        Some(_) => {}
    }
    out
}

/// The texts of the blocks, the empty ones as empty texts.
pub(super) fn block_texts(text: &str) -> Vec<String> {
    entries(text)
        .iter()
        .map(|e| text[e.start..e.end].to_owned())
        .collect()
}

/// Which block of [`block_texts`] has the byte `offset`: the one it lies in; in a gap the empty block there if the gap holds
/// one, else the block after.
pub(super) fn block_at(text: &str, offset: usize) -> usize {
    let all = entries(text);
    let inside = all
        .iter()
        .position(|e| !e.empty && e.start <= offset && offset <= e.end);
    inside.unwrap_or_else(|| {
        let before = all.iter().filter(|e| !e.empty && e.end < offset).count();
        let empties_before = all.iter().filter(|e| e.empty && e.start <= offset).count();
        (before + empties_before.saturating_sub(1)).min(all.len().saturating_sub(1))
    })
}

/// The byte range of the line that holds `offset` (without its line break).
pub(super) fn line_at(text: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(text.len());
    let start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let end = text[offset..].find('\n').map_or(text.len(), |i| offset + i);
    (start, end)
}

/// The text and the cursor after Enter at byte `at` of `before`. In a gap between blocks, in code and in a table the line break
/// stays a line break; in a paragraph the block splits there with a blank line between the two parts; in a list, a quote the
/// block goes on (a new item, a new `> `), and an empty item ends it.
pub(super) fn after_enter(before: &str, at: usize) -> (String, usize) {
    let plain = || (format!("{}\n{}", &before[..at], &before[at..]), at + 1);
    let Some(&(start, end)) = spans(before).iter().find(|&&(s, e)| s <= at && at <= e) else {
        return plain();
    };
    let (local, here) = (&before[start..end], at - start);
    let now = format!("{}\n{}", &local[..here], &local[here..]);
    let (text, cursor) = match enter(&now, here + 1) {
        Enter::Keep => return plain(),
        Enter::Carry { text, at } => (text, at),
        Enter::Split { head, tail } => {
            let cursor = head.len() + 2;
            (format!("{head}\n\n{tail}"), cursor)
        }
    };
    (
        format!("{}{}{}", &before[..start], text, &before[end..]),
        start + cursor,
    )
}

/// After a Backspace that took away one line break where two stood (the gap between two blocks): the text and the cursor with
/// the second line break gone too, so the block joins the one above, as in Notion. None when it was something else.
pub(super) fn after_backspace(before: &str, now: &str, cursor: usize) -> Option<(String, usize)> {
    let removed = before.len() == now.len() + 1
        && before.get(cursor..=cursor) == Some("\n")
        && before[..cursor].ends_with('\n');
    let above = now[..cursor].strip_suffix('\n')?;
    if !removed || above.ends_with('\n') || above.is_empty() {
        return None;
    }
    let last = above.rsplit('\n').next().unwrap_or("");
    let marked = last.starts_with(['-', '*', '+', '>', '|', '`', '~'])
        || last.chars().next().is_some_and(|c| c.is_ascii_digit());
    (!marked).then(|| (format!("{above}{}", &now[cursor..]), above.len()))
}
