//! Tests of the chain: program override, shared setting, `FS_LANG`, `LC_ALL`, `LC_MESSAGES`, `LANGUAGE`, `LANG`, English. Each link
//! is tested to win over the ones after it and to be skipped when it says nothing.

use std::collections::HashMap;

use fs_components_words::{LanguageSource as S, Locale, resolve};

const SUPPORTED: &[&str] = &["de", "en"];

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect()
}

fn loc(s: &str) -> Locale {
    Locale::parse(s).unwrap()
}

fn run(program: Option<&str>, shared: Option<&str>, pairs: &[(&str, &str)]) -> (String, S) {
    let (p, s) = (program.map(loc), shared.map(loc));
    let c = resolve(p.as_ref(), s.as_ref(), &env(pairs), SUPPORTED);
    (c.locale.to_string(), c.source)
}

const ALL: &[(&str, &str)] = &[
    ("FS_LANG", "fr"),
    ("LC_ALL", "it"),
    ("LC_MESSAGES", "es"),
    ("LANGUAGE", "pl:de"),
    ("LANG", "nl_NL.UTF-8"),
];

#[test]
fn each_link_beats_the_ones_after_it() {
    assert_eq!(run(Some("ja"), Some("ko"), ALL), ("ja".into(), S::Program));
    assert_eq!(run(None, Some("ko"), ALL), ("ko".into(), S::Shared));
    assert_eq!(run(None, None, ALL), ("fr".into(), S::Environment("FS_LANG")));
    assert_eq!(run(None, None, &ALL[1..]), ("it".into(), S::Environment("LC_ALL")));
    assert_eq!(run(None, None, &ALL[2..]), ("es".into(), S::Environment("LC_MESSAGES")));
    assert_eq!(run(None, None, &ALL[3..]), ("de".into(), S::Environment("LANGUAGE")));
    assert_eq!(run(None, None, &ALL[4..]), ("nl-NL".into(), S::Environment("LANG")));
    assert_eq!(run(None, None, &[]), ("en".into(), S::Fallback));
}

#[test]
fn c_posix_empty_and_broken_values_are_skipped() {
    let pairs = [("FS_LANG", ""), ("LC_ALL", "C"), ("LC_MESSAGES", "POSIX"), ("LANGUAGE", ""), ("LANG", "de_AT.UTF-8")];
    assert_eq!(run(None, None, &pairs), ("de-AT".into(), S::Environment("LANG")));
    let all_none = [("FS_LANG", "x"), ("LC_ALL", "C.UTF-8"), ("LANG", "POSIX")];
    assert_eq!(run(None, None, &all_none), ("en".into(), S::Fallback));
}

#[test]
fn language_takes_the_first_entry_with_a_supported_language() {
    let l = |v| run(None, None, &[("LANGUAGE", v), ("LANG", "en_US.UTF-8")]);
    assert_eq!(l("pl:de_DE:en"), ("de-DE".into(), S::Environment("LANGUAGE")));
    assert_eq!(l("pl:en:de"), ("en".into(), S::Environment("LANGUAGE")));
    // Counter-probe: no supported entry, so LANG answers.
    assert_eq!(l("pl:fr"), ("en-US".into(), S::Environment("LANG")));
    assert_eq!(l("C:de"), ("de".into(), S::Environment("LANGUAGE")));
}

#[test]
fn the_real_environment_can_be_read_but_a_test_never_needs_it() {
    // ProcessEnv exists for the programs; here only that it answers for a name that is surely not set.
    use fs_components_words::{Env, ProcessEnv};
    assert_eq!(ProcessEnv.var("FS_NO_SUCH_VARIABLE_FOR_THE_TEST"), None);
}
