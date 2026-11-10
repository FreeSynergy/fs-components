//! Why typed parts are not a value.
//!
//! Pattern: Value object — a plain code, compared by value; the sentence is the program's words
//! (ADR 0057), looked up by [`Problem::key`].

use fs_interfaces::values::ValueProblem;

/// Why the parts of a field are not (yet) a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Problem {
    /// The e-mail or phone rule of the contract broke; the code says which.
    Contact(ValueProblem),
    /// Neither a given name nor a surname.
    NoName,
    /// A part is there but another part it needs is not (address without place, date without day).
    Incomplete,
    /// A part that must be digits holds something else.
    NotANumber,
    /// The date does not exist (31 February).
    NoSuchDate,
}

impl Problem {
    /// The translation key of the sentence, stable.
    pub const fn key(self) -> &'static str {
        match self {
            Problem::Contact(ValueProblem::MissingAt) => "problem.email.missing-at",
            Problem::Contact(ValueProblem::ManyAt) => "problem.email.many-at",
            Problem::Contact(ValueProblem::EmptyLocal) => "problem.email.empty-local",
            Problem::Contact(ValueProblem::EmptyDomain) => "problem.email.empty-domain",
            Problem::Contact(ValueProblem::LocalCharacter) => "problem.email.local-character",
            Problem::Contact(ValueProblem::LocalTooLong) => "problem.email.local-too-long",
            Problem::Contact(ValueProblem::BadDomain) => "problem.email.bad-domain",
            Problem::Contact(ValueProblem::DomainWithoutDot) => "problem.email.domain-without-dot",
            Problem::Contact(ValueProblem::AddressTooLong) => "problem.email.too-long",
            Problem::Contact(ValueProblem::NoCountryCode) => "problem.phone.no-country-code",
            Problem::Contact(ValueProblem::PhoneCharacter) => "problem.phone.character",
            Problem::Contact(ValueProblem::PhoneLength) => "problem.phone.length",
            Problem::Contact(ValueProblem::Empty) => "problem.empty",
            Problem::NoName => "problem.name.none",
            Problem::Incomplete => "problem.incomplete",
            Problem::NotANumber => "problem.not-a-number",
            Problem::NoSuchDate => "problem.date.no-such-date",
        }
    }
}
