//! Can a picture be expected to load?
//!
//! Pattern: none — one pure check. Blinc's image shows a grey disc or nothing for a file it cannot open, never
//! an error, so a widget asks first and shows its own note or initials instead.

/// An address is left to Blinc; a path must be a file whose header is a picture.
pub(crate) fn loadable(source: &str) -> bool {
    if source.contains("://") || source.starts_with("data:") {
        return true;
    }
    image::ImageReader::open(source)
        .and_then(image::ImageReader::with_guessed_format)
        .is_ok_and(|r| r.into_dimensions().is_ok())
}
