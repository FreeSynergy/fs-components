//! The read-only test access to the text of the fields (feature `test-access`).
//!
//! Tests: the registry, the secret fields that are never in it, the writer of the file.
#![cfg(feature = "test-access")]

use blinc_layout::widgets::text_input::InputType;
use fs_components_primitives::FieldData;
use fs_components_primitives::test_access::{
    enable, field_texts, fields_json, registry_len, start_from_env,
};

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
    enable();
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
    enable();
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
    enable();
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

#[cfg(unix)]
#[test]
fn an_existing_file_or_a_link_of_the_name_is_replaced_never_written_through() {
    use fs_components_primitives::test_access::write_aside;
    use std::os::unix::fs::{PermissionsExt, symlink};
    let dir = std::env::temp_dir().join(format!("fs-ta-link-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let victim = dir.join("victim.txt");
    std::fs::write(&victim, "precious").unwrap();
    // the answer file is a link to the victim, and the side file is a link to the victim too
    let out = dir.join("out.json");
    symlink(&victim, &out).unwrap();
    symlink(&victim, dir.join("out.tmp")).unwrap();
    write_aside(&out, "{\"fields\":[]}\n").unwrap();
    assert_eq!(
        std::fs::read_to_string(&victim).unwrap(),
        "precious",
        "written through a link"
    );
    assert!(
        !std::fs::symlink_metadata(&out)
            .unwrap()
            .file_type()
            .is_symlink(),
        "the link must be replaced"
    );
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "{\"fields\":[]}\n");
    assert_eq!(
        std::fs::metadata(&out).unwrap().permissions().mode() & 0o777,
        0o600
    );
    // a second write over an existing plain file
    write_aside(&out, "second").unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), "second");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_registry_does_not_grow_without_bound_while_fields_come_and_go() {
    enable();
    for _ in 0..5_000 {
        let keeper = FieldData::new();
        let _field = keeper.text_input();
    }
    // 5000 short-lived fields: only the dead entries of the last stretch are left, never all of them
    assert!(
        registry_len() < 1_000,
        "registry grew to {}",
        registry_len()
    );
}
