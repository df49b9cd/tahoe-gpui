use std::borrow::Cow;

use super::emphasis::find_trailing_strikethrough;
use super::ranges::CodeBlockRanges;
use super::utils::{cow_append, ends_with_odd_backslashes, is_empty_or_markers};

/// Counts `~~` pairs in the text.
fn count_double_tildes(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'~' && bytes[i + 1] == b'~' {
            count += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    count
}

/// Finds `~~content~` half-complete pattern.
/// Returns the marker index of the opening `~~`.
fn find_half_complete_tilde(text: &str) -> Option<usize> {
    if !text.ends_with('~') || text.ends_with("~~") {
        return None;
    }
    let bytes = text.as_bytes();
    let content_end = bytes.len() - 1; // index of trailing `~`
    if content_end < 3 {
        return None;
    }
    let mut i = content_end - 1;
    while i >= 1 {
        if bytes[i] == b'~' && bytes[i - 1] == b'~' {
            let between = &text[i + 1..content_end];
            if !between.contains('~') {
                return Some(i - 1);
            }
        }
        i -= 1;
    }
    None
}

/// Test-only convenience wrapper that builds `CodeBlockRanges` on the fly.
#[cfg(test)]
fn handle(text: &str) -> Cow<'_, str> {
    handle_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete strikethrough formatting, using pre-computed code block ranges.
pub(crate) fn handle_with_ranges<'a>(text: &'a str, ranges: &CodeBlockRanges) -> Cow<'a, str> {
    if let Some((marker_index, content)) = find_trailing_strikethrough(text) {
        if content.is_empty() || is_empty_or_markers(content) {
            return Cow::Borrowed(text);
        }
        if ranges.is_inside_code(marker_index)
            || ranges.is_within_complete_inline_code(marker_index)
            || ranges.is_within_complete_math(marker_index)
        {
            return Cow::Borrowed(text);
        }

        let pairs = count_double_tildes(text);
        if pairs % 2 == 1 {
            if ends_with_odd_backslashes(text) {
                return Cow::Borrowed(text);
            }
            return cow_append(text, "~~");
        }
    } else {
        // Check for half-complete: ~~content~ → ~~content~~.
        if let Some(marker_index) = find_half_complete_tilde(text)
            && !ranges.is_inside_code(marker_index)
            && !ranges.is_within_complete_inline_code(marker_index)
            && !ranges.is_within_complete_math(marker_index)
        {
            let pairs = count_double_tildes(text);
            if pairs % 2 == 1 {
                if ends_with_odd_backslashes(text) {
                    return Cow::Borrowed(text);
                }
                return cow_append(text, "~");
            }
        }
    }

    Cow::Borrowed(text)
}

#[cfg(test)]
mod tests {
    use super::handle;
    use std::borrow::Cow;

    #[test]
    fn completes_strikethrough() {
        assert_eq!(handle("~~strike text").as_ref(), "~~strike text~~");
    }

    #[test]
    fn half_complete() {
        assert_eq!(handle("~~strike~").as_ref(), "~~strike~~");
    }

    #[test]
    fn leaves_complete() {
        assert!(matches!(handle("~~strike~~"), Cow::Borrowed(_)));
    }

    #[test]
    fn empty_content() {
        assert!(matches!(handle("~~"), Cow::Borrowed(_)));
    }

    #[test]
    fn inside_code_block() {
        assert!(matches!(handle("```\n~~strike\n```"), Cow::Borrowed(_)));
    }

    #[test]
    fn leaves_trailing_backslash() {
        assert!(matches!(handle("~~\\"), Cow::Borrowed(_)));
    }

    #[test]
    fn idempotent_with_trailing_backslash() {
        let once = handle("~~\\").into_owned();
        let twice = handle(&once).into_owned();
        assert_eq!(twice, once);
    }
}
