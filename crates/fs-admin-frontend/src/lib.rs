//! `fs-admin-frontend`: the Admin frontend of ADR 0066, generated from a type description.
//!
//! A program reports a `TypeDescription` (fields, commands) and gives a [`Backend`]; this crate makes the list, the
//! detail, the forms and the command buttons from them, one code for every program. The model in this crate is pure
//! (no Blinc, tested without a window); the views in `view` put it on the screen with the components of
//! `fs-components-fields` in the Admin (Fiori) style, inside the window shell of ADR 0065.
//!
//! Pattern: none — this file only declares modules and re-exports the public API.

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::must_use_candidate, clippy::module_name_repetitions)]

pub mod backend;
pub mod clock;
pub mod commands;
pub mod detail;
pub mod form;
mod form_edit;
pub mod list;
pub mod messages;
pub mod screen;
pub mod session;
mod session_done;
mod session_retry;
mod session_submit;
pub mod view;
pub mod window;

pub use backend::{Backend, BackendError, Outcome, Record, Values};
pub use clock::{Clock, ManualClock, SystemClock};
pub use commands::{CommandPlan, Placement, plan_commands};
pub use detail::{Entry, entries};
pub use form::FormModel;
pub use list::{Column, Direction, ListModel};
pub use messages::{Message, MessageKind, Messages};
pub use screen::Screen;
pub use session::Session;
pub use session_retry::{RetryJob, RetryOutcome};
pub use session_submit::{SubmitJob, SubmitOutcome};
