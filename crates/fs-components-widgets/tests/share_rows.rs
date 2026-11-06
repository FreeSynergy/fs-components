//! The share dialog's rows: duplicate keys, many permissions, unknown and unlisted pairs.
//!
//! Pattern: Pixel and layout tests in process (no window).

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, background, draw, ink, layout, serial};
use fs_components_widgets::share_dialog::{Bindings, Grant, Outcome};
use fs_components_widgets::{Choice, Permission, ShareDialog};

fn people() -> Vec<Choice> {
    vec![
        Choice::new("ada", "Ada Lovelace"),
        Choice::new("bob", "Bob Marley"),
    ]
}

fn perms() -> Vec<Permission> {
    vec![
        Permission::new("view", "Can view"),
        Permission::new("edit", "Can edit"),
    ]
}

#[test]
fn two_choices_with_one_key_do_not_crash_the_drawing() {
    let _g = serial();
    let d = ShareDialog::new(
        vec![Choice::new("same", "First"), Choice::new("same", "Second")],
        perms(),
    )
    .id("dup")
    .max_rows(1)
    .choose(vec![Grant::new("same", Some("view".into()))]);
    let img = draw(d.build(), 620, 400, "share-dialog-duplicate-keys.png");
    assert!(ink(&img, (0, 0, 620, 400), background(&img)) > 500);
}

#[test]
fn many_permissions_wrap_and_the_current_one_stays_visible() {
    let _g = serial();
    let many: Vec<Permission> = (0..6)
        .map(|i| Permission::new(format!("p{i}"), format!("Permission {i}")))
        .collect();
    let d = ShareDialog::new(people(), many)
        .id("six")
        .choose(vec![Grant::new("ada", Some("p5".into()))]);
    let tree = layout(d.build(), 600.0, 700.0);
    let root = tree
        .get_absolute_bounds(tree.query_by_id("six").unwrap())
        .unwrap();
    for i in 0..6 {
        let b = tree
            .get_absolute_bounds(tree.query_by_id(&format!("six-perm-ada-p{i}")).unwrap())
            .unwrap();
        assert!(
            b.x >= root.x && b.x + b.width <= root.x + root.width + 0.5,
            "button p{i} sticks out: {}..{} of {}..{}",
            b.x,
            b.x + b.width,
            root.x,
            root.x + root.width
        );
    }
    let _ = draw(d.build(), 600, 700, "share-dialog-six-permissions.png");
}

#[test]
fn an_unknown_permission_is_shown_and_survives_confirm() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let d = ShareDialog::new(
        people(),
        vec![
            Permission::new("owner", "Owner"),
            Permission::new("view", "Can view"),
        ],
    )
    .id("unk")
    .choose(vec![Grant::new("ada", Some("legacy-readonly".into()))])
    .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(d.build(), 620.0, 560.0);
    assert!(
        tree.query_by_id("unk-unknown-ada").is_some(),
        "the unknown permission is not shown"
    );
    Input::new().click(&mut tree, "unk-confirm");
    assert_eq!(
        got.lock().unwrap().last(),
        Some(&Outcome::Confirmed(vec![Grant::new(
            "ada",
            Some("legacy-readonly".into())
        )]))
    );
}

#[test]
fn a_pair_that_is_not_offered_is_shown_and_can_be_removed_by_click() {
    let _g = serial();
    let d = ShareDialog::new(people(), perms())
        .id("unl")
        .choose(vec![Grant::new("gone", Some("view".into()))]);
    let mut tree = layout(d.build(), 620.0, 560.0);
    assert!(tree.query_by_id("unl-row-gone").is_some());
    Input::new().click(&mut tree, "unl-pick-gone");
    assert!(d.chosen().is_empty());
}

#[test]
fn a_mouse_click_is_not_taken_for_enter_or_space() {
    let _g = serial();
    let d = ShareDialog::new(people(), perms())
        .default_permission("view")
        .id("mc")
        .keys(Bindings::default());
    let mut tree = layout(d.build(), 620.0, 560.0);
    let mut input = Input::new();
    input.focus(&tree, "mc");
    input.click(&mut tree, "mc-pick-bob");
    assert_eq!(
        d.chosen()
            .iter()
            .map(|g| (g.choice.as_str(), g.permission.as_deref()))
            .collect::<Vec<_>>(),
        [("bob", Some("view"))],
        "one click, one toggle"
    );
}

#[test]
fn text_focus_does_not_leak_into_the_next_test() {
    // Blinc keeps "a text input has the keyboard" process-wide; a widget ignores keys while it
    // is set. This once made `keys_only_reach_the_focused_dialog...` fail now and then, when it
    // ran after the test that typed into the filter field. `serial()` clears it.
    let g = serial();
    let d = ShareDialog::new(people(), perms()).id("tf");
    let mut tree = layout(d.build(), 620.0, 560.0);
    Input::new().click_at(&mut tree, 300.0, 80.0);
    assert!(blinc_layout::widgets::text_input::has_focused_text_input());
    drop(g);
    let _g = serial();
    assert!(!blinc_layout::widgets::text_input::has_focused_text_input());
}
