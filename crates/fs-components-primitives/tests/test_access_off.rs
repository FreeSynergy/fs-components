//! Without the switch (`FS_TEST_ACCESS_FILE` / `enable()`), nothing is noted: the registry cannot grow in a program
//! that has the feature but is not asked. Its own test binary, because the switch is global to the process.
//!
//! Tests: nothing noted while off.
#![cfg(feature = "test-access")]

use fs_components_primitives::FieldData;
use fs_components_primitives::test_access::{enabled, field_texts, registry_len};

#[test]
fn nothing_is_noted_while_the_switch_is_off() {
    assert!(!enabled());
    let keeper = FieldData::new();
    let field = keeper.text_input();
    field.lock().unwrap().value = "typed".into();
    for _ in 0..1_000 {
        let _ = keeper.text_input();
    }
    assert_eq!(registry_len(), 0);
    assert!(field_texts().is_empty());
}
