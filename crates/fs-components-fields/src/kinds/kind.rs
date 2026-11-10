//! The registry of the basic types that have components.
//!
//! Pattern: Registry — [`Kind`] names a type, [`Kind::field`] gives its [`BasicField`].

use super::{Address, Date, Email, Name, Phone};
use crate::field::BasicField;

/// The basic types done so far (the first five of `docs/en/programs/map.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// E-mail address (RFC 5322, JSContact `EmailAddress`).
    Email,
    /// Name of a person (JSContact `Name`).
    Name,
    /// Phone number (E.164).
    Phone,
    /// Postal address (JSContact `Address`).
    Address,
    /// Calendar date (RFC 3339 `full-date`).
    Date,
}

impl Kind {
    /// All kinds in a fixed order.
    pub const ALL: [Kind; 5] = [
        Kind::Email,
        Kind::Name,
        Kind::Phone,
        Kind::Address,
        Kind::Date,
    ];

    /// The type's rules and structure.
    pub fn field(self) -> &'static dyn BasicField {
        match self {
            Kind::Email => &Email,
            Kind::Name => &Name,
            Kind::Phone => &Phone,
            Kind::Address => &Address,
            Kind::Date => &Date,
        }
    }

    /// The kind with this key.
    pub fn from_key(key: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.field().key() == key)
    }
}
