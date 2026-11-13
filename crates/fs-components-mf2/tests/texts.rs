//! Tests of `Mf2Texts` (messages, plural, numbers, fallback chain) and `check_pairs` (each rule once, each with its
//! counter-probe). The MF2 spec examples at the top are spot checks of the library, not the official conformance suite.

use fs_components_mf2::{Mf2Texts, Source, check_pairs};
use fs_components_words::{Args, Locale, TextId, Words};

const EN: &str = "\
# English
hello = Hello, {$name}!
files =
    .input {$n :number}
    .match $n
    one {{{$n} file}}
    * {{{$n} files}}
price = {$p :number minimumFractionDigits=2} euros
escaped = Braces \\{ and \\} and a backslash \\\\
when = on {$day}, at {$time}
";
const DE: &str = "\
hello = Hallo, {$name}!
files =
    .input {$n :number}
    .match $n
    one {{{$n} Datei}}
    * {{{$n} Dateien}}
price = {$p :number minimumFractionDigits=2} Euro
escaped = Klammern \\{ und \\} und ein Backslash \\\\
when = am {$day}, um {$time} Uhr
";
const SOURCES: &[Source] = &[Source { language: "en", text: EN }, Source { language: "de", text: DE }];

fn say(locale: &str, id: &str, args: &Args) -> String {
    Mf2Texts::new(SOURCES, &Locale::parse(locale).unwrap()).say(&TextId::new(id), args).text().to_owned()
}

#[test]
fn a_text_comes_in_the_language_asked_for() {
    let a = Args::new().text("name", "Ada");
    assert_eq!(say("en", "hello", &a), "Hello, Ada!");
    assert_eq!(say("de", "hello", &a), "Hallo, Ada!");
    assert_eq!(say("de-AT", "hello", &a), "Hallo, Ada!", "the chain de-AT, de");
}

#[test]
fn plural_forms_come_from_the_language() {
    let n = |c| Args::new().count("n", c);
    assert_eq!(say("en", "files", &n(1)), "1 file");
    assert_eq!(say("en", "files", &n(3)), "3 files");
    assert_eq!(say("de", "files", &n(1)), "1 Datei");
    assert_eq!(say("de", "files", &n(0)), "0 Dateien");
    assert_eq!(say("en", "files", &n(0)), "0 files");
}

#[test]
fn numbers_follow_the_language_and_escapes_work() {
    let p = Args::new().number("p", 1234.5);
    assert_eq!(say("en", "price", &p), "1,234.50 euros");
    assert_eq!(say("de", "price", &p), "1.234,50 Euro");
    assert_eq!(say("en", "escaped", &Args::new()), "Braces { and } and a backslash \\");
    assert_eq!(say("de", "when", &Args::new().date("day", 86_400 * 365).time("time", 3600 * 7 + 60 * 5)), "am 1971-01-01, um 07:05 Uhr");
}

#[test]
fn a_text_missing_in_the_language_comes_in_english_and_one_missing_everywhere_shows_its_id() {
    let en_only = [Source { language: "en", text: "x = Only in English" }];
    let de = Mf2Texts::new(&en_only, &Locale::parse("de").unwrap());
    assert_eq!(de.plain("x"), "Only in English");
    let said = Mf2Texts::new(SOURCES, &Locale::parse("de").unwrap()).say(&TextId::new("nowhere"), &Args::new());
    assert!(!said.is_found());
    assert_eq!(said.text(), "nowhere");
    // A language without a source at all: English throughout.
    assert_eq!(say("fr", "hello", &Args::new().text("name", "Ada")), "Hello, Ada!");
}

#[test]
fn a_missing_value_is_a_missing_text_not_a_hole() {
    let said = Mf2Texts::new(SOURCES, &Locale::english()).say(&TextId::new("hello"), &Args::new());
    assert!(!said.is_found(), "no `name` given: {said}");
}

#[test]
fn the_shipped_sources_are_paired() {
    assert_eq!(check_pairs(SOURCES), Vec::<String>::new());
}

fn bad(en: &'static str, de: &'static str) -> Vec<String> {
    check_pairs(&[Source { language: "en", text: en }, Source { language: "de", text: de }])
}

#[test]
fn every_pairing_rule_fires_on_its_own_fault() {
    let has = |v: Vec<String>, what: &str| assert!(v.iter().any(|p| p.contains(what)), "{what}: {v:?}");
    has(bad("a = A\nb = B", "a = A"), "b: in en, missing in de");
    has(bad("a = A", "a = A\nb = B"), "b: in de, missing in en");
    has(bad("a = Hi {$x}", "a = Hallo {$y}"), "variables differ");
    has(bad("a = Hi {$x}", "a = Hallo"), "variables differ");
    has(bad("a = {$x", "a = A"), "a (line 1)");
    has(bad("a = A", "a = {$x"), "a (line 1)");
    has(bad("a = A\na = B", "a = A"), "given twice");
    has(bad("a = A", "a ="), "has no message");
    has(bad("just text", "a = A"), "no `=`");
    assert!(check_pairs(&[Source { language: "de", text: "a = A" }]).iter().any(|p| p == "no source for en"));
    assert!(check_pairs(&[Source { language: "en", text: "a = A" }]).iter().any(|p| p == "no source for de"));
    // Counter-probes: a good pair, and a variable used only in a plural branch, are sound.
    assert_eq!(bad("a = A {$x}", "a = Ein {$x}"), Vec::<String>::new());
    let plural = |a, b| bad(a, b);
    assert_eq!(
        plural("a = .input {$n :number} .match $n one {{{$n} x}} * {{{$n} xs}}", "a = .input {$n :number} .match $n one {{{$n} y}} * {{{$n} ys}}"),
        Vec::<String>::new()
    );
}
