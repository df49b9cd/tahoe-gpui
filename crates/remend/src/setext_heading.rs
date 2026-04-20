use std::borrow::Cow;

use super::utils::cow_append;

/// Detects if text ends with a potential incomplete setext heading underline
/// and adds a zero-width space to break the pattern.
pub fn handle(text: &str) -> Cow<'_, str> {
    if text.is_empty() {
        return Cow::Borrowed(text);
    }

    let last_newline = text.rfind('\n');
    // If there's no newline, we can't have a setext heading.
    let Some(last_newline_idx) = last_newline else {
        return Cow::Borrowed(text);
    };

    let last_line = &text[last_newline_idx + 1..];

    // CM: a setext heading underline must have < 4 columns of leading
    // whitespace — 4+ columns makes the line an indented code block.
    if leading_indent_cols(last_line) >= 4 {
        return Cow::Borrowed(text);
    }

    let trimmed = last_line.trim();

    // Check for 1-2 dashes/equals (not 3+, which is a valid horizontal rule /
    // setext heading).
    if !matches!(trimmed, "-" | "--" | "=" | "==") {
        return Cow::Borrowed(text);
    }

    // Don't modify if last line has trailing space (already breaks the pattern).
    if last_line.ends_with(' ') {
        return Cow::Borrowed(text);
    }

    // CM: the line immediately before the underline must be non-blank.
    // A blank line between the heading text and the underline invalidates
    // the setext heading (the "-"/"=" run becomes a thematic break or
    // list marker instead).
    let previous = &text[..last_newline_idx];
    let prev_line_start = previous.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let prev_line = &previous[prev_line_start..];

    if prev_line.trim().is_empty() {
        return Cow::Borrowed(text);
    }

    // Add zero-width space to break the setext heading pattern.
    cow_append(text, "\u{200B}")
}

fn leading_indent_cols(line: &str) -> usize {
    let mut cols = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => cols += 1,
            '\t' => cols = (cols / 4 + 1) * 4,
            _ => break,
        }
    }
    cols
}

#[cfg(test)]
mod tests {
    use super::handle;
    use std::borrow::Cow;

    #[test]
    fn breaks_dash_setext() {
        assert_eq!(handle("Heading\n-").as_ref(), "Heading\n-\u{200B}");
    }

    #[test]
    fn breaks_double_dash_setext() {
        assert_eq!(handle("Heading\n--").as_ref(), "Heading\n--\u{200B}");
    }

    #[test]
    fn breaks_equals_setext() {
        assert_eq!(handle("Heading\n=").as_ref(), "Heading\n=\u{200B}");
    }

    #[test]
    fn leaves_triple_dash() {
        // Three dashes is a valid horizontal rule, not incomplete.
        assert!(matches!(handle("Heading\n---"), Cow::Borrowed(_)));
    }

    #[test]
    fn no_preceding_content() {
        assert!(matches!(handle("\n-"), Cow::Borrowed(_)));
    }

    #[test]
    fn no_newline() {
        assert!(matches!(handle("just text"), Cow::Borrowed(_)));
    }

    #[test]
    fn four_space_indent_skipped() {
        assert!(matches!(handle("Head\n    -"), Cow::Borrowed(_)));
    }

    #[test]
    fn three_space_indent_still_fires() {
        assert_eq!(handle("Head\n   -").as_ref(), "Head\n   -\u{200B}");
    }

    #[test]
    fn tab_indent_skipped() {
        assert!(matches!(handle("Head\n\t-"), Cow::Borrowed(_)));
    }

    #[test]
    fn blank_line_between_returns_borrowed() {
        assert!(matches!(handle("a\n\n-"), Cow::Borrowed(_)));
    }

    #[test]
    fn whitespace_only_prev_line_returns_borrowed() {
        assert!(matches!(handle("a\n \n-"), Cow::Borrowed(_)));
    }
}
