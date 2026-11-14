//! The build-time check: every message in every language, the same variables, every message compiles.
//!
//! Pattern: none — a pure function over the sources. The variables are those `mf2` finds in a message (`manifest.slots`), so
//! two languages are compared by what they need, not by their wording. Run it as one test in every crate that has texts:
//! `assert_eq!(check_pairs(SOURCES), Vec::<String>::new())`.

use std::collections::{BTreeMap, BTreeSet};

use crate::Source;
use fs_components_words::Locale;

use crate::source::entries;

type Vars = BTreeSet<String>;

fn read(source: &Source, language: &str, problems: &mut Vec<String>) -> BTreeMap<String, Vars> {
    let (list, found) = entries(source.text);
    problems.extend(found.into_iter().map(|p| format!("{language}: {p}")));
    let mut out = BTreeMap::new();
    for e in list {
        match mf2::compile_str(&e.message, language) {
            Ok(c) => {
                let vars = c
                    .manifest
                    .slots
                    .first()
                    .map(|s| s.iter().cloned().collect())
                    .unwrap_or_default();
                out.insert(e.id, vars);
            }
            Err(why) => problems.push(format!("{language}: {} (line {}): {why}", e.id, e.line)),
        }
    }
    out
}

/// The problems of a set of sources; empty means sound. Needs a source for `en` and one for `de`; every other language is
/// compared with `en`. Reported: a line that is no entry, an id twice, a message that does not compile, a message that one
/// language has and the other lacks, and a message whose variables differ from English.
pub fn check_pairs(sources: &[Source]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut all: Vec<(String, _)> = Vec::new();
    for source in sources {
        // `de-at` and `de-AT` are the same language: the name is normalised before anything is compared.
        match Locale::parse(source.language) {
            Ok(tag) => {
                let name = tag.to_string();
                if all.iter().any(|(l, _)| *l == name) {
                    problems.push(format!("{name}: given by two sources"));
                } else {
                    let messages = read(source, &name, &mut problems);
                    all.push((name, messages));
                }
            }
            Err(why) => problems.push(format!("{:?}: {why}", source.language)),
        }
    }
    let Some(english) = all.iter().find(|(l, _)| l == "en").map(|(_, m)| m) else {
        problems.push("no source for en".to_owned());
        return problems;
    };
    if !all.iter().any(|(l, _)| l == "de") {
        problems.push("no source for de".to_owned());
    }
    for (language, messages) in all.iter().filter(|(l, _)| l != "en") {
        for (id, vars) in english {
            match messages.get(id) {
                None => problems.push(format!("{id}: in en, missing in {language}")),
                Some(other) if other != vars => {
                    problems.push(format!(
                        "{id}: variables differ, en {vars:?}, {language} {other:?}"
                    ));
                }
                Some(_) => {}
            }
        }
        for id in messages.keys().filter(|id| !english.contains_key(*id)) {
            problems.push(format!("{id}: in {language}, missing in en"));
        }
    }
    problems
}

/// The problems of a message that shows a number (`count_variable`, an integer): rendered for 0, 1 and 2 in `locale`, the
/// messages for 1 and 2 must both hold the number and must not be the same sentence (`1 pages` and `2 pages` differ in a
/// language with a singular only through a plural selector), and none may be empty or missing. Empty means sound.
pub fn check_counts(
    sources: &[Source],
    locale: &Locale,
    id: &str,
    count_variable: &str,
) -> Vec<String> {
    use fs_components_words::{Args, TextId, Words};
    let texts = crate::Mf2Texts::new(sources, locale);
    let mut said = Vec::new();
    let mut problems = Vec::new();
    for n in 0_u64..=2 {
        let args = Args::new().count(count_variable, n);
        let text = texts.say(&TextId::new(id), &args).text().to_owned();
        if text.trim().is_empty() || text == id {
            problems.push(format!("{id} for {n} in {locale}: no text ({text:?})"));
        }
        said.push(text);
    }
    if said.len() == 3 && !said[1].contains('1') {
        problems.push(format!(
            "{id} for 1 in {locale}: the number is missing ({:?})",
            said[1]
        ));
    }
    // The same sentence with the number taken out: for 1 and 2 it must differ (a singular and a plural).
    let bare = |t: &str, n: &str| t.replace(n, "#");
    if said.len() == 3 && bare(&said[1], "1") == bare(&said[2], "2") {
        problems.push(format!(
            "{id} in {locale}: one and two are said alike ({:?} / {:?})",
            said[1], said[2]
        ));
    }
    problems
}
