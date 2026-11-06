//! The handle of a share dialog.
//!
//! Pattern: Facade + Observer — a clonable handle over the [`Sharing`] state; no method returns a
//! lock guard; every step answers with an [`Outcome`], tells the callback and draws again. The
//! filter field keeps its text in its own shared state, outside the part that is drawn again.

use std::sync::{Arc, Mutex, PoisonError};

use fs_components_primitives::SearchField;

use super::bindings::{Bindings, Command};
use super::model::{Choice, Grant, Outcome, Permission};
use super::state::Sharing;
use crate::keys::KeyEvent;
use crate::live::Live;

/// Called with what every step did.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

/// The words of the dialog.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// The heading.
    pub title: String,
    /// Placeholder of the filter field.
    pub filter: String,
    /// Button: confirm.
    pub confirm: String,
    /// Button: cancel.
    pub cancel: String,
    /// How many pairs are chosen; `{}` = the number (also those hidden by the filter).
    pub chosen: String,
    /// Chosen pairs that are not among the rows drawn (filtered out, beyond the cap, or not
    /// offered any more); `{}` = how many.
    pub chosen_hidden: String,
    /// A chosen pair holds a permission that is not offered; `{}` = its key as the caller gave it.
    pub unknown_permission: String,
    /// A chosen pair has no permission yet.
    pub no_permission: String,
    /// A chosen pair whose choice is not offered (shown so it can be removed); `{}` = its key.
    pub unlisted: String,
    /// No choice fits the filter.
    pub none_found: String,
    /// Nothing is offered at all.
    pub nothing_offered: String,
    /// More rows than are drawn; `{}` = how many.
    pub more: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = |t: &str| t.to_string();
        Self {
            title: s("Share"),
            filter: s("Search"),
            confirm: s("Done"),
            cancel: s("Cancel"),
            chosen: s("{} chosen"),
            chosen_hidden: s("{} chosen not shown here."),
            unknown_permission: s("Unknown: {}"),
            no_permission: s("Choose a permission"),
            unlisted: s("{} (not in the list)"),
            none_found: s("Nobody found."),
            nothing_offered: s("Nobody to choose from."),
            more: s("{} more not shown, narrow the search."),
        }
    }
}

struct Inner {
    state: Sharing,
    on_outcome: Option<OnOutcome>,
}

/// The shared part: state, callback and the redraw handle.
#[derive(Clone)]
pub(super) struct Core {
    shared: Arc<Mutex<Inner>>,
    pub live: Live,
}

impl Core {
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn act(&self, f: impl FnOnce(&mut Sharing) -> Outcome) -> Outcome {
        let (outcome, handler) = {
            let mut inner = self.lock();
            (f(&mut inner.state), inner.on_outcome.clone())
        };
        if let (true, Some(handler)) = (outcome != Outcome::Ignored, handler) {
            handler(&outcome);
        }
        self.live.refresh();
        outcome
    }

    pub fn read<T>(&self, f: impl FnOnce(&Sharing) -> T) -> T {
        f(&self.lock().state)
    }
}

/// A share dialog. Cheap to clone; clones share the state.
#[derive(Clone)]
pub struct ShareDialog {
    pub(super) core: Core,
    pub(super) id: String,
    pub(super) bindings: Option<Bindings>,
    pub(super) labels: Labels,
    pub(super) max_rows: usize,
    pub(super) field: SearchField,
}

impl ShareDialog {
    /// A dialog offering `choices`, each with one of `permissions`.
    pub fn new(choices: Vec<Choice>, permissions: Vec<Permission>) -> Self {
        let core = Core {
            shared: Arc::new(Mutex::new(Inner {
                state: Sharing::new(choices, permissions),
                on_outcome: None,
            })),
            live: Live::new(),
        };
        let labels = Labels::default();
        Self {
            field: Self::field_for(&core, &labels),
            core,
            id: "fs-share".into(),
            bindings: None,
            labels,
            max_rows: 100,
        }
    }

    fn field_for(core: &Core, labels: &Labels) -> SearchField {
        let core = core.clone();
        SearchField::new(&labels.filter).on_change(Arc::new(move |text| {
            core.act(|s| s.set_filter(text));
        }))
    }

    /// The element id (rows are `<id>-row-<key>`, permission buttons `<id>-perm-<key>-<permission>`,
    /// and `<id>-confirm`, `<id>-cancel`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Take keys while the dialog has the keyboard, by these bindings.
    #[must_use]
    pub fn keys(mut self, bindings: Bindings) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.field = Self::field_for(&self.core, &labels);
        self.labels = labels;
        self
    }

    /// Draw at most `rows` rows (the others are counted in a line).
    #[must_use]
    pub fn max_rows(mut self, rows: usize) -> Self {
        self.max_rows = rows;
        self
    }

    /// Start with these pairs chosen.
    #[must_use]
    pub fn choose(self, grants: Vec<Grant>) -> Self {
        self.core.lock().state.choose(grants);
        self
    }

    /// The permission a newly chosen pair gets (the least-privilege one). Without it a new pair
    /// has none until the person picks one, and Done is refused until every pair has one.
    #[must_use]
    pub fn default_permission(self, key: impl Into<String>) -> Self {
        self.core
            .lock()
            .state
            .set_default_permission(Some(key.into()));
        self
    }

    /// Confirm only with at least one pair chosen.
    #[must_use]
    pub fn require_choice(self, require: bool) -> Self {
        self.core.lock().state.require_choice(require);
        self
    }

    /// Hear of every step.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.core.lock().on_outcome = Some(handler);
        self
    }

    /// A copy of the pairs chosen so far.
    pub fn chosen(&self) -> Vec<Grant> {
        self.core.read(|s| s.chosen().to_vec())
    }

    /// Key of the choice under the cursor.
    pub fn cursor_key(&self) -> Option<String> {
        self.core.read(|s| s.cursor_key().map(str::to_owned))
    }

    /// Choose or unchoose.
    pub fn toggle(&self, key: &str) -> Outcome {
        self.core.act(|s| s.toggle(key))
    }

    /// Give a chosen row a permission.
    pub fn set_permission(&self, key: &str, permission: &str) -> Outcome {
        self.core.act(|s| s.set_permission(key, permission))
    }

    /// Narrow the list (the filter field does this by itself when typed into).
    pub fn set_filter(&self, text: &str) -> Outcome {
        self.core.act(|s| s.set_filter(text))
    }

    /// Done: reports [`Outcome::Confirmed`] to the callback.
    pub fn confirm(&self) -> Outcome {
        self.core.act(|s| s.confirm())
    }

    /// Cancel: reports [`Outcome::Cancelled`].
    pub fn cancel(&self) -> Outcome {
        self.core.act(|_| Outcome::Cancelled)
    }

    /// Do one step.
    pub fn command(&self, command: Command) -> Outcome {
        self.core.act(|s| {
            let key = s.cursor_key().map(str::to_owned);
            let on_row = |s: &mut Sharing, f: &dyn Fn(&mut Sharing, &str) -> Outcome| match &key {
                Some(k) => f(s, k),
                None => Outcome::Refused(super::model::Refusal::Empty),
            };
            match command {
                Command::Up => s.move_cursor(-1),
                Command::Down => s.move_cursor(1),
                Command::Toggle => on_row(s, &|s, k| s.toggle(k)),
                Command::PermissionBack => on_row(s, &|s, k| s.step_permission(k, false)),
                Command::PermissionForward => on_row(s, &|s, k| s.step_permission(k, true)),
                Command::Confirm => s.confirm(),
                _ => Outcome::Cancelled,
            }
        })
    }

    /// A key press: through the bindings if there are any ([`Outcome::Ignored`] otherwise).
    pub fn press(&self, event: &KeyEvent) -> Outcome {
        match self.bindings.as_ref().and_then(|b| b.command(event)) {
            Some(command) => self.command(command),
            None => Outcome::Ignored,
        }
    }
}
