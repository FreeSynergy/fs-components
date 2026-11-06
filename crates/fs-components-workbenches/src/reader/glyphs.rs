//! Telling the person once when characters of a document cannot be drawn at all.
//!
//! Pattern: Observer of the fallback lookup — the text view looks fonts up per character; here
//! the characters of the document shown are looked up (cached) and the ones no installed font has
//! are announced once each, because a box on the screen says nothing about why.

use super::model::Reader;
use super::state::Shown;
use crate::labels::Labels;

impl Reader {
    /// Look at the characters of the document shown and tell about new ones nobody can draw.
    pub(super) fn tell_missing_glyphs(&mut self) {
        let Shown::Document { document, .. } = &self.shown else {
            return;
        };
        for block in &document.blocks {
            for c in block.plain_text().chars() {
                let _ = fs_components_widgets::fallback_family(c);
            }
        }
        let new: Vec<char> = fs_components_widgets::missing_glyphs()
            .into_iter()
            .filter(|c| self.glyphs_told.insert(*c))
            .collect();
        if !new.is_empty() {
            let list = new
                .iter()
                .take(8)
                .map(|c| format!("U+{:04X}", u32::from(*c)))
                .collect::<Vec<_>>()
                .join(" ");
            self.notice = Some(Labels::fill(&self.labels.glyphs_missing, &list));
        }
    }
}
