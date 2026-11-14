//! Tests of the block menu: what "/" is, how typing filters, what a choice writes.
//!
//! Pattern: none (tests).
//!
//! Tests: model tests over the pure functions (no window).

use fs_components_widgets::SlashLabels;
use fs_components_words::{Locale, Localized};

use super::slash::{Kind, matching, query};

#[test]
fn a_slash_at_the_start_without_a_space_is_a_menu_request() {
    assert_eq!(query("/"), Some(""));
    assert_eq!(query("/head"), Some("head"));
    for text in ["", "a/", "/ a", "/a b", "/a\nb", " /"] {
        assert_eq!(query(text), None, "{text:?}");
    }
}

#[test]
fn typing_filters_by_name_in_the_language_shown_and_by_marker() {
    let en = SlashLabels::default();
    let de = SlashLabels::for_locale(&Locale::parse("de").unwrap());
    assert_eq!(matching("", &en).len(), 10);
    assert_eq!(
        matching("heading", &en),
        [Kind::Heading(1), Kind::Heading(2), Kind::Heading(3)]
    );
    assert_eq!(matching("HEADING 2", &en), [Kind::Heading(2)]);
    assert_eq!(matching("überschrift", &de).len(), 3);
    // The English word finds nothing in the German menu; the marker does in both.
    assert_eq!(matching("heading", &de), []);
    assert_eq!(matching("```", &de), [Kind::Code]);
    assert_eq!(matching("##", &en), [Kind::Heading(2), Kind::Heading(3)]);
    assert_eq!(matching("zzz", &en), []);
}

#[test]
fn every_choice_writes_the_markdown_of_its_type() {
    let at = |k: Kind| {
        let w = k.written();
        (w.text, w.at, w.block_after)
    };
    assert_eq!(at(Kind::Heading(2)), ("## ".into(), 3, false));
    assert_eq!(at(Kind::Task), ("- [ ] ".into(), 6, false));
    assert_eq!(at(Kind::Code), ("```\n\n```".into(), 4, false));
    assert_eq!(at(Kind::Divider), ("---".into(), 3, true));
    for k in Kind::ALL {
        let w = k.written();
        assert!(w.at <= w.text.len(), "{k:?}");
        assert_ne!(w.text, "");
    }
}
