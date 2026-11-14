//! The words of the block menu: both languages, every id twinned.
//!
//! Tests: Model tests over the words.

use fs_components_mf2::check_pairs;
use fs_components_widgets::slash_menu::{SlashLabels, TEXTS};
use fs_components_words::{Locale, Localized};

#[test]
fn the_menu_has_german_and_english_words_for_every_block_type() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    let en = SlashLabels::default();
    let de = SlashLabels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(en.headings[0], "Heading 1");
    assert_eq!(de.headings[0], "Überschrift 1");
    assert_ne!(en, de);
    // Every word is filled in both.
    for l in [&en, &de] {
        for w in [
            &l.bullet,
            &l.numbered,
            &l.task,
            &l.table,
            &l.quote,
            &l.code,
            &l.divider,
            &l.none,
        ] {
            assert_ne!(w.as_str(), "");
        }
    }
}
