//! Tests of `Fallback` (the Decorator) and of loading a `Catalog` from a table.

use fs_components_words::{
    Args, Catalog, English, EnglishPlural, Fallback, PolishPlural, Said, TextId, Words,
};

const EN: &[(&str, &str)] = &[("a", "English A"), ("b", "English B {x}")];

fn pl() -> Catalog {
    Catalog::from_table(PolishPlural, &[("a", "Polskie A")])
}

#[test]
fn text_present_in_the_language_is_used() {
    let w = Fallback::new(pl(), English::new(EN));
    assert_eq!(
        w.say(&TextId::new("a"), &Args::new()),
        Said::found(&"a".into(), "Polskie A")
    );
}

#[test]
fn missing_text_comes_in_english() {
    let w = Fallback::new(pl(), English::new(EN));
    assert_eq!(
        w.say(&"b".into(), &Args::new().text("x", "1")).text(),
        "English B 1"
    );
}

#[test]
fn missing_everywhere_shows_the_id() {
    let w = Fallback::new(pl(), English::new(EN));
    assert_eq!(
        w.say(&"zz".into(), &Args::new()),
        Said::missing(&"zz".into())
    );
}

#[test]
fn a_text_the_language_cannot_fill_falls_back_too() {
    let broken = Catalog::from_table(PolishPlural, &[("b", "Polskie {nope}")]);
    let w: Box<dyn Words> = Box::new(Fallback::new(broken, English::new(EN)));
    assert_eq!(
        w.say(&"b".into(), &Args::new().text("x", "1")).text(),
        "English B 1"
    );
}

#[test]
fn fallbacks_nest() {
    let de = Catalog::from_table(EnglishPlural, &[("c", "Deutsch C")]);
    let w = Fallback::new(de, Fallback::new(pl(), English::new(EN)));
    assert_eq!(w.say(&"c".into(), &Args::new()).text(), "Deutsch C");
    assert_eq!(w.say(&"a".into(), &Args::new()).text(), "Polskie A");
    assert_eq!(
        w.say(&"b".into(), &Args::new().text("x", "!")).text(),
        "English B !"
    );
}

#[test]
fn catalog_from_text_table() {
    let text = "# comment\n\nfiles.one = One file\ngreet = Hallo {name}, a=b\n";
    let c = Catalog::from_text(EnglishPlural, text).unwrap();
    assert_eq!(c.say(&"files.one".into(), &Args::new()).text(), "One file");
    assert_eq!(
        c.say(&"greet".into(), &Args::new().text("name", "X"))
            .text(),
        "Hallo X, a=b"
    );
}

#[test]
fn catalog_from_text_reports_the_bad_line() {
    let e = Catalog::from_text(EnglishPlural, "ok = fine\nno equals sign\n").unwrap_err();
    assert_eq!(e.line(), 2);
    let e = Catalog::from_text(EnglishPlural, "ok = fine\nbad = {x\n").unwrap_err();
    assert_eq!(e.line(), 2);
    assert!(e.to_string().contains("line 2"));
}

#[test]
fn later_entry_replaces_earlier() {
    let c = Catalog::from_table(EnglishPlural, &[("a", "one"), ("a", "two")]);
    assert_eq!(c.say(&"a".into(), &Args::new()).text(), "two");
}

#[test]
fn dates_and_numbers_keep_their_kind_and_render_neutrally() {
    let c = Catalog::from_table(EnglishPlural, &[("t", "{d} {t} {n}")]);
    let args = Args::new()
        .date("d", 1_000_000_000)
        .time("t", 1_000_000_000)
        .number("n", 2.5);
    assert_eq!(c.say(&"t".into(), &args).text(), "2001-09-09 01:46 2.5");
    let args = Args::new()
        .date("d", -86_400)
        .time("t", -1)
        .number("n", 3.0);
    assert_eq!(c.say(&"t".into(), &args).text(), "1969-12-31 23:59 3");
}
