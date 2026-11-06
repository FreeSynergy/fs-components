//! The share dialog's state, without any drawing.
//!
//! Pattern: Model tests, pixel tests, an in-process mouse and keyboard.

use fs_components_widgets::share_dialog::{Grant, Outcome, Refusal, Sharing};
use fs_components_widgets::{Choice, Permission};

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

#[test]
fn choosing_gives_the_first_permission_and_can_be_changed() {
    let mut s = Sharing::new(people(), perms());
    assert_eq!(s.toggle("ada"), Outcome::Changed);
    assert_eq!(s.toggle("bob"), Outcome::Changed);
    assert_eq!(
        keys_of(s.chosen()),
        [("ada", Some("view")), ("bob", Some("view"))]
    );
    assert_eq!(s.set_permission("bob", "own"), Outcome::Changed);
    assert_eq!(
        s.set_permission("eve", "own"),
        Outcome::Refused(Refusal::NotChosen)
    );
    assert_eq!(
        s.set_permission("bob", "root"),
        Outcome::Refused(Refusal::NoSuchPermission)
    );
    assert_eq!(
        s.set_permission("nobody", "own"),
        Outcome::Refused(Refusal::NoSuchChoice)
    );
    assert_eq!(s.toggle("nobody"), Outcome::Refused(Refusal::NoSuchChoice));
    assert_eq!(s.step_permission("ada", true), Outcome::Changed);
    assert_eq!(s.step_permission("ada", false), Outcome::Changed);
    assert_eq!(
        s.step_permission("ada", false),
        Outcome::Refused(Refusal::Empty)
    );
    s.toggle("ada");
    assert_eq!(keys_of(s.chosen()), [("bob", Some("own"))]);
    assert_eq!(s.confirm(), Outcome::Confirmed(s.chosen().to_vec()));
}

#[test]
fn the_filter_hides_rows_but_never_drops_a_choice() {
    let mut s = Sharing::new(people(), perms());
    s.toggle("ada");
    s.set_filter("ärzt");
    assert_eq!(s.visible(), [1]);
    s.set_filter("  ÄRZTINNEN ");
    assert_eq!(
        s.visible(),
        [1],
        "case and surrounding spaces do not matter"
    );
    s.set_filter("example.org");
    assert_eq!(s.visible(), [0], "the detail is searched too");
    s.set_filter("李");
    assert_eq!(s.visible(), [3]);
    s.set_filter("nothing like this");
    assert!(s.visible().is_empty());
    assert_eq!(s.move_cursor(1), Outcome::Refused(Refusal::Empty));
    assert_eq!(s.cursor_key(), None);
    // ada is hidden but still chosen and still confirmed
    assert_eq!(
        s.confirm(),
        Outcome::Confirmed(vec![Grant::new("ada", Some("view".into()))])
    );
}

#[test]
fn earlier_choices_are_kept_even_if_not_offered_and_a_choice_can_be_required() {
    let mut s = Sharing::new(people(), perms());
    s.choose(vec![
        Grant::new("gone", Some("bogus".into())),
        Grant::new("ada", Some("edit".into())),
        Grant::new("ada", Some("own".into())),
    ]);
    assert_eq!(
        keys_of(s.chosen()),
        [("gone", Some("view")), ("ada", Some("own"))]
    );
    let mut r = Sharing::new(people(), perms());
    r.require_choice(true);
    assert_eq!(r.confirm(), Outcome::Refused(Refusal::NothingChosen));
    let mut none = Sharing::new(people(), vec![]);
    none.toggle("ada");
    assert_eq!(keys_of(none.chosen()), [("ada", None)]);
    assert_eq!(
        none.step_permission("ada", true),
        Outcome::Refused(Refusal::Empty)
    );
    let mut empty = Sharing::new(vec![], vec![]);
    assert_eq!(empty.move_cursor(1), Outcome::Refused(Refusal::Empty));
    assert_eq!(empty.confirm(), Outcome::Confirmed(vec![]));
}

#[test]
fn a_huge_offer_is_filtered_and_moved_through() {
    let many: Vec<Choice> = (0..100_000)
        .map(|i| Choice::new(format!("k{i}"), format!("Person {i}")))
        .collect();
    let mut s = Sharing::new(many, perms());
    s.set_filter("person 99999");
    assert_eq!(s.visible().len(), 1);
    s.set_filter("");
    s.move_cursor(isize::MAX);
    assert_eq!(s.cursor_key(), Some("k99999"));
    s.move_cursor(isize::MIN);
    assert_eq!(s.cursor_key(), Some("k0"));
}
