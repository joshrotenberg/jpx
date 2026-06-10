//! Small shared helpers for the CLI.

/// Truncate `s` to at most `max_chars` characters for display, appending an
/// ellipsis if it was longer.
///
/// Operates on characters, never byte offsets, so it is safe for any UTF-8
/// input -- truncating with `&s[..n]` panics when byte `n` falls inside a
/// multibyte code point.
pub(crate) fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let prefix: String = s.chars().take(max_chars.saturating_sub(3)).collect();
        format!("{prefix}...")
    }
}

/// Whether stdout is connected to a terminal.
///
/// Uses the std library's `IsTerminal` (stable since Rust 1.70) rather than the
/// unmaintained, unsound `atty` crate.
pub(crate) fn stdout_is_terminal() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_string_unchanged() {
        assert_eq!(truncate_str("hello", 10), "hello");
        assert_eq!(truncate_str("hello", 5), "hello");
    }

    #[test]
    fn long_ascii_truncated_with_ellipsis() {
        assert_eq!(truncate_str("abcdefghij", 8), "abcde...");
    }

    #[test]
    fn multibyte_does_not_panic_and_counts_chars() {
        // Each "é" is two bytes; a byte-index slice at 7 would panic mid-char.
        let s = "ééééééééé"; // 9 chars, 18 bytes
        let out = truncate_str(s, 7);
        assert_eq!(out, "éééé..."); // 4 chars + ellipsis
        // A 4-char string is under an 8-char limit -> unchanged.
        assert_eq!(truncate_str("é\u{1f600}本a", 8), "é\u{1f600}本a");
    }
}
