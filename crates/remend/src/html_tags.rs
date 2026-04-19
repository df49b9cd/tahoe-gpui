use std::borrow::Cow;

use super::utils::is_inside_code_block;

/// Strips incomplete HTML tags at the end of streaming text.
/// E.g. `text <custom` → `text`.
pub fn handle(text: &str) -> Cow<'_, str> {
    let bytes = text.as_bytes();

    // Scan backward for `<` that starts an incomplete tag.
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if bytes[i] == b'>' {
            // Found a closing `>` — no incomplete tag at end.
            return Cow::Borrowed(text);
        }
        if bytes[i] == b'<' {
            // Check that it starts a valid tag (followed by letter or /).
            let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
            if next.is_ascii_alphabetic() || next == b'/' {
                // Check there's no `>` after this `<` (which would close the tag).
                if !text[i..].contains('>') {
                    // Don't strip if inside a code block.
                    if is_inside_code_block(text, i) {
                        return Cow::Borrowed(text);
                    }
                    // Strip the incomplete tag and trailing whitespace.
                    let trimmed = text[..i].trim_end();
                    return Cow::Owned(trimmed.to_owned());
                }
            }
            return Cow::Borrowed(text);
        }
        if bytes[i] == b'\n' {
            // Tags don't span lines; stop scanning.
            return Cow::Borrowed(text);
        }
    }

    Cow::Borrowed(text)
}

#[cfg(test)]
mod tests {
    use super::handle;
    use std::borrow::Cow;

    #[test]
    fn strips_incomplete_tag() {
        assert_eq!(handle("text <custom").as_ref(), "text");
    }

    #[test]
    fn strips_incomplete_closing_tag() {
        assert_eq!(handle("text </div").as_ref(), "text");
    }

    #[test]
    fn leaves_complete_tag() {
        assert!(matches!(
            handle("text <div>content</div>"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn leaves_non_tag() {
        assert!(matches!(handle("5 < 10"), Cow::Borrowed(_)));
    }

    #[test]
    fn inside_code_block() {
        assert!(matches!(handle("```\n<custom\n```"), Cow::Borrowed(_)));
    }
}
