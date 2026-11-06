//! The move logic of the sortable list, without any drawing.
//!
//! Pattern: Pure state-machine tests, including a long pseudo-random run that checks nothing is
//! ever lost or doubled.

use fs_components_widgets::keys::{Key, KeyEvent};
use fs_components_widgets::sortable_list::{
    Bindings, Command, Item, Lane, Moved, Outcome, Place, Refusal, Sorter, window,
};

fn lane(key: &str, items: &[&str]) -> Lane {
    Lane::new(
        key,
        key.to_uppercase(),
        items
            .iter()
            .map(|k| Item::new(*k, k.to_uppercase()))
            .collect(),
    )
}

fn keys_of(s: &Sorter, lane: usize) -> Vec<String> {
    s.lanes()[lane]
        .items
        .iter()
        .map(|i| i.key.clone())
        .collect()
}

fn board() -> Sorter {
    Sorter::new(vec![
        lane("l1", &["a", "b", "c"]),
        lane("l2", &["x"]),
        lane("l3", &[]),
    ])
}

#[test]
fn keyboard_reorders_within_a_lane() {
    let mut s = board();
    assert_eq!(s.command(Command::Toggle), Outcome::Grabbed("a".into()));
    assert_eq!(s.command(Command::Down), Outcome::Carried);
    assert_eq!(s.command(Command::Down), Outcome::Carried);
    assert_eq!(s.command(Command::Down), Outcome::Refused(Refusal::AtEdge));
    assert_eq!(keys_of(&s, 0), ["b", "c", "a"]);
    assert_eq!(
        s.command(Command::Toggle),
        Outcome::Moved(Moved::new("a", Place::new("l1", 0), Place::new("l1", 2)))
    );
    assert_eq!(s.grabbed(), None);
    assert_eq!(s.cursor_key(), Some("a"));
}

#[test]
fn keyboard_moves_between_lanes_even_into_an_empty_one() {
    let mut s = board();
    s.command(Command::Down); // b
    s.command(Command::Toggle);
    assert_eq!(s.command(Command::Right), Outcome::Carried);
    assert_eq!(keys_of(&s, 1), ["x", "b"]);
    assert_eq!(s.command(Command::Right), Outcome::Carried);
    assert_eq!(keys_of(&s, 2), ["b"]);
    assert_eq!(s.command(Command::Right), Outcome::Refused(Refusal::AtEdge));
    let out = s.command(Command::Toggle);
    assert_eq!(
        out,
        Outcome::Moved(Moved::new("b", Place::new("l1", 1), Place::new("l3", 0)))
    );
    assert_eq!(keys_of(&s, 0), ["a", "c"]);
}

#[test]
fn cancel_puts_the_item_back_exactly() {
    let mut s = board();
    s.command(Command::Down);
    s.command(Command::Toggle);
    s.command(Command::Right);
    s.command(Command::Right);
    assert_eq!(s.command(Command::Cancel), Outcome::Cancelled);
    assert_eq!(keys_of(&s, 0), ["a", "b", "c"]);
    assert_eq!(keys_of(&s, 1), ["x"]);
    assert!(keys_of(&s, 2).is_empty());
    assert_eq!(s.grabbed(), None);
    assert_eq!(s.cursor_key(), Some("b"));
    // Escape with nothing held is not consumed
    assert_eq!(s.command(Command::Cancel), Outcome::Ignored);
}

#[test]
fn locked_items_and_closed_lanes_refuse() {
    let mut s = Sorter::new(vec![
        Lane::new(
            "l1",
            "",
            vec![Item::new("a", "A").locked(), Item::new("b", "B")],
        ),
        lane("l2", &["x"]).closed(),
        lane("l3", &[]),
    ]);
    assert_eq!(
        s.command(Command::Toggle),
        Outcome::Refused(Refusal::Locked)
    );
    s.command(Command::Down);
    assert_eq!(s.command(Command::Toggle), Outcome::Grabbed("b".into()));
    // right skips the closed lane and enters the next open one
    assert_eq!(s.command(Command::Right), Outcome::Carried);
    assert_eq!(s.cursor().map(|p| p.lane), Some(2));
    assert_eq!(
        s.hover(&Place::new("l2", 0)),
        Outcome::Refused(Refusal::LaneClosed)
    );
    // a closed lane can still be reordered by its own items
    s.cancel();
    assert_eq!(s.grab("x"), Outcome::Grabbed("x".into()));
    assert_eq!(s.hover(&Place::new("l2", 0)), Outcome::Carried);
    assert_eq!(
        s.hover(&Place::new("nope", 0)),
        Outcome::Refused(Refusal::NoSuchLane)
    );
    assert_eq!(s.grab("b"), Outcome::Refused(Refusal::Busy));
}

#[test]
fn empty_lists_answer_and_do_not_panic() {
    for lanes in [
        vec![],
        vec![lane("a", &[])],
        vec![lane("a", &[]), lane("b", &[])],
    ] {
        let mut s = Sorter::new(lanes);
        for c in [
            Command::Up,
            Command::Down,
            Command::Left,
            Command::Right,
            Command::First,
            Command::Last,
            Command::PageUp,
            Command::PageDown,
            Command::Toggle,
        ] {
            assert_eq!(s.command(c), Outcome::Refused(Refusal::Empty), "{c:?}");
        }
        assert_eq!(s.cursor(), None);
        assert_eq!(s.drop_grabbed(), Outcome::Refused(Refusal::NothingHeld));
        assert_eq!(s.cancel(), Outcome::Refused(Refusal::NothingHeld));
        assert_eq!(s.grab("zz"), Outcome::Refused(Refusal::NoSuchItem));
    }
}

#[test]
fn the_pointer_api_does_the_same_without_keys() {
    let mut s = board();
    assert_eq!(s.grab("a"), Outcome::Grabbed("a".into()));
    assert_eq!(s.hover(&Place::new("l2", 99)), Outcome::Carried); // too large means the end
    assert_eq!(keys_of(&s, 1), ["x", "a"]);
    assert_eq!(s.hover(&Place::new("l2", 0)), Outcome::Carried);
    let out = s.drop_grabbed();
    assert_eq!(
        out,
        Outcome::Moved(Moved::new("a", Place::new("l1", 0), Place::new("l2", 0)))
    );
    // dropping back where it was is unchanged
    assert_eq!(s.grab("a"), Outcome::Grabbed("a".into()));
    s.hover(&Place::new("l2", 0));
    assert_eq!(s.drop_grabbed(), Outcome::Unchanged);
    // one step
    let out = s.move_item("c", &Place::new("l3", 0));
    assert!(matches!(out, Outcome::Moved(_)));
    assert_eq!(
        s.move_item("c", &Place::new("zzz", 0)),
        Outcome::Refused(Refusal::NoSuchLane)
    );
    assert_eq!(keys_of(&s, 2), ["c"], "a refused move changes nothing");
}

#[test]
fn new_data_keeps_cursor_and_hold_by_key() {
    let mut s = board();
    s.grab("b");
    s.set_lanes(vec![lane("l1", &["z", "b", "a"]), lane("l2", &[])]);
    assert_eq!(s.grabbed(), Some("b"));
    assert_eq!(s.cursor_key(), Some("b"));
    s.set_lanes(vec![lane("l1", &["q"])]);
    assert_eq!(s.grabbed(), None, "the held item is gone");
    assert_eq!(s.cursor_key(), Some("q"));
    s.set_lanes(vec![]);
    assert_eq!(s.cursor(), None);
}

#[test]
fn a_huge_list_is_handled_and_unusual_keys_are_fine() {
    let items: Vec<Item> = (0..100_000)
        .map(|i| Item::new(format!("k{i}"), format!("Zeile {i} äöü 李雷 😀")))
        .collect();
    let mut s = Sorter::new(vec![Lane::new("big", "", items)]);
    s.command(Command::Toggle);
    for _ in 0..3 {
        s.command(Command::PageDown);
    }
    assert_eq!(s.cursor().map(|p| p.index), Some(30));
    s.command(Command::Last);
    assert_eq!(s.cursor().map(|p| p.index), Some(99_999));
    assert!(matches!(s.command(Command::Toggle), Outcome::Moved(_)));
    assert_eq!(s.lanes()[0].items.len(), 100_000);
    assert_eq!(s.lanes()[0].items[99_999].key, "k0");
    // duplicate keys: no panic, nothing lost
    let mut d = Sorter::new(vec![lane("a", &["x", "x", "x"]), lane("b", &["x"])]);
    d.command(Command::Toggle);
    d.command(Command::Right);
    d.command(Command::Toggle);
    assert_eq!(d.lanes().iter().map(|l| l.items.len()).sum::<usize>(), 4);
}

#[test]
fn nothing_is_lost_or_doubled_in_a_long_random_run() {
    let all = [
        Command::Up,
        Command::Down,
        Command::Left,
        Command::Right,
        Command::First,
        Command::Last,
        Command::PageUp,
        Command::PageDown,
        Command::Toggle,
        Command::Toggle,
        Command::Cancel,
    ];
    let mut s = Sorter::new(vec![
        lane("a", &["1", "2", "3", "4"]),
        lane("b", &["5"]),
        lane("c", &[]),
        lane("d", &["6", "7"]).closed(),
    ]);
    let mut seed = 12345u64;
    for _ in 0..20_000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        s.command(all[(seed >> 33) as usize % all.len()]);
        let mut seen: Vec<String> = s
            .lanes()
            .iter()
            .flat_map(|l| l.items.iter().map(|i| i.key.clone()))
            .collect();
        seen.sort();
        assert_eq!(seen, ["1", "2", "3", "4", "5", "6", "7"]);
        // the closed lane never takes an item of another lane
        assert!(keys_of(&s, 3).iter().all(|k| k == "6" || k == "7"));
    }
}

#[test]
fn windows_are_bounded_and_contain_the_focus() {
    assert_eq!(window(0, None, 10), 0..0);
    assert_eq!(window(5, Some(2), 10), 0..5);
    assert_eq!(window(1000, Some(0), 10), 0..10);
    assert_eq!(window(1000, Some(999), 10), 990..1000);
    let w = window(1000, Some(500), 10);
    assert!(w.contains(&500) && w.len() == 10);
    assert_eq!(window(1000, Some(5000), 10).end, 1000);
    assert_eq!(
        window(3, None, 0).len(),
        1,
        "a window of zero rows is one row"
    );
}

#[test]
fn bindings_are_optional_replaceable_and_leave_ctrl_alone() {
    let b = Bindings::default();
    assert_eq!(b.command(&KeyEvent::new(Key::Space)), Some(Command::Toggle));
    assert_eq!(b.command(&KeyEvent::new(Key::Other(65))), None);
    let mut ctrl = KeyEvent::new(Key::Up);
    ctrl.ctrl = true;
    assert_eq!(b.command(&ctrl), None);
    let mine = Bindings::none()
        .bind(Key::Other(74), Command::Down)
        .bind(Key::Other(74), Command::Up);
    assert_eq!(
        mine.command(&KeyEvent::new(Key::Other(74))),
        Some(Command::Up)
    );
    assert_eq!(mine.command(&KeyEvent::new(Key::Up)), None);
}
