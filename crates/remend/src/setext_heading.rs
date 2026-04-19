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

    // Check if previous line has content.
    let previous = &text[..last_newline_idx];
    let prev_line = previous
        .rfind('\n')
        .map(|p| &previous[p + 1..])
        .unwrap_or(previous);

    if prev_line.trim().is_empty() {
        return Cow::Borrowed(text);
    }

    // Add zero-width space to break the setext heading pattern.
    cow_append(text, "\u{200B}")
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
}
