//! The views of the generated Admin frontend.
//!
//! Pattern: none — declares the modules and re-exports.

mod chrome;
mod detail;
mod findings;
mod form;
mod inputs;
mod kinds;
mod list;
mod redraw;
mod screens;
mod surface;
mod value;

pub use kinds::{WITHOUT_COMPONENT, kind_of};
pub use list::rows as list_rows;
pub use surface::{AboutInfo, AdminSurface};

/// The one lock of the lib tests that count the signals of the process-wide reactive graph. The count is global: a test
/// that makes or gives back signals while another counts would move the other's numbers, so every test of this crate
/// that makes signals, counts them or drops something that gives them back holds this lock for its whole run.
#[cfg(test)]
pub(crate) fn graph_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static GRAPH: std::sync::Mutex<()> = std::sync::Mutex::new(());
    GRAPH
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
