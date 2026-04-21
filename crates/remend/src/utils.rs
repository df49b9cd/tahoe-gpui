use std::borrow::Cow;
use std::ops::ControlFlow;

use super::fence::{CodeRegion, FenceRegion, InlineRegion, InlineTerminator, scan_code_regions};

/// Returns `true` if `ch` is a Unicode letter, digit, or underscore.
/// Matches the TS `isWordChar` / `[\p{L}\p{N}_]`.
pub fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Returns `true` if the byte at `pos` is preceded by a backslash that is not
/// itself escaped. In other words, the character at `pos` is backslash-escaped.
///
/// Returns `false` when `pos` is 0 (nothing precedes it) or when the preceding
/// backslash is itself escaped by another backslash.
pub(crate) fn is_escaped(bytes: &[u8], pos: usize) -> bool {
    if pos == 0 || bytes[pos - 1] != b'\\' {
        return false;
    }
    // The backslash at pos-1 might itself be escaped. Count consecutive
    // backslashes ending at pos-1; an odd count means the character is escaped.
    let mut backslashes = 0usize;
    let mut j = pos;
    while j > 0 && bytes[j - 1] == b'\\' {
        backslashes += 1;
        j -= 1;
    }
    backslashes % 2 == 1
}

/// Returns `true` if `s` contains only whitespace and emphasis marker characters
/// (`_`, `~`, `*`, `` ` ``). Matches the TS `whitespaceOrMarkersPattern`.
pub fn is_empty_or_markers(s: &str) -> bool {
    s.bytes()
        .all(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'_' | b'~' | b'*' | b'`'))
}

/// Returns `true` if the line is a list item marker line (e.g. `  - `, `  * `, `  + `).
pub fn is_list_marker_line(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    // Skip leading whitespace.
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t') {
        i += 1;
    }
    // Expect one of `-`, `*`, `+`.
    if i >= bytes.len() || !matches!(bytes[i], b'-' | b'*' | b'+') {
        return false;
    }
    i += 1;
    // Must be followed by at least one space or tab, then only whitespace.
    if i >= bytes.len() || !matches!(bytes[i], b' ' | b'\t') {
        return false;
    }
    bytes[i..].iter().all(|&b| matches!(b, b' ' | b'\t'))
}

/// Returns `true` if the position is inside a fenced code block (between ``` markers)
/// or an inline code span (between `` ` `` markers).
///
/// Thin adapter over `scan_code_regions` (shared with `ranges.rs` and the
/// cross-validation reference impls). Regions are emitted in increasing
/// open-position order, so we break as soon as we see a region whose opener
/// has already passed `position`.
pub fn is_inside_code_block(text: &str, position: usize) -> bool {
    let len = text.len();
    let mut inside = false;
    scan_code_regions(text, |region| {
        let (open_pos, start, end) = match region {
            CodeRegion::Fence(f) => (f.open_run_start, f.open_run_start + 1, fence_end(&f, len)),
            CodeRegion::Inline(s) => (s.open_pos, s.open_pos + 1, inline_end(&s, len)),
        };
        if start <= position && position < end {
            inside = true;
            return ControlFlow::Break(());
        }
        if open_pos >= position {
            // Regions are emitted in opening-position order; no later region
            // can contain `position`.
            return ControlFlow::Break(());
        }
        ControlFlow::Continue(())
    });
    inside
}

/// Returns `true` if the position is inside a *complete* inline code span
/// (both opening and closing backtick present). Returns `false` for incomplete
/// spans so emphasis markers can still be completed during streaming.
///
/// Prefer `CodeBlockRanges::is_within_complete_inline_code` for repeated queries
/// — this function scans the full text each time (O(n)).
#[cfg(test)]
pub(crate) fn is_within_complete_inline_code(text: &str, position: usize) -> bool {
    let mut inside = false;
    scan_code_regions(text, |region| {
        if let CodeRegion::Inline(s) = region
            && let InlineTerminator::Closed(close_pos) = s.terminator
        {
            if s.open_pos < position && position < close_pos {
                inside = true;
                return ControlFlow::Break(());
            }
            if s.open_pos >= position {
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    });
    inside
}

/// Exclusive end boundary of a fenced code region matching the historical
/// `is_inside_code_block` semantics: the first byte of the closing fence run
/// is still "inside"; unterminated fences extend to `len + 1` so queries at
/// `position == len` agree with the per-position scan.
#[inline]
pub(crate) fn fence_end(region: &FenceRegion, len: usize) -> usize {
    if region.closed {
        region.close_run_start + 1
    } else {
        len + 1
    }
}

/// Exclusive end boundary of an inline code region matching the historical
/// `is_inside_code_block` semantics: the closing backtick (or terminating
/// newline) is still "inside"; EOF-unterminated spans extend to `len + 1`.
#[inline]
pub(crate) fn inline_end(region: &InlineRegion, len: usize) -> usize {
    match region.terminator {
        InlineTerminator::Closed(pos) | InlineTerminator::Newline(pos) => pos + 1,
        InlineTerminator::Eof => len + 1,
    }
}

/// Returns `true` if the backtick at `pos` is part of a run of 3+ backticks.
pub fn is_part_of_triple_backtick(text: &str, pos: usize) -> bool {
    let bytes = text.as_bytes();
    if pos >= bytes.len() || bytes[pos] != b'`' {
        return false;
    }
    // Find the start of the backtick run containing pos.
    let mut start = pos;
    while start > 0 && bytes[start - 1] == b'`' {
        start -= 1;
    }
    // Find the end of the backtick run containing pos.
    let mut end = pos + 1;
    while end < bytes.len() && bytes[end] == b'`' {
        end += 1;
    }
    // Part of a fence if the run is 3+ backticks.
    (end - start) >= 3
}

/// Counts single backticks that are not part of triple backticks or escaped.
pub fn count_single_backticks(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        // Skip escaped backticks.
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'`' {
            i += 2;
            continue;
        }
        if bytes[i] == b'`' && !is_part_of_triple_backtick(text, i) {
            count += 1;
        }
        i += 1;
    }
    count
}

/// Returns `true` if `position` is inside a math block (`$` or `$$`).
pub fn is_within_math_block(text: &str, position: usize) -> bool {
    let bytes = text.as_bytes();
    let mut in_inline_math = false;
    let mut in_block_math = false;
    let mut i = 0;

    while i < bytes.len() && i < position {
        // Skip escaped dollar signs.
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'$' {
            i += 2;
            continue;
        }
        if bytes[i] == b'$' {
            // Check for block math ($$).
            if i + 1 < bytes.len() && bytes[i + 1] == b'$' {
                in_block_math = !in_block_math;
                i += 2;
                in_inline_math = false; // Block math takes precedence.
                continue;
            } else if !in_block_math {
                // Only toggle inline math if not in block math.
                in_inline_math = !in_inline_math;
            }
        }
        i += 1;
    }

    in_inline_math || in_block_math
}

/// Returns `true` if `position` is inside the URL portion of a link/image `](url)`.
pub fn is_within_link_or_image_url(text: &str, position: usize) -> bool {
    let bytes = text.as_bytes();

    // Search backwards from position for `(` preceded by `]`.
    let mut i = position.saturating_sub(1);
    loop {
        if i >= bytes.len() {
            break;
        }
        match bytes[i] {
            b')' => return false,
            b'(' => {
                if i > 0 && bytes[i - 1] == b']' {
                    // We're potentially inside a link/image URL.
                    // Check if we're before the closing `)`.
                    for &b in &bytes[position..] {
                        if b == b')' {
                            return true;
                        }
                        if b == b'\n' {
                            return false;
                        }
                    }
                }
                return false;
            }
            b'\n' => return false,
            _ => {}
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }

    false
}

/// Returns `true` if `remainder` (content after `<`) looks like a plausible
/// HTML tag prefix per CommonMark: optional `/`, then an ASCII letter,
/// then zero or more `[A-Za-z0-9-]`, then either EOF or whitespace
/// (attributes begin). Any other byte means it is not a tag.
pub(crate) fn is_plausible_tag_remainder(remainder: &[u8]) -> bool {
    let mut j = 0;
    if remainder.first() == Some(&b'/') {
        j = 1;
    }
    if j >= remainder.len() {
        // `</` alone is a valid incomplete close-tag prefix; bare `<` is not.
        return j > 0;
    }
    if !remainder[j].is_ascii_alphabetic() {
        return false;
    }
    j += 1;
    while j < remainder.len() {
        let b = remainder[j];
        if b.is_ascii_alphanumeric() || b == b'-' {
            j += 1;
        } else if b == b' ' || b == b'\t' {
            return true;
        } else if b == b'/' && j == remainder.len() - 1 {
            // Self-closing `/` at EOF: `<br/`.
            return true;
        } else {
            return false;
        }
    }
    true
}

/// Returns `true` if `position` is inside an HTML tag (between `<` and `>`).
///
/// Production callers should use `CodeBlockRanges::is_within_html_tag` for
/// O(log n) lookups. This O(n) per-query helper is retained as the reference
/// implementation that `compute_html_tag_ranges` cross-validates against.
#[cfg(test)]
pub(crate) fn is_within_html_tag(text: &str, position: usize) -> bool {
    let bytes = text.as_bytes();

    let mut i = position.saturating_sub(1);
    loop {
        if i >= bytes.len() {
            break;
        }
        match bytes[i] {
            b'>' => return false,
            b'<' => {
                // Must look like a real HTML tag, not inline text like `a<b` or
                // `name@<example.com`.
                if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
                    return false;
                }
                return is_plausible_tag_remainder(&bytes[i + 1..]);
            }
            b'\n' => return false,
            _ => {}
        }
        if i == 0 {
            break;
        }
        i -= 1;
    }

    false
}

/// Returns `true` if the marker at `marker_index` is on a line that forms a
/// horizontal rule (3+ of the same marker with optional spaces, nothing else).
pub fn is_horizontal_rule(text: &str, marker_index: usize, marker: u8) -> bool {
    let bytes = text.as_bytes();

    // Find line start.
    let line_start = bytes[..marker_index]
        .iter()
        .rposition(|&b| b == b'\n')
        .map(|p| p + 1)
        .unwrap_or(0);

    // Find line end.
    let line_end = bytes[marker_index..]
        .iter()
        .position(|&b| b == b'\n')
        .map(|p| marker_index + p)
        .unwrap_or(bytes.len());

    let line = &bytes[line_start..line_end];

    let mut marker_count = 0;
    let mut has_other = false;

    for &b in line {
        if b == marker {
            marker_count += 1;
        } else if b != b' ' && b != b'\t' {
            has_other = true;
            break;
        }
    }

    marker_count >= 3 && !has_other
}

/// Walks backwards through `text` looking for the last occurrence of `delimiter`
/// (a byte sequence like `b"**"`, `b"~~"`, `b"__"`) followed by trailing content
/// that does not contain the delimiter's first byte.
///
/// Returns `(marker_start_index, content_after_delimiter)` on success.
///
/// This is the generic form of functions like `find_trailing_double_underscore`
/// and `find_trailing_strikethrough`.
pub(crate) fn find_trailing_delimiter<'a>(
    text: &'a str,
    delimiter: &[u8],
) -> Option<(usize, &'a str)> {
    let bytes = text.as_bytes();
    let dlen = delimiter.len();
    if dlen == 0 || bytes.len() < dlen {
        return None;
    }
    let forbidden = delimiter[0];
    // Single backward pass: once we see a `forbidden` byte, any earlier
    // delimiter match would have `forbidden` in its trailing content and
    // would be rejected — so we can stop. This keeps the function O(n)
    // even on pathologically dense inputs (the prior implementation
    // rescanned the tail on every candidate, which was O(n²)).
    //
    // Byte-level comparison is also safer than the previous
    // `content.contains(forbidden as char)` — a `u8` cast produces garbage
    // `char` values for multi-byte UTF-8 leading bytes, though in practice
    // all callers use ASCII delimiters.
    let mut saw_forbidden = false;
    let mut i = bytes.len();
    while i >= dlen {
        i -= 1;
        let candidate_end = i;
        let candidate_start = candidate_end + 1 - dlen;
        let candidate_ok = bytes[candidate_end] == delimiter[dlen - 1]
            && bytes[candidate_start..=candidate_end] == *delimiter;
        if candidate_ok && !saw_forbidden {
            let content = &text[candidate_end + 1..];
            return Some((candidate_start, content));
        }
        if bytes[i] == forbidden {
            saw_forbidden = true;
        }
    }
    None
}

/// Helper: make an owned Cow by appending a suffix.
pub(crate) fn cow_append<'a>(text: &str, suffix: &str) -> Cow<'a, str> {
    let mut s = String::with_capacity(text.len() + suffix.len());
    s.push_str(text);
    s.push_str(suffix);
    Cow::Owned(s)
}

/// Returns `true` if the text ends with an odd number of backslashes,
/// meaning the next character appended would be treated as backslash-escaped
/// on a subsequent pass (breaking idempotency).
pub(crate) fn ends_with_odd_backslashes(text: &str) -> bool {
    let count = text.bytes().rev().take_while(|&b| b == b'\\').count();
    count % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::{
        count_single_backticks, find_trailing_delimiter, is_empty_or_markers, is_horizontal_rule,
        is_inside_code_block, is_within_html_tag, is_within_link_or_image_url,
        is_within_math_block, is_word_char,
    };

    #[test]
    fn test_is_word_char() {
        assert!(is_word_char('a'));
        assert!(is_word_char('Z'));
        assert!(is_word_char('0'));
        assert!(is_word_char('_'));
        assert!(!is_word_char(' '));
        assert!(!is_word_char('*'));
        // Unicode
        assert!(is_word_char('é'));
        assert!(is_word_char('中'));
    }

    #[test]
    fn test_is_empty_or_markers() {
        assert!(is_empty_or_markers(""));
        assert!(is_empty_or_markers("  "));
        assert!(is_empty_or_markers("*_~`"));
        assert!(!is_empty_or_markers("hello"));
        assert!(!is_empty_or_markers("*a"));
    }

    #[test]
    fn test_is_inside_code_block() {
        assert!(is_inside_code_block("```code", 5));
        // Per CommonMark §4.5 (fixed in #50): the second ``` is mid-line and
        // does NOT close the fence opened at position 0, so "after" is inside.
        assert!(is_inside_code_block("```code```after", 12));
        assert!(is_inside_code_block("`code", 3));
        assert!(!is_inside_code_block("`code`after", 8));
        // Unterminated fence at EOF: every position past the opener is inside.
        assert!(is_inside_code_block("```\ncode", 8));
        // Char-mismatch closer: `~~~` on a line does not close a ``` fence.
        assert!(is_inside_code_block("```\ncode\n~~~\nmore", 14));
        // Too-short closer: 3 backticks do not close a 4-backtick fence.
        assert!(is_inside_code_block("````\ncode\n```\nmore", 15));
        // 3 leading spaces is still a fence.
        assert!(is_inside_code_block("   ```\ninside", 10));
        // 4 leading spaces disqualifies the fence (indented code block syntax).
        assert!(!is_inside_code_block("    ```\nnot-inside", 10));
        // Newline resets inline code span — unclosed backtick doesn't cross \n.
        assert!(is_inside_code_block("`unclosed", 5));
        assert!(!is_inside_code_block("`unclosed\nnext", 14));
        // Fence opens after a newline: prose line is outside, code line is inside.
        assert!(!is_inside_code_block("plain\n```\ncode", 5));
        assert!(is_inside_code_block("plain\n```\ncode", 10));
    }

    #[test]
    fn test_is_within_math_block() {
        assert!(is_within_math_block("$x+y", 2));
        assert!(!is_within_math_block("$x+y$z", 6));
        assert!(is_within_math_block("$$x+y", 3));
        assert!(!is_within_math_block("\\$x", 2));
    }

    #[test]
    fn test_is_horizontal_rule() {
        assert!(is_horizontal_rule("---", 0, b'-'));
        assert!(is_horizontal_rule("***", 0, b'*'));
        assert!(is_horizontal_rule("- - -", 0, b'-'));
        assert!(!is_horizontal_rule("--", 0, b'-'));
        assert!(!is_horizontal_rule("--x", 0, b'-'));
    }

    #[test]
    fn test_count_single_backticks() {
        assert_eq!(count_single_backticks("`hello`"), 2);
        assert_eq!(count_single_backticks("```hello```"), 0);
        assert_eq!(count_single_backticks("`hello"), 1);
        assert_eq!(count_single_backticks("\\`hello"), 0);
    }

    #[test]
    fn test_is_within_link_or_image_url() {
        assert!(is_within_link_or_image_url(
            "[text](http://example.com)",
            15
        ));
        assert!(!is_within_link_or_image_url(
            "[text](http://example.com)",
            3
        ));
        assert!(!is_within_link_or_image_url("just text", 3));
    }

    #[test]
    fn test_is_within_html_tag() {
        assert!(is_within_html_tag("<a href=\"test\">", 5));
        assert!(!is_within_html_tag("<a href=\"test\">after", 16));
        assert!(!is_within_html_tag("text", 2));
        // Issue #15: `<example.com` is not a valid HTML tag.
        assert!(!is_within_html_tag("name@<example.com", 10));
        // Issue #15: `a<b` is inline text, not a tag.
        assert!(!is_within_html_tag("a<b", 2));
    }

    #[test]
    fn test_find_trailing_delimiter_double_underscore() {
        assert_eq!(
            find_trailing_delimiter("hello __world", b"__"),
            Some((6, "world"))
        );
        assert_eq!(
            find_trailing_delimiter("__bold__inner__text", b"__"),
            Some((13, "text"))
        );
        // Content contains delimiter char -> skip, find earlier match.
        assert_eq!(
            find_trailing_delimiter("__a_b", b"__"),
            None // only match at 0 but content "a_b" contains '_'
        );
        assert_eq!(find_trailing_delimiter("no delimiters", b"__"), None);
        assert_eq!(find_trailing_delimiter("__", b"__"), Some((0, "")));
    }

    #[test]
    fn test_find_trailing_delimiter_double_tilde() {
        assert_eq!(
            find_trailing_delimiter("hello ~~strike", b"~~"),
            Some((6, "strike"))
        );
        assert_eq!(find_trailing_delimiter("no tildes", b"~~"), None);
        // Content with tilde should be skipped.
        assert_eq!(find_trailing_delimiter("~~a~b", b"~~"), None);
    }

    #[test]
    fn is_inside_code_block_rejects_mid_line_backtick_run() {
        // Issue #50: mid-line ``` is not a fence.
        assert!(!is_inside_code_block("hello ```\ncode", 12));
    }

    #[test]
    fn is_inside_code_block_rejects_mid_line_tilde_run() {
        // Issue #50: mid-line ~~~ is not a fence.
        assert!(!is_inside_code_block("hello ~~~\ncode", 12));
    }

    #[test]
    fn is_inside_code_block_mid_line_fence_leaves_opener_unclosed() {
        // "```code```after" — the second ``` is mid-line, so it doesn't close
        // the fence opened at position 0. Position 12 (inside "after") IS
        // therefore inside an unclosed fenced code block.
        assert!(is_inside_code_block("```code```after", 12));
    }
}
