//! The state of the shell, without a window.
//!
//! Pattern: State tests — transitions only.

use std::sync::Arc;

use blinc_layout::div::div;
use fs_components_app::{AppError, Element, Screen, Shell, Status, Surface};

struct Sheet(&'static str);

impl Surface for Sheet {
    fn build(&self, _w: f32, _h: f32) -> Box<dyn Element> {
        Box::new(div())
    }
    fn styles(&self) -> String {
        self.0.to_owned()
    }
}

fn screen(id: &str) -> Screen {
    Screen::new(id, id.to_uppercase(), Arc::new(Sheet("")))
}

#[test]
fn the_first_screen_is_shown() {
    let shell = Shell::new("T", vec![screen("a"), screen("b")]).unwrap();
    assert_eq!(shell.title(), "T");
    assert_eq!(shell.current().unwrap().id, "a");
    assert_eq!(shell.current().unwrap().label, "A");
}

#[test]
fn selecting_an_unknown_screen_changes_nothing() {
    let mut shell = Shell::new("T", vec![screen("a"), screen("b")]).unwrap();
    assert!(shell.select("b"));
    assert!(!shell.select("c"));
    assert_eq!(shell.current().unwrap().id, "b");
}

#[test]
fn two_screens_with_one_id_are_refused() {
    let result = Shell::new("T", vec![screen("a"), screen("a")]);
    assert_eq!(
        result.err(),
        Some(AppError::DuplicateScreen("a".to_owned()))
    );
}

#[test]
fn a_program_without_screens_shows_none() {
    let mut shell = Shell::new("T", Vec::new()).unwrap();
    assert!(shell.current().is_none());
    assert!(!shell.select("a"));
}

#[test]
fn the_status_is_replaced_and_emptied() {
    let mut shell = Shell::new("T", vec![screen("a")]).unwrap();
    assert_eq!(shell.status(), None);
    shell.set_status(Some(Status::Info("one".into())));
    shell.set_status(Some(Status::Problem("two".into())));
    assert_eq!(shell.status(), Some(&Status::Problem("two".into())));
    shell.set_status(None);
    assert_eq!(shell.status(), None);
}
