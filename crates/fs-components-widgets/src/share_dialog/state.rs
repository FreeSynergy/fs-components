//! The state of the dialog: what is offered, what is chosen, what is typed.
//!
//! Pattern: State machine, pure — no Blinc. Choices that are not in the offered list (given as
//! already chosen) are kept and returned on confirm, never dropped.

use super::model::{Choice, Grant, Outcome, Permission, Refusal};

/// Choices, permissions, the pairs chosen, the filter and the cursor.
#[derive(Clone, Debug, Default)]
pub struct Sharing {
    pub(super) choices: Vec<Choice>,
    pub(super) permissions: Vec<Permission>,
    pub(super) chosen: Vec<Grant>,
    pub(super) filter: String,
    pub(super) cursor: Option<String>,
    pub(super) require_choice: bool,
}

impl Sharing {
    /// A dialog state over these choices and permissions; nothing chosen.
    pub fn new(choices: Vec<Choice>, permissions: Vec<Permission>) -> Self {
        Self {
            choices,
            permissions,
            ..Self::default()
        }
    }

    /// Start with these pairs chosen (a permission the caller does not offer becomes the first).
    pub fn choose(&mut self, grants: Vec<Grant>) {
        for g in grants {
            let permission = self.valid_permission(g.permission);
            match self.chosen.iter_mut().find(|c| c.choice == g.choice) {
                Some(c) => c.permission = permission,
                None => self.chosen.push(Grant::new(g.choice, permission)),
            }
        }
    }

    /// Confirm only with at least one pair chosen.
    pub fn require_choice(&mut self, require: bool) {
        self.require_choice = require;
    }

    fn valid_permission(&self, wanted: Option<String>) -> Option<String> {
        let known = |k: &String| self.permissions.iter().any(|p| &p.key == k);
        wanted
            .filter(known)
            .or_else(|| self.permissions.first().map(|p| p.key.clone()))
    }

    /// The pairs chosen, in order.
    pub fn chosen(&self) -> &[Grant] {
        &self.chosen
    }

    /// The text typed to narrow the list.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Indexes of the choices that fit the filter (case-insensitive, in name, kind and detail).
    pub fn visible(&self) -> Vec<usize> {
        let needle = self.filter.trim().to_lowercase();
        (0..self.choices.len())
            .filter(|&i| {
                let c = &self.choices[i];
                needle.is_empty()
                    || [&c.name, &c.kind, &c.detail]
                        .iter()
                        .any(|t| t.to_lowercase().contains(&needle))
            })
            .collect()
    }

    /// Key of the choice under the cursor (always one of the visible ones, if there are any).
    pub fn cursor_key(&self) -> Option<&str> {
        let visible = self.visible();
        let at = self
            .cursor
            .as_deref()
            .and_then(|k| visible.iter().find(|&&i| self.choices[i].key == k))
            .or(visible.first())?;
        Some(self.choices[*at].key.as_str())
    }

    /// Narrow the list.
    pub fn set_filter(&mut self, text: &str) -> Outcome {
        self.filter = text.to_string();
        Outcome::Changed
    }

    /// Choose or unchoose; a new pair gets the first permission.
    pub fn toggle(&mut self, key: &str) -> Outcome {
        if !self.choices.iter().any(|c| c.key == key) {
            return Outcome::Refused(Refusal::NoSuchChoice);
        }
        if let Some(i) = self.chosen.iter().position(|g| g.choice == key) {
            self.chosen.remove(i);
        } else {
            let permission = self.valid_permission(None);
            self.chosen.push(Grant::new(key, permission));
        }
        self.cursor = Some(key.to_string());
        Outcome::Changed
    }

    /// Give a chosen row another permission.
    pub fn set_permission(&mut self, key: &str, permission: &str) -> Outcome {
        if !self.permissions.iter().any(|p| p.key == permission) {
            return Outcome::Refused(Refusal::NoSuchPermission);
        }
        match self.chosen.iter_mut().find(|g| g.choice == key) {
            Some(g) => {
                g.permission = Some(permission.to_string());
                Outcome::Changed
            }
            None if self.choices.iter().any(|c| c.key == key) => {
                Outcome::Refused(Refusal::NotChosen)
            }
            None => Outcome::Refused(Refusal::NoSuchChoice),
        }
    }

    /// The next or previous permission of a chosen row (stops at the ends).
    pub fn step_permission(&mut self, key: &str, forward: bool) -> Outcome {
        let Some(g) = self.chosen.iter().find(|g| g.choice == key) else {
            return Outcome::Refused(Refusal::NotChosen);
        };
        let at = self
            .permissions
            .iter()
            .position(|p| Some(&p.key) == g.permission.as_ref())
            .unwrap_or(0);
        let to = if forward { at + 1 } else { at.wrapping_sub(1) };
        match self.permissions.get(to).map(|p| p.key.clone()) {
            Some(next) => self.set_permission(key, &next),
            None => Outcome::Refused(Refusal::Empty),
        }
    }

    /// Move the cursor over the visible rows.
    pub fn move_cursor(&mut self, delta: isize) -> Outcome {
        let visible = self.visible();
        if visible.is_empty() {
            return Outcome::Refused(Refusal::Empty);
        }
        let now = self
            .cursor_key()
            .and_then(|k| visible.iter().position(|&i| self.choices[i].key == k))
            .unwrap_or(0);
        let to = now.saturating_add_signed(delta).min(visible.len() - 1);
        self.cursor = Some(self.choices[visible[to]].key.clone());
        Outcome::Changed
    }

    /// The person is done: all pairs chosen, also those hidden by the filter.
    pub fn confirm(&self) -> Outcome {
        if self.require_choice && self.chosen.is_empty() {
            return Outcome::Refused(Refusal::NothingChosen);
        }
        Outcome::Confirmed(self.chosen.clone())
    }
}
