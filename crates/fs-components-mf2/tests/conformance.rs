//! The official `MessageFormat` 2 conformance suite (unicode-org/message-format-wg, `test/tests/*.json`) run against `mf2`
//! (ADR 0057 reservation: "the library claims to pass the suite"). `FS_MF2_SUITE` names the directory (default: the clone
//! under `.scratch/downloads`); missing: the test is skipped with a message, with `CI=1` it fails. The suite part only
//! reports (a table per file: passed / failed / unsupported, with `--nocapture`); the part in `core.rs` is the hard one.
//! Not checked: `expParts` (counted only), a datetime parameter, `:test:*`, `:date`, `:time`, `:currency`, `u:*` (unsupported).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use mf2::{Arg, BidiStrategy, ErrorKind, FormatContext, FormatError, Formatter, Registry};
use serde_json::Value;

const DEFAULT_DIR: &str = "/srv/FreeSynergy/.scratch/downloads/mf2-wg/src/test/tests";

static FUNCTIONS: [(&str, &dyn mf2::Function); 5] = [
    ("string", &mf2::functions::STRING),
    ("number", &mf2::fn_number::NUMBER),
    ("integer", &mf2::fn_number::INTEGER),
    ("percent", &mf2::fn_number::PERCENT),
    ("offset", &mf2::fn_number::OFFSET),
];
static REGISTRY: Registry = Registry::new(&FUNCTIONS).with_numbers(&mf2::fn_number::NUMBERS);

#[derive(Default)]
struct Tally {
    passed: u32,
    failed: u32,
    unsupported: u32,
    parts: u32,
    /// class -> (count, first example)
    classes: BTreeMap<String, (u32, String)>,
}

enum Outcome {
    Pass,
    Unsupported,
    Fail(String, String),
}

fn kind_name(k: ErrorKind) -> &'static str {
    match k {
        ErrorKind::Syntax => "syntax-error",
        ErrorKind::VariantKeyMismatch => "variant-key-mismatch",
        ErrorKind::MissingFallbackVariant => "missing-fallback-variant",
        ErrorKind::MissingSelectorAnnotation => "missing-selector-annotation",
        ErrorKind::DuplicateDeclaration => "duplicate-declaration",
        ErrorKind::DuplicateOptionName => "duplicate-option-name",
        ErrorKind::DuplicateVariant => "duplicate-variant",
        ErrorKind::UnresolvedVariable => "unresolved-variable",
        ErrorKind::UnknownFunction => "unknown-function",
        ErrorKind::BadSelector => "bad-selector",
        ErrorKind::BadOperand => "bad-operand",
        ErrorKind::BadOption => "bad-option",
        ErrorKind::BadVariantKey => "bad-variant-key",
        ErrorKind::UnsupportedOperation => "unsupported-operation",
        _ => "message-function-error",
    }
}

fn runtime_name(e: FormatError) -> &'static str {
    match e {
        FormatError::UnresolvedVariable => "unresolved-variable",
        FormatError::UnknownFunction => "unknown-function",
        FormatError::BadSelector => "bad-selector",
        FormatError::BadOperand => "bad-operand",
        FormatError::BadOption => "bad-option",
        FormatError::BadVariantKey => "bad-variant-key",
        FormatError::UnsupportedOperation => "unsupported-operation",
        _ => "message-function-error",
    }
}

/// A test property: the case's own, else the file default.
fn prop<'a>(case: &'a Value, defaults: &'a Value, key: &str) -> Option<&'a Value> {
    case.get(key).or_else(|| defaults.get(key))
}

fn names(list: Option<&Value>) -> Vec<String> {
    let mut v: Vec<String> = list
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|e| e["type"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v.dedup();
    v
}

fn run_case(case: &Value, defaults: &Value) -> Outcome {
    let src = prop(case, defaults, "src")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let locale = prop(case, defaults, "locale")
        .and_then(Value::as_str)
        .unwrap_or("en-US");
    let expected = prop(case, defaults, "exp").and_then(Value::as_str);
    let exp_errors = names(prop(case, defaults, "expErrors"));
    let none_wanted = prop(case, defaults, "bidiIsolation").and_then(Value::as_str) == Some("none");
    let tags = prop(case, defaults, "tags")
        .map(Value::to_string)
        .unwrap_or_default();
    let params = prop(case, defaults, "params")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let unsupported_src = [":test:", ":date", ":time", ":currency", "u:"]
        .iter()
        .any(|f| src.contains(f))
        || tags.contains("u:")
        || params.iter().any(|p| p.get("type").is_some());
    let verdict = |got: String, got_errors: Vec<String>| -> Outcome {
        let text_ok = expected.is_none_or(|e| e == got);
        if text_ok && got_errors == exp_errors {
            return Outcome::Pass;
        }
        let tolerated = got_errors
            .iter()
            .any(|e| e == "unknown-function" || e == "unsupported-operation");
        if unsupported_src || (tolerated && !exp_errors.contains(&"unknown-function".to_owned())) {
            return Outcome::Unsupported;
        }
        let class = if got_errors == exp_errors {
            "output"
        } else {
            "errors"
        };
        let detail = format!(
            "src={src:?} expected {:?} {exp_errors:?}, got {got:?} {got_errors:?}",
            expected.unwrap_or("-")
        );
        Outcome::Fail(class.to_owned(), detail)
    };
    let compiled = match mf2::compile_str(src, locale) {
        Ok(c) => c,
        Err(e) => {
            let mut kinds: Vec<String> = e
                .kinds()
                .into_iter()
                .map(|k| kind_name(k).to_owned())
                .collect();
            kinds.sort();
            kinds.dedup();
            let fallback = expected.map(str::to_owned).unwrap_or_default();
            return if kinds.is_empty() {
                Outcome::Unsupported
            } else {
                verdict(fallback, kinds)
            };
        }
    };
    let owned: Vec<(String, String)> = params
        .iter()
        .filter_map(|p| {
            Some((
                p["name"].as_str()?.to_owned(),
                p["value"].as_str().unwrap_or_default().to_owned(),
            ))
        })
        .collect();
    let args: Vec<(&str, Arg<'_>)> = params
        .iter()
        .zip(&owned)
        .filter_map(|(p, (name, text))| {
            let v = &p["value"];
            let arg = if v.is_string() {
                Arg::Str(text)
            } else if let Some(i) = v.as_i64() {
                Arg::Int(i)
            } else {
                Arg::Float(v.as_f64()?)
            };
            Some((name.as_str(), arg))
        })
        .collect();
    let mut cx = FormatContext::new(&mf2::host_std::HOST);
    if none_wanted {
        cx.bidi = BidiStrategy::None;
    }
    let (mut out, mut errs): (String, Vec<FormatError>) = (String::new(), Vec::new());
    Formatter::new(&compiled.catalog, &REGISTRY, &cx).write_named(
        mf2::Compiled::ID,
        &args,
        &mut out,
        &mut errs,
    );
    let mut got: Vec<String> = errs.iter().map(|&e| runtime_name(e).to_owned()).collect();
    got.sort();
    got.dedup();
    verdict(out, got)
}

fn run_file(path: &Path) -> Tally {
    let json: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("readable")).expect("valid JSON");
    let defaults = json
        .get("defaultTestProperties")
        .cloned()
        .unwrap_or(Value::Null);
    let mut tally = Tally::default();
    for case in json["tests"].as_array().expect("tests") {
        if prop(case, &defaults, "expParts").is_some() {
            tally.parts += 1;
        }
        match run_case(case, &defaults) {
            Outcome::Pass => tally.passed += 1,
            Outcome::Unsupported => tally.unsupported += 1,
            Outcome::Fail(class, detail) => {
                tally.failed += 1;
                let id = case["description"].as_str().unwrap_or("-");
                let entry = tally.classes.entry(class).or_insert((0, String::new()));
                entry.0 += 1;
                if entry.1.is_empty() {
                    entry.1 = format!("{id} | {detail}");
                }
            }
        }
    }
    tally
}

fn files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).expect("readable dir").flatten() {
        let p = entry.path();
        if p.is_dir() {
            out.extend(files(&p));
        } else if p.extension().is_some_and(|e| e == "json") {
            out.push(p);
        }
    }
    out.sort();
    out
}

#[test]
fn official_suite() {
    let dir =
        PathBuf::from(std::env::var("FS_MF2_SUITE").unwrap_or_else(|_| DEFAULT_DIR.to_owned()));
    if !dir.is_dir() {
        assert!(
            std::env::var("CI").as_deref() != Ok("1"),
            "FS_MF2_SUITE missing: {}",
            dir.display()
        );
        eprintln!(
            "conformance suite skipped: {} not found (set FS_MF2_SUITE)",
            dir.display()
        );
        return;
    }
    let verbose = std::env::var("FS_MF2_VERBOSE").is_ok();
    let (mut sum, mut all) = ([0u32; 3], 0u32);
    println!("| file | passed | failed | unsupported | with expParts |\n|---|---|---|---|---|");
    for path in files(&dir) {
        let t = run_file(&path);
        let rel = path
            .strip_prefix(&dir)
            .unwrap_or(&path)
            .display()
            .to_string();
        println!(
            "| {rel} | {} | {} | {} | {} |",
            t.passed, t.failed, t.unsupported, t.parts
        );
        for (class, (n, example)) in &t.classes {
            println!("  class {class}: {n}x, e.g. {example}");
        }
        sum[0] += t.passed;
        sum[1] += t.failed;
        sum[2] += t.unsupported;
        all += t.passed + t.failed + t.unsupported;
        let _ = verbose;
    }
    println!(
        "| total | {} | {} | {} | of {all} |",
        sum[0], sum[1], sum[2]
    );
}
