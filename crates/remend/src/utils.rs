use std::borrow::Cow;

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

/// Counts the length of a consecutive run of the given byte starting at `i`.
#[inline]
pub(crate) fn fence_run_length(bytes: &[u8], i: usize, ch: u8) -> usize {
    let mut len = 0;
    while i + len < bytes.len() && bytes[i + len] == ch {
        len += 1;
    }
    len
}

/// Result of a successful fence parse: the fence character, run length, and
/// byte offsets of the fence run (just past any leading spaces).
pub(crate) struct FenceHit {
    pub(crate) ch: u8,
    pub(crate) len: usize,
    /// Offset of the first fence char (backtick or tilde), past ≤3 leading spaces.
    pub(crate) run_start: usize,
    /// Offset just past the fence run.
    pub(crate) run_end: usize,
}

/// Attempts to parse a CommonMark §4.5 code fence at the current logical line
/// beginning at `line_start`. Skips up to 3 leading spaces (tab = 4, so any
/// leading tab disqualifies); then requires 3+ consecutive backticks or tildes.
///
/// Returns `None` if the line does not open (or close) a fence. This is the
/// single source of truth for fence detection across `utils.rs`, `ranges.rs`,
/// and `incomplete_code.rs`.
#[inline]
pub(crate) fn parse_fence_at_line_start(bytes: &[u8], line_start: usize) -> Option<FenceHit> {
    let mut i = line_start;
    let mut leading = 0usize;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        leading += if bytes[i] == b'\t' { 4 } else { 1 };
        if leading > 3 {
            return None;
        }
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let ch = bytes[i];
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let run = fence_run_length(bytes, i, ch);
    if run < 3 {
        return None;
    }
    Some(FenceHit {
        ch,
        len: run,
        run_start: i,
        run_end: i + run,
    })
}

/// Tracks whether a byte-wise scan is currently inside a fenced code block
/// (`` ``` `` or `~~~` runs of 3+, per CommonMark §4.5). A single scanner
/// instance is threaded through a loop and consulted at each line start.
///
/// Consolidates the fence-tracking state machine used by the emphasis and
/// katex counting helpers — previously duplicated across 8 call sites.
#[derive(Default)]
pub(crate) struct FenceScanner {
    in_code_block: bool,
    opening_fence_char: u8,
    opening_fence_len: usize,
}

impl FenceScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the most recent `consume_fence_at_line_start` call
    /// left the scanner inside a fenced code block.
    pub fn in_code_block(&self) -> bool {
        self.in_code_block
    }

    /// If `line_start` begins a CommonMark fence line (≤3 leading spaces then
    /// 3+ backticks or tildes), update the fence state and return
    /// `Some(run_end)` — the position just past the fence run. Returns `None`
    /// when no fence starts at this line.
    ///
    /// A closing fence must use the same character and be at least as long as
    /// the opening fence: `` ```` `` is not closed by ``` ``` ``, and a
    /// backtick fence is not closed by a tilde run.
    pub fn consume_fence_at_line_start(
        &mut self,
        bytes: &[u8],
        line_start: usize,
    ) -> Option<usize> {
        let hit = parse_fence_at_line_start(bytes, line_start)?;
        if !self.in_code_block {
            self.in_code_block = true;
            self.opening_fence_char = hit.ch;
            self.opening_fence_len = hit.len;
        } else if hit.ch == self.opening_fence_char && hit.len >= self.opening_fence_len {
            self.in_code_block = false;
            self.opening_fence_char = 0;
            self.opening_fence_len = 0;
        }
        Some(hit.run_end)
    }
}

/// Returns `true` if the position is inside a fenced code block (between ``` markers)
/// or an inline code span (between `` ` `` markers).
///
/// Fenced code blocks are detected per CommonMark §4.5: the opening and closing
/// fences must start at the beginning of a line with ≤3 leading spaces. A
/// 3+ backtick or tilde run in the middle of a line is NOT a fence.
pub fn is_inside_code_block(text: &str, position: usize) -> bool {
    let bytes = text.as_bytes();
    let mut in_code_block = false;
    let mut opening_fence_char: u8 = 0;
    let mut opening_fence_len: usize = 0;
    let mut in_inline_code = false;
    let mut fence_on_line = parse_fence_at_line_start(bytes, 0);
    let mut i = 0;

    while i < position && i < bytes.len() {
        // Toggle fence state when we reach the first fence char on a fence line.
        if !in_inline_code
            && let Some(hit) = fence_on_line.as_ref()
            && i == hit.run_start
        {
            if !in_code_block {
                in_code_block = true;
                opening_fence_char = hit.ch;
                opening_fence_len = hit.len;
            } else if hit.ch == opening_fence_char && hit.len >= opening_fence_len {
                in_code_block = false;
                opening_fence_char = 0;
                opening_fence_len = 0;
            }
            i = hit.run_end;
            continue;
        }

        // Skip escaped backticks.
        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'`' {
            i += 2;
            continue;
        }

        // Mid-line 3+ backtick/tilde runs are inert literals — never a fence
        // (those are handled at line-start above) and never a valid inline-code
        // closer for a single-backtick opener, so skip the whole run without
        // toggling `in_inline_code`.
        if bytes[i] == b'`' || bytes[i] == b'~' {
            let run = fence_run_length(bytes, i, bytes[i]);
            if run >= 3 {
                i += run;
                continue;
            }
        }

        // Only check for inline code if not in multiline code.
        if !in_code_block && bytes[i] == b'`' {
            in_inline_code = !in_inline_code;
        }
        if bytes[i] == b'\n' {
            fence_on_line = parse_fence_at_line_start(bytes, i + 1);
            // CommonMark inline code spans do not span hard line breaks in this
            // streaming approximation — reset to keep fence detection honest
            // after a newline (see issue #50 follow-up).
            in_inline_code = false;
        }
        i += 1;
    }

    in_inline_code || in_code_block
}

/// Returns `true` if the position is inside a *complete* inline code span
/// (both opening and closing backtick present). Returns `false` for incomplete
/// spans so emphasis markers can still be completed during streaming.
///
/// Prefer `CodeBlockRanges::is_within_complete_inline_code` for repeated queries
/// — this function scans the full text each time (O(n)).
#[cfg(test)]
pub(crate) fn is_within_complete_inline_code(text: &str, position: usize) -> bool {
    let bytes = text.as_bytes();
    let mut in_multiline_code = false;
    let mut opening_fence_char: u8 = 0;
    let mut opening_fence_len: usize = 0;
    let mut in_inline_code = false;
    let mut inline_code_start: Option<usize> = None;
    let mut fence_on_line = parse_fence_at_line_start(bytes, 0);
    let mut i = 0;

    while i < bytes.len() {
        if !in_inline_code
            && let Some(hit) = fence_on_line.as_ref()
            && i == hit.run_start
        {
            if !in_multiline_code {
                in_multiline_code = true;
                opening_fence_char = hit.ch;
                opening_fence_len = hit.len;
            } else if hit.ch == opening_fence_char && hit.len >= opening_fence_len {
                in_multiline_code = false;
                opening_fence_char = 0;
                opening_fence_len = 0;
            }
            i = hit.run_end;
            continue;
        }

        if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i + 1] == b'`' {
            i += 2;
            continue;
        }

        // Mid-line 3+ runs are inert literals (issue #50 follow-up).
        if bytes[i] == b'`' || bytes[i] == b'~' {
            let run = fence_run_length(bytes, i, bytes[i]);
            if run >= 3 {
                i += run;
                continue;
            }
        }

        if !in_multiline_code && bytes[i] == b'`' {
            if in_inline_code {
                if let Some(start) = inline_code_start
                    && start < position
                    && position < i
                {
                    return true;
                }
                in_inline_code = false;
                inline_code_start = None;
            } else {
                in_inline_code = true;
                inline_code_start = Some(i);
            }
        }
        if bytes[i] == b'\n' {
            fence_on_line = parse_fence_at_line_start(bytes, i + 1);
            in_inline_code = false;
            inline_code_start = None;
        }
        i += 1;
    }

    false
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
pub fn is_within_html_tag(text: &str, position: usize) -> bool {
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

/// Finds the matching opening bracket `[` for a closing bracket at `close_index`,
/// handling nested brackets.
pub fn find_matching_opening_bracket(text: &str, close_index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: i32 = 1;
    let mut i = close_index;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b']' => depth += 1,
            b'[' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Finds the matching closing bracket `]` for an opening bracket at `open_index`,
/// handling nested brackets.
pub fn find_matching_closing_bracket(text: &str, open_index: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: i32 = 1;
    let mut i = open_index + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
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

#[cfg(test)]
mod tests {
    use super::{
        FenceScanner, count_single_backticks, find_matching_closing_bracket,
        find_matching_opening_bracket, find_trailing_delimiter, is_empty_or_markers,
        is_horizontal_rule, is_inside_code_block, is_within_html_tag, is_within_link_or_image_url,
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
    }

    #[test]
    fn test_is_within_math_block() {
        assert!(is_within_math_block("$x+y", 2));
        assert!(!is_within_math_block("$x+y$z", 6));
        assert!(is_within_math_block("$$x+y", 3));
        assert!(!is_within_math_block("\\$x", 2));
    }

    #[test]
    fn test_find_matching_brackets() {
        assert_eq!(find_matching_opening_bracket("[hello]", 6), Some(0));
        assert_eq!(find_matching_closing_bracket("[hello]", 0), Some(6));
        assert_eq!(find_matching_opening_bracket("a[b[c]]", 6), Some(1));
        assert_eq!(find_matching_opening_bracket("hello]", 5), None);
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
    fn fence_scanner_consumes_backtick_fence() {
        let mut scanner = FenceScanner::new();
        let bytes = b"```rust\ncode\n```";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(3));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 13), Some(16));
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_consumes_tilde_fence() {
        let mut scanner = FenceScanner::new();
        let bytes = b"~~~\nx\n~~~";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(3));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 6), Some(9));
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_ignores_short_runs() {
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"`x`", 0), None);
        assert_eq!(scanner.consume_fence_at_line_start(b"``x``", 0), None);
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_ignores_non_fence_bytes() {
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"abc", 0), None);
        assert_eq!(scanner.consume_fence_at_line_start(b"", 0), None);
    }

    #[test]
    fn fence_scanner_rejects_prose_line_with_mid_line_run() {
        // A prose line that happens to contain a mid-line ``` run must NOT be
        // treated as a fence line when the scanner is invoked at its actual
        // line start (offset 0). The mid-line run at offset 6 is irrelevant —
        // `consume_fence_at_line_start` only inspects the byte at/after the
        // leading spaces of `line_start`.
        let mut scanner = FenceScanner::new();
        assert_eq!(
            scanner.consume_fence_at_line_start(b"hello ```\ncode", 0),
            None
        );
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_accepts_indented_fence() {
        // 3 leading spaces are allowed by CommonMark §4.5.
        let mut scanner = FenceScanner::new();
        assert_eq!(
            scanner.consume_fence_at_line_start(b"   ```\nx", 0),
            Some(6)
        );
        assert!(scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_rejects_four_space_indent() {
        // 4+ leading spaces is an indented code block, not a fence.
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"    ```\nx", 0), None);
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_rejects_leading_tab() {
        // A tab counts as 4 columns; no fence can open after a leading tab.
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"\t```\nx", 0), None);
    }

    #[test]
    fn fence_scanner_backtick_not_closed_by_tilde() {
        // CommonMark §4.5: closing fence char must match opening.
        let mut scanner = FenceScanner::new();
        let bytes = b"```\ncode\n~~~\nmore";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(3));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 9), Some(12));
        assert!(scanner.in_code_block()); // tilde run did NOT close the backtick fence
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

    #[test]
    fn fence_scanner_short_closer_does_not_close_long_opener() {
        // 4-backtick opener cannot be closed by 3 backticks (CommonMark rule).
        let mut scanner = FenceScanner::new();
        let bytes = b"````\nx\n```\ny\n````";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(4));
        assert!(scanner.in_code_block());
        // The 3-backtick run at line_start=7 does NOT close the 4-backtick fence.
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 7), Some(10));
        assert!(scanner.in_code_block());
        // The 4-backtick run at line_start=13 closes it.
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 13), Some(17));
        assert!(!scanner.in_code_block());
    }
}
