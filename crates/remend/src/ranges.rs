use super::utils::fence_run_length;

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
}

impl CodeBlockRanges {
    /// Build ranges by scanning the text once.
    pub fn new(text: &str) -> Self {
        let code_ranges = Self::compute_code_ranges(text);
        let complete_inline_code_ranges = Self::compute_complete_inline_code_ranges(text);
        let math_ranges = Self::compute_math_ranges(text);
        Self {
            code_ranges,
            complete_inline_code_ranges,
            math_ranges,
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
    /// Ports the logic from `utils::is_inside_code_block` but collects ALL ranges
    /// instead of checking a single position.
    ///
    /// The original function scans `while i < position` and checks the toggle state.
    /// A position is "inside code" if, after scanning everything before it, the
    /// state is toggled on. This means:
    /// - For a fence like ``` at position 0, position 0 is NOT inside code (nothing
    ///   scanned yet), but position 3 (after the fence) IS inside code.
    /// - For inline code `x`, position of ` is NOT inside, but position of x IS.
    ///   The closing ` toggles back, so position after closing ` is NOT inside.
    /// Compute code ranges matching the semantics of `utils::is_inside_code_block`.
    ///
    /// The original scans bytes 0..position-1 and checks toggle state. A fence at
    /// offset j processes all its bytes at once (i jumps past the run). So:
    /// - Opening fence at j: position j is NOT inside (not yet processed), but j+1 IS.
    /// - Closing fence at k: position k IS inside (not yet processed), k+1 is NOT.
    /// - Opening inline `` ` `` at j: j is NOT inside, j+1 IS.
    /// - Closing inline `` ` `` at k: k IS inside, k+1 is NOT.
    ///
    /// Range for each region: [start+1, end+1) where start/end are the first bytes
    /// of the opening/closing delimiters.
    fn compute_code_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut ranges = Vec::new();
        let mut in_code_block = false;
        let mut opening_fence_len: usize = 0;
        let mut code_block_start: usize = 0;
        let mut in_inline_code = false;
        let mut inline_code_start: usize = 0;
        let mut i = 0;

        while i < len {
            // Skip escaped backticks.
            if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1] == b'`' {
                i += 2;
                continue;
            }

            // Check for backtick/tilde fence runs (3+ chars).
            if (bytes[i] == b'`' || bytes[i] == b'~') && !in_inline_code {
                let ch = bytes[i];
                let run = fence_run_length(bytes, i, ch);
                if run >= 3 {
                    if !in_code_block {
                        in_code_block = true;
                        opening_fence_len = run;
                        code_block_start = i + 1; // Position after first byte of opening fence.
                    } else if run >= opening_fence_len {
                        // Closing fence at i: position i is still inside, i+1 is not.
                        let end = i + 1;
                        if code_block_start <= end {
                            ranges.push(code_block_start..end);
                        }
                        in_code_block = false;
                        opening_fence_len = 0;
                    }
                    i += run;
                    continue;
                }
            }

            // Only check for inline code if not in multiline code.
            if !in_code_block && bytes[i] == b'`' {
                if in_inline_code {
                    // Closing backtick at i: position i is still inside, i+1 is not.
                    ranges.push(inline_code_start..i + 1);
                    in_inline_code = false;
                } else {
                    in_inline_code = true;
                    inline_code_start = i + 1; // Position after opening backtick.
                }
            }
            i += 1;
        }

        // If still in a code block or inline code at end of text, the rest is "inside code".
        if in_code_block && code_block_start <= len {
            ranges.push(code_block_start..len);
        } else if in_inline_code && inline_code_start <= len {
            ranges.push(inline_code_start..len);
        }

        ranges
    }

    /// Compute complete inline code ranges.
    ///
    /// Ports the logic from `utils::is_within_complete_inline_code`: only includes
    /// spans where both opening and closing backtick are present.
    fn compute_complete_inline_code_ranges(text: &str) -> Vec<std::ops::Range<usize>> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut ranges = Vec::new();
        let mut in_inline_code = false;
        let mut in_multiline_code = false;
        let mut inline_code_start: usize = 0;
        let mut i = 0;

        while i < len {
            // Skip escaped backticks.
            if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1] == b'`' {
                i += 2;
                continue;
            }

            // Check for backtick fence runs (3+ chars).
            if bytes[i] == b'`' {
                let run = fence_run_length(bytes, i, b'`');
                if run >= 3 {
                    in_multiline_code = !in_multiline_code;
                    i += run;
                    continue;
                }
            }

            // Only check for inline code if not in multiline code.
            if !in_multiline_code && bytes[i] == b'`' {
                if in_inline_code {
                    // Found closing backtick -- this is a complete span.
                    // Range is the interior (between the backticks), matching the
                    // original `start < position && position < i` check.
                    ranges.push(inline_code_start + 1..i);
                    in_inline_code = false;
                } else {
                    in_inline_code = true;
                    inline_code_start = i;
                }
            }
            i += 1;
        }

        // Incomplete inline code spans are NOT included (that's the point).
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
        if (in_block_math || in_inline_math) && math_start < len {
            ranges.push(math_start..len);
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
        ];
        for text in &texts {
            let ranges = CodeBlockRanges::new(text);
            for pos in 0..text.len() {
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
        ];
        for text in &texts {
            let ranges = CodeBlockRanges::new(text);
            for pos in 0..text.len() {
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
            for pos in 0..text.len() {
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
}
