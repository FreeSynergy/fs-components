//! What may be shown of bytes nobody has checked.
//!
//! Pattern: Specification — pure functions over bytes. A text is shown only if it is text: no NUL, and
//! valid UTF-8 (a character cut by the end of a part read does not count against it). Control and
//! direction characters are replaced, so a name or a line cannot dress up as something else. Lines and
//! their length are bounded, because Blinc measures long text slowly. A picture is handed to Blinc only
//! if its header says it is small enough to decode: Blinc decodes at full size.

/// Lines of a text shown at most.
pub const MAX_LINES: usize = 200;
/// Characters of a line shown at most.
pub const MAX_LINE_CHARS: usize = 200;

/// Text that is safe to draw.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeText {
    /// The lines.
    pub lines: Vec<String>,
    /// Something was left out: lines, the end of a line, or the rest of the file.
    pub cut: bool,
}

fn drawable(c: char) -> char {
    let direction = matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}');
    if (c.is_control() && c != '\t') || direction {
        '\u{FFFD}'
    } else if c == '\t' {
        ' '
    } else {
        c
    }
}

/// The text in `bytes`, `more` saying that the file goes on after them; `None` if it is not text.
pub fn safe_text(bytes: &[u8], more: bool) -> Option<SafeText> {
    if bytes.contains(&0) {
        return None;
    }
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(e) if more && e.error_len().is_none() => {
            std::str::from_utf8(&bytes[..e.valid_up_to()]).ok()?
        }
        Err(_) => return None,
    };
    let mut cut = more;
    let mut lines = Vec::new();
    for line in text.split('\n') {
        if lines.len() == MAX_LINES {
            cut = true;
            break;
        }
        let chars = line.trim_end_matches('\r').chars();
        let mut shown: String = chars.clone().take(MAX_LINE_CHARS).map(drawable).collect();
        if chars.count() > MAX_LINE_CHARS {
            cut = true;
            shown.push('\u{2026}');
        }
        lines.push(shown);
    }
    Some(SafeText { lines, cut })
}

/// Do these bytes start like a PNG or a JPEG of at most `max_side` pixels each way? Anything else,
/// also a header that cannot be read, is no.
pub fn picture_fits(bytes: &[u8], max_side: u32) -> bool {
    size_of_png(bytes)
        .or_else(|| size_of_jpeg(bytes))
        .is_some_and(|(w, h)| w > 0 && h > 0 && w <= max_side && h <= max_side)
}

fn size_of_png(bytes: &[u8]) -> Option<(u32, u32)> {
    let signature = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[..8] != signature || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let number =
        |at: usize| u32::from_be_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    Some((number(16), number(20)))
}

fn size_of_jpeg(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return None;
    }
    let mut at = 2;
    while at + 9 < bytes.len() {
        if bytes[at] != 0xFF {
            return None;
        }
        let marker = bytes[at + 1];
        match marker {
            // fill: the next 0xFF may be the marker
            0xFF => {
                at += 1;
                continue;
            }
            // TEM, RSTn and SOI stand alone, with no length after them
            0x01 | 0xD0..=0xD8 => {
                at += 2;
                continue;
            }
            // EOI: the picture ends without a frame
            0xD9 => return None,
            _ => {}
        }
        let frame = matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC);
        if frame {
            let number = |i: usize| u32::from(bytes[i]) << 8 | u32::from(bytes[i + 1]);
            return Some((number(at + 7), number(at + 5)));
        }
        let length = usize::from(bytes[at + 2]) << 8 | usize::from(bytes[at + 3]);
        at += 2 + length.max(2);
    }
    None
}

/// `bytes` as an address Blinc's image element takes (`data:<type>;base64,...`).
pub fn picture_address(media_type: &str, bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = format!("data:{media_type};base64,");
    out.reserve(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let joined = chunk
            .iter()
            .fold(0u32, |all, byte| (all << 8) | u32::from(*byte))
            << (8 * (3 - chunk.len()));
        for place in 0..4 {
            if place <= chunk.len() {
                out.push(char::from(
                    ALPHABET[(joined >> (18 - 6 * place)) as usize & 63],
                ));
            } else {
                out.push('=');
            }
        }
    }
    out
}
