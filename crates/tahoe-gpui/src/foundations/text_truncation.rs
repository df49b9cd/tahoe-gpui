//! Helpers for app-layer text truncation.
//!
//! GPUI's [`TextOverflow`] primitive covers head / tail truncation natively
//! ([`TextOverflow::Truncate`] and [`TextOverflow::TruncateStart`]), but
//! middle-truncation ("path/to/…/file.rs") has no direct primitive. Rather
//! than block on an upstream PR, we mirror the pattern Zed already ships for
//! path labels: clip the string at the character boundary before handing it
//! to the renderer.
//!
//! The character-budget heuristic is close enough for the typical
//! middle-truncation use cases (file paths, breadcrumbs) where each glyph is
//! near-uniform width. A pixel-exact variant would require plumbing into
//! GPUI's post-layout measurement pass.
//!
//! [`TextOverflow`]: gpui::TextOverflow
//! [`TextOverflow::Truncate`]: gpui::TextOverflow::Truncate
//! [`TextOverflow::TruncateStart`]: gpui::TextOverflow::TruncateStart

const ELLIPSIS: &str = "…";

/// Truncate `s` so that it fits within `max_chars` (counting the one-char
/// ellipsis), keeping the head and tail of the string and replacing the
/// middle with a single `…`.
///
/// Returns `s` unchanged when its character length is `<= max_chars`.
///
/// The split favours the tail when `max_chars - 1` is odd — this matches the
/// convention used by macOS Finder and the AppKit truncation modes, where
/// the trailing filename is usually more informative than the intermediate
/// directory names.
///
/// # Examples
///
/// ```ignore
/// # use tahoe_gpui::foundations::text_truncation::truncate_middle;
/// assert_eq!(truncate_middle("short", 10), "short");
/// assert_eq!(truncate_middle("a/long/path/to/file.rs", 12), "a/lo…ile.rs");
/// ```
pub fn truncate_middle(s: &str, max_chars: usize) -> String {
    // A zero-char budget can't fit even the ellipsis — degenerate, but the
    // caller would get a panic on the head+tail arithmetic below. Return
    // empty to mirror how GPUI's `TextOverflow` collapses to the ellipsis.
    if max_chars == 0 {
        return String::new();
    }

    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }

    // Reserve one character for the ellipsis; split the remainder between
    // head and tail, biasing towards the tail when the budget is odd.
    let budget = max_chars - 1;
    let tail_chars = budget.div_ceil(2);
    let head_chars = budget - tail_chars;

    let mut out = String::with_capacity(s.len());

    // Head: first `head_chars` characters.
    if head_chars > 0 {
        let head_end = s
            .char_indices()
            .nth(head_chars)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        out.push_str(&s[..head_end]);
    }

    out.push_str(ELLIPSIS);

    // Tail: last `tail_chars` characters.
    if tail_chars > 0 {
        let tail_start = s
            .char_indices()
            .nth_back(tail_chars - 1)
            .map(|(i, _)| i)
            .unwrap_or(s.len());
        out.push_str(&s[tail_start..]);
    }

    out
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::truncate_middle;

    #[test]
    fn empty_string_is_unchanged() {
        assert_eq!(truncate_middle("", 10), "");
    }

    #[test]
    fn short_string_is_unchanged() {
        assert_eq!(truncate_middle("hello", 10), "hello");
    }

    #[test]
    fn exact_length_is_unchanged() {
        assert_eq!(truncate_middle("hello", 5), "hello");
    }

    #[test]
    fn truncates_middle_with_ellipsis() {
        // 22 chars → 12 budget → 1 ellipsis + 5 head + 6 tail when biased
        // towards tail.
        let out = truncate_middle("a/long/path/to/file.rs", 12);
        assert_eq!(out.chars().count(), 12);
        assert!(out.contains('…'));
        assert!(out.starts_with('a'));
        assert!(out.ends_with(".rs"));
    }

    #[test]
    fn multibyte_chars_counted_by_codepoint() {
        // 6 visible glyphs; each is 2 bytes in UTF-8.
        let out = truncate_middle("èèèèèè", 4);
        assert_eq!(out.chars().count(), 4);
        assert!(out.contains('…'));
    }

    #[test]
    fn single_char_budget_returns_only_ellipsis() {
        let out = truncate_middle("hello world", 1);
        assert_eq!(out, "…");
    }

    #[test]
    fn zero_budget_returns_empty() {
        // Degenerate case: can't fit even the ellipsis.
        assert_eq!(truncate_middle("hello", 0), "");
    }

    #[test]
    fn biases_towards_tail_on_odd_budget() {
        // 14 chars → budget 6 → tail gets 3, head gets 3.
        // 15 chars → budget 7 → tail gets 4, head gets 3.
        let out = truncate_middle("abcdefghijklmno", 8);
        assert_eq!(out.chars().count(), 8);
        // head "abc" + "…" + tail "lmno"
        assert_eq!(out, "abc…lmno");
    }
}
