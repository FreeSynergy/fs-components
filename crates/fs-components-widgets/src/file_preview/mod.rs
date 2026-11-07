//! `file-preview`: what can safely be shown of a file, and the facts about it.
//!
//! Pattern: Facade + Adapter over plain Blinc
//! elements. The widget has no words and no colours of its own and reads nothing: the caller hands over
//! bytes it read, and what cannot be shown safely is not handed to Blinc. Marks and the note stand
//! outside the scroller, so they are seen wherever it was left.

mod safe;
mod styles;
mod view;

pub use safe::{MAX_LINE_CHARS, MAX_LINES, SafeText, picture_address, picture_fits, safe_text};
pub use styles::STYLES;
pub use view::{Fact, FilePreview, PreviewContent};
