//! The shapes the dialog works with.
//!
//! Pattern: none — neutral. The dialog does not know what a person, a group or a
//! permission is: a choice has a key, a name and two texts; a permission has a key and a label.

/// Someone or something that can be chosen.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Choice {
    /// Identity given back in a [`Grant`].
    pub key: String,
    /// The name shown.
    pub name: String,
    /// What kind it is, in the caller's words ("Group"); may be empty.
    pub kind: String,
    /// A second line (an address, a member count); may be empty.
    pub detail: String,
}

impl Choice {
    /// A choice with a key and a name.
    pub fn new(key: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            ..Self::default()
        }
    }

    /// With a kind and a detail line.
    #[must_use]
    pub fn with_details(mut self, kind: impl Into<String>, detail: impl Into<String>) -> Self {
        self.kind = kind.into();
        self.detail = detail.into();
        self
    }
}

/// A permission, by the caller's key and the caller's words.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Permission {
    /// Identity given back in a [`Grant`].
    pub key: String,
    /// What is shown.
    pub label: String,
}

impl Permission {
    /// A permission.
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

/// One chosen pair.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    /// Key of the choice.
    pub choice: String,
    /// Key of the permission; none when the caller offered no permissions.
    pub permission: Option<String>,
}

impl Grant {
    /// A pair.
    pub fn new(choice: impl Into<String>, permission: Option<String>) -> Self {
        Self {
            choice: choice.into(),
            permission,
        }
    }
}

/// Why a step did nothing.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No choice with that key.
    NoSuchChoice,
    /// No permission with that key.
    NoSuchPermission,
    /// Confirm with nobody chosen, while a choice is required.
    NothingChosen,
    /// The row is not chosen, so it has no permission to change.
    NotChosen,
    /// Nothing to act on (no choices fit).
    Empty,
    /// Confirm while a chosen pair has no permission yet (permissions are offered, and the
    /// caller named no default for new pairs).
    PermissionMissing,
}

/// What a step did.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Something in the dialog changed (selection, permission, filter, cursor).
    Changed,
    /// The person confirmed: all the pairs chosen, in the order they were chosen.
    Confirmed(Vec<Grant>),
    /// The person cancelled; the caller removes the dialog.
    Cancelled,
    /// The step was not possible.
    Refused(Refusal),
    /// The key is not bound to anything.
    Ignored,
}
