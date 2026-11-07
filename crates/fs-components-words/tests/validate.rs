//! Tests of `validate()` and of loading from text: what counts as a faulty table.

use fs_components_words::{Args, Catalog, English, EnglishPlural, TextId, Words};

fn en(table: &[(&str, &str)]) -> Catalog {
    Catalog::from_table(EnglishPlural, table)
}

#[test]
fn templates_that_always_print_nothing_are_faulty() {
    for empty in [
        "\\ ",
        "{n, plural, other {}}",
        "{n, plural, other { }}",
        " \\  ",
    ] {
        let c = en(&[("t", empty)]);
        assert_eq!(c.validate(), vec![TextId::new("t")], "{empty:?}");
        assert!(
            !c.say(&"t".into(), &Args::new().count("n", 1)).is_found(),
            "{empty:?}"
        );
        let e = Catalog::from_text(EnglishPlural, &format!("ok = x\nt = {empty}\n")).unwrap_err();
        assert_eq!(e.line(), 2, "{empty:?}");
    }
    // One branch that can show something is enough.
    assert!(
        en(&[("t", "{n, plural, one {} other {x}}")])
            .validate()
            .is_empty()
    );
}

#[test]
fn a_byte_order_mark_at_the_start_is_ignored() {
    let c = Catalog::from_text(EnglishPlural, "\u{feff}t = hello\nu = x\n").unwrap();
    assert_eq!(c.say(&"t".into(), &Args::new()).text(), "hello");
    assert!(c.validate().is_empty());
}

#[test]
fn validate_reports_a_plural_without_other() {
    let c = en(&[
        ("p", "{n, plural, one {# a}}"),
        ("ok", "{n, plural, one {a} other {b}}"),
    ]);
    assert_eq!(c.validate(), vec![TextId::new("p")]);
    let nested = en(&[("q", "{n, plural, other {{m, plural, one {x}}}}")]);
    assert_eq!(nested.validate(), vec![TextId::new("q")]);
}

#[test]
fn validate_reports_repeated_ids_of_a_table() {
    let dup = en(&[("a", "one"), ("b", "x"), ("a", "two"), ("a", "three")]);
    assert_eq!(dup.validate(), vec![TextId::new("a")]);
    assert_eq!(
        English::new(&[("a", "1"), ("a", "2")]).validate(),
        vec![TextId::new("a")]
    );
    assert_eq!(dup.say(&"a".into(), &Args::new()).text(), "three");
}
