//! `email-input` and `phone-input`: a text field for one contact value, shared by every program.
//!
//! Pattern: Decorator over `blinc_cn`'s input, with the value objects of `fs-interfaces`
//! (`values`) as the rule. [`email_input`] and [`phone_input`] differ only in the Blinc input type (the
//! keyboard of a phone), the placeholder and the rule; the shape and the colours come from one place.
//! The component offers the rule ([`email_problem`], [`phone_problem`]: a code) and the field; it does not word: a program turns the code into a sentence and gives it back (ADR 0057).

mod input;

pub use input::{OnChange, email_input, email_problem, phone_input, phone_problem};
