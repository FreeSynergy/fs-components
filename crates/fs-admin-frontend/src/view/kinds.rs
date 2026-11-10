//! Which basic types have a component yet.
//!
//! Pattern: Registry (basic type to component kind) — `fs-components-fields` has components for five basic types
//! (`Kind`); every other type is shown as its canonical text and typed into a plain input, named in `DESIGN.md` as
//! waiting for its own component (ADR 0066 point 4).

use fs_components_fields::Kind;
use fs_types::BasicType;

/// The component kind of a basic type, if `fs-components-fields` has one.
pub fn kind_of(basic: BasicType) -> Option<Kind> {
    match basic {
        BasicType::Email => Some(Kind::Email),
        BasicType::Name => Some(Kind::Name),
        BasicType::Phone => Some(Kind::Phone),
        BasicType::Address => Some(Kind::Address),
        BasicType::Date => Some(Kind::Date),
        _ => None,
    }
}

/// The basic types still without a component of their own (for the list in `DESIGN.md` and the tests).
pub const WITHOUT_COMPONENT: [BasicType; 11] = [
    BasicType::Text,
    BasicType::MultilineText,
    BasicType::Place,
    BasicType::DateTime,
    BasicType::Language,
    BasicType::Link,
    BasicType::Image,
    BasicType::Key,
    BasicType::Choice,
    BasicType::Keywords,
    BasicType::Reference,
];
