//! Count texts: a message that shows a number says 1 and 2 differently where the language has a singular (finding W3 of
//! 2026-11-14: "1 Seiten").
//!
//! Tests: model tests over the `check_counts` render for n = 0, 1, 2 in German and English; the counter-test is a message
//! that was written without a plural selector.

use fs_components_mf2::{Source, check_counts};
use fs_components_words::{Locale, Words};

const EN_GOOD: &str = "pages =\n  .input {$count :number}\n  .match $count one {{{$count} page}} * {{{$count} pages}}\n";
const DE_GOOD: &str = "pages =\n  .input {$count :number}\n  .match $count one {{{$count} Seite}} * {{{$count} Seiten}}\n";
const EN_BAD: &str = "pages = {$count} pages\n";
const DE_BAD: &str = "pages = {$count} Seiten\n";

fn sources(en: &'static str, de: &'static str) -> [Source; 2] {
    [
        Source {
            language: "en",
            text: en,
        },
        Source {
            language: "de",
            text: de,
        },
    ]
}

#[test]
fn a_count_text_with_a_plural_selector_says_one_and_many_in_both_languages() {
    let s = sources(EN_GOOD, DE_GOOD);
    for lang in ["en", "de"] {
        let locale = Locale::parse(lang).unwrap();
        assert_eq!(
            check_counts(&s, &locale, "pages", "count"),
            Vec::<String>::new(),
            "{lang}"
        );
    }
    let texts = fs_components_mf2::Mf2Texts::new(&s, &Locale::parse("de").unwrap());
    let say = |n| {
        texts
            .say(
                &fs_components_words::TextId::new("pages"),
                &fs_components_words::Args::new().count("count", n),
            )
            .text()
            .to_owned()
    };
    assert_eq!(
        (say(0).as_str(), say(1).as_str(), say(2).as_str()),
        ("0 Seiten", "1 Seite", "2 Seiten")
    );
}

#[test]
fn a_count_text_without_a_plural_selector_is_found() {
    let s = sources(EN_BAD, DE_BAD);
    for lang in ["en", "de"] {
        let found = check_counts(&s, &Locale::parse(lang).unwrap(), "pages", "count");
        assert_eq!(found.len(), 1, "{lang}: {found:?}");
        assert!(found[0].contains("said alike"), "{found:?}");
    }
}
