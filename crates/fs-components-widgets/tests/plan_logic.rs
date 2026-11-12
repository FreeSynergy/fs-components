//! The plan widgets without a window: sizes, what Enter hands over, the heading of a column, the tones.
//!
//! Tests: pure functions and data. Not covered here (a harness limit, named): the drawing and the clicks of the
//! board, the tabs, the entry line and the panel; they were seen in pictures of the tasks program, not asserted.

use fs_components_widgets::plan::{
    Chip, Column, Events, PlanCard, Tab, Tone, board_layout, submitted,
};

#[test]
fn the_panel_exists_only_when_a_card_is_chosen() {
    let (side, column, none) = board_layout(1400.0, false);
    let (_, _, some) = board_layout(1400.0, true);
    assert_eq!((side, none), (220.0, 0.0));
    assert!(some > 0.0 && some <= 360.0);
    assert!(
        (260.0..=280.0).contains(&column),
        "a column is 260 to 280 px wide: {column}"
    );
}

#[test]
fn a_narrow_window_drops_the_project_list_and_keeps_the_panel_in_bounds() {
    let (side, _, panel) = board_layout(600.0, true);
    assert_eq!(side, 0.0);
    assert!(panel <= 600.0 * 0.45);
}

#[test]
fn enter_hands_over_trimmed_text_and_never_an_empty_line() {
    assert_eq!(
        submitted("  Call Zoë about the Größe  "),
        Some("Call Zoë about the Größe".into())
    );
    assert_eq!(
        submitted("€ { } [ ] \\ | ~ @ µ"),
        Some("€ { } [ ] \\ | ~ @ µ".into())
    );
    assert_eq!(submitted("   "), None);
    assert_eq!(submitted(""), None);
}

#[test]
fn a_column_heads_with_its_name_and_its_count() {
    let mut c = Column::new("todo", "To do");
    assert_eq!(c.head(), ("To do".into(), "0".into()));
    c.cards.push(PlanCard::new("a", "A"));
    assert_eq!(c.head().1, "1");
}

#[test]
fn a_locked_tab_is_marked_and_a_working_one_is_not() {
    assert!(Tab::new("calendar", "Calendar").locked().locked);
    assert!(!Tab::new("board", "Board").locked);
}

#[test]
fn every_tone_has_its_own_class() {
    let tones = [
        Tone::Neutral,
        Tone::Accent,
        Tone::Warning,
        Tone::Danger,
        Tone::Success,
    ];
    let classes: std::collections::BTreeSet<_> = tones.iter().map(|t| t.class()).collect();
    assert_eq!(classes.len(), tones.len());
    assert_eq!(Chip::new("x", Tone::Danger).tone, Tone::Danger);
    assert!(
        Events::default().pick.is_none(),
        "a missing callback does nothing"
    );
}
