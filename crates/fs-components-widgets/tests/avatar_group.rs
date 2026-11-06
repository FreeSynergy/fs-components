//! Avatar group: the model, and the picture.
//!
//! Pattern: Model tests plus pixel tests on an offscreen drawing (no window).

mod common;

use common::{background, draw, draw_settled, extent_x, ink, serial};
use fs_components_widgets::{
    AvatarGroup, Person, Presence, Size, avatar_group_initials as initials,
};

fn people(n: usize) -> Vec<Person> {
    (0..n)
        .map(|i| Person::new(format!("p{i}"), format!("Person {i}")))
        .collect()
}

#[test]
fn initials_cope_with_any_name() {
    assert_eq!(initials("Ada Lovelace"), "AL");
    assert_eq!(initials("ada"), "A");
    assert_eq!(initials("  "), "?");
    assert_eq!(initials(""), "?");
    assert_eq!(initials("Jean Claude Van Damme"), "JD");
    assert_eq!(initials("élodie Ça"), "ÉÇ");
    assert_eq!(initials("e\u{301}ric z"), "E\u{301}Z");
    assert_eq!(initials("ßeta"), "SS");
    assert_eq!(initials("李 雷"), "李雷");
    assert_eq!(initials("👍🏽 ok"), "O");
    assert_eq!(initials("\u{200B}Bob"), "B");
    assert_eq!(initials("\u{202E}x"), "X");
    assert_eq!(initials("🇩🇪 Anna"), "A");
    assert_eq!(initials("🇩🇪"), "?");
    assert_eq!(initials("(Dr.) Who"), "DW");
}

#[test]
fn what_does_not_fit_is_counted_not_dropped() {
    let g = AvatarGroup::new(people(12)).max(4);
    assert_eq!(g.shown().len(), 4);
    assert_eq!(g.hidden().len(), 8);
    assert_eq!(g.hidden()[0].key, "p4");
    let all = AvatarGroup::new(people(3)).max(10);
    assert_eq!((all.shown().len(), all.hidden().len()), (3, 0));
    let none = AvatarGroup::new(people(3)).max(0);
    assert_eq!((none.shown().len(), none.hidden().len()), (0, 3));
    let empty = AvatarGroup::new(vec![]);
    assert!(empty.shown().is_empty() && empty.hidden().is_empty());
    let huge = AvatarGroup::new(people(100_000)).max(usize::MAX);
    assert_eq!(
        huge.shown().len(),
        fs_components_widgets::AVATAR_GROUP_MAX_SHOWN
    );
    assert_eq!(huge.hidden().len(), 100_000 - huge.shown().len());
}

#[test]
fn the_picture_shows_circles_and_the_count() {
    let _g = serial();
    let ui = AvatarGroup::new(people(9)).max(3).build();
    let img = draw(ui, 300, 80, "avatar-group-9-max-3.png");
    let bg = background(&img);
    let (l, r) = extent_x(&img, 0, 80).expect("something is drawn");
    // 3 circles + "+6" circle of 40 px, overlapping by 8 px each: about 4*40 - 3*8 = 136 px
    assert!((110..=170).contains(&(r - l)), "width {}", r - l);
    assert!(ink(&img, (0, 0, 300, 80), bg) > 800);
    // nothing to the right of the group
    assert_eq!(ink(&img, (200, 0, 100, 80), bg), 0);

    // fewer shown, same people: the picture differs
    let wide = AvatarGroup::new(people(9)).max(8).build();
    let img2 = draw(wide, 400, 80, "avatar-group-9-max-8.png");
    let (l2, r2) = extent_x(&img2, 0, 80).unwrap();
    assert!(r2 - l2 > r - l + 100, "{} vs {}", r2 - l2, r - l);
}

#[test]
fn edge_cases_draw_without_a_crash() {
    let _g = serial();
    let odd = vec![
        Person::new("", ""),
        Person::new(
            "k",
            "A very long name that goes on and on and on and on and on".repeat(20),
        )
        .with_presence(Presence::Online),
        Person::new("u", "👩🏽‍🚀 李雷 Ünï").with_image("/does/not/exist.png"),
    ];
    for size in [Size::ExtraSmall, Size::Large, Size::ExtraLarge] {
        let ui = AvatarGroup::new(odd.clone()).size(size).build();
        let img = draw(ui, 300, 90, "avatar-group-odd.png");
        assert!(ink(&img, (0, 0, 300, 90), background(&img)) > 100);
    }
    // empty and huge
    let img = draw(
        AvatarGroup::new(vec![]).build(),
        100,
        50,
        "avatar-group-empty.png",
    );
    assert_eq!(ink(&img, (0, 0, 100, 50), background(&img)), 0);
    let huge = AvatarGroup::new(people(100_000)).max(5).build();
    let img = draw(huge, 300, 60, "avatar-group-huge.png");
    assert!(ink(&img, (0, 0, 300, 60), background(&img)) > 500);
    // zero size: laid out, not drawn
    let _ = common::layout(AvatarGroup::new(people(3)).build(), 0.0, 0.0);
}

#[test]
fn a_picture_that_cannot_be_loaded_falls_back_to_initials() {
    let _g = serial();
    let plain = AvatarGroup::new(vec![Person::new("a", "Bob Marley")]).build();
    let broken = AvatarGroup::new(vec![
        Person::new("a", "Bob Marley").with_image("/no/such/file.png"),
    ])
    .build();
    let a = draw(plain, 100, 60, "avatar-plain.png");
    let b = draw(broken, 100, 60, "avatar-broken-image.png");
    assert_eq!(a, b, "a missing picture must look like no picture");
    assert!(ink(&a, (0, 0, 100, 60), background(&a)) > 500);
}

#[test]
fn a_picture_that_exists_is_drawn() {
    let _g = serial();
    let dir = common::Scratch::new("avatar");
    let png = common::write_png(&dir, "p.png", 64, 64, [220, 30, 30]);
    let ui = AvatarGroup::new(vec![Person::new("a", "Bob Marley").with_image(png)]).build();
    let img = common::draw_settled(ui, 100, 60, "avatar-with-picture.png");
    let reds = img
        .pixels()
        .filter(|p| p.0[0] > 150 && p.0[1] < 90 && p.0[2] < 90)
        .count();
    assert!(reds > 300, "red pixels: {reds}");
}

#[test]
fn a_file_that_is_no_picture_falls_back_to_initials_too() {
    let _g = serial();
    let dir = common::Scratch::new("avatar-text");
    let path = dir.path("notes.png");
    std::fs::write(&path, "this is text, not a picture").unwrap();
    let plain = AvatarGroup::new(vec![Person::new("a", "Bob Marley")]).build();
    let fake = AvatarGroup::new(vec![
        Person::new("a", "Bob Marley").with_image(path.to_string_lossy()),
    ])
    .build();
    let a = draw(plain, 100, 60, "avatar-plain2.png");
    let b = draw_settled(fake, 100, 60, "avatar-not-a-picture.png");
    assert_eq!(a, b);
}

#[test]
fn the_group_takes_no_keys() {
    let _g = serial();
    let mut tree = common::layout(AvatarGroup::new(people(3)).build(), 300.0, 80.0);
    let mut input = common::Input::new();
    for code in [13, 27, 32, 37, 38, 39, 40] {
        input.key_broadcast(&mut tree, code);
        input.key(&mut tree, code);
    }
}
