//! Which language a program speaks: the chain from the person's choice down to English.
//!
//! Pattern: Chain of Responsibility (each link may decline) + Strategy for the environment (`Env`), so a test never reads
//! the real one. Order: the program's own override, the setting shared by all FreeSynergy programs, `FS_LANG`, `LC_ALL`,
//! `LC_MESSAGES`, `LANGUAGE` (GNU list: the first entry with a supported language), `LANG`, then English. `C`, `POSIX` and
//! an empty or malformed value are no language and the chain goes on. The desktop portal is not asked (open, DESIGN).

use std::collections::HashMap;

use crate::Locale;

/// Where the chosen language came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageSource {
    /// The program's own override.
    Program,
    /// The setting shared by all programs.
    Shared,
    /// This environment variable.
    Environment(&'static str),
    /// Nothing else said anything: English.
    Fallback,
}

/// The language and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chosen {
    /// The language to speak.
    pub locale: Locale,
    /// Which link of the chain chose it.
    pub source: LanguageSource,
}

/// Reads environment variables.
pub trait Env {
    /// The value of `name`, if set.
    fn var(&self, name: &str) -> Option<String>;
}

/// The environment of this process.
#[derive(Debug, Clone, Copy, Default)]
pub struct ProcessEnv;

impl Env for ProcessEnv {
    fn var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
}

/// A fixed set of variables, for tests.
impl<S: std::hash::BuildHasher> Env for HashMap<String, String, S> {
    fn var(&self, name: &str) -> Option<String> {
        self.get(name).cloned()
    }
}

/// The variables that name one language, in the order they are asked.
const VARIABLES: [&str; 3] = ["FS_LANG", "LC_ALL", "LC_MESSAGES"];

/// Choose the language. `program` and `shared` are the two settings (already read); `supported` are the languages the
/// program has texts for (primary subtags), used to pick from a `LANGUAGE` list.
pub fn resolve(
    program: Option<&Locale>,
    shared: Option<&Locale>,
    env: &dyn Env,
    supported: &[&str],
) -> Chosen {
    let chosen = |locale: &Locale, source| Chosen {
        locale: locale.clone(),
        source,
    };
    if let Some(l) = program {
        return chosen(l, LanguageSource::Program);
    }
    if let Some(l) = shared {
        return chosen(l, LanguageSource::Shared);
    }
    let posix = |name: &'static str| {
        let locale = env.var(name).and_then(|v| Locale::from_posix(&v))?;
        Some(Chosen {
            locale,
            source: LanguageSource::Environment(name),
        })
    };
    for name in VARIABLES {
        if let Some(found) = posix(name) {
            return found;
        }
    }
    let listed = env.var("LANGUAGE").and_then(|list| {
        list.split(':')
            .filter_map(Locale::from_posix)
            .find(|l| supported.contains(&l.primary()))
    });
    if let Some(locale) = listed {
        return Chosen {
            locale,
            source: LanguageSource::Environment("LANGUAGE"),
        };
    }
    posix("LANG").unwrap_or_else(|| chosen(&Locale::english(), LanguageSource::Fallback))
}
