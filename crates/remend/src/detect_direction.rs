//! Text direction detection using the "first strong character" Unicode algorithm.
//!
//! Ported from Streamdown's `detect-direction.ts`.

/// Text direction: left-to-right or right-to-left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextDirection {
    /// Left-to-right (default for Latin, CJK, Cyrillic, etc.)
    #[default]
    Ltr,
    /// Right-to-left (Hebrew, Arabic, Syriac, Thaana, etc.)
    Rtl,
}

/// Returns `true` if the character is in a Unicode RTL "strong" range.
///
/// Covers: Hebrew, Arabic, Syriac, Thaana, NKo, Samaritan, Mandaic,
/// Arabic Supplement/Extended, and RTL presentation forms.
fn is_rtl_char(ch: char) -> bool {
    let cp = ch as u32;
    (0x0590..=0x08FF).contains(&cp)        // Hebrew through Arabic Extended
        || (0xFB1D..=0xFDFF).contains(&cp)  // Alphabetic Presentation Forms + Arabic Presentation Forms-A
        || (0xFE70..=0xFEFF).contains(&cp) // Arabic Presentation Forms-B
}

/// Detects text direction using the "first strong character" algorithm.
///
/// Strips common markdown syntax (headings, bold/italic markers, inline code,
/// links, list markers) then finds the first Unicode letter with strong
/// directionality.
///
/// Returns [`TextDirection::Rtl`] if the first strong character is RTL,
/// [`TextDirection::Ltr`] otherwise.
pub fn detect_text_direction(text: &str) -> TextDirection {
    // Iterate directly over char_indices to avoid allocating a Vec<char>.
    let bytes = text.as_bytes();
    let mut iter = text.char_indices().peekable();

    while let Some((byte_pos, ch)) = iter.next() {
        // Skip heading markers at line start: #{1,6} followed by space.
        if ch == '#' && is_line_start(bytes, byte_pos) {
            while iter.peek().is_some_and(|(_, c)| *c == '#') {
                let _ = iter.next();
            }
            while iter.peek().is_some_and(|(_, c)| *c == ' ') {
                let _ = iter.next();
            }
            continue;
        }

        // Skip bold/italic markers (*, _).
        if ch == '*' || ch == '_' {
            continue;
        }

        // Skip inline code: `...`
        if ch == '`' {
            while iter.peek().is_some_and(|(_, c)| *c != '`') {
                let _ = iter.next();
            }
            if iter.peek().is_some() {
                let _ = iter.next(); // skip closing `
            }
            continue;
        }

        // Skip links: [text](url) — keep the text, skip the bracket itself.
        if ch == '[' {
            continue;
        }
        // When we see `](`, skip through the closing `)`.
        if ch == ']' {
            if iter.peek().is_some_and(|(_, c)| *c == '(') {
                let _ = iter.next(); // skip `(`
                while iter.peek().is_some_and(|(_, c)| *c != ')') {
                    let _ = iter.next();
                }
                if iter.peek().is_some() {
                    let _ = iter.next(); // skip `)`
                }
                continue;
            }
            continue;
        }

        // Skip line-start markers: >, -, +, digits followed by ., spaces.
        if is_line_start(bytes, byte_pos) && is_list_or_quote_char(ch) {
            while iter
                .peek()
                .is_some_and(|(_, c)| is_list_or_quote_char(*c) || c.is_ascii_digit() || *c == '.')
            {
                let _ = iter.next();
            }
            while iter.peek().is_some_and(|(_, c)| *c == ' ') {
                let _ = iter.next();
            }
            continue;
        }

        // Check if this is a strong directional character (any letter).
        if ch.is_alphabetic() {
            if is_rtl_char(ch) {
                return TextDirection::Rtl;
            }
            return TextDirection::Ltr;
        }
    }

    TextDirection::Ltr
}

/// Returns true if `byte_offset` is at the start of a line.
fn is_line_start(bytes: &[u8], byte_offset: usize) -> bool {
    if byte_offset == 0 {
        return true;
    }
    let mut j = byte_offset;
    while j > 0 {
        j -= 1;
        if bytes[j] == b'\n' {
            return true;
        }
        if bytes[j] != b' ' && bytes[j] != b'\t' {
            return false;
        }
    }
    true
}

fn is_list_or_quote_char(ch: char) -> bool {
    matches!(ch, '>' | '-' | '+' | '*')
}

#[cfg(test)]
mod tests {
    use super::{TextDirection, detect_text_direction};

    #[test]
    fn english_text() {
        assert_eq!(detect_text_direction("Hello world"), TextDirection::Ltr);
    }

    #[test]
    fn hebrew_text() {
        assert_eq!(detect_text_direction("שלום עולם"), TextDirection::Rtl);
    }

    #[test]
    fn arabic_text() {
        assert_eq!(detect_text_direction("مرحبا بالعالم"), TextDirection::Rtl);
    }

    #[test]
    fn heading_with_hebrew() {
        assert_eq!(detect_text_direction("## שלום"), TextDirection::Rtl);
    }

    #[test]
    fn bold_english() {
        assert_eq!(detect_text_direction("**hello**"), TextDirection::Ltr);
    }

    #[test]
    fn bold_arabic() {
        assert_eq!(detect_text_direction("**مرحبا**"), TextDirection::Rtl);
    }

    #[test]
    fn link_with_hebrew_text() {
        assert_eq!(
            detect_text_direction("[שלום](https://example.com)"),
            TextDirection::Rtl
        );
    }

    #[test]
    fn inline_code_then_hebrew() {
        assert_eq!(detect_text_direction("`code` שלום"), TextDirection::Rtl);
    }

    #[test]
    fn numbers_then_english() {
        assert_eq!(detect_text_direction("123 hello"), TextDirection::Ltr);
    }

    #[test]
    fn numbers_then_arabic() {
        assert_eq!(detect_text_direction("123 مرحبا"), TextDirection::Rtl);
    }

    #[test]
    fn empty_string() {
        assert_eq!(detect_text_direction(""), TextDirection::Ltr);
    }

    #[test]
    fn only_numbers() {
        assert_eq!(detect_text_direction("12345"), TextDirection::Ltr);
    }

    #[test]
    fn list_item_hebrew() {
        assert_eq!(detect_text_direction("- שלום"), TextDirection::Rtl);
    }

    #[test]
    fn blockquote_arabic() {
        assert_eq!(detect_text_direction("> مرحبا"), TextDirection::Rtl);
    }

    #[test]
    fn mixed_ltr_first() {
        assert_eq!(detect_text_direction("Hello שלום"), TextDirection::Ltr);
    }

    #[test]
    fn mixed_rtl_first() {
        assert_eq!(detect_text_direction("שלום Hello"), TextDirection::Rtl);
    }

    #[test]
    fn cjk_is_ltr() {
        assert_eq!(detect_text_direction("你好世界"), TextDirection::Ltr);
    }

    #[test]
    fn cyrillic_is_ltr() {
        assert_eq!(detect_text_direction("Привет мир"), TextDirection::Ltr);
    }
}
