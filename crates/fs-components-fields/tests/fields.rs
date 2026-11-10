//! Tests: the 20 components (5 types x 4 modes) in the 4 styles, drawn offscreen; the input typed
//! into with the window's own router; sizes measured against the style.
//!
//! Pattern: none (tests). They need a graphics adapter (software Vulkan on the runner).
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use common::{Input, background, draw, has_fonts, ink, layout, serial, size_of};
use fs_components_fields::{
    FieldInput, KeysAsText, Kind, Look, Say, SuggestFromList, cell, compact, display,
};
use fs_components_styles::StyleId;

/// A page in the ground colour of the look: a frontend paints the ground, the components do not.
fn on_ground<I: blinc_layout::prelude::ElementBuilder + 'static>(
    look: &Look,
    w: f32,
    h: f32,
    ui: I,
) -> impl blinc_layout::prelude::ElementBuilder + use<I> {
    blinc_layout::div::div()
        .w(w)
        .h(h)
        .bg(Look::color(look.palette().ground))
        .child(ui)
}

fn sample(kind: Kind) -> Vec<String> {
    let v: &[&str] = match kind {
        Kind::Email => &["ada", "example.org"],
        Kind::Name => &["Dr.", "Ada", "Augusta", "Lovelace", ""],
        Kind::Phone => &["+49", "30 1234567"],
        Kind::Address => &["Main St 1", "10115", "Berlin", "", "Germany"],
        Kind::Date => &["2026", "11", "10"],
    };
    v.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn every_component_draws_something_in_every_style() {
    let _g = serial();
    for id in StyleId::ALL {
        for dark in [false, true] {
            let look = Look::new(id, dark);
            let tone = if dark { "dark" } else { "light" };
            for kind in Kind::ALL {
                let parts = sample(kind);
                let say: Arc<dyn Say> = Arc::new(KeysAsText);
                let file =
                    |mode: &str| format!("{}-{tone}-{}-{mode}.png", id.name(), kind.field().key());
                let input = FieldInput::new(kind, look, say)
                    .value(&kind.field().join(&parts))
                    .build();
                let shots = [
                    (
                        "input",
                        draw(
                            on_ground(&look, 640.0, 140.0, input),
                            640,
                            140,
                            &file("input"),
                        ),
                    ),
                    (
                        "display",
                        draw(
                            on_ground(
                                &look,
                                640.0,
                                200.0,
                                display(kind, &parts, &look, &KeysAsText),
                            ),
                            640,
                            200,
                            &file("display"),
                        ),
                    ),
                    (
                        "compact",
                        draw(
                            on_ground(&look, 640.0, 40.0, compact(kind, &parts, &look)),
                            640,
                            40,
                            &file("compact"),
                        ),
                    ),
                    (
                        "cell",
                        draw(
                            on_ground(&look, 640.0, 80.0, cell(kind, &parts, &look, false)),
                            640,
                            80,
                            &file("cell"),
                        ),
                    ),
                ];
                assert!(
                    has_fonts(),
                    "no system fonts: the pictures would have no text (install DejaVu or Noto)"
                );
                for (mode, img) in shots {
                    let n = ink(&img, (0, 0, img.width(), img.height()), background(&img));
                    assert!(
                        n > 20,
                        "{} dark={dark} {} {mode}: no ink ({n})",
                        id.name(),
                        kind.field().key()
                    );
                }
            }
        }
    }
}

#[test]
fn the_row_of_a_cell_has_the_height_of_the_style() {
    let _g = serial();
    for id in StyleId::ALL {
        let look = Look::new(id, false);
        let ui = blinc_layout::div::div()
            .id("probe")
            .w(400.0)
            .h_fit()
            .child(cell(Kind::Name, &sample(Kind::Name), &look, false));
        let tree = layout(ui, 400.0, 200.0);
        let (_, h) = size_of(&tree, "probe");
        assert!(
            (h - look.style.heights.row).abs() < 0.6,
            "{}: {} vs {}",
            id.name(),
            h,
            look.style.heights.row
        );
    }
}

#[test]
fn the_input_field_has_the_control_height_of_the_style() {
    let _g = serial();
    for id in StyleId::ALL {
        let look = Look::new(id, false);
        let f = FieldInput::new(Kind::Email, look, Arc::new(KeysAsText)).name("m");
        let tree = layout(f.build(), 600.0, 200.0);
        let (_, h) = size_of(&tree, "m-local");
        assert!(
            (h - look.style.heights.control).abs() < 0.6,
            "{}: input is {h}, style says {}",
            id.name(),
            look.style.heights.control
        );
    }
}

fn type_into(
    tree: &mut blinc_layout::prelude::RenderTree,
    input: &mut Input,
    id: &str,
    text: &str,
) {
    input.click(tree, id);
    for ch in text.chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        common::frame(tree, 600.0, 200.0);
    }
}

#[test]
fn typing_an_at_into_the_first_part_moves_the_rest_to_the_domain() {
    let _g = serial();
    let look = Look::new(StyleId::Admin, false);
    let seen = Arc::new(std::sync::Mutex::new(String::new()));
    let sink = Arc::clone(&seen);
    let f = FieldInput::new(Kind::Email, look, Arc::new(KeysAsText))
        .name("m")
        .on_change(move |t| *sink.lock().unwrap() = t.to_string());
    let mut tree = layout(f.build(), 600.0, 200.0);
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "m-local", "ada@exa");
    assert_eq!(f.parts(), ["ada", "exa"]);
    assert_eq!(*seen.lock().unwrap(), "ada@exa");
    type_into(&mut tree, &mut input, "m-domain", "mple.org");
    assert_eq!(f.text(), "ada@example.org");
    assert_eq!(f.problem(), None);
}

#[test]
fn an_at_typed_into_the_first_part_suggests_for_the_domain_at_once() {
    let _g = serial();
    let list = SuggestFromList {
        field: "email",
        segment: 1,
        seen: vec!["example.org".into()],
    };
    let f = FieldInput::new(
        Kind::Email,
        Look::new(StyleId::Admin, false),
        Arc::new(KeysAsText),
    )
    .name("m")
    .suggest(Arc::new(list));
    let mut tree = layout(f.build(), 600.0, 200.0);
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "m-local", "ada@ex");
    common::frame(&mut tree, 600.0, 200.0);
    assert_eq!(f.parts(), ["ada", "ex"]);
    assert!(tree.query_by_id("m-suggest-0").is_some());
}

#[test]
fn a_wrong_value_shows_its_sentence_only_after_a_change_and_a_suggestion_fills_the_part() {
    let _g = serial();
    let look = Look::new(StyleId::Windows, false);
    let list = SuggestFromList {
        field: "email",
        segment: 1,
        seen: vec![
            "example.org".into(),
            "example.org".into(),
            "example.net".into(),
        ],
    };
    let seen = Arc::new(std::sync::Mutex::new(String::new()));
    let sink = Arc::clone(&seen);
    let f = FieldInput::new(Kind::Email, look, Arc::new(KeysAsText))
        .name("m")
        .suggest(Arc::new(list))
        .on_change(move |t| *sink.lock().unwrap() = t.to_string());
    let mut tree = layout(f.build(), 600.0, 200.0);
    assert!(
        tree.query_by_id("m-problem").is_none(),
        "an untouched form shows no error"
    );
    assert!(
        tree.query_by_id("m-suggest-0").is_none(),
        "nothing typed, nothing suggested"
    );
    let mut input = Input::new();
    type_into(&mut tree, &mut input, "m-local", "ada");
    common::frame(&mut tree, 600.0, 200.0);
    assert!(
        tree.query_by_id("m-problem").is_some(),
        "after a change the missing domain is said"
    );
    type_into(&mut tree, &mut input, "m-domain", "ex");
    common::frame(&mut tree, 600.0, 200.0);
    assert!(
        tree.query_by_id("m-suggest-0").is_some(),
        "a suggestion after typing"
    );
    input.click(&mut tree, "m-suggest-0");
    common::frame(&mut tree, 600.0, 200.0);
    assert_eq!(f.parts(), ["ada", "example.org"]);
    assert_eq!(
        *seen.lock().unwrap(),
        "ada@example.org",
        "the owner is told about a click too"
    );
    assert!(
        tree.query_by_id("m-problem").is_none(),
        "the sentence goes when the value is right"
    );
}

#[test]
fn a_key_costs_little_in_our_code_with_the_whole_form_on_screen() {
    let _g = serial();
    let look = Look::new(StyleId::Admin, false);
    let say: Arc<dyn Say> = Arc::new(KeysAsText);
    let fields: Vec<FieldInput> = Kind::ALL
        .into_iter()
        .map(|k| FieldInput::new(k, look, Arc::clone(&say)).name(k.field().key()))
        .collect();
    let mut page = blinc_layout::div::div().w(900.0).flex_col();
    for f in &fields {
        page = page.child(f.build());
    }
    let mut tree = layout(page, 900.0, 700.0);
    let mut input = Input::new();
    input.click(&mut tree, "email-local");
    let start = std::time::Instant::now();
    let keys = 40;
    for _ in 0..keys {
        tree.broadcast_text_input_event('a', false, false, false, false);
        common::frame(&mut tree, 900.0, 700.0);
    }
    let per_key = start.elapsed().as_secs_f64() * 1000.0 / f64::from(keys);
    println!(
        "MEASURE one key with 5 fields (debug build): {per_key:.2} ms (handler, redraw of the hint area, layout)"
    );
    assert_eq!(fields[0].parts()[0].len(), 40);
    assert!(per_key < 50.0, "{per_key} ms per key");
}
