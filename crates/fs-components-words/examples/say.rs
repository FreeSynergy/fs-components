//! Example: one call, English and Polish, with a fallback to English.
//!
//! Pattern: none — an example that uses `Words`, `Catalog` and `Fallback`.

use fs_components_words::{Args, Catalog, English, Fallback, PolishPlural, TextId, Words};

const EN: &[(&str, &str)] = &[
    (
        "files.deleted",
        "{who} deleted {count, plural, one {# file} other {# files}}",
    ),
    ("hello", "Hello"),
];

fn main() {
    let polish = Catalog::from_text(
        PolishPlural,
        "files.deleted = {who} usunął {count, plural, one {# plik} few {# pliki} many {# plików}}",
    )
    .expect("valid table");
    let words = Fallback::new(polish, English::new(EN));

    for n in [1, 3, 5] {
        let args = Args::new().text("who", "Ada").count("count", n);
        println!("{}", words.say(&TextId::new("files.deleted"), &args));
    }
    // Not in Polish: comes in English. Not anywhere: shows the identifier.
    println!("{}", words.say(&TextId::new("hello"), &Args::new()));
    println!("{}", words.say(&TextId::new("nope"), &Args::new()));
}
