//! The build-time check: every message in every language, the same variables, every message compiles.
//!
//! Pattern: none — a pure function over the sources. The variables are those `mf2` finds in a message (`manifest.slots`), so
//! two languages are compared by what they need, not by their wording. Run it as one test in every crate that has texts:
//! `assert_eq!(check_pairs(SOURCES), Vec::<String>::new())`.

use std::collections::{BTreeMap, BTreeSet};

use crate::Source;
use crate::source::entries;

type Vars = BTreeSet<String>;

fn read(source: &Source, problems: &mut Vec<String>) -> BTreeMap<String, Vars> {
    let (list, found) = entries(source.text);
    problems.extend(
        found
            .into_iter()
            .map(|p| format!("{}: {p}", source.language)),
    );
    let mut out = BTreeMap::new();
    for e in list {
        match mf2::compile_str(&e.message, source.language) {
            Ok(c) => {
                let vars = c
                    .manifest
                    .slots
                    .first()
                    .map(|s| s.iter().cloned().collect())
                    .unwrap_or_default();
                out.insert(e.id, vars);
            }
            Err(why) => problems.push(format!(
                "{}: {} (line {}): {why}",
                source.language, e.id, e.line
            )),
        }
    }
    out
}

/// The problems of a set of sources; empty means sound. Needs a source for `en` and one for `de`; every other language is
/// compared with `en`. Reported: a line that is no entry, an id twice, a message that does not compile, a message that one
/// language has and the other lacks, and a message whose variables differ from English.
pub fn check_pairs(sources: &[Source]) -> Vec<String> {
    let mut problems = Vec::new();
    let all: Vec<_> = sources
        .iter()
        .map(|s| (s.language, read(s, &mut problems)))
        .collect();
    let Some(english) = all.iter().find(|(l, _)| *l == "en").map(|(_, m)| m) else {
        problems.push("no source for en".to_owned());
        return problems;
    };
    if !all.iter().any(|(l, _)| *l == "de") {
        problems.push("no source for de".to_owned());
    }
    for (language, messages) in all.iter().filter(|(l, _)| *l != "en") {
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
