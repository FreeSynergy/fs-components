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
    pub(super) default_permission: Option<String>,
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

    /// Start with these pairs chosen. A permission is kept exactly as given, also one that is not
    /// offered (it stays visibly unknown and is returned unchanged unless the person picks
    /// another): the dialog never turns a grant into a different one by itself.
    pub fn choose(&mut self, grants: Vec<Grant>) {
        for g in grants {
            match self.chosen.iter_mut().find(|c| c.choice == g.choice) {
                Some(c) => c.permission = g.permission,
                None => self.chosen.push(g),
            }
        }
    }

    /// Confirm only with at least one pair chosen.
    pub fn require_choice(&mut self, require: bool) {
        self.require_choice = require;
    }

    /// The permission a newly chosen pair gets: the least-privilege one, named by the caller.
    /// Without one (or if it is not offered) a new pair has no permission until the person picks
    /// one, and confirming is refused until then.
    pub fn set_default_permission(&mut self, key: Option<String>) {
        self.default_permission = key;
    }

    fn default_for_new(&self) -> Option<String> {
        self.default_permission
            .clone()
            .filter(|k| self.permissions.iter().any(|p| &p.key == k))
    }

    /// Is this permission one of those offered?
    pub fn offers(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p.key == permission)
    }

    /// Chosen pairs whose choice is not offered (given as already chosen). They are shown, can be
    /// removed, and are returned on confirm unless removed.
    pub fn unlisted(&self) -> Vec<&Grant> {
        self.chosen
            .iter()
            .filter(|g| !self.choices.iter().any(|c| c.key == g.choice))
            .collect()
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

    /// Choose or unchoose; a new pair gets the caller's default permission, or none yet.
    pub fn toggle(&mut self, key: &str) -> Outcome {
        let known = self.choices.iter().any(|c| c.key == key);
        let held = self.chosen.iter().position(|g| g.choice == key);
        match (held, known) {
            (Some(i), _) => {
                self.chosen.remove(i);
            }
            (None, true) => {
                let permission = self.default_for_new();
                self.chosen.push(Grant::new(key, permission));
            }
            (None, false) => return Outcome::Refused(Refusal::NoSuchChoice),
        }
        if known {
            self.cursor = Some(key.to_string());
        }
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

    /// The next or previous permission of a chosen row (stops at the ends). A row without a
    /// known permission goes to the first (forward) or last (back) one offered: the person asked.
    pub fn step_permission(&mut self, key: &str, forward: bool) -> Outcome {
        let Some(g) = self.chosen.iter().find(|g| g.choice == key) else {
            return Outcome::Refused(Refusal::NotChosen);
        };
        let at = self
            .permissions
            .iter()
            .position(|p| Some(&p.key) == g.permission.as_ref());
        let to = match (at, forward) {
            (None, true) => Some(0),
            (None, false) => self.permissions.len().checked_sub(1),
            (Some(at), true) => Some(at + 1),
            (Some(at), false) => at.checked_sub(1),
        };
        match to
            .and_then(|i| self.permissions.get(i))
            .map(|p| p.key.clone())
        {
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
        if !self.permissions.is_empty() && self.chosen.iter().any(|g| g.permission.is_none()) {
            return Outcome::Refused(Refusal::PermissionMissing);
        }
        Outcome::Confirmed(self.chosen.clone())
    }
}
