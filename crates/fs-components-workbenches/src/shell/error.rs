//! What can go wrong when drawing.
//!
//! Pattern: none.

/// The shell could not draw.
#[derive(Debug)]
pub struct ShellError(pub String);

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot draw: {}", self.0)
    }
}

impl std::error::Error for ShellError {}
