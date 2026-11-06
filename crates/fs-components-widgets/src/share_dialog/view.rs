//! The share dialog as elements.
//!
//! Pattern: Adapter — the state in, Blinc layout, `blinc_cn` buttons and our search field out.
//! The filter field sits outside the part that is drawn again, so typing never loses focus.

use std::sync::Arc;

use blinc_cn::components::button::{ButtonBuilder, ButtonSize, ButtonVariant};
use blinc_layout::div::{Div, div};
use blinc_layout::prelude::text;

use super::handle::ShareDialog;
use super::model::Outcome;
use super::state::Sharing;
use crate::Element;
use crate::keys::bind_keys;
use crate::sortable_list::window;

impl ShareDialog {
    /// The element to place in a layout (the frontend puts it in a modal).
    pub fn build(&self) -> impl Element + use<> {
        let this = self.clone();
        let root = div()
            .id(&self.id)
            .class("fs-sd")
            .child(text(self.labels.title.clone()).class("fs-sd-title"))
            .child(self.field.build())
            .child(self.core.live.view(move || this.render()));
        if self.bindings.is_none() {
            return root;
        }
        let this = self.clone();
        bind_keys(
            root,
            Arc::new(move |event| this.press(&event) != Outcome::Ignored),
        )
    }

    fn render(&self) -> Div {
        self.core.read(|state| {
            let l = &self.labels;
            let visible = state.visible();
            let focus = state
                .cursor_key()
                .and_then(|k| visible.iter().position(|&i| state.choices[i].key == k));
            let rows = window(visible.len(), focus, self.max_rows);
            let mut list = div().class("fs-sd-list");
            for &i in &visible[rows.clone()] {
                list = list.child(self.row(state, i));
            }
            if state.choices.is_empty() {
                list = list.child(text(l.nothing_offered.clone()).class("fs-sd-note"));
            } else if visible.is_empty() {
                list = list.child(text(l.none_found.clone()).class("fs-sd-note"));
            }
            let hidden = visible.len() - rows.len();
            if hidden > 0 {
                let note = l.more.replacen("{}", &hidden.to_string(), 1);
                list = list.child(text(note).class("fs-sd-note"));
            }
            // Chosen rows the window or the filter keeps out of sight are said, not forgotten.
            let drawn = visible[rows.clone()]
                .iter()
                .filter(|&&i| {
                    state
                        .chosen
                        .iter()
                        .any(|g| g.choice == state.choices[i].key)
                })
                .count();
            let unseen = state.chosen.len() - drawn;
            if unseen > 0 {
                let note = l.chosen_hidden.replacen("{}", &unseen.to_string(), 1);
                list = list.child(text(note).class("fs-sd-note"));
            }
            let count = l.chosen.replacen("{}", &state.chosen.len().to_string(), 1);
            let buttons = div()
                .class("fs-sd-buttons")
                .child(self.action("cancel", &l.cancel, ButtonVariant::Ghost))
                .child(self.action("confirm", &l.confirm, ButtonVariant::Primary));
            div().w_full().flex_col().gap_px(12.0).child(list).child(
                div()
                    .class("fs-sd-foot")
                    .child(text(count).class("fs-sd-note"))
                    .child(buttons),
            )
        })
    }

    fn row(&self, state: &Sharing, index: usize) -> Div {
        let choice = &state.choices[index];
        let grant = state.chosen.iter().find(|g| g.choice == choice.key);
        let mut row = div()
            .class("fs-sd-row")
            .id(format!("{}-row-{}", self.id, choice.key));
        if grant.is_some() {
            row = row.class("fs-sd-row-on");
        }
        if state.cursor_key() == Some(choice.key.as_str()) {
            row = row.class("fs-sd-row-cursor");
        }
        let mark = if grant.is_some() {
            div().class("fs-sd-mark").class("fs-sd-mark-on")
        } else {
            div().class("fs-sd-mark")
        };
        let meta = [choice.kind.as_str(), choice.detail.as_str()]
            .iter()
            .filter(|t| !t.is_empty())
            .copied()
            .collect::<Vec<_>>()
            .join(" - ");
        let mut who = div()
            .class("fs-sd-who")
            .child(text(choice.name.clone()).class("fs-sd-name"));
        if !meta.is_empty() {
            who = who.child(text(meta).class("fs-sd-meta"));
        }
        // Only this part toggles: a click on a permission button must not unchoose the row.
        let (this, key) = (self.clone(), choice.key.clone());
        let pick = div()
            .class("fs-sd-pick")
            .id(format!("{}-pick-{}", self.id, choice.key))
            .child(mark)
            .child(who)
            .on_click(move |_| {
                this.toggle(&key);
            });
        row = row.child(pick);
        if let Some(grant) = grant {
            row = row.child(self.permissions(&choice.key, grant.permission.as_deref(), state));
        }
        row
    }

    fn permissions(&self, key: &str, current: Option<&str>, state: &Sharing) -> Div {
        let mut perms = div().class("fs-sd-perms");
        for p in &state.permissions {
            let id = format!("{}-perm-{key}-{}", self.id, p.key);
            let variant = if Some(p.key.as_str()) == current {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            };
            let (this, k, pk) = (self.clone(), key.to_string(), p.key.clone());
            perms = perms.child(
                ButtonBuilder::with_key(id.clone(), p.label.clone())
                    .variant(variant)
                    .size(ButtonSize::Small)
                    .on_click(move |_| {
                        this.set_permission(&k, &pk);
                    })
                    .build_component()
                    .id(&id),
            );
        }
        perms
    }

    fn action(&self, tag: &str, word: &str, variant: ButtonVariant) -> impl Element + use<> {
        let id = format!("{}-{tag}", self.id);
        let this = self.clone();
        let confirm = tag == "confirm";
        ButtonBuilder::with_key(id.clone(), word.to_string())
            .variant(variant)
            .on_click(move |_| {
                if confirm {
                    this.confirm();
                } else {
                    this.cancel();
                }
            })
            .build_component()
            .id(&id)
    }
}
