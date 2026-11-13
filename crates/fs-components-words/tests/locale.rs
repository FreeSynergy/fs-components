//! Tests of `Locale`: the BCP 47 parser, the normal form, the POSIX form and the chain.

use fs_components_words::Locale;

fn tag(s: &str) -> String {
    Locale::parse(s).unwrap().to_string()
}

#[test]
fn a_tag_is_normalised() {
    assert_eq!(tag("de"), "de");
    assert_eq!(tag("DE-at"), "de-AT");
    assert_eq!(tag("zh-hans-cn"), "zh-Hans-CN");
    assert_eq!(tag("es-419"), "es-419");
    assert_eq!(tag("de-1996"), "de-1996");
    assert_eq!(tag("de-DE-u-co-phonebk"), "de-DE-u-co-phonebk");
    assert_eq!(tag("en-x-Private"), "en-x-private");
}

#[test]
fn what_is_no_tag_is_refused() {
    for bad in [
        "", "d", "de_DE", "de--AT", "de-", "-de", "1de", "de-Hans-Hant", "de-AT-DE", "de-A", "deutsch1", "de-Latn-x",
        "toolonglang", "de-\u{e4}t", "de AT",
    ] {
        assert!(Locale::parse(bad).is_err(), "{bad:?} must be refused");
    }
}

#[test]
fn the_posix_form_loses_code_set_and_modifier_and_c_is_no_language() {
    let p = |s| Locale::from_posix(s).map(|l| l.to_string());
    assert_eq!(p("de_DE.UTF-8@euro").as_deref(), Some("de-DE"));
    assert_eq!(p("en_US").as_deref(), Some("en-US"));
    assert_eq!(p("de").as_deref(), Some("de"));
    assert_eq!(p(" fr_CA.utf8 ").as_deref(), Some("fr-CA"));
    for none in ["", "C", "POSIX", "C.UTF-8", "  ", "_DE", "12_34"] {
        assert_eq!(p(none), None, "{none:?}");
    }
}

#[test]
fn the_chain_goes_from_the_long_tag_to_the_language() {
    let chain = |s: &str| {
        Locale::parse(s)
            .unwrap()
            .chain()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    };
    assert_eq!(chain("zh-Hans-CN"), ["zh-Hans-CN", "zh-Hans", "zh"]);
    assert_eq!(chain("de"), ["de"]);
    assert_eq!(Locale::parse("de-AT").unwrap().primary(), "de");
    assert_eq!(Locale::english().to_string(), "en");
}
