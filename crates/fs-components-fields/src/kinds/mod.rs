//! The basic types that have components.
//!
//! Pattern: none — declares the modules and re-exports; [`Kind`] (the registry) is in `kind.rs`.

mod address;
mod date;
mod email;
mod kind;
mod name;
mod phone;

pub use address::Address;
pub use date::Date;
pub use email::Email;
pub use kind::Kind;
pub use name::Name;
pub use phone::Phone;
