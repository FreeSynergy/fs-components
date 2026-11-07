//! Exact text: what the text area cannot hold.
//!
//! Pattern: none — compare and describe texts that differ only in line ends and blank
//! lines at the end.

/// Equal apart from line ends and blank lines at the end.
pub(super) fn same_text(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.replace("\r\n", "\n").trim_end_matches('\n').to_string();
    norm(a) == norm(b)
}

/// What saving would change in `original` beyond the person's edits, because the text area cannot
/// hold it: Windows or mixed line ends (they are unified), blank lines at the end (they are
/// reduced to one line end). `None` when the text survives as it is.
pub fn lossy_note(original: &str) -> Option<String> {
    let mut reasons = Vec::new();
    let crlf = original.matches("\r\n").count();
    let lf_only = original.matches('\n').count() - crlf;
    if crlf > 0 && lf_only > 0 {
        reasons.push("mixed line ends become one kind");
    }
    if original.ends_with("\n\n") || original.ends_with("\r\n\r\n") {
        reasons.push("blank lines at the end are dropped");
    }
    (!reasons.is_empty()).then(|| reasons.join("; "))
}
