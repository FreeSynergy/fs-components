//! `Source` and the text file format: one language, many messages.
//!
//! Pattern: Value object + a small parser. `MessageFormat` 2 defines the message, not a file of messages, so the container is
//! the plainest one (the same idea as `Catalog::from_text` of `fs-components-words`): `id = message`, a message goes on
//! over the following lines when they are indented, lines starting with `#` and blank lines between messages are skipped.
//! An id is letters, digits, `-`, `_`, `.`. The message is MF2 as it stands (`Hello, {$name}!`, or
//! `.input {$n :number} .match $n one {{{$n} file}} * {{{$n} files}}`).

/// The text of one language file with its language (BCP 47, for example `de`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source {
    /// The language of the file.
    pub language: &'static str,
    /// The content: `id = message` entries.
    pub text: &'static str,
}

/// An entry: id, message (MF2) and the 1-based line it starts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub id: String,
    pub message: String,
    pub line: usize,
}

fn id_ok(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
}

/// The entries of a text and the problems on the way (a line that is no entry, an id given twice, an empty message).
pub(crate) fn entries(text: &str) -> (Vec<Entry>, Vec<String>) {
    let (mut out, mut problems): (Vec<Entry>, Vec<String>) = (Vec::new(), Vec::new());
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for (i, raw) in text.lines().enumerate() {
        let n = i + 1;
        let continued = raw.starts_with([' ', '\t']) && !raw.trim().is_empty();
        let line = raw.trim_end();
        if continued {
            match out.last_mut() {
                Some(last) => {
                    last.message.push('\n');
                    last.message.push_str(line.trim_start());
                }
                None => problems.push(format!("line {n}: indented line before any entry")),
            }
        } else if line.is_empty() || line.starts_with('#') {
            // a blank line or a comment: nothing to take
        } else if let Some((id, rest)) = line.split_once('=') {
            let id = id.trim();
            if !id_ok(id) {
                problems.push(format!("line {n}: `{id}` is no identifier"));
            } else if out.iter().any(|e| e.id == id) {
                problems.push(format!("line {n}: {id} is given twice"));
            } else {
                out.push(Entry {
                    id: id.to_owned(),
                    message: rest.trim().to_owned(),
                    line: n,
                });
            }
        } else {
            problems.push(format!("line {n}: no `=`"));
        }
    }
    for e in &out {
        if e.message.trim().is_empty() {
            problems.push(format!("line {}: {} has no message", e.line, e.id));
        }
    }
    (out, problems)
}
