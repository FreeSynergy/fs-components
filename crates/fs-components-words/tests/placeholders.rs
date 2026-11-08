//! Tests of named placeholders, against the `Words` trait only.

use fs_components_words::{Args, English, Said, TextId, Value, Words};

fn say(table: &[(&str, &str)], id: &str, args: &Args) -> Said {
    let words: Box<dyn Words> = Box::new(English::new(table));
    words.say(&TextId::new(id), args)
}

#[test]
fn named_placeholder_is_filled() {
    let s = say(
        &[("hi", "Hello {name}!")],
        "hi",
        &Args::new().text("name", "Ada"),
    );
    assert_eq!(s, Said::found(&"hi".into(), "Hello Ada!"));
}

#[test]
fn order_in_template_is_free() {
    let t = [
        ("a", "{who} deleted {what}"),
        ("b", "{what} wurde von {who} gelöscht"),
    ];
    let args = Args::new().text("who", "Ada").text("what", "notes.md");
    assert_eq!(say(&t, "a", &args).text(), "Ada deleted notes.md");
    assert_eq!(
        say(&t, "b", &args).text(),
        "notes.md wurde von Ada gelöscht"
    );
}

#[test]
fn same_name_twice_and_unused_value() {
    let args = Args::new().text("x", "1").text("unused", "u");
    assert_eq!(say(&[("t", "{x}-{x}")], "t", &args).text(), "1-1");
}

#[test]
fn missing_value_shows_the_id_not_a_guess() {
    let s = say(&[("hi", "Hello {name}!")], "hi", &Args::new());
    assert_eq!(s, Said::missing(&"hi".into()));
    assert!(!s.is_found());
}

#[test]
fn missing_text_shows_the_id() {
    assert_eq!(say(&[], "no.such", &Args::new()).text(), "no.such");
}

#[test]
fn empty_id_and_empty_template_are_never_empty_text() {
    assert_ne!(say(&[], "", &Args::new()).text(), "");
    assert_ne!(say(&[("e", "")], "e", &Args::new()).text(), "");
}

#[test]
fn broken_templates_do_not_panic() {
    for bad in [
        "{",
        "}",
        "{x",
        "a {x, plural, one {#}",
        "{x, plural}",
        "{x, nope, a {b}}",
        "\\",
        "{ }",
    ] {
        let s = say(&[("t", bad)], "t", &Args::new().text("x", "1"));
        assert_eq!(s, Said::missing(&"t".into()), "template {bad:?}");
    }
}

#[test]
fn braces_and_hash_in_values_are_not_evaluated_again() {
    let args = Args::new().text("v", "{name} # {x, plural, other {z}} }");
    let s = say(&[("t", "[{v}]")], "t", &args);
    assert_eq!(s.text(), "[{name} # {x, plural, other {z}} }]");
}

#[test]
fn escaped_literals_and_plain_hash() {
    let s = say(&[("t", "\\{a\\} # \\\\")], "t", &Args::new());
    assert_eq!(s.text(), "{a} # \\");
}

#[test]
fn unicode_in_template_and_values() {
    let args = Args::new().text("n", "Zoë 🎉 日本語");
    assert_eq!(
        say(&[("t", "Grüß dich, {n} – ✓")], "t", &args).text(),
        "Grüß dich, Zoë 🎉 日本語 – ✓"
    );
}

#[test]
fn values_keep_their_kind_and_last_set_wins() {
    let args = Args::new().text("n", "a").count("n", 4);
    assert_eq!(args.get("n"), Some(&Value::Count(4)));
    assert_eq!(args.get("zz"), None);
}
