//! Shared helpers for the tests that refuse specs.

use fs_components_charts::spec::{ErrorKind, parse};

pub const XY: &str =
    r#""x":{"field":"a","type":"nominal"},"y":{"field":"b","type":"quantitative"}"#;

pub fn bar(extra_top: &str, enc: &str) -> String {
    format!(r#"{{"data":{{"values":[]}},"mark":"bar",{extra_top}"encoding":{{{enc}}}}}"#)
}

pub fn refused(text: &str, kind: ErrorKind, path: &str, word: &str) {
    let e = parse(text).expect_err(text);
    assert_eq!(e.kind, kind, "{text}: {e}");
    assert_eq!(e.path, path, "{text}: {e}");
    assert!(
        e.message.contains(word),
        "{text}: message {:?} lacks {word:?}",
        e.message
    );
    assert!(e.to_string().contains(path), "{e}");
}
