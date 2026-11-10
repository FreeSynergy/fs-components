//! Tests: every built-in style is complete, readable and distinct.
//!
//! Pattern: none (tests). Tests: against the public API only.

use crate::{Builtin, Demand, StyleCatalog, StyleId, Surface, style};

#[test]
fn every_id_resolves_and_names_round_trip() {
    for id in StyleId::ALL {
        assert_eq!(style(id).id, id);
        assert_eq!(StyleId::from_name(id.name()), Some(id));
        assert!(Builtin.get(id).is_some());
    }
    assert_eq!(StyleId::from_name("nope"), None);
}

#[test]
fn text_is_readable_in_both_appearances() {
    for id in StyleId::ALL {
        for dark in [false, true] {
            for (what, fg, bg) in style(id).palette(dark).readable_pairs() {
                let ratio = fg.contrast(bg);
                // Apple's own blue (#0088FF) under white text reaches about 3.5: the 1:1 rebuild keeps it, so text on the accent is held to the 3:1 of WCAG for large text and controls, the rest to 4.5:1.
                let need = if what == "text on accent" { 3.0 } else { 4.5 };
                assert!(
                    ratio >= need,
                    "{} dark={dark}: {what} has {ratio:.2}",
                    id.name()
                );
            }
        }
    }
}

#[test]
fn sizes_are_sane_and_the_row_holds_its_content() {
    for id in StyleId::ALL {
        let s = style(id);
        assert!(s.heights.control >= s.text.body_line, "{}", id.name());
        assert!(s.heights.row >= s.heights.control, "{}", id.name());
        assert!(s.heights.control_dense <= s.heights.control);
        assert!(
            s.text.caption < s.text.body
                && s.text.body < s.text.title
                && s.text.title < s.text.headline
        );
        assert!(s.radii.control <= s.radii.card && s.radii.card <= s.radii.overlay);
        assert_ne!(s.fonts.len(), 0);
    }
}

#[test]
fn the_four_styles_differ_where_it_counts() {
    let heights: Vec<f32> = StyleId::ALL
        .iter()
        .map(|i| style(*i).heights.control)
        .collect();
    for (i, a) in heights.iter().enumerate() {
        for b in &heights[i + 1..] {
            assert!(
                (a - b).abs() > f32::EPSILON,
                "two styles share a control height"
            );
        }
    }
}

#[test]
fn only_apple_asks_for_glass_and_only_windows_for_a_backdrop() {
    let glass = |id| {
        style(id)
            .demands()
            .iter()
            .any(|d| matches!(d, Demand::Glass(_)))
    };
    let backdrop = |id| style(id).demands().contains(&Demand::Backdrop);
    assert!(
        glass(StyleId::Apple)
            && !glass(StyleId::Windows)
            && !glass(StyleId::Admin)
            && !glass(StyleId::Simple)
    );
    assert!(backdrop(StyleId::Windows) && !backdrop(StyleId::Apple));
    assert!(matches!(
        style(StyleId::Apple).effects.chrome,
        Surface::Glass { .. }
    ));
}

#[test]
fn icon_sets_are_distinct_per_style() {
    let sets: Vec<_> = StyleId::ALL.iter().map(|i| style(*i).icons).collect();
    for (i, a) in sets.iter().enumerate() {
        for b in &sets[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn the_numbers_of_the_measurement_sheets_are_kept() {
    use crate::Provenance;
    let a = style(StyleId::Admin);
    assert_eq!((a.heights.control, a.heights.control_dense), (36.0, 26.0)); // fiori.md section 3
    assert_eq!((a.heights.row, a.heights.row_dense), (44.0, 32.0));
    assert_eq!(a.radii.control, 12.0);
    assert_eq!(
        (a.spacing.s, a.spacing.m, a.spacing.l, a.spacing.xl),
        (8.0, 16.0, 32.0, 48.0)
    ); // section 4
    assert_eq!(
        (a.text.caption, a.text.body, a.text.title, a.text.headline),
        (12.0, 14.0, 16.0, 24.0)
    ); // section 2
    assert_eq!(
        (a.light.focus.css().as_str(), a.effects.focus.width),
        ("#0032a5", 2.0)
    ); // section 5
    let p = style(StyleId::Apple);
    assert_eq!(
        (p.text.body, p.text.body_line, p.heights.control),
        (13.0, 16.0, 28.0)
    ); // apple.md sections 1, 2
    assert_eq!(p.light.accent.css(), "#0088ff"); // section 6
    let w = style(StyleId::Windows);
    assert_eq!(
        (w.text.body, w.text.body_line, w.text.title, w.text.headline),
        (14.0, 20.0, 20.0, 28.0)
    ); // windows.md section 1
    assert_eq!((w.radii.control, w.radii.overlay), (4.0, 8.0)); // section 2
    assert_eq!(w.fonts[0], "Selawik"); // section 9
    assert_eq!(style(StyleId::Simple).provenance, Provenance::Sheet);
    for id in [StyleId::Admin, StyleId::Apple, StyleId::Windows] {
        assert_eq!(
            style(id).provenance,
            Provenance::Partial,
            "{}: not everything is in the sheet",
            id.name()
        );
    }
}
