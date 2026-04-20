use std::borrow::Cow;

use super::options::LinkMode;
use super::utils::{
    find_matching_closing_bracket, find_matching_opening_bracket, is_inside_code_block,
    is_list_marker_line,
};

/// Handles incomplete URLs in links/images: `[text](partial-url`.
fn handle_incomplete_url(
    text: &str,
    bracket_paren_index: usize,
    link_mode: LinkMode,
) -> Option<Cow<'_, str>> {
    // `bracket_paren_index` points to `]` in `](`.
    // Only consider `)` on the same line — a `)` later in the document
    // (e.g., from an emoticon or another link) should not prevent completion.
    let after_paren = &text[bracket_paren_index + 2..];
    if after_paren.lines().next().unwrap_or("").contains(')') {
        return None; // URL is complete.
    }

    // Find matching `[` for the `]`.
    let open = find_matching_opening_bracket(text, bracket_paren_index)?;

    if is_inside_code_block(text, open) {
        return None;
    }

    // `open` is a byte offset pointing at `[`. Using `str::ends_with` here
    // is UTF-8-safe whereas raw byte indexing could land on a continuation
    // byte of a multibyte code point. `b'!'` (0x21) never collides with a
    // UTF-8 continuation byte (0x80–0xBF), but the safer form also reads
    // more clearly to future maintainers.
    let is_image = text[..open].ends_with('!');
    let start = if is_image { open - 1 } else { open };
    let before = &text[..start];

    if is_image {
        // Incomplete images are removed entirely (trim trailing whitespace).
        return Some(Cow::Owned(before.trim_end().to_owned()));
    }

    let link_text = &text[open + 1..bracket_paren_index];

    match link_mode {
        LinkMode::TextOnly => {
            // Display only the link text without markup.
            let mut result = String::with_capacity(before.len() + link_text.len());
            result.push_str(before);
            result.push_str(link_text);
            Some(Cow::Owned(result))
        }
        LinkMode::Protocol => {
            // Replace URL with placeholder.
            let mut result = String::with_capacity(before.len() + link_text.len() + 32);
            result.push_str(before);
            result.push('[');
            result.push_str(link_text);
            result.push_str("](streamdown:incomplete-link)");
            Some(Cow::Owned(result))
        }
    }
}

/// Handles incomplete link text: `[partial-text` without closing `]`.
fn handle_incomplete_text(
    text: &str,
    open_index: usize,
    link_mode: LinkMode,
) -> Option<Cow<'_, str>> {
    let is_image = text[..open_index].ends_with('!');
    let start = if is_image { open_index - 1 } else { open_index };

    // Check if there's a closing bracket after this.
    let after = &text[open_index + 1..];
    if !after.contains(']') {
        // Incomplete link/image.
        let before = &text[..start];

        if is_image {
            return Some(Cow::Owned(before.trim_end().to_owned()));
        }

        return Some(make_incomplete_link(text, open_index, link_mode));
    }

    // Check if the closing bracket actually matches (accounting for nesting).
    let closing = find_matching_closing_bracket(text, open_index);
    if closing.is_none() {
        let before = &text[..start];
        if is_image {
            return Some(Cow::Owned(before.trim_end().to_owned()));
        }
        return Some(make_incomplete_link(text, open_index, link_mode));
    }

    None
}

/// Finds the first incomplete `[` by scanning forward, skipping complete links.
/// `max_pos` is the position of a known incomplete `[` (fallback).
fn find_first_incomplete_bracket(text: &str, max_pos: usize) -> usize {
    let bytes = text.as_bytes();
    let mut j = 0;
    while j < max_pos {
        if bytes[j] == b'[' && !is_inside_code_block(text, j) {
            // Skip images.
            if text[..j].ends_with('!') {
                j += 1;
                continue;
            }
            // Check if this `[` has a matching `]`.
            if let Some(close_idx) = find_matching_closing_bracket(text, j) {
                // Check if it's a full link `[text](url)`.
                if close_idx + 1 < bytes.len()
                    && bytes[close_idx + 1] == b'('
                    && let Some(url_end) = text[close_idx + 2..].find(')')
                {
                    // Skip past this complete link.
                    j = close_idx + 2 + url_end + 1;
                    continue;
                }
                j = close_idx + 1;
            } else {
                // This is an incomplete `[`.
                return j;
            }
        } else {
            j += 1;
        }
    }
    // Fallback: the bracket at max_pos is always incomplete by contract.
    max_pos
}

/// Creates the appropriate incomplete link output based on link mode.
fn make_incomplete_link<'a>(text: &str, open_index: usize, link_mode: LinkMode) -> Cow<'a, str> {
    match link_mode {
        LinkMode::TextOnly => {
            // Find the first incomplete `[` (scanning forward) and strip just that bracket.
            let first_incomplete = find_first_incomplete_bracket(text, open_index);
            let mut result = String::with_capacity(text.len());
            result.push_str(&text[..first_incomplete]);
            result.push_str(&text[first_incomplete + 1..]);
            Cow::Owned(result)
        }
        LinkMode::Protocol => {
            let mut result = String::with_capacity(text.len() + 32);
            result.push_str(text);
            result.push_str("](streamdown:incomplete-link)");
            Cow::Owned(result)
        }
    }
}

/// Returns `true` if the `[` at `bracket_pos` is the start of a GFM
/// task-list marker (`- [`, `- [x`, `  * [X]`, etc.). These must not be
/// auto-completed as incomplete links during streaming.
fn is_task_list_marker_start(text: &str, bracket_pos: usize) -> bool {
    let bytes = text.as_bytes();

    let line_start = bytes[..bracket_pos]
        .iter()
        .rposition(|&b| b == b'\n')
        .map(|p| p + 1)
        .unwrap_or(0);

    if !is_list_marker_line(&text[line_start..bracket_pos]) {
        return false;
    }

    matches!(bytes.get(bracket_pos + 1), Some(b' ' | b'x' | b'X'))
}

/// Handles incomplete links and images by auto-completing or removing them.
///
/// When `links_enabled` is false, incomplete links are left untouched.
/// When `images_enabled` is false, incomplete images are left untouched.
pub fn handle(
    text: &str,
    link_mode: LinkMode,
    links_enabled: bool,
    images_enabled: bool,
) -> Cow<'_, str> {
    if !links_enabled && !images_enabled {
        return Cow::Borrowed(text);
    }

    let bytes = text.as_bytes();

    // Phase 1: Look for `](` pattern — incomplete URL.
    if let Some(pos) = text.rfind("](")
        && !is_inside_code_block(text, pos)
    {
        // Check if this is an image (preceded by `![`).
        let open = find_matching_opening_bracket(text, pos);
        let is_image = open.is_some_and(|o| text[..o].ends_with('!'));
        if ((is_image && images_enabled) || (!is_image && links_enabled))
            && let Some(result) = handle_incomplete_url(text, pos, link_mode)
        {
            return result;
        }
    }

    // Phase 2: Scan backward for unmatched `[`.
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if bytes[i] == b'[' && !is_inside_code_block(text, i) {
            let is_image = text[..i].ends_with('!');
            if (is_image && !images_enabled) || (!is_image && !links_enabled) {
                continue;
            }
            if !is_image && is_task_list_marker_start(text, i) {
                continue;
            }
            if let Some(result) = handle_incomplete_text(text, i, link_mode) {
                return result;
            }
        }
    }

    Cow::Borrowed(text)
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::options::LinkMode;
    use std::borrow::Cow;

    fn h(text: &str) -> Cow<'_, str> {
        handle(text, LinkMode::Protocol, true, true)
    }

    fn h_text_only(text: &str) -> Cow<'_, str> {
        handle(text, LinkMode::TextOnly, true, true)
    }

    #[test]
    fn completes_incomplete_link_url() {
        assert_eq!(
            h("[Click here](http://exam").as_ref(),
            "[Click here](streamdown:incomplete-link)"
        );
    }

    #[test]
    fn completes_incomplete_link_text() {
        assert_eq!(
            h("[Click here").as_ref(),
            "[Click here](streamdown:incomplete-link)"
        );
    }

    #[test]
    fn removes_incomplete_image() {
        assert_eq!(h("text ![alt](http://").as_ref(), "text");
    }

    #[test]
    fn removes_incomplete_image_text() {
        assert_eq!(h("text ![alt").as_ref(), "text");
    }

    #[test]
    fn leaves_complete_link() {
        assert!(matches!(h("[text](http://example.com)"), Cow::Borrowed(_)));
    }

    #[test]
    fn inside_code_block() {
        assert!(matches!(h("```\n[incomplete\n```"), Cow::Borrowed(_)));
    }

    // Text-only mode tests
    #[test]
    fn text_only_incomplete_url() {
        assert_eq!(
            h_text_only("[Click here](http://exam").as_ref(),
            "Click here"
        );
    }

    #[test]
    fn text_only_incomplete_text() {
        assert_eq!(h_text_only("Text [partial").as_ref(), "Text partial");
    }

    #[test]
    fn text_only_complete_unchanged() {
        assert_eq!(
            h_text_only("[text](http://example.com)").as_ref(),
            "[text](http://example.com)"
        );
    }

    #[test]
    fn leaves_unordered_task_list_marker_x() {
        assert_eq!(h("- [x").as_ref(), "- [x");
    }

    #[test]
    fn leaves_unordered_task_list_marker_space() {
        assert_eq!(h("- [ ").as_ref(), "- [ ");
    }

    #[test]
    fn leaves_unordered_task_list_marker_capital_x() {
        assert_eq!(h("- [X").as_ref(), "- [X");
    }

    #[test]
    fn leaves_task_list_with_other_bullet_styles() {
        assert_eq!(h("* [x").as_ref(), "* [x");
        assert_eq!(h("+ [ ").as_ref(), "+ [ ");
    }

    #[test]
    fn leaves_indented_task_list_marker() {
        assert_eq!(h("  - [x").as_ref(), "  - [x");
    }

    #[test]
    fn completes_real_link_in_list_item() {
        assert_eq!(h("- [foo").as_ref(), "- [foo](streamdown:incomplete-link)");
    }

    #[test]
    fn text_only_leaves_task_list_marker() {
        assert_eq!(h_text_only("- [x").as_ref(), "- [x");
        assert_eq!(h_text_only("- [ ").as_ref(), "- [ ");
    }

    #[test]
    fn task_list_marker_then_incomplete_link_later() {
        assert_eq!(
            h("- [x] prefix [link").as_ref(),
            "- [x] prefix [link](streamdown:incomplete-link)"
        );
    }
}
