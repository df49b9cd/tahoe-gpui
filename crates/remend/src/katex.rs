use std::borrow::Cow;

use super::ranges::CodeBlockRanges;
use super::utils::cow_append;

/// What the dollar-scanner sees at each non-escaped, non-code-block position.
enum DollarToken {
    /// `$$` run (two or more dollars). Argument is the run length.
    Double,
    /// A single `$` (not part of `$$`).
    Single,
}

/// Walk `text` using pre-computed code block ranges to skip code regions.
/// Only checks `ranges.is_inside_code(i)` at `$` positions, which is
/// much cheaper than per-byte fence/inline-code tracking.
fn scan_dollars_with_ranges(text: &str, ranges: &CodeBlockRanges, mut f: impl FnMut(DollarToken)) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Skip escaped characters.
        if bytes[i] == b'\\' && i + 1 < len {
            i += 2;
            continue;
        }
        if bytes[i] == b'$' && !ranges.is_inside_code(i) {
            if i + 1 < len && bytes[i + 1] == b'$' {
                f(DollarToken::Double);
                i += 2;
            } else {
                f(DollarToken::Single);
                i += 1;
            }
            continue;
        }
        i += 1;
    }
}

/// Counts `$$` pairs outside of code blocks, using pre-computed ranges.
fn count_dollar_pairs_with_ranges(text: &str, ranges: &CodeBlockRanges) -> usize {
    let mut pairs = 0;
    scan_dollars_with_ranges(text, ranges, |tok| {
        if let DollarToken::Double = tok {
            pairs += 1;
        }
    });
    pairs
}

/// Counts single `$` signs outside of code blocks, using pre-computed ranges.
fn count_single_dollars_with_ranges(text: &str, ranges: &CodeBlockRanges) -> usize {
    let mut count = 0;
    scan_dollars_with_ranges(text, ranges, |tok| {
        if let DollarToken::Single = tok {
            count += 1;
        }
    });
    count
}

/// Completes incomplete block KaTeX formatting (`$$`).
pub fn handle_block(text: &str) -> Cow<'_, str> {
    handle_block_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete block KaTeX formatting, using pre-computed code block ranges.
pub fn handle_block_with_ranges<'a>(text: &'a str, ranges: &CodeBlockRanges) -> Cow<'a, str> {
    let pairs = count_dollar_pairs_with_ranges(text, ranges);
    if pairs.is_multiple_of(2) {
        return Cow::Borrowed(text);
    }

    // If text already ends with a single $ (but not $$), just add one more.
    if text.ends_with('$') && !text.ends_with("$$") {
        return cow_append(text, "$");
    }

    // If there's a newline after the opening $$ and text doesn't end with newline,
    // add newline before closing $$.
    if let Some(first_dollar) = text.find("$$") {
        let has_newline_after = text[first_dollar..].contains('\n');
        if has_newline_after && !text.ends_with('\n') {
            return cow_append(text, "\n$$");
        }
    }

    cow_append(text, "$$")
}

/// Completes incomplete inline KaTeX formatting (`$`).
pub fn handle_inline(text: &str) -> Cow<'_, str> {
    handle_inline_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete inline KaTeX formatting, using pre-computed code block ranges.
pub fn handle_inline_with_ranges<'a>(text: &'a str, ranges: &CodeBlockRanges) -> Cow<'a, str> {
    let count = count_single_dollars_with_ranges(text, ranges);
    if count % 2 == 1 {
        return cow_append(text, "$");
    }
    Cow::Borrowed(text)
}

#[cfg(test)]
mod tests {
    use super::{handle_block, handle_inline};
    use std::borrow::Cow;

    #[test]
    fn completes_block_katex() {
        assert_eq!(handle_block("$$x + y").as_ref(), "$$x + y$$");
    }

    #[test]
    fn completes_block_katex_multiline() {
        assert_eq!(handle_block("$$\nx + y").as_ref(), "$$\nx + y\n$$");
    }

    #[test]
    fn leaves_complete_block_katex() {
        assert!(matches!(handle_block("$$x + y$$"), Cow::Borrowed(_)));
    }

    #[test]
    fn half_complete_dollar() {
        assert_eq!(handle_block("$$x + y$").as_ref(), "$$x + y$$");
    }

    #[test]
    fn completes_inline_katex() {
        assert_eq!(handle_inline("$x + y").as_ref(), "$x + y$");
    }

    #[test]
    fn leaves_complete_inline_katex() {
        assert!(matches!(handle_inline("$x + y$"), Cow::Borrowed(_)));
    }

    #[test]
    fn ignores_dollar_pairs_inside_fenced_code() {
        // $$ inside ``` should not be counted as math delimiters.
        assert!(matches!(
            handle_block("```\n$$x + y\n```"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn ignores_escaped_dollar_pairs() {
        // Escaped \$$ should not be counted.
        assert!(matches!(handle_block("\\$$x"), Cow::Borrowed(_)));
    }

    #[test]
    fn ignores_single_dollar_inside_fenced_code() {
        assert!(matches!(
            handle_inline("```\n$x + y\n```"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn ignores_dollar_pairs_inside_tilde_fence() {
        assert!(matches!(
            handle_block("~~~\n$$x + y\n~~~"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn ignores_single_dollar_inside_tilde_fence() {
        assert!(matches!(
            handle_inline("~~~\n$x + y\n~~~"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn four_backtick_fence_not_closed_by_three() {
        // 4-backtick fence should not be closed by 3 backticks.
        assert!(matches!(
            handle_block("````\n```\n$$x + y"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn four_tilde_fence_not_closed_by_three() {
        assert!(matches!(
            handle_block("~~~~\n~~~\n$$x + y"),
            Cow::Borrowed(_)
        ));
    }
}
