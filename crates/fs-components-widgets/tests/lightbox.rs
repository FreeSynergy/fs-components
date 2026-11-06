//! The lightbox: stepping logic, pictures really drawn, buttons and keys in process.
//!
//! Pattern: Model tests, pixel tests on offscreen drawings, an in-process mouse and keyboard.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, Scratch, background, draw, draw_settled, ink, layout, serial, write_png};
use fs_components_widgets::lightbox::{Bindings, Command, Outcome, Refusal, Viewer};
use fs_components_widgets::{LightBox, Media};

fn items(n: usize) -> Vec<Media> {
    (0..n)
        .map(|i| {
            Media::picture(
                format!("m{i}"),
                format!("/none/{i}.png"),
                format!("Picture {i}"),
            )
        })
        .collect()
}

#[test]
fn stepping_stops_at_the_edges_or_wraps() {
    let mut v = Viewer::new(items(3));
    assert_eq!(v.position(), (1, 3));
    assert_eq!(v.back(), Outcome::Refused(Refusal::AtEdge));
    assert_eq!(v.forward(), Outcome::Shown("m1".into()));
    assert_eq!(v.last(), Outcome::Shown("m2".into()));
    assert_eq!(v.forward(), Outcome::Refused(Refusal::AtEdge));
    assert_eq!(v.show("m0"), Outcome::Shown("m0".into()));
    assert_eq!(v.show("zz"), Outcome::Refused(Refusal::NoSuchItem));
    let mut w = Viewer::new(items(3)).wrap(true);
    assert_eq!(w.back(), Outcome::Shown("m2".into()));
    assert_eq!(w.forward(), Outcome::Shown("m0".into()));
    let mut one = Viewer::new(items(1));
    assert_eq!(one.forward(), Outcome::Refused(Refusal::AtEdge));
}

#[test]
fn no_items_never_panics_and_new_items_keep_the_place() {
    let mut e = Viewer::new(vec![]);
    for o in [e.forward(), e.back(), e.first(), e.last()] {
        assert_eq!(o, Outcome::Refused(Refusal::Empty));
    }
    assert_eq!((e.current(), e.position()), (None, (0, 0)));
    let mut v = Viewer::new(items(5));
    v.show("m3");
    v.set_items(items(10));
    assert_eq!(v.current().map(|m| m.key.as_str()), Some("m3"));
    v.set_items(items(2));
    assert_eq!(
        v.position(),
        (2, 2),
        "a vanished item leaves the nearest one"
    );
    v.set_items(vec![]);
    assert_eq!(v.position(), (0, 0));
    let mut huge = Viewer::new(items(200_000));
    assert_eq!(huge.last(), Outcome::Shown("m199999".into()));
}

fn bounds_of(
    img: &image::RgbaImage,
    hit: impl Fn(&[u8; 4]) -> bool,
) -> (u32, u32, u32, u32, usize) {
    let (mut x0, mut y0, mut x1, mut y1, mut n) = (u32::MAX, u32::MAX, 0, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if hit(&p.0) {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
            n += 1;
        }
    }
    (x0, y0, x1, y1, n)
}

fn is_red(p: &[u8; 4]) -> bool {
    p[0] > 150 && p[1] < 90 && p[2] < 90
}

fn is_blue(p: &[u8; 4]) -> bool {
    p[2] > 150 && p[0] < 90 && p[1] < 120
}

#[test]
fn the_picture_is_drawn_fitted_and_stepping_changes_it() {
    let _g = serial();
    let dir = Scratch::new("lightbox");
    let wide = write_png(&dir, "wide.png", 400, 200, [220, 30, 30]);
    let tall = write_png(&dir, "tall.png", 100, 300, [30, 30, 220]);
    let lb = LightBox::new(vec![
        Media::picture("w", wide, "A wide one"),
        Media::picture("t", tall, "A tall one"),
    ])
    .id("lb1");
    let img = draw_settled(lb.build(), 700, 500, "lightbox-wide.png");
    let (x0, y0, x1, y1, n) = bounds_of(&img, is_red);
    assert!(n > 20_000, "the red picture is not drawn: {n}");
    let (w, h) = (f64::from(x1 - x0 + 1), f64::from(y1 - y0 + 1));
    assert!(
        (1.8..=2.2).contains(&(w / h)),
        "aspect of the shown picture {w}x{h}"
    );
    assert!(
        x0 > 40 && x1 < 660,
        "the picture stays between the buttons: {x0}..{x1}"
    );
    assert_eq!(bounds_of(&img, is_blue).4, 0);

    assert_eq!(lb.command(Command::Next), Outcome::Shown("t".into()));
    let img2 = draw_settled(lb.build(), 700, 500, "lightbox-tall.png");
    let (x0, y0, x1, y1, n) = bounds_of(&img2, is_blue);
    assert!(n > 10_000, "the blue picture is not drawn: {n}");
    let (w, h) = (f64::from(x1 - x0 + 1), f64::from(y1 - y0 + 1));
    assert!((0.28..=0.38).contains(&(w / h)), "aspect {w}x{h}");
    assert_eq!(bounds_of(&img2, is_red).4, 0);
}

#[test]
fn buttons_step_and_close_and_report() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let lb = LightBox::new(items(3))
        .id("bt")
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let mut tree = layout(lb.build(), 700.0, 500.0);
    let mut input = Input::new();
    input.click(&mut tree, "bt-next");
    assert_eq!(lb.current_key().as_deref(), Some("m1"));
    input.click(&mut tree, "bt-prev");
    input.click(&mut tree, "bt-close");
    let got = got.lock().unwrap().clone();
    assert_eq!(got.first(), Some(&Outcome::Shown("m1".into())));
    assert_eq!(got.last(), Some(&Outcome::Closed));
}

#[test]
fn keys_are_optional_and_only_the_focused_box_takes_them() {
    let _g = serial();
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    let a = LightBox::new(items(4))
        .id("ka")
        .keys(Bindings::default())
        .on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone())));
    let b = LightBox::new(items(4)).id("kb").keys(Bindings::default());
    let plain = LightBox::new(items(4)).id("kc");
    let ui = blinc_layout::div::div()
        .flex_col()
        .child(a.build())
        .child(b.build())
        .child(plain.build());
    let mut tree = layout(ui, 700.0, 1500.0);
    let mut input = Input::new();
    input.focus(&tree, "ka");
    input.key_broadcast(&mut tree, 39); // Right
    input.key_broadcast(&mut tree, 39);
    assert_eq!(
        (a.position().0, b.position().0, plain.position().0),
        (3, 1, 1)
    );
    input.key_broadcast(&mut tree, 27); // Escape
    assert_eq!(got.lock().unwrap().last(), Some(&Outcome::Closed));
    input.focus(&tree, "kc");
    input.key_broadcast(&mut tree, 39);
    assert_eq!(
        (a.position().0, plain.position().0),
        (3, 1),
        "no bindings, no keys"
    );
}

#[test]
fn what_cannot_be_shown_is_said() {
    let _g = serial();
    let missing = LightBox::new(vec![Media::picture("a", "/no/such/file.png", "")]).id("n1");
    let other = LightBox::new(vec![Media::other("v", "A video")]).id("n2");
    let none = LightBox::new(vec![]).id("n3");
    for (name, id, lb) in [
        ("missing", "n1", missing),
        ("other", "n2", other),
        ("none", "n3", none),
    ] {
        let tree = layout(lb.build(), 700.0, 400.0);
        assert!(
            tree.query_by_id(&format!("{id}-note")).is_some(),
            "{name}: no note on the stage"
        );
        let _ = common::draw_tree(&tree, 700, 400, &format!("lightbox-{name}.png"));
    }
}

#[test]
fn odd_input_and_size_do_not_crash() {
    let _g = serial();
    let lb = LightBox::new(vec![Media::other(
        "é\u{0}",
        "Ein sehr langer Titel 李雷 😀 ".repeat(6),
    )]);
    let img = draw(lb.build(), 300, 200, "lightbox-odd.png");
    assert!(ink(&img, (0, 0, 300, 200), background(&img)) > 300);
    let _ = layout(LightBox::new(items(3)).build(), 0.0, 0.0);
    let _ = draw(
        LightBox::new(items(100_000)).build(),
        200,
        120,
        "lightbox-huge.png",
    );
}

#[test]
fn every_default_key_does_its_job_through_the_element() {
    let _g = serial();
    let lb = LightBox::new(items(5)).id("ek").keys(Bindings::default());
    let mut tree = layout(lb.build(), 700.0, 500.0);
    let mut input = Input::new();
    input.focus(&tree, "ek");
    for (code, at) in [(39, 2), (37, 1), (35, 5), (36, 1), (34, 2), (33, 1)] {
        input.key(&mut tree, code);
        assert_eq!(lb.position().0, at, "key {code}");
    }
    input.key(&mut tree, 65);
    assert_eq!(lb.position().0, 1);
}

#[test]
fn a_file_that_is_no_picture_is_announced_and_long_captions_wrap() {
    let _g = serial();
    let dir = Scratch::new("lb-text");
    let path = dir.path("x.png");
    std::fs::write(&path, "not a picture").unwrap();
    let note = LightBox::new(vec![Media::picture("a", path.to_string_lossy(), "")]).id("fk");
    let tree = layout(note.build(), 700.0, 400.0);
    assert!(
        tree.query_by_id("fk-note").is_some(),
        "a fake picture file must be announced"
    );
    let long = "A caption that is much longer than the box is wide and must not be cut off at the end: THE END ".repeat(3);
    let tall = |c: &str| {
        let t = layout(
            LightBox::new(vec![Media::other("a", c)]).id("cap").build(),
            400.0,
            500.0,
        );
        common::size_of(&t, "cap-caption").1
    };
    assert!(
        tall(&long) >= tall("short"),
        "a caption never gets smaller than a short one"
    );
    if common::has_fonts() {
        assert!(
            tall(&long) > tall("short") + 30.0,
            "a long caption must wrap onto more lines"
        );
    }
}
