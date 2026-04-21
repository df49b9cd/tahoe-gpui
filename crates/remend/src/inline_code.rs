use std::borrow::Cow;

use super::utils::{count_single_backticks, cow_append, ends_with_odd_backslashes};

/// Returns `true` if the text is inside an incomplete fenced code block.
///
/// Delegates to the CommonMark-aware fence parser which handles both
/// backtick and tilde fences with proper length matching.
fn is_inside_incomplete_code_block(text: &str) -> bool {
    crate::incomplete_code::has_incomplete_code_fence(text)
}

/// Handles inline triple backticks on a single line (not a fence).
/// E.g. `` ```code`` `` → `` ```code``` ``.
fn handle_inline_triple_backticks(text: &str) -> Option<Cow<'_, str>> {
    // Must be single-line and start with ```.
    if text.contains('\n') || !text.starts_with("```") {
        return None;
    }
    // Must have a stray backtick after the opening ``` without a complete
    // closing ``` already in place.
    let rest = &text[3..];
    if !rest.contains('`') || rest.ends_with("```") {
        return None;
    }
    // Dangling `` at the end means only one more backtick is needed to close.
    // (The "already ends in ```" case was ruled out above, so that branch
    // was unreachable in the previous implementation.)
    if text.ends_with("``") {
        return Some(cow_append(text, "`"));
    }
    None
}

/// Completes incomplete inline code formatting (`` ` ``).
pub fn handle(text: &str) -> Cow<'_, str> {
    // Check for inline triple backticks first.
    if let Some(result) = handle_inline_triple_backticks(text) {
        return result;
    }

    // Check if we're inside an incomplete fenced code block — don't close backticks there.
    if is_inside_incomplete_code_block(text) {
        return Cow::Borrowed(text);
    }

    // Look for incomplete inline code: odd number of single backticks.
    // First, check if there's even a backtick.
    if !text.contains('`') {
        return Cow::Borrowed(text);
    }

    // Find the last single backtick and check content after it.
    let bytes = text.as_bytes();
    let mut last_backtick = None;
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if bytes[i] == b'`' {
            // Skip if part of ```.
            if (i + 2 < bytes.len() && bytes[i + 1] == b'`' && bytes[i + 2] == b'`')
                || (i >= 1 && bytes[i - 1] == b'`')
            {
                continue;
            }
            // Skip if escaped.
            if i >= 1 && bytes[i - 1] == b'\\' {
                continue;
            }
            last_backtick = Some(i);
            break;
        }
    }

    if last_backtick.is_none() {
        return Cow::Borrowed(text);
    }

    // Check if content after the opening backtick is meaningful.
    if let Some(pos) = last_backtick {
        let content = &text[pos + 1..];
        if content.is_empty()
            || content
                .bytes()
                .all(|b| matches!(b, b' ' | b'\t' | b'\n' | b'_' | b'~' | b'*' | b'`'))
        {
            return Cow::Borrowed(text);
        }
    }

    let count = count_single_backticks(text);
    if count % 2 == 1 {
        if ends_with_odd_backslashes(text) {
            return Cow::Borrowed(text);
        }
        return cow_append(text, "`");
    }

    Cow::Borrowed(text)
}

#[cfg(test)]
mod tests {
    use super::handle;
    use std::borrow::Cow;

    #[test]
    fn completes_inline_code() {
        assert_eq!(handle("`code").as_ref(), "`code`");
    }

    #[test]
    fn leaves_complete_inline_code() {
        assert!(matches!(handle("`code`"), Cow::Borrowed(_)));
    }

    #[test]
    fn does_not_complete_in_code_block() {
        assert!(matches!(handle("```\n`code\n"), Cow::Borrowed(_)));
    }

    #[test]
    fn empty_content() {
        assert!(matches!(handle("`"), Cow::Borrowed(_)));
    }

    #[test]
    fn escaped_backtick() {
        assert!(matches!(handle("\\`code"), Cow::Borrowed(_)));
    }

    #[test]
    fn leaves_trailing_backslash() {
        assert!(matches!(handle("`\\"), Cow::Borrowed(_)));
    }

    #[test]
    fn idempotent_with_trailing_backslash() {
        let once = handle("`\\").into_owned();
        let twice = handle(&once).into_owned();
        assert_eq!(twice, once);
    }
}
