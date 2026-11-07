//! Tests of plural selection and of the "a library fits behind it" proof.

use fs_components_words::{
    Args, Catalog, EnglishPlural, PluralCategory, PluralRules, PolishPlural, Said, TextId, Words,
};

const EN: &[(&str, &str)] = &[(
    "files.deleted",
    "{who} deleted {count, plural, one {# file} other {# files}} in {where}",
)];
const PL: &[(&str, &str)] = &[(
    "files.deleted",
    "W {where} {who} usunął {count, plural, one {# plik} few {# pliki} many {# plików} other {# pliku}}",
)];

/// The one call every language gets: same id, same args.
fn call(words: &dyn Words, n: u64) -> Said {
    let args = Args::new()
        .text("who", "Ada")
        .count("count", n)
        .text("where", "Docs");
    words.say(&TextId::new("files.deleted"), &args)
}

#[test]
fn english_one_and_other() {
    let w = Catalog::from_table(EnglishPlural, EN);
    assert_eq!(call(&w, 1).text(), "Ada deleted 1 file in Docs");
    assert_eq!(call(&w, 0).text(), "Ada deleted 0 files in Docs");
    assert_eq!(call(&w, 5).text(), "Ada deleted 5 files in Docs");
}

#[test]
fn polish_three_forms_and_other_order_with_the_same_call() {
    let w = Catalog::from_table(PolishPlural, PL);
    assert_eq!(call(&w, 1).text(), "W Docs Ada usunął 1 plik");
    assert_eq!(call(&w, 2).text(), "W Docs Ada usunął 2 pliki");
    assert_eq!(call(&w, 24).text(), "W Docs Ada usunął 24 pliki");
    assert_eq!(call(&w, 5).text(), "W Docs Ada usunął 5 plików");
    assert_eq!(call(&w, 12).text(), "W Docs Ada usunął 12 plików");
    assert_eq!(call(&w, 22).text(), "W Docs Ada usunął 22 pliki");
    assert_eq!(call(&w, 0).text(), "W Docs Ada usunął 0 plików");
}

#[test]
fn polish_categories() {
    let r = PolishPlural;
    assert_eq!(r.category(1), PluralCategory::One);
    assert_eq!(r.category(112), PluralCategory::Many);
    assert_eq!(r.category(103), PluralCategory::Few);
}

#[test]
fn a_category_missing_in_the_template_falls_to_other() {
    let w = Catalog::from_table(PolishPlural, &[("t", "{n, plural, one {# a} other {# b}}")]);
    assert_eq!(w.say(&"t".into(), &Args::new().count("n", 3)).text(), "3 b");
}

#[test]
fn no_matching_branch_and_no_other_is_missing() {
    let w = Catalog::from_table(EnglishPlural, &[("t", "{n, plural, one {# a}}")]);
    assert_eq!(
        w.say(&"t".into(), &Args::new().count("n", 3)),
        Said::missing(&"t".into())
    );
}

#[test]
fn plural_needs_a_count() {
    let w = Catalog::from_table(EnglishPlural, EN);
    let args = Args::new()
        .text("who", "A")
        .text("where", "B")
        .text("count", "3");
    assert_eq!(
        w.say(&"files.deleted".into(), &args),
        Said::missing(&"files.deleted".into())
    );
    let args = Args::new().text("who", "A").text("where", "B");
    assert!(!w.say(&"files.deleted".into(), &args).is_found());
}

#[test]
fn hash_is_only_the_count_inside_a_branch() {
    let w = Catalog::from_table(EnglishPlural, &[("t", "No. # of {n, plural, other {#/#}}")]);
    assert_eq!(
        w.say(&"t".into(), &Args::new().count("n", 7)).text(),
        "No. # of 7/7"
    );
}

#[test]
fn braces_in_values_inside_a_branch_are_not_evaluated() {
    let w = Catalog::from_table(EnglishPlural, &[("t", "{n, plural, other {# x {v}}}")]);
    let args = Args::new().count("n", 2).text("v", "{n} #");
    assert_eq!(w.say(&"t".into(), &args).text(), "2 x {n} #");
}

#[test]
fn count_as_plain_placeholder_is_decimal() {
    let w = Catalog::from_table(EnglishPlural, &[("t", "{n} items")]);
    assert_eq!(
        w.say(&"t".into(), &Args::new().count("n", 12)).text(),
        "12 items"
    );
}
