//! The next level is as wide as its longest line (no wrapped "Systemsprache (Standard)"), within a minimum and a maximum and never
//! wider than the window; and a frontend with a translated display name is shown with it.

#[path = "menu_common/mod.rs"]
mod menu_common;

use fs_components_app::{FrontendItem, LanguageChoice, LanguageControl, Scope};
use fs_components_words::Locale;
use menu_common::fixture::entry;
use menu_common::standard::{SUB, fixture_in, labels, menus};

#[test]
fn the_longest_line_of_the_next_level_does_not_wrap_in_german_and_english() {
    for (lang, other) in [("de_DE.UTF-8", "en"), ("en_US.UTF-8", "de")] {
        for shared in [false, true] {
            let mut f = fixture_in("width", lang, 800.0);
            if shared {
                // The longest system line: "Wie alle Programme (English)" / "Like all programs (Deutsch)".
                f.control
                    .choose(
                        Scope::AllPrograms,
                        &LanguageChoice::Language(Locale::parse(other).unwrap()),
                    )
                    .unwrap();
                f.h.app.set_active_frontend("board").unwrap();
            }
            f.h.click("fs-app-menu-0");
            f.h.click(&entry("fs.language"));
            let system = f.h.bounds(&entry("fs.language.system"));
            let short = f.h.bounds(&entry("fs.language.en"));
            assert!(
                (system.3 - short.3).abs() < 8.0,
                "{lang} shared={shared}: the longest line is one line high like the others (a wrapped line would be 17 px or more taller): {system:?} against {short:?}"
            );
            let sub = f.h.bounds(SUB);
            assert!(
                sub.2 >= 140.0 && sub.2 <= 360.0,
                "{lang}: within the minimum and maximum: {sub:?}"
            );
            assert!(
                menus(&f.h.app)[0].entries[1]
                    .children
                    .iter()
                    .all(|c| !labels(std::slice::from_ref(c))[0].is_empty())
            );
            let _ = std::fs::remove_dir_all(f.dir);
        }
    }
}

#[test]
fn a_short_next_level_is_narrower_than_a_long_one_and_never_wider_than_the_window() {
    // View > Frontends has the short lines "Classic", "Board", "Day list": narrower than the language menu's long line.
    let (short, long) = {
        let mut f = fixture_in("short", "en_US.UTF-8", 800.0);
        f.h.click("fs-app-menu-1");
        f.h.click(&entry("fs.frontends"));
        let short = f.h.bounds(SUB).2;
        f.h.click("fs-app-menu-1");
        f.h.settle();
        f.h.click("fs-app-menu-0");
        f.h.click(&entry("fs.language"));
        let long = f.h.bounds(SUB).2;
        let _ = std::fs::remove_dir_all(&f.dir);
        (short, long)
    }; // the first window is gone before the next one opens: one at a time
    assert!(
        short < long,
        "narrower for short lines: {short} against {long}"
    );
    assert!(short >= 140.0, "but not below the minimum: {short}");
    // A window narrower than the maximum: the next level fits into it.
    let mut f = fixture_in("narrow", "de_DE.UTF-8", 200.0);
    f.h.click("fs-app-menu-0");
    f.h.click(&entry("fs.language"));
    let (x, _, w, _) = f.h.bounds(SUB);
    assert!(
        x >= 0.0 && x + w <= 200.0,
        "inside a window of 200 px: {x} {w}"
    );
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn a_frontend_with_a_translated_display_name_shows_it_and_a_proper_name_stays() {
    let item = FrontendItem::new("storefront", "Storefront")
        .named_in("de", "Schaufenster")
        .named_in("fr-CA", "Vitrine");
    let l = |t: &str| Locale::parse(t).unwrap();
    assert_eq!(item.label_for(&l("de")), "Schaufenster");
    assert_eq!(
        item.label_for(&l("de-AT")),
        "Schaufenster",
        "a region uses its language"
    );
    assert_eq!(
        item.label_for(&l("en")),
        "Storefront",
        "no English name: the plain label"
    );
    assert_eq!(item.label_for(&l("fr")), "Storefront", "fr-CA is not fr");
    assert_eq!(item.label_for(&l("fr-CA")), "Vitrine");
    let plain = FrontendItem::new("compact", "Compact");
    assert_eq!(
        plain.label_for(&l("de")),
        "Compact",
        "a proper name stays in every language"
    );
    // In the menu.
    let f = fixture_in("names", "de_DE.UTF-8", 800.0);
    f.h.app
        .set_frontends(vec![item.clone(), plain], "storefront", |_| {})
        .unwrap();
    assert_eq!(
        labels(&menus(&f.h.app)[1].entries[2].children),
        ["Schaufenster", "Compact"]
    );
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn a_line_wider_than_the_maximum_ends_the_next_level_at_the_maximum_and_wraps() {
    let long = "A frontend whose display name is much longer than any next level of a menu may be wide, so that it has to wrap";
    let mut f = fixture_in("longline", "en_US.UTF-8", 800.0);
    f.h.app
        .set_frontends(
            vec![FrontendItem::new("long", long), FrontendItem::new("b", "B")],
            "long",
            |_| {},
        )
        .unwrap();
    f.h.click("fs-app-menu-1");
    f.h.click(&entry("fs.frontends"));
    let sub = f.h.bounds(SUB);
    assert!((sub.2 - 360.0).abs() < 1.0, "ends at the maximum: {sub:?}");
    assert!(sub.0 + sub.2 <= 800.0, "inside the window: {sub:?}");
    let line = f.h.bounds(&entry("fs.frontend.long"));
    let short = f.h.bounds(&entry("fs.frontend.b"));
    assert!(
        line.3 > short.3 + 8.0,
        "the long line wraps: {line:?} against {short:?}"
    );
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn the_most_specific_name_wins_in_either_order_of_naming() {
    let l = |t: &str| Locale::parse(t).unwrap();
    let a = FrontendItem::new("x", "Plain")
        .named_in("de", "Deutsch")
        .named_in("de-AT", "Oesterreichisch");
    let b = FrontendItem::new("x", "Plain")
        .named_in("de-AT", "Oesterreichisch")
        .named_in("de", "Deutsch");
    for item in [&a, &b] {
        assert_eq!(item.label_for(&l("de-AT")), "Oesterreichisch");
        assert_eq!(
            item.label_for(&l("de")),
            "Deutsch",
            "counter-probe: de is not de-AT"
        );
    }
    // An empty name is skipped: the language's name or the plain label shows.
    let empty = FrontendItem::new("x", "Plain").named_in("de", "");
    assert_eq!(empty.label_for(&l("de")), "Plain");
}
