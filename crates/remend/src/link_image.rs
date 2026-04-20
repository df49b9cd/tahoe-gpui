use std::borrow::Cow;

use super::options::LinkMode;
use super::ranges::CodeBlockRanges;
use super::utils::{
    find_matching_closing_bracket, find_matching_opening_bracket, is_list_marker_line,
};

/// Handles incomplete URLs in links/images: `[text](partial-url`.
fn handle_incomplete_url<'a>(
    text: &'a str,
    bracket_paren_index: usize,
    link_mode: LinkMode,
    ranges: &CodeBlockRanges,
) -> Option<Cow<'a, str>> {
    // `bracket_paren_index` points to `]` in `](`.
    // Only consider `)` on the same line — a `)` later in the document
    // (e.g., from an emoticon or another link) should not prevent completion.
    //
    // Skip any leading CR/LF so a URL placed on a subsequent line (e.g. a
    // stream-chunk boundary that splits `](` from the URL) is still recognized.
    // Then byte-scan to the next CR/LF — treating both as terminators keeps
    // behavior consistent for lone `\r` input too.
    let after_paren = text[bracket_paren_index + 2..].trim_start_matches(['\r', '\n']);
    let line_end = after_paren
        .bytes()
        .position(|b| matches!(b, b'\n' | b'\r'))
        .unwrap_or(after_paren.len());
    if after_paren[..line_end].contains(')') {
        return None; // URL is complete.
    }

    // Find matching `[` for the `]`.
    let open = find_matching_opening_bracket(text, bracket_paren_index, ranges)?;

    if ranges.is_inside_code(open) {
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
fn handle_incomplete_text<'a>(
    text: &'a str,
    open_index: usize,
    link_mode: LinkMode,
    ranges: &CodeBlockRanges,
) -> Option<Cow<'a, str>> {
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

        return Some(make_incomplete_link(text, open_index, link_mode, ranges));
    }

    // Check if the closing bracket actually matches (accounting for nesting).
    let closing = find_matching_closing_bracket(text, open_index, ranges);
    if closing.is_none() {
        let before = &text[..start];
        if is_image {
            return Some(Cow::Owned(before.trim_end().to_owned()));
        }
        return Some(make_incomplete_link(text, open_index, link_mode, ranges));
    }

    None
}

/// Finds the first incomplete `[` by scanning forward, skipping complete links.
/// `max_pos` is the position of a known incomplete `[` (fallback).
fn find_first_incomplete_bracket(text: &str, max_pos: usize, ranges: &CodeBlockRanges) -> usize {
    let bytes = text.as_bytes();
    let mut j = 0;
    while j < max_pos {
        if bytes[j] == b'[' && !ranges.is_inside_code(j) {
            // Skip images.
            if text[..j].ends_with('!') {
                j += 1;
                continue;
            }
            // Check if this `[` has a matching `]`.
            if let Some(close_idx) = find_matching_closing_bracket(text, j, ranges) {
                // Check if it's a full link `[text](url)` — but only if `)` is
                // on the same line (links cannot span lines per CommonMark).
                if close_idx + 1 < bytes.len() && bytes[close_idx + 1] == b'(' {
                    let after_paren = &text[close_idx + 2..];
                    let line_end = after_paren
                        .bytes()
                        .position(|b| matches!(b, b'\n' | b'\r'))
                        .unwrap_or(after_paren.len());
                    if let Some(url_end) = after_paren[..line_end].find(')') {
                        // Skip past this complete link.
                        j = close_idx + 2 + url_end + 1;
                        continue;
                    }
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
fn make_incomplete_link<'a>(
    text: &str,
    open_index: usize,
    link_mode: LinkMode,
    ranges: &CodeBlockRanges,
) -> Cow<'a, str> {
    match link_mode {
        LinkMode::TextOnly => {
            // Find the first incomplete `[` (scanning forward) and strip just that bracket.
            let first_incomplete = find_first_incomplete_bracket(text, open_index, ranges);
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

/// Test-only convenience wrapper that builds `CodeBlockRanges` on the fly.
#[cfg(test)]
fn handle(
    text: &str,
    link_mode: LinkMode,
    links_enabled: bool,
    images_enabled: bool,
) -> Cow<'_, str> {
    handle_with_ranges(
        text,
        link_mode,
        links_enabled,
        images_enabled,
        &CodeBlockRanges::new(text),
    )
}

/// Handles incomplete links and images, using pre-computed code block ranges.
pub(crate) fn handle_with_ranges<'a>(
    text: &'a str,
    link_mode: LinkMode,
    links_enabled: bool,
    images_enabled: bool,
    ranges: &CodeBlockRanges,
) -> Cow<'a, str> {
    if !links_enabled && !images_enabled {
        return Cow::Borrowed(text);
    }

    let bytes = text.as_bytes();

    // Phase 1: Look for `](` pattern — incomplete URL.
    if let Some(pos) = text.rfind("](")
        && !ranges.is_inside_code(pos)
    {
        // Check if this is an image (preceded by `![`).
        let open = find_matching_opening_bracket(text, pos, ranges);
        let is_image = open.is_some_and(|o| text[..o].ends_with('!'));
        if ((is_image && images_enabled) || (!is_image && links_enabled))
            && let Some(result) = handle_incomplete_url(text, pos, link_mode, ranges)
        {
            return result;
        }
    }

    // Phase 2: Scan backward for unmatched `[`.
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if bytes[i] == b'[' && !ranges.is_inside_code(i) {
            let is_image = text[..i].ends_with('!');
            if (is_image && !images_enabled) || (!is_image && !links_enabled) {
                continue;
            }
            if !is_image && is_task_list_marker_start(text, i) {
                continue;
            }
            if let Some(result) = handle_incomplete_text(text, i, link_mode, ranges) {
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

    #[test]
    fn leaves_complete_link_with_crlf_before_url() {
        assert!(matches!(
            h("[text](\r\nhttp://example.com)"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn leaves_complete_link_with_lf_before_url() {
        assert!(matches!(
            h("[text](\nhttp://example.com)"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn leaves_complete_link_with_crlf_trailing() {
        assert!(matches!(
            h("[text](http://example.com)\r\nNext"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn completes_incomplete_url_with_crlf_trailing() {
        assert_eq!(
            h("[text](http://exam\r\n").as_ref(),
            "[text](streamdown:incomplete-link)"
        );
    }

    #[test]
    fn completes_incomplete_url_with_lone_cr() {
        assert_eq!(
            h("[text](http://exam\r").as_ref(),
            "[text](streamdown:incomplete-link)"
        );
    }

    #[test]
    fn removes_incomplete_image_with_crlf() {
        assert_eq!(h("text ![alt](\r\nhttp://exam").as_ref(), "text");
    }

    // Issue #78: brackets inside inline code must not participate in link matching.
    #[test]
    fn bracket_inside_inline_code_not_matched() {
        // The [ inside backticks should not form a link.
        assert!(matches!(h("`[incomplete`"), Cow::Borrowed(_)));
    }

    #[test]
    fn closing_bracket_inside_inline_code_ignored() {
        // The ] inside backticks should not close the link; the [ is still incomplete.
        assert_eq!(
            h("[text `code]more`").as_ref(),
            "[text `code]more`](streamdown:incomplete-link)"
        );
    }

    #[test]
    fn complete_link_with_inline_code_in_text() {
        // A complete link whose text contains inline code should be left alone.
        assert!(matches!(h("[text `code]more`](url)"), Cow::Borrowed(_)));
    }

    #[test]
    fn inline_code_brackets_dont_interfere_with_real_link() {
        // Inline code brackets should not prevent detecting a real incomplete link.
        assert_eq!(
            h("`[code]` [link](url").as_ref(),
            "`[code]` [link](streamdown:incomplete-link)"
        );
    }

    #[test]
    fn inline_code_closing_bracket_doesnt_false_match() {
        // `](` outside code but `[` inside code — not a link.
        assert!(matches!(h("`[text` ](url"), Cow::Borrowed(_)));
    }

    #[test]
    fn close_paren_on_next_line_does_not_complete_link() {
        // `)` on line 2 must not close the URL — the link is incomplete.
        assert_eq!(
            h("[a](url\nother)").as_ref(),
            "[a](streamdown:incomplete-link)"
        );
    }

    #[test]
    fn text_only_close_paren_on_next_line_does_not_complete_link() {
        assert_eq!(h_text_only("[a](url\nother)").as_ref(), "a");
    }
}
