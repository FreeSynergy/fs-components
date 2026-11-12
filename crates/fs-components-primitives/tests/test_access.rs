//! The read-only test access to the text of the fields (feature `test-access`).
//!
//! Tests: the registry, the secret fields that are never in it, the writer of the file.
#![cfg(feature = "test-access")]

use blinc_layout::widgets::text_input::InputType;
use fs_components_primitives::FieldData;
use fs_components_primitives::test_access::{field_texts, fields_json, start_from_env};

fn set(data: &blinc_layout::widgets::text_input::SharedTextInputData, text: &str) {
    data.lock().unwrap().value = text.to_string();
}

fn texts_of(keeper_ids: &[u64]) -> Vec<String> {
    field_texts()
        .into_iter()
        .filter(|f| keeper_ids.contains(&f.id))
        .map(|f| f.text)
        .collect()
}

#[test]
fn the_text_of_a_field_is_read_exactly_and_a_secret_field_gives_nothing() {
    let keeper = FieldData::new();
    let plain = keeper.text_input();
    let named = keeper.text_input_with_placeholder("Name");
    let secret = keeper.text_input();
    let masked = keeper.text_input();
    set(&plain, "a\u{c4}\u{e4}@\u{20ac}{}[]\\|~\u{b5}\"x");
    set(&named, "Name \u{df}");
    set(&secret, "hunter2");
    secret.lock().unwrap().input_type = InputType::Password;
    set(&masked, "also secret");
    masked.lock().unwrap().masked = true;

    let all: Vec<String> = field_texts().into_iter().map(|f| f.text).collect();
    assert!(all.contains(&"a\u{c4}\u{e4}@\u{20ac}{}[]\\|~\u{b5}\"x".to_string()));
    assert!(all.contains(&"Name \u{df}".to_string()));
    assert!(
        !all.iter()
            .any(|t| t.contains("hunter2") || t.contains("also secret"))
    );
    let json = fields_json();
    assert!(
        !json.contains("hunter2") && !json.contains("also secret"),
        "{json}"
    );
    assert!(
        json.contains("\\\\|~") && json.contains("\\\"x"),
        "escaped: {json}"
    );

    // a field that is gone is not in the answer any more
    let id_before = field_texts().len();
    drop((plain, named, secret, masked));
    drop(keeper);
    assert!(field_texts().len() < id_before);
    let _ = texts_of(&[]);
}

#[test]
fn a_wrong_text_is_not_the_text_that_was_typed() {
    // The counter test of the typing test in miniature: what is read is what is in the field, not what was meant.
    let keeper = FieldData::new();
    let field = keeper.text_input();
    set(&field, ")");
    let read: Vec<String> = field_texts().into_iter().map(|f| f.text).collect();
    assert!(read.contains(&")".to_string()));
    assert!(
        !read.contains(&"}".to_string()),
        "a `)` in the field is never reported as `}}`"
    );
}

#[test]
fn the_writer_writes_the_file_whole_when_the_environment_names_one() {
    let path = std::env::temp_dir().join(format!("fs-test-access-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    // SAFETY: this test is the only one of this binary that reads or sets the variable, and it runs before the
    // thread of the writer exists.
    unsafe { std::env::set_var("FS_TEST_ACCESS_FILE", &path) };
    let keeper = FieldData::new();
    let field = keeper.text_input();
    set(&field, "typed");
    start_from_env();
    let mut seen = String::new();
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(40));
        if let Ok(text) = std::fs::read_to_string(&path)
            && text.contains("typed")
        {
            seen = text;
            break;
        }
    }
    assert!(seen.contains("\"text\":\"typed\""), "{seen}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "only the owner may read it");
    }
    let _ = std::fs::remove_file(&path);
}
