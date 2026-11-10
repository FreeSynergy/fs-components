//! The size of a file for people.
//!
//! Pattern: none — one pure function. Binary steps (1024) with the units KB, MB, GB, TB as in the file managers the
//! analysis lists; one decimal below ten, none above.

/// `0` -> `0 B`, `1536` -> `1.5 KB`, `10_485_760` -> `10 MB`.
#[allow(clippy::cast_precision_loss)]
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value < 10.0 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.0} {}", UNITS[unit])
    }
}
