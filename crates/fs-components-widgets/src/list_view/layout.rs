//! What a person decides about the columns of a list: which are shown, in what order, how wide.
//!
//! Pattern: Value object (+ Memento through `to_text` / `from_text`) — the caller gives the columns and their default
//! order, the person changes them, and the result is plain text that a store keeps (see `layout_store.rs`; ADR 0069:
//! the key is the view, the value says order, hidden and widths, never a position on the screen). The list has no data
//! of its own and no idea what a column means: a column is a key. Rules: a column the caller marks as not hideable (the
//! name) stays; at least one column stays shown; a width has a least value; a key the layout does not know is
//! dropped when read back, a new column of the caller is appended.

use std::collections::{BTreeMap, BTreeSet};

/// The narrowest a column can be made, in pixels.
pub const LEAST_WIDTH: f32 = 48.0;
/// The same in whole pixels.
const LEAST_WIDTH_PX: u32 = 48;
/// The widest column a stored text may give (the same bound `set_width` has).
const MOST_WIDTH_PX: u32 = 100_000;

/// Order, hidden columns and widths of the columns of one list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColumnLayout {
    defaults: Vec<String>,
    order: Vec<String>,
    hidden: BTreeSet<String>,
    widths: BTreeMap<String, u32>,
    fixed: BTreeSet<String>,
}

impl ColumnLayout {
    /// The default order of the columns, by key.
    pub fn new<S: AsRef<str>>(keys: &[S]) -> Self {
        let order: Vec<String> = keys.iter().map(|k| k.as_ref().to_string()).collect();
        Self {
            defaults: order.clone(),
            order,
            ..Self::default()
        }
    }

    /// This column cannot be hidden (the name of a thing).
    #[must_use]
    pub fn keep_shown(mut self, key: &str) -> Self {
        self.fixed.insert(key.to_string());
        self.hidden.remove(key);
        self
    }

    /// The keys in the order they are shown, hidden ones included.
    pub fn order(&self) -> &[String] {
        &self.order
    }

    /// The keys that are shown, in order.
    pub fn shown(&self) -> Vec<&str> {
        self.order
            .iter()
            .filter(|k| self.is_shown(k))
            .map(String::as_str)
            .collect()
    }

    /// Is this column shown?
    pub fn is_shown(&self, key: &str) -> bool {
        !self.hidden.contains(key)
    }

    /// May this column be hidden now? Not the ones the caller keeps, and not the last one shown.
    pub fn can_hide(&self, key: &str) -> bool {
        !self.fixed.contains(key) && (!self.is_shown(key) || self.shown().len() > 1)
    }

    /// Show a hidden column or hide a shown one; `false` if that is refused (see [`ColumnLayout::can_hide`]).
    pub fn toggle(&mut self, key: &str) -> bool {
        if !self.order.iter().any(|k| k == key) {
            return false;
        }
        if self.is_shown(key) {
            if !self.can_hide(key) {
                return false;
            }
            self.hidden.insert(key.to_string());
        } else {
            self.hidden.remove(key);
        }
        true
    }

    /// Move a column by `delta` places among all columns (negative = to the left); stops at the ends.
    pub fn move_by(&mut self, key: &str, delta: isize) {
        let Some(from) = self.order.iter().position(|k| k == key) else {
            return;
        };
        let last = self.order.len().saturating_sub(1);
        let to = from.saturating_add_signed(delta).min(last);
        let moved = self.order.remove(from);
        self.order.insert(to, moved);
    }

    /// Put a column where `target` is (the drag of a heading onto another one).
    pub fn move_to(&mut self, key: &str, target: &str) {
        let (Some(from), Some(to)) = (
            self.order.iter().position(|k| k == key),
            self.order.iter().position(|k| k == target),
        ) else {
            return;
        };
        let moved = self.order.remove(from);
        self.order.insert(to, moved);
    }

    /// Give a column this width (not less than [`LEAST_WIDTH`]).
    pub fn set_width(&mut self, key: &str, pixels: f32) {
        // Whole pixels, 0..=100000: the cast cannot lose more than the fraction.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let px = pixels.clamp(LEAST_WIDTH, 100_000.0).round() as u32;
        self.widths.insert(key.to_string(), px);
    }

    /// The width the person gave, if any.
    #[allow(clippy::cast_precision_loss)]
    pub fn width_of(&self, key: &str) -> Option<f32> {
        self.widths.get(key).map(|w| *w as f32)
    }

    /// Forget the width of a column (it goes back to what the caller says).
    pub fn forget_width(&mut self, key: &str) {
        self.widths.remove(key);
    }

    /// Back to the order and the shown columns the caller gave, with no widths.
    pub fn reset(&mut self) {
        self.order = self.defaults.clone();
        self.hidden.clear();
        self.widths.clear();
    }

    /// The layout as text for a store: `order=a,b,c;hidden=b;width.a=120`.
    pub fn to_text(&self) -> String {
        let mut parts = vec![format!("order={}", self.order.join(","))];
        if !self.hidden.is_empty() {
            let hidden: Vec<&str> = self.hidden.iter().map(String::as_str).collect();
            parts.push(format!("hidden={}", hidden.join(",")));
        }
        parts.extend(self.widths.iter().map(|(k, w)| format!("width.{k}={w}")));
        parts.join(";")
    }

    /// This layout with what `text` says. What the text names and the layout does not have is dropped; a column the
    /// text does not name stays where it is, after the others; a column that may not be hidden is shown; if all would be
    /// hidden the first is shown. Text that does not make sense changes nothing.
    #[must_use]
    pub fn with_text(mut self, text: &str) -> Self {
        let known: BTreeSet<String> = self.defaults.iter().cloned().collect();
        let mut order: Vec<String> = Vec::new();
        let (mut hidden, mut widths) = (BTreeSet::new(), BTreeMap::new());
        for part in text.split(';') {
            let Some((name, value)) = part.split_once('=') else {
                continue;
            };
            match name {
                "order" => {
                    for k in value.split(',').filter(|k| known.contains(*k)) {
                        if !order.iter().any(|o| o == k) {
                            order.push(k.to_string());
                        }
                    }
                }
                "hidden" => hidden.extend(
                    value
                        .split(',')
                        .filter(|k| known.contains(*k))
                        .map(String::from),
                ),
                _ => {
                    if let Some(key) = name.strip_prefix("width.")
                        && known.contains(key)
                        && let Ok(w) = value.parse::<u32>()
                    {
                        widths.insert(key.to_string(), w.clamp(LEAST_WIDTH_PX, MOST_WIDTH_PX));
                    }
                }
            }
        }
        if order.is_empty() {
            return self;
        }
        let rest: Vec<String> = self
            .order
            .iter()
            .filter(|k| !order.contains(k))
            .cloned()
            .collect();
        order.extend(rest);
        hidden.retain(|k| !self.fixed.contains(k));
        if order.iter().all(|k| hidden.contains(k)) {
            hidden.remove(&order[0]);
        }
        self.order = order;
        self.hidden = hidden;
        self.widths = widths;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> ColumnLayout {
        ColumnLayout::new(&["name", "size", "modified", "type"]).keep_shown("name")
    }

    #[test]
    fn columns_are_hidden_and_shown_and_the_name_and_the_last_one_stay() {
        let mut l = layout();
        assert!(l.toggle("size") && !l.is_shown("size"));
        assert_eq!(l.shown(), ["name", "modified", "type"]);
        assert!(!l.toggle("name"), "the caller keeps the name");
        assert!(l.toggle("size") && l.is_shown("size"));
        let mut one = ColumnLayout::new(&["a", "b"]);
        assert!(one.toggle("a"));
        assert!(!one.toggle("b"), "the last column stays");
        assert_eq!(one.shown(), ["b"]);
        assert!(!l.toggle("nothing"), "an unknown key changes nothing");
    }

    #[test]
    fn columns_move_by_places_and_onto_each_other_and_stop_at_the_ends() {
        let mut l = layout();
        l.move_by("type", -1);
        assert_eq!(l.order(), ["name", "size", "type", "modified"]);
        l.move_by("name", 9);
        assert_eq!(l.order(), ["size", "type", "modified", "name"]);
        l.move_by("size", -5);
        assert_eq!(l.order()[0], "size");
        l.move_to("name", "type");
        assert_eq!(l.order(), ["size", "name", "type", "modified"]);
    }

    #[test]
    fn a_width_has_a_least_value_and_can_be_forgotten() {
        let mut l = layout();
        l.set_width("size", 10.0);
        assert_eq!(l.width_of("size"), Some(LEAST_WIDTH));
        l.set_width("size", 130.4);
        assert_eq!(l.width_of("size"), Some(130.0));
        l.forget_width("size");
        assert_eq!(l.width_of("size"), None);
    }

    #[test]
    fn a_stored_width_is_bounded_both_ways_when_the_text_is_read() {
        // Red without the upper bound: 4294967295 px came back as it was stored.
        let read = layout().with_text("order=name,size;width.name=4294967295;width.size=3");
        assert_eq!(read.width_of("name"), Some(100_000.0));
        assert_eq!(read.width_of("size"), Some(LEAST_WIDTH));
    }

    #[test]
    fn the_text_gives_the_layout_back_and_survives_a_changed_list_of_columns() {
        let mut l = layout();
        l.toggle("modified");
        l.move_by("type", -2);
        l.set_width("name", 220.0);
        let text = l.to_text();
        assert_eq!(layout().with_text(&text), l, "{text}");
        // the caller has a new column and no longer has `type`: the new one is appended, the old one is dropped
        let newer = ColumnLayout::new(&["name", "size", "modified", "owner"]).keep_shown("name");
        let read = newer.with_text(&text);
        assert_eq!(read.order(), ["name", "size", "modified", "owner"]);
        assert!(!read.is_shown("modified") && read.width_of("name") == Some(220.0));
        // nonsense changes nothing, and a text that hides everything still shows one column
        assert_eq!(layout().with_text("???"), layout());
        let all = ColumnLayout::new(&["a", "b"]).with_text("order=a,b;hidden=a,b");
        assert_eq!(all.shown().len(), 1);
        let kept = layout().with_text("order=name,size;hidden=name");
        assert!(
            kept.is_shown("name"),
            "a column the caller keeps cannot be hidden by a text either"
        );
    }

    #[test]
    fn reset_goes_back_to_what_the_caller_gave() {
        let mut l = layout();
        l.toggle("type");
        l.move_by("size", 2);
        l.set_width("name", 300.0);
        l.reset();
        assert_eq!(l, layout());
    }
}
