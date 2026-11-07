//! Telling the person when characters cannot be drawn.
//!
//! Pattern: State, part of `Reader` (nothing subscribes) — the text view looks fonts up per character; here the
//! characters of the document shown, and of the titles in the list, are looked up (cached) and
//! two things are announced, because a box on the screen says nothing about why: characters no
//! installed font has, and characters in titles of the list (the Blinc tree cannot set a run in
//! another font, so they are boxes even where a font exists).

use std::collections::BTreeSet;

use super::model::Reader;
use super::state::Shown;
use crate::labels::Labels;

/// How many characters the notice lists before it says "and N more".
const LISTED: usize = 8;

impl Reader {
    /// A notice about characters that cannot be drawn in the document shown or in the list, if
    /// there are any. Shown as long as they are there (not consumed by other notices).
    pub fn glyph_notice(&self) -> Option<&str> {
        self.glyph_notice.as_deref()
    }

    /// Look at the characters and (re)build the notice.
    pub(super) fn tell_missing_glyphs(&mut self) {
        let mut parts = Vec::new();
        if let Shown::Document { document, .. } = &self.shown {
            let chars: BTreeSet<char> = document
                .blocks
                .iter()
                .flat_map(|b| b.plain_text().chars().collect::<Vec<_>>())
                .collect();
            for c in &chars {
                let _ = fs_components_widgets::fallback_family(*c);
            }
            let missing = fs_components_widgets::missing_glyphs();
            let here: Vec<char> = chars.into_iter().filter(|c| missing.contains(c)).collect();
            if !here.is_empty() {
                parts.push(Labels::fill(
                    &self.labels.glyphs_missing,
                    &self.listed(&here),
                ));
            }
        }
        let titles: BTreeSet<char> = self
            .entries()
            .iter()
            .flat_map(|e| e.title.chars().collect::<Vec<_>>())
            .collect();
        // Characters that need another font, or that no font has: the list cannot draw either.
        let in_titles: Vec<char> = titles
            .into_iter()
            .filter(|c| {
                fs_components_widgets::fallback_family(*c).is_some()
                    || fs_components_widgets::missing_glyphs().contains(c)
            })
            .collect();
        if !in_titles.is_empty() {
            parts.push(Labels::fill(
                &self.labels.glyphs_titles,
                &self.listed(&in_titles),
            ));
        }
        self.glyph_notice = (!parts.is_empty()).then(|| parts.join(" "));
    }

    /// `U+XXXX` for the first few, then "and N more".
    fn listed(&self, chars: &[char]) -> String {
        let mut list = chars
            .iter()
            .take(LISTED)
            .map(|c| format!("U+{:04X}", u32::from(*c)))
            .collect::<Vec<_>>()
            .join(" ");
        if chars.len() > LISTED {
            list.push(' ');
            list.push_str(&Labels::fill(
                &self.labels.glyphs_more,
                &(chars.len() - LISTED).to_string(),
            ));
        }
        list
    }
}
