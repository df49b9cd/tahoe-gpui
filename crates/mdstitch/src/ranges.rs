use std::ops::ControlFlow;

use super::fence::{CodeRegion, InlineTerminator, scan_code_regions};
use super::utils::is_plausible_tag_remainder;

/// Pre-computed ranges of text that are inside code blocks, inline code spans, or math blocks.
/// Used to skip these regions in emphasis/katex handlers without redundant O(n) scans.
///
/// Each handler previously called `is_inside_code_block()` which does a full O(n) scan.
/// With ~15 handlers running per streaming delta, that meant ~15 full scans.
/// This struct computes ranges once (O(n)) and answers queries in O(log n) via binary search.
pub struct CodeBlockRanges {
    /// Sorted, non-overlapping byte ranges that are inside fenced code blocks or inline code.
    /// Each range spans the interior of a code region (between the delimiters).
    code_ranges: Vec<std::ops::Range<usize>>,
    /// Sorted, non-overlapping byte ranges inside *complete* inline code spans
    /// (both opening and closing backtick present).
    complete_inline_code_ranges: Vec<std::ops::Range<usize>>,
    /// Sorted, non-overlapping byte ranges inside math blocks ($..$ or $$..$$).
    math_ranges: Vec<std::ops::Range<usize>>,
    /// Sorted, non-overlapping byte ranges inside *complete* math spans (both
    /// opening and closing `$`/`$$` present). Unterminated math is excluded.
    complete_math_ranges: Vec<std::ops::Range<usize>>,
    /// Sorted, non-overlapping byte ranges inside the URL portion of a link/image
    /// `](url)` whose `)` is on the same line.
    link_url_ranges: Vec<std::ops::Range<usize>>,
    /// Sorted, non-overlapping byte ranges inside a plausible HTML tag opened by `<`.
    html_tag_ranges: Vec<std::ops::Range<usize>>,
}

impl CodeBlockRanges {
    /// Build ranges by scanning the text once.
    pub fn new(text: &str) -> Self {
        let code_ranges = Self::compute_code_ranges(text);
        let complete_inline_code_ranges = Self::compute_complete_inline_code_ranges(text);
        let math_ranges = Self::compute_math_ranges(text);
        let complete_math_ranges = Self::compute_complete_math_ranges(text);
        let link_url_ranges = Self::compute_link_url_ranges(text);
        let html_tag_ranges = Self::compute_html_tag_ranges(text);
        Self {
            code_ranges,
            complete_inline_code_ranges,
            math_ranges,
            complete_math_ranges,
            link_url_ranges,
            html_tag_ranges,
        }
    }

    /// Returns true if the byte position is inside a code block or inline code span.
    ///
    /// Equivalent to `utils::is_inside_code_block(text, position)` but O(log n).
    pub fn is_inside_code(&self, position: usize) -> bool {
        Self::position_in_ranges(&self.code_ranges, position)
    }

    /// Returns true if the position is inside a complete inline code span
    /// (both opening and closing backtick present).
    ///
    /// Equivalent to `utils::is_within_complete_inline_code(text, position)` but O(log n).
    pub fn is_within_complete_inline_code(&self, position: usize) -> bool {
        Self::position_in_ranges(&self.complete_inline_code_ranges, position)
    }

    /// Returns true if the position is inside a math block.
    ///
    /// Equivalent to `utils::is_within_math_block(text, position)` but O(log n).
    pub fn is_within_math(&self, position: usize) -> bool {
        Self::position_in_ranges(&self.math_ranges, position)
    }

    /// Returns true if the position is inside a *complete* math span (both
    /// opening and closing `$`/`$$` present). Unlike `is_within_math`, this
    /// returns `false` for positions after an unclosed `$` — emphasis counters
    /// rely on this so a lone dollar sign doesn't swallow their own trailing
    /// completion markers across passes.
    pub fn is_within_complete_math(&self, position: usize) -> bool {
        Self::position_in_ranges(&self.complete_math_ranges, position)
    }

    /// Returns true if the position is inside a link/image URL `](url)` whose
    /// `)` is on the same line.
    ///
    /// Equivalent to `utils::is_within_link_or_image_url(text, position)` but O(log n).
    pub fn is_within_link_url(&self, position: usize) -> bool {
        Self::position_in_ranges(&self.link_url_ranges, position)
    }

    /// Returns true if the position is inside a plausible HTML tag (between `<`
    /// and the next `>`, `\n`, or EOF).
    ///
    /// Equivalent to `utils::is_within_html_tag(text, position)` but O(log n).
    pub fn is_within_html_tag(&self, position: usize) -> bool {
        Self::position_in_ranges(&self.html_tag_ranges, position)
    }

    /// Binary search to check if `position` falls inside any of the sorted, non-overlapping ranges.
    fn position_in_ranges(ranges: &[std::ops::Range<usize>], position: usize) -> bool {
        // Binary search: find the last range whose start <= position.
        let idx = ranges.partition_point(|r| r.start <= position);
        if idx == 0 {
            return false;
        }
        let range = &ranges[idx - 1];
        position < range.end
    }

    /// Compute code ranges (fenced code blocks + inline code).
    ///
    /// Thin adapter over `scan_code_regions` — the canonical fence/inline state
    /// machine lives in `fence.rs`. Boundary conventions (`start + 1` for the
    /// first delimiter byte, `end + 1` for the first byte of the closer,
    /// `len + 1` for unterminated regions) keep `is_inside_code(pos)` in
    /// agreement with `utils::is_inside_code_block(text, pos)` for every
    /// `pos` in `0..=len`.
    fn compute_code_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let len = text.len();
        let mut ranges = Vec::new();
        scan_code_regions(text, |region| {
            let (start, end) = match region {
                CodeRegion::Fence(f) => {
                    let end = if f.closed {
                        f.close_run_start + 1
                    } else {
                        len + 1
                    };
                    (f.open_run_start + 1, end)
                }
                CodeRegion::Inline(s) => {
                    let end = match s.terminator {
                        InlineTerminator::Closed(p) | InlineTerminator::Newline(p) => p + 1,
                        InlineTerminator::Eof => len + 1,
                    };
                    (s.open_pos + 1, end)
                }
            };
            if start <= end {
                ranges.push(start..end);
            }
            ControlFlow::Continue(())
        });
        ranges
    }

    /// Compute complete inline code ranges.
    ///
    /// Thin adapter over `scan_code_regions` that keeps only inline spans with a
    /// `Closed` terminator. Range interior is `(open_pos, close_pos)` — both
    /// delimiters are excluded, matching `utils::is_within_complete_inline_code`.
    fn compute_complete_inline_code_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let mut ranges = Vec::new();
        scan_code_regions(text, |region| {
            if let CodeRegion::Inline(s) = region
                && let InlineTerminator::Closed(close_pos) = s.terminator
            {
                ranges.push(s.open_pos + 1..close_pos);
            }
            ControlFlow::Continue(())
        });
        ranges
    }

    /// Compute math block ranges (`$..$` and `$$..$$`).
    ///
    /// Ports `utils::is_within_math_block` from a per-position query into a
    /// set of ranges. The original function scans `0..position` and returns
    /// the final toggle state, which gives these boundary rules:
    ///
    /// - Opening `$` (or `$$`) at `j`: the delimiter's first byte is NOT
    ///   inside math (not yet processed at position j), but every position
    ///   past it is — so the range starts at `j + 1`.
    /// - Closing `$` (or `$$`) at `k`: position `k` IS still inside math
    ///   (the toggle fires only when the scan processes `k`), but `k + 1`
    ///   is outside — so the range ends at `k + 1`.
    /// - Unterminated math at end of text: range extends to `len`.
    ///
    /// For `$$xy$$` that yields `[1, 5)` (positions 1..4 inside); for `$x$`
    /// it yields `[1, 3)` (positions 1..2 inside). Cross-validated against
    /// the original function in `tests::matches_original_is_within_math_block`.
    fn compute_math_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut ranges = Vec::new();
        let mut in_inline_math = false;
        let mut in_block_math = false;
        let mut math_start: usize = 0;
        let mut i = 0;

        while i < len {
            // Skip escaped dollar signs.
            if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1] == b'$' {
                i += 2;
                continue;
            }

            if bytes[i] == b'$' {
                // Check for block math ($$).
                if i + 1 < len && bytes[i + 1] == b'$' {
                    if in_block_math {
                        // Closing $$: range ends at i+1 (first $ of closer is still inside).
                        ranges.push(math_start..i + 1);
                        in_block_math = false;
                    } else {
                        in_block_math = true;
                        math_start = i + 1; // After the first byte of opening $$.
                    }
                    i += 2;
                    in_inline_math = false;
                    continue;
                } else if !in_block_math {
                    if in_inline_math {
                        // Closing $: range ends at i+1 (the $ itself is still inside).
                        ranges.push(math_start..i + 1);
                        in_inline_math = false;
                    } else {
                        in_inline_math = true;
                        math_start = i + 1; // After the opening $.
                    }
                }
            }
            i += 1;
        }

        // If still in math at end of text, the rest is "inside math".
        // See matches_original_is_within_math_block (uses 0..=len); using len+1
        // keeps agreement at pos == len for unterminated math, matching the
        // code/inline-code trailing-range convention.
        if (in_block_math || in_inline_math) && math_start <= len {
            ranges.push(math_start..len + 1);
        }

        ranges
    }

    /// Like `compute_math_ranges` but only emits ranges for math spans that
    /// actually close. Unterminated math at EOF produces no range — emphasis
    /// counters rely on this so a lone dollar sign doesn't swallow their
    /// trailing completion markers across passes. Boundary rules otherwise
    /// match `compute_math_ranges` (see its doc comment for worked examples).
    fn compute_complete_math_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut ranges = Vec::new();
        let mut in_inline_math = false;
        let mut in_block_math = false;
        let mut math_start: usize = 0;
        let mut i = 0;

        while i < len {
            if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1] == b'$' {
                i += 2;
                continue;
            }
            if bytes[i] == b'$' {
                if i + 1 < len && bytes[i + 1] == b'$' {
                    if in_block_math {
                        ranges.push(math_start..i + 1);
                        in_block_math = false;
                    } else {
                        in_block_math = true;
                        math_start = i + 1;
                    }
                    i += 2;
                    in_inline_math = false;
                    continue;
                } else if !in_block_math {
                    if in_inline_math {
                        ranges.push(math_start..i + 1);
                        in_inline_math = false;
                    } else {
                        in_inline_math = true;
                        math_start = i + 1;
                    }
                }
            }
            i += 1;
        }

        ranges
    }

    /// Compute link/image URL ranges matching `utils::is_within_link_or_image_url`.
    ///
    /// A range covers positions strictly inside `](url)` where the `)` is on the
    /// same line as the `]`. Incomplete URLs (hitting `\n` or EOF before `)`)
    /// produce no range — matching the original function which returns `false`
    /// for those positions.
    ///
    /// Range boundaries agree with the backward-walk semantics:
    /// - `(` itself is NOT inside (the backward walk stops at `(` and returns
    ///   only when preceded by `]`, then forward-walks from `position`).
    /// - `)` IS inside (backward walk reaches `(`, forward walk sees `)` at
    ///   `bytes[position]`).
    ///
    /// So for `[a](bc)` at offsets 0..=6, the range is `4..7` (covering b, c, `)`).
    fn compute_link_url_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut ranges = Vec::new();
        let mut i = 0;
        while i + 1 < len {
            if bytes[i] == b']' && bytes[i + 1] == b'(' {
                let url_start = i + 2;
                let mut j = url_start;
                while j < len && bytes[j] != b')' && bytes[j] != b'\n' {
                    j += 1;
                }
                if j < len && bytes[j] == b')' {
                    ranges.push(url_start..j + 1);
                    i = j + 1;
                    continue;
                }
                // Incomplete URL — no range, resume scanning past `](`.
                i = url_start;
                continue;
            }
            i += 1;
        }
        ranges
    }

    /// Compute HTML tag ranges matching `utils::is_within_html_tag`.
    ///
    /// A range covers positions inside `<tag...` that look like a plausible
    /// HTML tag (per CommonMark: `<` not preceded by alphanumeric/underscore,
    /// then `is_plausible_tag_remainder` over the rest of the text).
    ///
    /// Range boundaries mirror the backward-walk semantics:
    /// - For `<` at position > 0, the `<` itself is NOT inside (the backward
    ///   walk from that position lands on the preceding byte first).
    /// - For `<` at position 0, the `<` IS inside: `position.saturating_sub(1)`
    ///   re-examines `bytes[0]`, which matches the `b'<'` arm and returns the
    ///   plausibility result.
    /// - `>` and `\n` ARE inside (the backward walk from `>+1` / `\n+1` lands
    ///   on `>` / `\n` and returns false, but walking from `>` / `\n` itself
    ///   continues back to `<`).
    /// - An unterminated tag extends to `len + 1` so `is_within_html_tag(text, len)`
    ///   agrees at the EOF position.
    fn compute_html_tag_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut ranges = Vec::new();
        let mut i = 0;
        while i < len {
            if bytes[i] != b'<' {
                i += 1;
                continue;
            }
            // `<` preceded by an identifier byte is not a tag opener (e.g., `a<b`).
            if i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_') {
                i += 1;
                continue;
            }
            if !is_plausible_tag_remainder(&bytes[i + 1..]) {
                i += 1;
                continue;
            }
            let mut j = i + 1;
            while j < len && bytes[j] != b'>' && bytes[j] != b'\n' {
                j += 1;
            }
            // Include `<` itself only when it sits at offset 0 (saturating_sub quirk
            // in the original backward walk).
            let range_start = if i == 0 { 0 } else { i + 1 };
            let range_end = if j < len { j + 1 } else { len + 1 };
            ranges.push(range_start..range_end);
            // Resume past `>` or `\n`; for EOF, the outer loop exits naturally.
            i = if j < len { j + 1 } else { len };
        }
        ranges
    }
}

#[cfg(test)]
mod tests {
    use super::CodeBlockRanges;
    use crate::utils;

    #[test]
    fn empty_text_produces_no_ranges() {
        let ranges = CodeBlockRanges::new("");
        assert!(!ranges.is_inside_code(0));
        assert!(!ranges.is_within_complete_inline_code(0));
        assert!(!ranges.is_within_math(0));
    }

    #[test]
    fn backtick_fence_range() {
        // "before\n```\ncode here\n```\nafter"
        //  0123456 789...
        let text = "before\n```\ncode here\n```\nafter";
        let ranges = CodeBlockRanges::new(text);
        assert!(!ranges.is_inside_code(5)); // "before"
        assert!(!ranges.is_inside_code(7)); // first byte of opening ``` (not yet processed)
        assert!(ranges.is_inside_code(8)); // second byte of opening ``` (toggle happened at 7)
        assert!(ranges.is_inside_code(15)); // inside "code here"
        assert!(!ranges.is_inside_code(25)); // "after"
    }

    #[test]
    fn tilde_fence_range() {
        let text = "~~~\nhello\n~~~\nafter";
        let ranges = CodeBlockRanges::new(text);
        assert!(!ranges.is_inside_code(0)); // first byte of opening ~~~ (not yet processed)
        assert!(ranges.is_inside_code(1)); // second byte (toggle happened)
        assert!(ranges.is_inside_code(5)); // inside "hello"
        assert!(!ranges.is_inside_code(14)); // "after"
    }

    #[test]
    fn inline_code_range() {
        // "before `code` after"
        //  01234567890123456789
        let text = "before `code` after";
        let ranges = CodeBlockRanges::new(text);
        assert!(!ranges.is_inside_code(5)); // "before"
        assert!(!ranges.is_inside_code(7)); // opening backtick (not yet processed)
        assert!(ranges.is_inside_code(8)); // 'c' of "code" (toggle happened)
        assert!(ranges.is_inside_code(12)); // closing backtick (still inside, not yet processed)
        assert!(!ranges.is_inside_code(13)); // space after closing backtick (toggle happened)
    }

    #[test]
    fn complete_inline_code_range() {
        let text = "before `code` after";
        let ranges = CodeBlockRanges::new(text);
        // Complete inline code interior is between backticks (exclusive on both ends).
        assert!(!ranges.is_within_complete_inline_code(7)); // the opening backtick itself
        assert!(ranges.is_within_complete_inline_code(8)); // 'c' of "code"
        assert!(ranges.is_within_complete_inline_code(11)); // 'e' of "code"
        assert!(!ranges.is_within_complete_inline_code(12)); // closing backtick
    }

    #[test]
    fn incomplete_inline_code_not_in_complete_ranges() {
        let text = "before `incomplete code";
        let ranges = CodeBlockRanges::new(text);
        // Should be inside code (open inline code)
        assert!(ranges.is_inside_code(10));
        // But NOT inside *complete* inline code
        assert!(!ranges.is_within_complete_inline_code(10));
    }

    #[test]
    fn math_block_range() {
        // "before $x+y$ after"
        //  0123456789...
        let text = "before $x+y$ after";
        let ranges = CodeBlockRanges::new(text);
        assert!(!ranges.is_within_math(5)); // "before"
        assert!(!ranges.is_within_math(7)); // the $ itself (not yet processed)
        assert!(ranges.is_within_math(8)); // 'x' (toggle happened)
        assert!(ranges.is_within_math(11)); // closing $ (still inside, not yet processed)
        assert!(!ranges.is_within_math(12)); // space after (toggle happened)
    }

    #[test]
    fn double_dollar_math_range() {
        // "before $$x+y$$ after"
        //  01234567890123456789
        let text = "before $$x+y$$ after";
        let ranges = CodeBlockRanges::new(text);
        assert!(!ranges.is_within_math(7)); // first $ of opening $$ (not yet processed)
        assert!(ranges.is_within_math(8)); // second $ of opening $$ (toggle happened at 7)
        assert!(ranges.is_within_math(10)); // 'x+y'
        assert!(ranges.is_within_math(12)); // first $ of closing $$ (still inside)
        assert!(!ranges.is_within_math(14)); // space after
    }

    #[test]
    fn escaped_backtick_not_code() {
        let text = "\\`not code`";
        let ranges = CodeBlockRanges::new(text);
        assert!(!ranges.is_inside_code(3));
    }

    #[test]
    fn binary_search_boundary_conditions() {
        // "a`b`c`d`e"
        //  012345678
        // Opening ` at 1: range starts at 2. Closing ` at 3: range ends at 4. => [2, 4)
        // Opening ` at 5: range starts at 6. Closing ` at 7: range ends at 8. => [6, 8)
        let text = "a`b`c`d`e";
        let ranges = CodeBlockRanges::new(text);
        assert!(!ranges.is_inside_code(0)); // 'a'
        assert!(!ranges.is_inside_code(1)); // opening '`' (not yet processed)
        assert!(ranges.is_inside_code(2)); // 'b' (inside)
        assert!(ranges.is_inside_code(3)); // closing '`' (still inside)
        assert!(!ranges.is_inside_code(4)); // 'c' (outside)
        assert!(!ranges.is_inside_code(5)); // opening '`' (not yet processed)
        assert!(ranges.is_inside_code(6)); // 'd' (inside)
        assert!(ranges.is_inside_code(7)); // closing '`' (still inside)
        assert!(!ranges.is_inside_code(8)); // 'e' (outside)
    }

    // Cross-validate against the original utility functions for various inputs.
    #[test]
    fn matches_original_is_inside_code_block() {
        let texts = [
            "```code",
            "```code```after",
            "`code",
            "`code`after",
            "~~~\nhello\n~~~",
            "normal text",
            "before `inline` after `more code`",
            "```\n**bold\n```",
            "````\n```\nstill inside\n````",
            // Issue #50: mid-line fence runs are literal text, not fences.
            "hello ```\ncode",
            "text ~~~ more",
            "a ```inline fence``` b",
            "   ```\ncode",
            "    ```\nnot a fence",
            // Issue #50 follow-up: char mismatch, unterminated, and tilde-symmetric.
            "```\ncode\n~~~\nmore",
            "~~~\ncode\n```\nmore",
            "```\nunclosed",
            "~~~\nunclosed",
            "````\n```\nshort closer no good\n````",
            "text ~~~ more\n**bold",
        ];
        for text in &texts {
            let ranges = CodeBlockRanges::new(text);
            // Cover position == len too — C10 (trailing-range push) depends on it.
            for pos in 0..=text.len() {
                let expected = utils::is_inside_code_block(text, pos);
                let actual = ranges.is_inside_code(pos);
                assert_eq!(
                    actual, expected,
                    "is_inside_code mismatch at pos {} in {:?}: expected {}, got {}",
                    pos, text, expected, actual
                );
            }
        }
    }

    #[test]
    fn matches_original_is_within_complete_inline_code() {
        let texts = [
            "before `code` after",
            "`incomplete",
            "`a` `b` `c`",
            "```fence``` not inline",
            "\\`escaped`",
            // Issue #50: mid-line 3+ backtick runs are inert, not inline code.
            "hello ```\ncode",
            "a ```run``` b",
            // Issue #50 follow-up: newline closes any in-progress inline span.
            "`unclosed\nnext",
            "`one`\n`two`",
            // Unterminated inline span at EOF — hits trailing-range push.
            "`still open",
        ];
        for text in &texts {
            let ranges = CodeBlockRanges::new(text);
            for pos in 0..=text.len() {
                let expected = utils::is_within_complete_inline_code(text, pos);
                let actual = ranges.is_within_complete_inline_code(pos);
                assert_eq!(
                    actual, expected,
                    "is_within_complete_inline_code mismatch at pos {} in {:?}: expected {}, got {}",
                    pos, text, expected, actual
                );
            }
        }
    }

    #[test]
    fn matches_original_is_within_math_block() {
        let texts = [
            "$x+y",
            "$x+y$z",
            "$$x+y",
            "\\$x",
            "before $a$ middle $$b$$ after",
        ];
        for text in &texts {
            let ranges = CodeBlockRanges::new(text);
            for pos in 0..=text.len() {
                let expected = utils::is_within_math_block(text, pos);
                let actual = ranges.is_within_math(pos);
                assert_eq!(
                    actual, expected,
                    "is_within_math mismatch at pos {} in {:?}: expected {}, got {}",
                    pos, text, expected, actual
                );
            }
        }
    }

    #[test]
    fn matches_original_is_within_link_or_image_url() {
        let texts = [
            "[a](bc)",
            "[a](bc)d",
            "text [foo](http://example.com) tail",
            "prefix [x](url) middle [y](url2) end",
            "[incomplete](url",
            "[linebreak](\nurl)",
            "[cross-line](url\nmore)",
            "no link here",
            "nested [[inner](url)]",
            "[empty]()",
            "just ](paren?",
        ];
        for text in &texts {
            let ranges = CodeBlockRanges::new(text);
            for pos in 0..=text.len() {
                let expected = utils::is_within_link_or_image_url(text, pos);
                let actual = ranges.is_within_link_url(pos);
                assert_eq!(
                    actual, expected,
                    "is_within_link_url mismatch at pos {} in {:?}: expected {}, got {}",
                    pos, text, expected, actual
                );
            }
        }
    }

    #[test]
    fn matches_original_is_within_html_tag() {
        let texts = [
            "<a href=\"test\">",
            "<a href=\"test\">after",
            "<br/",
            "<br/>",
            "plain text",
            "a<b",
            "name@<example.com",
            "<div class=\"x\">body</div>",
            "<unclosed attr=\"y\"",
            "<p a>next line\ncontent",
            "<p a\nx",
            "<a><br c>",
            "<a> text <br x>",
        ];
        for text in &texts {
            let ranges = CodeBlockRanges::new(text);
            for pos in 0..=text.len() {
                let expected = utils::is_within_html_tag(text, pos);
                let actual = ranges.is_within_html_tag(pos);
                assert_eq!(
                    actual, expected,
                    "is_within_html_tag mismatch at pos {} in {:?}: expected {}, got {}",
                    pos, text, expected, actual
                );
            }
        }
    }
}
