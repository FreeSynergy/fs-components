//! Tests of hostile or odd input: depth, extreme values, strange templates, foreign `Words`.

use fs_components_words::{
    Args, Catalog, English, EnglishPlural, Fallback, PolishPlural, Said, TextId, Words,
};

fn en(table: &[(&str, &str)]) -> Catalog {
    Catalog::from_table(EnglishPlural, table)
}

fn one(c: &Catalog, args: &Args) -> Said {
    c.say(&TextId::new("t"), args)
}

#[test]
fn deep_nesting_is_refused_not_a_crash() {
    let deep = "{n, plural, other {".repeat(100_000);
    let e = Catalog::from_text(EnglishPlural, &format!("ok = fine\nt = {deep}\n")).unwrap_err();
    assert_eq!(e.line(), 2);
    let c = en(&[("t", &deep)]);
    let args = Args::new().count("n", 1);
    assert_eq!(one(&c, &args), Said::missing(&"t".into()));
    assert_eq!(c.validate(), vec![TextId::new("t")]);
}

#[test]
fn nesting_within_the_limit_works() {
    let open = "{n, plural, other {".repeat(10);
    let t = format!("{open}#{}", "}}".repeat(10));
    let c = en(&[("t", &t)]);
    assert_eq!(one(&c, &Args::new().count("n", 4)).text(), "4");
}

#[test]
fn whitespace_only_template_is_faulty() {
    let c = en(&[("t", "  \t ")]);
    assert!(!one(&c, &Args::new()).is_found());
    assert_eq!(c.validate().len(), 1);
}

#[test]
fn unknown_escapes_are_errors() {
    for bad in ["a\\nb", "\\t", "\\x", "ends\\"] {
        let c = en(&[("t", bad)]);
        assert!(!one(&c, &Args::new()).is_found(), "{bad:?}");
    }
    assert_eq!(one(&en(&[("t", "a\\ b")]), &Args::new()).text(), "a b");
}

#[test]
fn duplicate_plural_branches_and_odd_names_are_refused() {
    for bad in [
        "{n, plural, one {a} one {b} other {c}}",
        "{a b}",
        "{a\tb}",
        "{n, plural, other {a} other {b}}",
    ] {
        let c = en(&[("t", bad)]);
        let args = Args::new().count("n", 1).text("a b", "x");
        assert!(!one(&c, &args).is_found(), "{bad:?}");
    }
}

#[test]
fn from_text_keeps_an_escaped_trailing_space_and_refuses_duplicates() {
    let c = Catalog::from_text(EnglishPlural, "t = Name:\\ \nu = x  \n").unwrap();
    assert_eq!(one(&c, &Args::new()).text(), "Name: ");
    assert_eq!(c.say(&"u".into(), &Args::new()).text(), "x");
    let e = Catalog::from_text(EnglishPlural, "a = 1\nb = 2\na = 3\n").unwrap_err();
    assert_eq!(e.line(), 3);
}

#[test]
fn validate_lists_every_faulty_template() {
    let c = en(&[("good", "fine {x}"), ("bad2", "{x"), ("bad1", "}")]);
    assert_eq!(c.validate(), vec![TextId::new("bad1"), TextId::new("bad2")]);
    assert!(en(&[("a", "ok")]).validate().is_empty());
    assert_eq!(
        English::new(&[("z", "{")]).validate(),
        vec![TextId::new("z")]
    );
}

#[test]
fn extreme_values_render_sensibly() {
    let c = en(&[("t", "{d}|{t}|{n}|{c}")]);
    let show = |d: i64, n: f64, k: u64| {
        let args = Args::new()
            .date("d", d)
            .time("t", d)
            .number("n", n)
            .count("c", k);
        one(&c, &args).text().to_owned()
    };
    assert_eq!(
        show(i64::MAX, 1.0, u64::MAX),
        "292277026596-12-04|15:30|1|18446744073709551615"
    );
    assert_eq!(show(i64::MIN, 1.5, 0), "-292277022657-01-27|08:29|1.5|0");
    assert!(show(-62_167_219_200 - 86_400, 0.0, 0).starts_with("-0001-12-31|"));
    assert_eq!(show(0, f64::NAN, 0).split('|').nth(2), Some("NaN"));
    assert_eq!(show(0, f64::INFINITY, 0).split('|').nth(2), Some("∞"));
    assert_eq!(show(0, f64::NEG_INFINITY, 0).split('|').nth(2), Some("-∞"));
    assert_eq!(show(0, -0.0, 0).split('|').nth(2), Some("0"));
    let big = show(0, 1e300, 0);
    assert!(big.len() < 40, "{big}");
}

#[test]
fn polish_forms_around_the_teens() {
    let c = Catalog::from_table(
        PolishPlural,
        &[(
            "t",
            "{n, plural, one {# a} few {# b} many {# c} other {# d}}",
        )],
    );
    let say = |n| one(&c, &Args::new().count("n", n)).text().to_owned();
    assert_eq!(say(11), "11 c");
    assert_eq!(say(21), "21 c");
    assert_eq!(say(22), "22 b");
    assert_eq!(say(112), "112 c");
    assert_eq!(say(u64::MAX), format!("{} c", u64::MAX));
}

/// A foreign implementation that answers with nothing useful.
struct Blank(&'static str);

impl Words for Blank {
    fn say(&self, id: &TextId, _: &Args) -> Said {
        Said::found(id, self.0)
    }
}

#[test]
fn empty_or_blank_answers_of_a_foreign_words_fall_through() {
    for blank in ["", "   ", "\n\t"] {
        let w = Fallback::new(Blank(blank), English::new(&[("t", "English")]));
        assert_eq!(
            w.say(&"t".into(), &Args::new()).text(),
            "English",
            "{blank:?}"
        );
    }
    let both = Fallback::new(Blank(""), Blank(" "));
    let s = both.say(&"t".into(), &Args::new());
    assert_eq!(s, Said::missing(&"t".into()));
    assert_eq!(s.text(), "t");
    assert_eq!(both.say(&"".into(), &Args::new()).text(), "(no id)");
}
