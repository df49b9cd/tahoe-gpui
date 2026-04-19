use std::borrow::Cow;

use super::utils::{FenceScanner, cow_append, is_part_of_triple_backtick};

/// What the dollar-scanner sees at each non-escaped, non-code-block position.
enum DollarToken {
    /// `$$` run (two or more dollars). Argument is the run length.
    Double,
    /// A single `$` (not part of `$$`).
    Single,
}

/// Walk `text`, skipping escapes, fenced code blocks, and inline code spans,
/// and invoke `f` for every `$` / `$$` occurrence found outside those
/// regions. Lets `count_dollar_pairs` and `count_single_dollars` share a
/// single scanning loop instead of duplicating the fence/inline-code
/// bookkeeping.
fn scan_dollars(text: &str, mut f: impl FnMut(DollarToken)) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut in_inline_code = false;
    let mut scanner = FenceScanner::new();
    let mut i = 0;

    while i < len {
        // Skip escaped characters.
        if bytes[i] == b'\\' && i + 1 < len {
            i += 2;
            continue;
        }
        if let Some(next) = scanner.consume_fence(bytes, i) {
            i = next;
            continue;
        }
        if scanner.in_code_block() {
            i += 1;
            continue;
        }
        if bytes[i] == b'`' && !is_part_of_triple_backtick(text, i) {
            in_inline_code = !in_inline_code;
            i += 1;
            continue;
        }
        if !in_inline_code && bytes[i] == b'$' {
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

/// Counts `$$` pairs outside of inline code blocks and fenced code blocks.
fn count_dollar_pairs(text: &str) -> usize {
    let mut pairs = 0;
    scan_dollars(text, |tok| {
        if let DollarToken::Double = tok {
            pairs += 1;
        }
    });
    pairs
}

/// Counts single `$` signs (excluding `$$`) outside of code blocks.
fn count_single_dollars(text: &str) -> usize {
    let mut count = 0;
    scan_dollars(text, |tok| {
        if let DollarToken::Single = tok {
            count += 1;
        }
    });
    count
}

/// Completes incomplete block KaTeX formatting (`$$`).
pub fn handle_block(text: &str) -> Cow<'_, str> {
    let pairs = count_dollar_pairs(text);
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
    let count = count_single_dollars(text);
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
