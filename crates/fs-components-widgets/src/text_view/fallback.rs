//! Fonts for characters the default font does not have.
//!
//! Pattern: Facade over `fontdb` and `ttf-parser` — Blinc draws a character from one default font, and from one
//! emoji font that has only a bundled subset; everything else is an empty box. For the scripts and
//! symbol ranges where that happens, this module looks through the fonts installed on the system
//! (`fontdb`, `ttf-parser`) and names the family that has the character, so that the text view
//! can set that run in it. Blinc cannot draw colour-vector emoji fonts (the system's "Noto Color
//! Emoji" is one), so those are skipped; the monochrome "Noto Emoji" and the symbol fonts are used.
//! A character no font has stays a box, and is counted (see [`missing`]).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// After this many characters of a block found in no font (and none found), the rest of the
/// block is not searched.
const BLOCK_GIVE_UP: u32 = 8;

/// One system font that can be asked for characters.
struct Candidate {
    id: fontdb::ID,
    family: Arc<str>,
}

struct Index {
    db: fontdb::Database,
    ranked: Vec<Candidate>,
    cache: Mutex<HashMap<char, Option<Arc<str>>>>,
    absent: Mutex<HashSet<char>>,
    /// Per block of 256 code points: (found, not found) so far, to stop searching a block that no
    /// installed font covers (thousands of CJK characters would each scan every font).
    blocks: Mutex<HashMap<u32, (u32, u32)>>,
}

fn index() -> &'static Index {
    static INDEX: OnceLock<Index> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        let mut ranked: Vec<(u8, Candidate)> = db
            .faces()
            .filter_map(|f| {
                let family: Arc<str> = f.families.first()?.0.as_str().into();
                let rank = rank(&family)?;
                Some((rank, Candidate { id: f.id, family }))
            })
            .collect();
        ranked.sort_by_key(|(r, _)| *r);
        Index {
            db,
            ranked: ranked.into_iter().map(|(_, c)| c).collect(),
            cache: Mutex::new(HashMap::new()),
            absent: Mutex::new(HashSet::new()),
            blocks: Mutex::new(HashMap::new()),
        }
    })
}

/// How much we like a family as a fallback (lower is better); `None` = never use.
fn rank(family: &str) -> Option<u8> {
    if family.contains("Color Emoji") || family.contains("Mono") || family.contains("Italic") {
        return None; // colour-vector emoji cannot be drawn; monospace/italic are not fallbacks
    }
    Some(if family == "Noto Emoji" {
        0
    } else if family.starts_with("Noto Sans") {
        1
    } else if family.starts_with("DejaVu Sans") {
        2
    } else if family.starts_with("Noto") {
        3
    } else {
        4
    })
}

/// Does `ch` lie in a range where the default fonts often have no glyph?
fn in_doubtful_range(ch: char) -> bool {
    matches!(u32::from(ch),
        0x0900..=0x1DFF
            | 0x2300..=0x23FF
            | 0x2600..=0x27BF
            | 0x2900..=0x2BFF
            | 0x2E80..=0x9FFF // CJK
            | 0xAC00..=0xD7AF // Hangul
            | 0xFF00..=0xFFEF
            | 0x1F000..=0x1FAFF)
}

/// Is `ch` in the colour subset that Blinc draws as emoji?
fn in_bundled_emoji(ch: char) -> bool {
    static FACE: OnceLock<Option<ttf_parser::Face<'static>>> = OnceLock::new();
    FACE.get_or_init(|| ttf_parser::Face::parse(blinc_noto_emoji::NOTO_COLOR_EMOJI_SUBSET, 0).ok())
        .as_ref()
        .is_some_and(|f| f.glyph_index(ch).is_some())
}

/// The family to set `ch` in, or `None` when the default way of drawing it is the best there is.
pub(super) fn font_for(ch: char) -> Option<Arc<str>> {
    if !in_doubtful_range(ch) || in_bundled_emoji(ch) {
        return None;
    }
    let idx = index();
    if let Some(hit) = idx
        .cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&ch)
    {
        return hit.clone();
    }
    let block = u32::from(ch) >> 8;
    let hopeless = idx
        .blocks
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&block)
        .is_some_and(|&(found, missed)| found == 0 && missed >= BLOCK_GIVE_UP);
    let found = if hopeless {
        None
    } else {
        idx.ranked.iter().find_map(|c| {
            idx.db
                .with_face_data(c.id, |data, index| {
                    ttf_parser::Face::parse(data, index).is_ok_and(|f| f.glyph_index(ch).is_some())
                })
                .unwrap_or(false)
                .then(|| c.family.clone())
        })
    };
    {
        let mut blocks = idx.blocks.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = blocks.entry(block).or_insert((0, 0));
        if found.is_some() {
            entry.0 += 1;
        } else {
            entry.1 += 1;
        }
    }
    if found.is_none() {
        idx.absent
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(ch);
    }
    idx.cache
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(ch, found.clone());
    found
}

/// Does any character of `text` need another font?
pub(super) fn needs_fallback(text: &str) -> bool {
    text.chars().any(|c| font_for(c).is_some())
}

/// Characters looked for so far that no installed font has (they are drawn as boxes).
pub fn missing() -> Vec<char> {
    let mut v: Vec<char> = index()
        .absent
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .copied()
        .collect();
    v.sort_unstable();
    v
}

/// The family a character is set in when the default fonts lack it; `None` if the default way is
/// used (or no installed font has it: then it is in [`missing`] afterwards).
pub fn fallback_family(ch: char) -> Option<String> {
    font_for(ch).map(|f| f.to_string())
}
