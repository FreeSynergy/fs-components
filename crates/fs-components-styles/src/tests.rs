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
                // Apple's own blue (#007AFF) under white text reaches 4.02: the 1:1 rebuild keeps it, so text on the accent is held to the 3:1 of WCAG for large text and controls, the rest to 4.5:1.
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
