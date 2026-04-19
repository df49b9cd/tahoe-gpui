use std::borrow::Cow;

use super::utils::{is_inside_code_block, is_word_char};

/// Escapes single `~` characters between word characters to prevent
/// false strikethrough interpretation. E.g. `20~25°C` → `20\~25°C`.
pub fn handle(text: &str) -> Cow<'_, str> {
    if !text.contains('~') {
        return Cow::Borrowed(text);
    }

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut result: Option<String> = None;
    let mut last_copy = 0;

    for i in 0..len {
        if bytes[i] != b'~' {
            continue;
        }
        // Must be a single ~ (not ~~).
        if i > 0 && bytes[i - 1] == b'~' {
            continue;
        }
        if i + 1 < len && bytes[i + 1] == b'~' {
            continue;
        }
        // Must be between word characters.
        if i == 0 || i + 1 >= len {
            continue;
        }

        // Get the surrounding characters as Unicode chars.
        let prev_str = &text[..i];
        let next_str = &text[i + 1..];
        let prev_char = prev_str.chars().next_back();
        let next_char = next_str.chars().next();

        let between_words = matches!((prev_char, next_char), (Some(p), Some(n)) if is_word_char(p) && is_word_char(n));

        if !between_words {
            continue;
        }

        // Don't escape inside code blocks.
        if is_inside_code_block(text, i) {
            continue;
        }

        // Escape this tilde.
        let buf = result.get_or_insert_with(|| String::with_capacity(text.len() + 8));
        buf.push_str(&text[last_copy..i]);
        buf.push_str("\\~");
        last_copy = i + 1;
    }

    match result {
        Some(mut buf) => {
            buf.push_str(&text[last_copy..]);
            Cow::Owned(buf)
        }
        None => Cow::Borrowed(text),
    }
}

#[cfg(test)]
mod tests {
    use super::handle;
    use std::borrow::Cow;

    #[test]
    fn escapes_tilde_between_words() {
        assert_eq!(handle("20~25").as_ref(), "20\\~25");
    }

    #[test]
    fn leaves_double_tilde() {
        assert!(matches!(handle("~~strike~~"), Cow::Borrowed(_)));
    }

    #[test]
    fn no_tilde() {
        assert!(matches!(handle("hello world"), Cow::Borrowed(_)));
    }

    #[test]
    fn tilde_at_boundaries() {
        assert!(matches!(handle("~start"), Cow::Borrowed(_)));
        assert!(matches!(handle("end~"), Cow::Borrowed(_)));
    }

    #[test]
    fn tilde_between_spaces() {
        assert!(matches!(handle("a ~ b"), Cow::Borrowed(_)));
    }
}
