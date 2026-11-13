//! `Mf2Texts`: the messages of one locale and its fallback chain, as `Words`.
//!
//! Pattern: Adapter + Chain of Responsibility — the first language of the chain (`de-AT`, `de`, then English) that has the
//! message and can fill it wins; nothing found is `Said::missing(id)`. A message is compiled with the `mf2` library the
//! first time it is asked for and kept (a program shows a few hundred, a window asks for tens). Numbers and plural forms
//! come from the CLDR data inside `mf2` (ICU4X is not needed for them; dates are shown neutrally, see `date.rs`).
//! The catalogs are embedded (`Source`, made with `include_str!`); loading a language package later (ADR 0036) only has to
//! make a `Source` from a file: `new` takes sources, it does not know where they come from.

use std::collections::HashMap;
use std::sync::Mutex;

use fs_components_words::{Args, Locale, Said, TextId, Value, Words};
use mf2::{Arg, BidiStrategy, Compiled, FormatContext, Formatter, Function, Registry};

use crate::source::entries;
use crate::{Source, date};

static FUNCTIONS: [(&str, &dyn Function); 5] = [
    ("string", &mf2::functions::STRING),
    ("number", &mf2::fn_number::NUMBER),
    ("integer", &mf2::fn_number::INTEGER),
    ("percent", &mf2::fn_number::PERCENT),
    ("offset", &mf2::fn_number::OFFSET),
];
static REGISTRY: Registry = Registry::new(&FUNCTIONS);
/// No isolation marks (U+2066 to U+2069) around inserted values: they are for mixed left-to-right and right-to-left text, and
/// only show as boxes in a font that lacks them. Right to left is not supported yet (DESIGN); a language that needs it
/// switches this on.
static CONTEXT: FormatContext = {
    let mut cx = FormatContext::new(&mf2::host_std::HOST);
    cx.bidi = BidiStrategy::None;
    cx
};

struct Language {
    tag: String,
    messages: HashMap<String, String>,
    compiled: Mutex<HashMap<String, Option<Compiled>>>,
}

/// The texts of one locale.
pub struct Mf2Texts {
    chain: Vec<Language>,
}

impl Language {
    fn say(&self, id: &str, args: &Args) -> Option<String> {
        let source = self.messages.get(id)?;
        let mut cache = self
            .compiled
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let compiled = cache
            .entry(id.to_owned())
            .or_insert_with(|| mf2::compile_str(source, &self.tag).ok())
            .as_ref()?;
        let names = compiled.manifest.slots.first()?;
        let owned: Vec<String> = names.iter().map(|n| text_of(args.get(n))).collect();
        let values: Vec<Arg<'_>> = names
            .iter()
            .zip(&owned)
            .map(|(n, text)| arg_of(args.get(n), text))
            .collect();
        let (mut out, mut errors) = (String::new(), Vec::new());
        Formatter::new(&compiled.catalog, &REGISTRY, &CONTEXT).write(
            Compiled::ID,
            &values,
            &mut out,
            &mut errors,
        );
        (errors.is_empty() && !out.trim().is_empty()).then_some(out)
    }
}

/// The text form of a date or time (only those two need one).
fn text_of(value: Option<&Value>) -> String {
    match value {
        Some(Value::Date(d)) => date::day(*d),
        Some(Value::Time(t)) => date::clock(*t),
        _ => String::new(),
    }
}

fn arg_of<'a>(value: Option<&'a Value>, rendered: &'a str) -> Arg<'a> {
    match value {
        Some(Value::Text(t)) => Arg::Str(t),
        Some(Value::Count(n)) => i64::try_from(*n).map_or(Arg::Unset, Arg::Int),
        Some(Value::Number(n)) => Arg::Float(*n),
        Some(Value::Date(_) | Value::Time(_)) => Arg::Str(rendered),
        None => Arg::Unset,
    }
}

impl Mf2Texts {
    /// The texts of `locale` from `sources`: for each tag of the chain (`de-AT`, `de`) the source of that language if there
    /// is one, then English. A malformed message is left out (it comes in English); `check_pairs` reports it.
    pub fn new(sources: &[Source], locale: &Locale) -> Self {
        let mut wanted = locale.chain();
        wanted.push(Locale::english());
        let mut chain: Vec<Language> = Vec::new();
        for tag in wanted {
            if chain.iter().any(|l| l.tag == tag.as_str()) {
                continue;
            }
            if let Some(s) = sources.iter().find(|s| s.language == tag.as_str()) {
                let messages = entries(s.text)
                    .0
                    .into_iter()
                    .map(|e| (e.id, e.message))
                    .collect();
                chain.push(Language {
                    tag: tag.to_string(),
                    messages,
                    compiled: Mutex::default(),
                });
            }
        }
        Self { chain }
    }

    /// Shorthand: the sentence for `id` without values.
    pub fn plain(&self, id: &str) -> String {
        self.say(&TextId::new(id), &Args::new()).text().to_owned()
    }
}

impl Words for Mf2Texts {
    fn say(&self, id: &TextId, args: &Args) -> Said {
        self.chain
            .iter()
            .find_map(|l| l.say(id.as_str(), args))
            .map_or_else(|| Said::missing(id), |text| Said::found(id, text))
    }
}
