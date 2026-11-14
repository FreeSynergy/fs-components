//! What Enter does in a block: split it in two, carry a list on, or leave a line break where it is.
//!
//! Pattern: Strategy by the kind of block (pure functions over the text; the window applies the result).
//!
//! Limits: the kind is read from the lines of the block as Markdown writes them (a fence, a table row, a list marker, a
//! quote mark); a nested list keeps its indentation, a loose list is not made.

/// What the block does after the text area has put a line break at the cursor.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Enter {
    /// The block ends before the line break; what follows is a new block below it.
    Split {
        /// The text that stays.
        head: String,
        /// The text of the new block.
        tail: String,
    },
    /// The block stays one block: this is its new text, the cursor stands `at` bytes into it.
    Carry {
        /// The new text of the block.
        text: String,
        /// Byte offset of the cursor.
        at: usize,
    },
    /// The line break stays as the text area made it (code, table).
    Keep,
}

/// The list line `line` as (the start of the next line, the text after the marker, is it a task item): `- [x] a` gives
/// (`- [ ] `, `a`, true), `3. a` gives (`4. `, `a`, false). None for a line that is no list item.
fn markers(line: &str) -> Option<(String, String, bool)> {
    let indent = line.len() - line.trim_start().len();
    let (pad, rest) = line.split_at(indent);
    let (next, body) = if let Some((m, body)) = ["- ", "* ", "+ "]
        .iter()
        .find_map(|m| rest.strip_prefix(m).map(|b| (*m, b)))
    {
        (m.to_string(), body)
    } else {
        let digits = rest.chars().take_while(char::is_ascii_digit).count();
        let (number, after) = rest.split_at(digits);
        let sep = after.chars().next().filter(|c| matches!(c, '.' | ')'))?;
        let body = after[1..].strip_prefix(' ')?;
        let n: u64 = number.parse().ok().filter(|_| number.len() < 10)?;
        (format!("{}{sep} ", n + 1), body)
    };
    let task = ["[ ]", "[x]", "[X]"].iter().any(|t| body.starts_with(t));
    let body = if task { body[3..].trim_start() } else { body };
    let next = if task { format!("{next}[ ] ") } else { next };
    Some((format!("{pad}{next}"), body.to_string(), task))
}

/// Is `before` inside a fenced code block (an open fence)?
fn in_fence(before: &str) -> bool {
    let mut open: Option<&str> = None;
    for line in before.lines() {
        let t = line.trim_start();
        let tick = ["```", "~~~"].into_iter().find(|f| t.starts_with(f));
        match (open, tick) {
            (None, Some(f)) => open = Some(f),
            (Some(f), Some(t)) if f == t => open = None,
            _ => {}
        }
    }
    open.is_some()
}

/// The action for a block whose text area now holds `now`, with a line break just before byte `cursor` (the cursor
/// stands after it).
pub(super) fn enter(now: &str, cursor: usize) -> Enter {
    let (before, after) = (&now[..cursor - 1], &now[cursor..]);
    if in_fence(before) || before.trim_start().starts_with('|') {
        return Enter::Keep;
    }
    let last = before.rsplit('\n').next().unwrap_or("");
    if let Some((next, body, _)) = markers(last) {
        let empty = body.trim().is_empty();
        if empty && after.is_empty() {
            // An empty item ends the list: the item goes, the new block below is a paragraph.
            let kept = before[..before.len() - last.len()].trim_end_matches('\n');
            return Enter::Split {
                head: kept.to_string(),
                tail: String::new(),
            };
        }
        let text = format!("{before}\n{next}{after}");
        let at = before.len() + 1 + next.len();
        return Enter::Carry { text, at };
    }
    if let Some(rest) = last.strip_prefix("> ") {
        if rest.trim().is_empty() && after.is_empty() {
            let kept = before[..before.len() - last.len()].trim_end_matches('\n');
            return Enter::Split {
                head: kept.to_string(),
                tail: String::new(),
            };
        }
        return Enter::Carry {
            text: format!("{before}\n> {after}"),
            at: before.len() + 3,
        };
    }
    Enter::Split {
        head: before.to_string(),
        tail: after.to_string(),
    }
}
