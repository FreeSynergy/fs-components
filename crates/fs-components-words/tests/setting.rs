//! Tests of the language setting: where the files are, reading, and the atomic write. Temp folders only.

#![allow(clippy::unnecessary_wraps)]

use std::ffi::OsString;
use std::path::PathBuf;

use fs_components_words::{LanguageSetting, Locale};

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fs-words-setting-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn os(s: &str) -> Option<OsString> {
    Some(s.into())
}

#[test]
fn the_files_follow_the_xdg_rule() {
    let shared = |x, h| LanguageSetting::shared(x, h).map(|s| s.file().to_path_buf());
    assert_eq!(
        shared(os("/c"), os("/h")),
        Some("/c/freesynergy/language".into())
    );
    assert_eq!(
        shared(None, os("/h")),
        Some("/h/.config/freesynergy/language".into())
    );
    // A relative $XDG_CONFIG_HOME is ignored, a relative HOME is no home.
    assert_eq!(
        shared(os("rel"), os("/h")),
        Some("/h/.config/freesynergy/language".into())
    );
    assert_eq!(shared(os("rel"), os("rel")), None);
    assert_eq!(shared(None, None), None);
    let own = LanguageSetting::program("tasks", os("/c"), os("/h")).unwrap();
    assert_eq!(own.file(), PathBuf::from("/c/freesynergy/tasks/language"));
    let own = LanguageSetting::program("wiki-2", None, os("/h")).unwrap();
    assert_eq!(
        own.file(),
        PathBuf::from("/h/.config/freesynergy/wiki-2/language")
    );
}

#[test]
fn a_program_name_is_an_identifier_never_a_path() {
    for bad in [
        "", "../x", "..", ".", "/etc", "a/b", "Tasks", "-x", "a b", "a\u{e4}", "x/../y", "a.b",
    ] {
        assert!(
            LanguageSetting::program(bad, os("/c"), os("/h")).is_none(),
            "{bad:?}"
        );
    }
    assert!(LanguageSetting::program("fs-files2", os("/c"), os("/h")).is_some());
}

#[test]
fn a_saved_language_is_read_back_and_none_removes_it() {
    let d = dir("roundtrip");
    let s = LanguageSetting::new(d.join("deep/er/language"));
    assert_eq!(s.load(), Ok(None), "no file is no setting");
    s.save(Some(&Locale::parse("de-AT").unwrap())).unwrap();
    assert_eq!(s.load().unwrap().unwrap().to_string(), "de-AT");
    assert_eq!(std::fs::read_to_string(s.file()).unwrap(), "de-AT\n");
    s.save(Some(&Locale::english())).unwrap();
    assert_eq!(
        s.load().unwrap().unwrap().to_string(),
        "en",
        "a second save replaces the first"
    );
    s.save(None).unwrap();
    assert_eq!(s.load(), Ok(None));
    s.save(None).unwrap(); // removing what is not there is fine
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_write_leaves_no_temporary_file_and_a_failed_write_keeps_the_old_choice() {
    let d = dir("atomic");
    let s = LanguageSetting::new(d.join("language"));
    s.save(Some(&Locale::english())).unwrap();
    let names: Vec<_> = std::fs::read_dir(&d)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names, ["language"], "only the file, no .tmp");
    // Make the folder unwritable by putting a directory where the temporary file would go: the save fails, the old one stays.
    let temp = d.join(format!("language.{}.tmp", std::process::id()));
    std::fs::create_dir(&temp).unwrap();
    assert!(s.save(Some(&Locale::parse("de").unwrap())).is_err());
    assert_eq!(s.load().unwrap().unwrap().to_string(), "en");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_broken_file_is_an_error_not_a_language_and_a_blank_one_is_no_setting() {
    let d = dir("broken");
    let s = LanguageSetting::new(d.join("language"));
    std::fs::write(s.file(), "not a tag at all\n").unwrap();
    assert!(s.load().unwrap_err().contains("holds no language"));
    std::fs::write(s.file(), "  \n").unwrap();
    assert_eq!(s.load(), Ok(None));
    std::fs::write(s.file(), "\u{feff}de\n").unwrap();
    assert!(
        s.load().is_err(),
        "a byte order mark is not part of a tag: refused, never guessed"
    );
    // A directory where the file should be: unreadable, an error.
    std::fs::remove_file(s.file()).unwrap();
    std::fs::create_dir(s.file()).unwrap();
    assert!(s.load().unwrap_err().contains("cannot read"));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_file_larger_than_a_tag_or_no_regular_file_is_refused_unread() {
    let d = dir("limit");
    let s = LanguageSetting::new(d.join("language"));
    std::fs::write(s.file(), "de\n".repeat(100)).unwrap();
    assert!(s.load().unwrap_err().contains("too large"));
    std::fs::write(s.file(), [0xff, 0xfe, b'd']).unwrap();
    assert!(s.load().unwrap_err().contains("not text"));
    // A link to a folder (a stand-in for a pipe or a device) is no regular file.
    std::fs::remove_file(s.file()).unwrap();
    std::os::unix::fs::symlink(&d, s.file()).unwrap();
    assert!(s.load().unwrap_err().contains("not a regular file"));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_planted_link_at_the_temporary_name_is_not_followed() {
    let d = dir("planted");
    let s = LanguageSetting::new(d.join("language"));
    let victim = d.join("victim");
    std::fs::write(&victim, "keep me").unwrap();
    let temp = d.join(format!("language.{}.tmp", std::process::id()));
    std::os::unix::fs::symlink(&victim, &temp).unwrap();
    s.save(Some(&Locale::parse("de").unwrap())).unwrap();
    assert_eq!(
        std::fs::read_to_string(&victim).unwrap(),
        "keep me",
        "the link was replaced, not written through"
    );
    assert_eq!(s.load().unwrap().unwrap().to_string(), "de");
    let _ = std::fs::remove_dir_all(d);
}
