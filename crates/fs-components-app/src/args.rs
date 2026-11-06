//! What the command lines of all programs share.
//!
//! Pattern: Value parser — a pure function; each program keeps its own list of options.

/// The largest side the graphics can draw on (wgpu's guaranteed texture size); more panics there.
const MAX_SIDE: u32 = 8192;

/// The value of `--size`: `WIDTHxHEIGHT`, both above zero and at most 8192.
///
/// # Errors
/// A message for the person.
pub fn parse_size(text: &str) -> Result<(u32, u32), String> {
    let bad = || format!("--size wants WIDTHxHEIGHT, not {text}");
    let (w, h) = text.split_once('x').ok_or_else(bad)?;
    let (w, h): (u32, u32) = (w.parse().map_err(|_| bad())?, h.parse().map_err(|_| bad())?);
    if w == 0 || h == 0 {
        return Err(bad());
    }
    if w > MAX_SIDE || h > MAX_SIDE {
        return Err(format!(
            "--size allows at most {MAX_SIDE} on a side, not {text}"
        ));
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests {
    use super::parse_size;

    #[test]
    fn a_size_is_two_numbers_above_zero() {
        assert_eq!(parse_size("1300x800"), Ok((1300, 800)));
        assert_eq!(parse_size("8192x8192"), Ok((8192, 8192)));
        for bad in [
            "8193x1",
            "1x8193",
            "99999999999x1",
            "",
            "1300",
            "x800",
            "0x800",
            "800x0",
            "ax1",
            "-1x1",
            "1x1x1",
        ] {
            assert!(parse_size(bad).is_err(), "{bad}");
        }
    }
}
