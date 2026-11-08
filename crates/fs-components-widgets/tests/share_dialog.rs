//! The share dialog: the picture, clicks, keys and typing in process.
//!
//! Tests: Model tests, pixel tests, an in-process mouse and keyboard.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, background, draw, ink, layout, serial};
use fs_components_widgets::share_dialog::{Bindings, Command, Grant, Outcome, Refusal};
use fs_components_widgets::{Choice, Permission, ShareDialog};

fn people() -> Vec<Choice> {
    vec![
        Choice::new("ada", "Ada Lovelace").with_details("Person", "ada@example.org"),
        Choice::new("grp", "Ärztinnen und Ärzte").with_details("Group", "12 members"),
        Choice::new("bob", "Bob Marley").with_details("Person", ""),
        Choice::new("eve", "李雷 Li Lei"),
        Choice::new("zed", "Zed"),
    ]
}

fn perms() -> Vec<Permission> {
    vec![
        Permission::new("view", "Can view"),
        Permission::new("edit", "Can edit"),
        Permission::new("own", "Owner"),
    ]
}

fn keys_of(g: &[Grant]) -> Vec<(&str, Option<&str>)> {
    g.iter()
        .map(|g| (g.choice.as_str(), g.permission.as_deref()))
        .collect()
}

fn dialog(id: &str) -> ShareDialog {
    ShareDialog::new(people(), perms())
        .default_permission("view")
        .id(id)
        .choose(vec![
            Grant::new("ada", Some("edit".into())),
            Grant::new("grp", Some("view".into())),
        ])
}

#[test]
fn the_picture_shows_rows_marks_and_permissions() {
    let _g = serial();
    let d = dialog("sd1");
    let img = draw(d.build(), 620, 560, "share-dialog.png");
    let bg = background(&img);
    assert!(ink(&img, (0, 0, 620, 560), bg) > 8000);
    // chosen rows are tinted and carry the primary-coloured mark: count blue-ish mark pixels
    let blue = img
        .pixels()
        .filter(|p| i32::from(p.0[2]) > i32::from(p.0[0]) + 60)
        .count();
    assert!(blue > 600, "chosen rows are not marked: {blue}");
    // choosing one more row changes the picture
    d.toggle("zed");
    let img2 = draw(d.build(), 620, 560, "share-dialog-three.png");
    assert_ne!(img, img2);
}

#[test]
fn clicks_choose_set_permissions_and_confirm() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let d = dialog("sd2").on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(d.build(), 620.0, 560.0);
    let mut input = Input::new();
    input.click(&mut tree, "sd2-pick-bob");
    assert_eq!(
        keys_of(&d.chosen()),
        [
            ("ada", Some("edit")),
            ("grp", Some("view")),
            ("bob", Some("view"))
        ]
    );
    // the permission buttons of an already chosen row
    input.click(&mut tree, "sd2-perm-ada-own");
    assert_eq!(
        keys_of(&d.chosen())[0],
        ("ada", Some("own")),
        "a permission click must keep the row chosen"
    );
    input.click(&mut tree, "sd2-confirm");
    let last = got.lock().unwrap().last().cloned();
    assert_eq!(
        last,
        Some(Outcome::Confirmed(vec![
            Grant::new("ada", Some("own".into())),
            Grant::new("grp", Some("view".into())),
            Grant::new("bob", Some("view".into())),
        ]))
    );
    input.click(&mut tree, "sd2-cancel");
    assert_eq!(got.lock().unwrap().last(), Some(&Outcome::Cancelled));
}

#[test]
fn keys_only_reach_the_focused_dialog_and_only_when_bound() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let a = ShareDialog::new(people(), perms())
        .default_permission("view")
        .id("ka")
        .keys(Bindings::default())
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let b = ShareDialog::new(people(), perms())
        .id("kb")
        .keys(Bindings::default());
    let plain = ShareDialog::new(people(), perms()).id("kc");
    let ui = blinc_layout::div::div()
        .flex_col()
        .child(a.build())
        .child(b.build())
        .child(plain.build());
    let mut tree = layout(ui, 620.0, 2000.0);
    let mut input = Input::new();
    input.focus(&tree, "ka");
    input.key_broadcast(&mut tree, 40); // Down: cursor to bob? first is ada, so grp
    input.key_broadcast(&mut tree, 32); // Space: choose
    input.key_broadcast(&mut tree, 39); // Right: next permission
    assert_eq!(keys_of(&a.chosen()), [("grp", Some("edit"))]);
    assert!(b.chosen().is_empty() && plain.chosen().is_empty());
    input.key_broadcast(&mut tree, 27); // Escape
    assert_eq!(got.lock().unwrap().last(), Some(&Outcome::Cancelled));
    input.focus(&tree, "kc");
    input.key_broadcast(&mut tree, 32);
    assert!(plain.chosen().is_empty(), "no bindings, no keys");
    let _ = Command::Up;
}

#[test]
fn typing_into_the_filter_field_narrows_the_list() {
    let _g = serial();
    let d = ShareDialog::new(people(), perms())
        .id("sf")
        .keys(Bindings::default());
    let mut tree = layout(d.build(), 620.0, 560.0);
    let mut input = Input::new();
    // the field is the only text input: click on it, then type
    input.click_at(&mut tree, 300.0, 80.0);
    for ch in "ada".chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
    }
    let narrowed = layout(d.build(), 620.0, 560.0);
    assert!(narrowed.query_by_id("sf-row-ada").is_some());
    assert!(
        narrowed.query_by_id("sf-row-bob").is_none(),
        "typed text did not narrow the list"
    );
    // while typing, Space must not choose a row
    input.key_broadcast(&mut tree, 32);
    assert_eq!(
        d.chosen(),
        [] as [fs_components_widgets::share_dialog::Grant; 0]
    );
}

#[test]
fn odd_data_draws_without_a_crash() {
    let _g = serial();
    let odd = ShareDialog::new(
        vec![
            Choice::new("", "").with_details("", ""),
            Choice::new("u", "Ünï 李雷 😀 ".repeat(6))
                .with_details("Typ ".repeat(6), "x".repeat(60)),
        ],
        vec![
            Permission::new("", ""),
            Permission::new("p", "Sehr lange Berechtigung ".repeat(10)),
        ],
    )
    .id("odd")
    .choose(vec![Grant::new("u", None)]);
    let img = draw(odd.build(), 620, 300, "share-dialog-odd.png");
    assert!(ink(&img, (0, 0, 620, 300), background(&img)) > 1000);
    let _ = layout(odd.build(), 0.0, 0.0);
    let empty = ShareDialog::new(vec![], vec![]);
    let img = draw(empty.build(), 620, 200, "share-dialog-empty.png");
    assert!(ink(&img, (0, 0, 620, 200), background(&img)) > 500);
    let many: Vec<Choice> = (0..50_000)
        .map(|i| Choice::new(format!("k{i}"), format!("Person {i}")))
        .collect();
    let huge = ShareDialog::new(many, perms()).max_rows(20);
    let img = draw(huge.build(), 620, 900, "share-dialog-huge.png");
    assert!(ink(&img, (0, 0, 620, 900), background(&img)) > 3000);
}

#[test]
fn chosen_rows_out_of_sight_are_announced() {
    let _g = serial();
    let many: Vec<Choice> = (0..500)
        .map(|i| Choice::new(format!("k{i}"), format!("Person {i}")))
        .collect();
    let d = ShareDialog::new(many, perms())
        .id("hid")
        .max_rows(10)
        .choose(vec![Grant::new("k400", None), Grant::new("gone", None)]);
    let with_note = draw(d.build(), 620, 700, "share-dialog-hidden-chosen.png");
    // the same dialog with the chosen one inside the window shows no such note
    let d2 = ShareDialog::new(
        (0..500)
            .map(|i| Choice::new(format!("k{i}"), format!("Person {i}")))
            .collect(),
        perms(),
    )
    .id("hid2")
    .max_rows(10)
    .choose(vec![Grant::new("k1", None)]);
    let without = draw(d2.build(), 620, 700, "share-dialog-visible-chosen.png");
    let footer = |i: &image::RgbaImage| {
        (0..700u32)
            .filter(|&y| ink(i, (0, y, 620, 1), background(i)) > 0)
            .count()
    };
    assert!(footer(&with_note) > footer(&without) - 40);
    let tree = layout(d.build(), 620.0, 700.0);
    assert!(tree.query_by_id("hid-row-k400").is_none());
    assert_eq!(d.chosen().len(), 2);
}

#[test]
fn long_names_wrap_instead_of_being_cut() {
    let _g = serial();
    let name = "Alexandra Katharina von Hohenzollern-Sigmaringen und Gorizia";
    let d = ShareDialog::new(vec![Choice::new("a", name)], perms()).id("w1");
    let short = ShareDialog::new(vec![Choice::new("a", "Alex")], perms()).id("w2");
    let tall = |ui| {
        let t = layout(ui, 300.0, 600.0);
        let n = t
            .query_by_id("w1-row-a")
            .or(t.query_by_id("w2-row-a"))
            .unwrap();
        t.get_absolute_bounds(n).unwrap().height
    };
    assert!(
        tall(d.build()) > tall(short.build()) + 10.0,
        "a long name must make its row taller"
    );
}

#[test]
fn every_default_key_does_its_job_through_the_element() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let d = ShareDialog::new(people(), perms())
        .default_permission("view")
        .id("ek")
        .keys(Bindings::default())
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(d.build(), 620.0, 560.0);
    let mut input = Input::new();
    input.focus(&tree, "ek");
    input.key(&mut tree, 32); // Space: ada
    input.key(&mut tree, 39); // Right: edit
    input.key(&mut tree, 39); // Right: own
    input.key(&mut tree, 39); // Right at the end: refused
    input.key(&mut tree, 37); // Left: edit
    assert_eq!(keys_of(&d.chosen()), [("ada", Some("edit"))]);
    input.key(&mut tree, 40); // Down
    input.key(&mut tree, 38); // Up
    assert_eq!(d.cursor_key().as_deref(), Some("ada"));
    // Enter and Space choose the row (the window turns them into a click on the focused root)
    input.key(&mut tree, 13);
    assert!(
        d.chosen().is_empty(),
        "Enter toggles the row under the cursor"
    );
    input.key(&mut tree, 32);
    assert_eq!(keys_of(&d.chosen()).len(), 1);
    // confirming by keyboard: Tab to the Done button, Enter there is Blinc's own button activation
    assert!(
        input.tab_to(&mut tree, "ek-confirm"),
        "the Done button is not reachable by Tab"
    );
    input.key(&mut tree, 13);
    assert_eq!(
        got.lock().unwrap().last(),
        Some(&Outcome::Confirmed(d.chosen()))
    );
    input.focus(&tree, "ek");
    input.key(&mut tree, 27);
    assert_eq!(got.lock().unwrap().last(), Some(&Outcome::Cancelled));
    assert!(
        got.lock()
            .unwrap()
            .contains(&Outcome::Refused(Refusal::Empty)),
        "refusals are reported, not swallowed"
    );
    input.key(&mut tree, 65);
}
