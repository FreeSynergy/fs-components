//! The hard part: the cases FreeSynergy uses (plain text, `{$var}`, `.match` with `:plural` / `:number` / `:string`,
//! escapes, error fallback) must pass 100 %. Own cases, not from the suite; the suite is `conformance.rs`.

// One table of cases in one test: long on purpose.
#![allow(clippy::too_many_lines)]

use mf2::{Arg, BidiStrategy, FormatContext, FormatError, Formatter, Registry};

static FUNCTIONS: [(&str, &dyn mf2::Function); 3] = [
    ("string", &mf2::functions::STRING),
    ("number", &mf2::fn_number::NUMBER),
    ("integer", &mf2::fn_number::INTEGER),
];
static REGISTRY: Registry = Registry::new(&FUNCTIONS).with_numbers(&mf2::fn_number::NUMBERS);

fn run(src: &str, locale: &str, args: &[(&str, Arg<'_>)]) -> (String, Vec<FormatError>) {
    let compiled = mf2::compile_str(src, locale).expect("compiles");
    let mut cx = FormatContext::new(&mf2::host_std::HOST);
    cx.bidi = BidiStrategy::None;
    let (mut out, mut errs) = (String::new(), Vec::new());
    Formatter::new(&compiled.catalog, &REGISTRY, &cx).write_named(
        mf2::Compiled::ID,
        args,
        &mut out,
        &mut errs,
    );
    (out, errs)
}

type Case<'a> = (&'a str, &'a str, Vec<(&'a str, Arg<'a>)>, &'a str, bool);

const FILES: &str = ".input {$n :number} .match $n one {{{$n} file}} * {{{$n} files}}";

#[test]
fn core_cases() {
    // (source, locale, args, expected text, error expected)
    let cases: Vec<Case<'_>> = vec![
        ("plain text", "en", vec![], "plain text", false),
        ("", "en", vec![], "", false),
        (
            "Hello, {$name}!",
            "en",
            vec![("name", Arg::Str("Kal"))],
            "Hello, Kal!",
            false,
        ),
        (
            "Hallo, {$name}!",
            "de",
            vec![("name", Arg::Str("Kal"))],
            "Hallo, Kal!",
            false,
        ),
        (FILES, "en", vec![("n", Arg::Int(1))], "1 file", false),
        (FILES, "en", vec![("n", Arg::Int(0))], "0 files", false),
        (FILES, "en", vec![("n", Arg::Int(5))], "5 files", false),
        (FILES, "de", vec![("n", Arg::Int(1))], "1 file", false),
        (
            ".input {$n :number} .match $n one {{eins}} few {{wenige}} many {{viele}} * {{andere}}",
            "pl",
            vec![("n", Arg::Int(3))],
            "wenige",
            false,
        ),
        (
            ".input {$n :number select=ordinal} .match $n one {{{$n}st}} two {{{$n}nd}} few {{{$n}rd}} * {{{$n}th}}",
            "en",
            vec![("n", Arg::Int(22))],
            "22nd",
            false,
        ),
        (
            ".input {$n :integer} .match $n 1 {{one}} * {{other}}",
            "en",
            vec![("n", Arg::Int(1))],
            "one",
            false,
        ),
        (
            "{$p :number minimumFractionDigits=2} EUR",
            "en",
            vec![("p", Arg::Float(3.5))],
            "3.50 EUR",
            false,
        ),
        (
            "{$p :number}",
            "en",
            vec![("p", Arg::Int(1234))],
            "1,234",
            false,
        ),
        (
            "{$p :number}",
            "de",
            vec![("p", Arg::Int(1234))],
            "1.234",
            false,
        ),
        ("{$p}", "de", vec![("p", Arg::Float(1.5))], "1,5", false),
        (
            ".input {$s :string} .match $s yes {{Y}} * {{N}}",
            "en",
            vec![("s", Arg::Str("yes"))],
            "Y",
            false,
        ),
        (
            ".input {$s :string} .match $s yes {{Y}} * {{N}}",
            "en",
            vec![("s", Arg::Str("no"))],
            "N",
            false,
        ),
        (
            ".local $x = {$a} {{{$x}}}",
            "en",
            vec![("a", Arg::Str("v"))],
            "v",
            false,
        ),
        ("\\{ \\} \\\\ \\|", "en", vec![], "{ } \\ |", false),
        (
            "{|literal with spaces|} {42}",
            "en",
            vec![],
            "literal with spaces 42",
            false,
        ),
        ("{$missing}", "en", vec![], "{$missing}", true),
        (
            "{$n :nosuchfunction}",
            "en",
            vec![("n", Arg::Int(1))],
            "{$n}",
            true,
        ),
        (
            "{$n :number}",
            "en",
            vec![("n", Arg::Str("abc"))],
            "{$n}",
            true,
        ),
    ];
    let mut failures = Vec::new();
    for (src, locale, args, want, error) in &cases {
        let (out, errs) = run(src, locale, args);
        if out != *want || errs.is_empty() == *error {
            failures.push(format!(
                "{src:?} [{locale}]: want {want:?} (error {error}), got {out:?} ({} errors)",
                errs.len()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "core cases failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn core_syntax_errors_refuse_to_compile() {
    for src in [
        "{",
        "}",
        "Hello {$",
        ".match $x * {{a}}",
        ".input {$n :number} .match $n one {{a}}",
    ] {
        assert!(
            mf2::compile_str(src, "en").is_err(),
            "{src:?} should be refused"
        );
    }
}
