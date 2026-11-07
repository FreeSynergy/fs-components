//! Template syntax: parse a sentence template into parts and fill it with `Args`.
//!
//! Pattern: Interpreter — a small language (`{name}`, `{n, plural, one {# x} other {# y}}`,
//! escapes `\{ \} \# \\ \<space>`) evaluated over its parsed tree. Values are inserted as data
//! and never parsed again. Nesting is limited to [`MAX_DEPTH`] so hostile input cannot exhaust
//! the stack.

use crate::format::show;
use crate::{Args, PluralCategory, PluralRules, Value};

/// Deepest nesting of plural branches a template may have.
pub(crate) const MAX_DEPTH: usize = 32;

/// One piece of a parsed template.
#[derive(Debug)]
pub(crate) enum Part {
    Lit(String),
    Arg(String),
    Hash,
    Plural {
        name: String,
        branches: Vec<(PluralCategory, Vec<Part>)>,
    },
}

/// Parse a template; `None` if it is malformed, blank or nested too deeply.
pub(crate) fn parse(src: &str) -> Option<Vec<Part>> {
    if src.trim().is_empty() {
        return None;
    }
    let chars: Vec<char> = src.chars().collect();
    let parts = Parser { c: &chars, i: 0 }.parts(0)?;
    may_show(&parts).then_some(parts)
}

/// Can this template print something that is not blank? One that never can is faulty.
fn may_show(parts: &[Part]) -> bool {
    parts.iter().any(|p| match p {
        Part::Lit(s) => !s.trim().is_empty(),
        Part::Arg(_) | Part::Hash => true,
        Part::Plural { branches, .. } => branches.iter().any(|(_, body)| may_show(body)),
    })
}

/// Does every plural in the template have an `other` branch (as ICU asks)?
pub(crate) fn all_have_other(parts: &[Part]) -> bool {
    parts.iter().all(|p| match p {
        Part::Plural { branches, .. } => {
            branches.iter().any(|(c, _)| *c == PluralCategory::Other)
                && branches.iter().all(|(_, body)| all_have_other(body))
        }
        _ => true,
    })
}

/// Fill a parsed template. `None` if a value is missing or of the wrong kind, or no branch fits.
pub(crate) fn render(
    parts: &[Part],
    args: &Args,
    rules: &dyn PluralRules,
    hash: Option<u64>,
    out: &mut String,
) -> Option<()> {
    for part in parts {
        match part {
            Part::Lit(s) => out.push_str(s),
            Part::Arg(name) => out.push_str(&show(args.get(name)?)),
            Part::Hash => out.push_str(&hash?.to_string()),
            Part::Plural { name, branches } => {
                let Value::Count(n) = args.get(name)? else {
                    return None;
                };
                let cat = rules.category(*n);
                let pick = |c| branches.iter().find(|(b, _)| *b == c);
                let (_, body) = pick(cat).or_else(|| pick(PluralCategory::Other))?;
                render(body, args, rules, Some(*n), out)?;
            }
        }
    }
    Some(())
}

struct Parser<'a> {
    c: &'a [char],
    i: usize,
}

fn flush(lit: &mut String, out: &mut Vec<Part>) {
    if !lit.is_empty() {
        out.push(Part::Lit(std::mem::take(lit)));
    }
}

impl Parser<'_> {
    fn next(&mut self) -> Option<char> {
        let ch = self.c.get(self.i).copied();
        self.i += 1;
        ch
    }

    /// Parts up to the end (depth 0) or up to the closing `}` of a branch (depth >= 1).
    fn parts(&mut self, depth: usize) -> Option<Vec<Part>> {
        if depth > MAX_DEPTH {
            return None;
        }
        let (mut out, mut lit) = (Vec::new(), String::new());
        loop {
            let Some(ch) = self.next() else {
                flush(&mut lit, &mut out);
                return (depth == 0).then_some(out);
            };
            match ch {
                '\\' => match self.next()? {
                    e @ ('{' | '}' | '#' | '\\' | ' ') => lit.push(e),
                    _ => return None,
                },
                '{' => {
                    flush(&mut lit, &mut out);
                    out.push(self.placeholder(depth)?);
                }
                '}' if depth > 0 => {
                    flush(&mut lit, &mut out);
                    return Some(out);
                }
                '}' => return None,
                '#' if depth > 0 => {
                    flush(&mut lit, &mut out);
                    out.push(Part::Hash);
                }
                c => lit.push(c),
            }
        }
    }

    /// Read up to (and past) one of `stops`; returns the trimmed text and the stop.
    fn until(&mut self, stops: &[char]) -> Option<(String, char)> {
        let mut s = String::new();
        loop {
            let ch = self.next()?;
            if stops.contains(&ch) {
                return Some((s.trim().to_owned(), ch));
            }
            s.push(ch);
        }
    }

    /// After `{`: `name}` or `name, plural, sel {..} sel {..}}`.
    fn placeholder(&mut self, depth: usize) -> Option<Part> {
        let (name, stop) = self.until(&[',', '}'])?;
        if name.is_empty() || name.contains(|c: char| c.is_whitespace() || "{\\#".contains(c)) {
            return None;
        }
        if stop == '}' {
            return Some(Part::Arg(name));
        }
        let (kind, _) = self.until(&[','])?;
        if kind != "plural" {
            return None;
        }
        let mut branches: Vec<(PluralCategory, Vec<Part>)> = Vec::new();
        loop {
            let (sel, stop) = self.until(&['{', '}'])?;
            if stop == '}' {
                return (sel.is_empty() && !branches.is_empty())
                    .then_some(Part::Plural { name, branches });
            }
            let cat = PluralCategory::from_name(&sel)?;
            if branches.iter().any(|(c, _)| *c == cat) {
                return None;
            }
            branches.push((cat, self.parts(depth + 1)?));
        }
    }
}
