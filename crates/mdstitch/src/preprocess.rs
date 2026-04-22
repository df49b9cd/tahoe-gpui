//! Markdown preprocessing for custom and literal HTML tags.
//!
//! Prepares HTML tag bodies so CommonMark treats them as intended: keeps
//! blank-line-containing custom tags in one block, renders literal-tag
//! contents as plain text, and strips indentation that would turn an HTML
//! tag into an indented code block.

use std::borrow::Cow;

/// Preprocesses custom HTML tags to prevent blank lines within them from
/// causing CommonMark to split the block.
///
/// For each registered tag name, replaces `\n\n` inside the tag with
/// `\n<!---->\n` (HTML comment that acts as a spacer without splitting).
///
/// # Limitations
///
/// Nested instances of the same tag name are not supported — only the
/// outermost `<tag>...</tag>` pair is processed per tag name.
pub fn preprocess_custom_tags<'a>(markdown: &'a str, tag_names: &[&str]) -> Cow<'a, str> {
    if tag_names.is_empty() || markdown.is_empty() {
        return Cow::Borrowed(markdown);
    }

    // Each pass processes one tag name, building a fresh `String` in a single
    // left-to-right scan of the input rather than re-allocating per match. The
    // previous implementation copied the entire string on every match, which
    // was O(n²) on inputs with many blank-line blocks.
    let mut current: Cow<'a, str> = Cow::Borrowed(markdown);
    let mut changed_any = false;
    for &tag_name in tag_names {
        if let Some(next) = rewrite_custom_tag_pass(&current, tag_name) {
            current = Cow::Owned(next);
            changed_any = true;
        }
    }
    if changed_any {
        current
    } else {
        Cow::Borrowed(markdown)
    }
}

/// Run one tag's `<!---->` substitution pass, returning `Some(String)` if the
/// source was rewritten or `None` if the source does not reference the tag.
fn rewrite_custom_tag_pass(source: &str, tag_name: &str) -> Option<String> {
    find_tag_open(source, tag_name)?;

    let close_pattern = format!("</{tag_name}");
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut changed = false;

    while let Some(open_rel) = find_tag_open(&source[cursor..], tag_name) {
        let open_tag_start = cursor + open_rel;
        let Some(open_close_rel) = source[open_tag_start..].find('>') else {
            break;
        };
        let open_tag_end = open_tag_start + open_close_rel + 1;

        let Some(close_rel) = find_case_insensitive(&source[open_tag_end..], &close_pattern) else {
            break;
        };
        let close_tag_start = open_tag_end + close_rel;
        let Some(close_close_rel) = source[close_tag_start..].find('>') else {
            break;
        };
        let close_tag_end = close_tag_start + close_close_rel + 1;

        let content = &source[open_tag_end..close_tag_start];
        if content.contains("\n\n") {
            changed = true;
            // Emit: …source[cursor..open_tag_end] | padded fixed content | close tag | (pad).
            out.push_str(&source[cursor..open_tag_end]);
            if !content.starts_with('\n') {
                out.push('\n');
            }
            // Single pass over `content`, replacing `\n\n` with `\n<!---->\n`.
            let mut tail = content;
            while let Some(idx) = tail.find("\n\n") {
                out.push_str(&tail[..idx]);
                out.push_str("\n<!---->\n");
                tail = &tail[idx + 2..];
            }
            out.push_str(tail);
            if !content.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&source[close_tag_start..close_tag_end]);
            if !source[close_tag_end..].starts_with('\n') {
                out.push_str("\n\n");
            }
            cursor = close_tag_end;
        } else {
            // Preserve untouched; keep scanning after this close tag.
            out.push_str(&source[cursor..close_tag_end]);
            cursor = close_tag_end;
        }
    }

    if !changed {
        return None;
    }
    out.push_str(&source[cursor..]);
    Some(out)
}

/// Escapes markdown metacharacters inside specified HTML tags so their content
/// renders as plain text.
pub fn preprocess_literal_tag_content<'a>(markdown: &'a str, tag_names: &[&str]) -> Cow<'a, str> {
    if tag_names.is_empty() || markdown.is_empty() {
        return Cow::Borrowed(markdown);
    }

    let mut current: Cow<'a, str> = Cow::Borrowed(markdown);
    let mut changed_any = false;
    for &tag_name in tag_names {
        if let Some(next) = rewrite_literal_tag_pass(&current, tag_name) {
            current = Cow::Owned(next);
            changed_any = true;
        }
    }
    if changed_any {
        current
    } else {
        Cow::Borrowed(markdown)
    }
}

/// Run one tag's markdown-escape pass, returning `Some(String)` if the source
/// was rewritten or `None` if no literal content needed escaping.
fn rewrite_literal_tag_pass(source: &str, tag_name: &str) -> Option<String> {
    find_tag_open(source, tag_name)?;

    let close_pattern = format!("</{tag_name}");
    let mut out = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut changed = false;

    while let Some(open_rel) = find_tag_open(&source[cursor..], tag_name) {
        let open_tag_start = cursor + open_rel;
        let Some(open_close_rel) = source[open_tag_start..].find('>') else {
            break;
        };
        let open_tag_end = open_tag_start + open_close_rel + 1;

        let Some(close_rel) = find_case_insensitive(&source[open_tag_end..], &close_pattern) else {
            break;
        };
        let close_tag_start = open_tag_end + close_rel;
        let Some(close_close_rel) = source[close_tag_start..].find('>') else {
            break;
        };
        let close_tag_end = close_tag_start + close_close_rel + 1;

        let content = &source[open_tag_end..close_tag_start];
        let escaped = escape_markdown(content);

        if escaped != content {
            changed = true;
            out.push_str(&source[cursor..open_tag_end]);
            out.push_str(&escaped);
            out.push_str(&source[close_tag_start..close_tag_end]);
        } else {
            out.push_str(&source[cursor..close_tag_end]);
        }
        cursor = close_tag_end;
    }

    if !changed {
        return None;
    }
    out.push_str(&source[cursor..]);
    Some(out)
}

/// Escapes markdown metacharacters: `\`, `` ` ``, `*`, `_`, `~`, `[`, `]`, `|`.
/// Also replaces `\n\n` with `&#10;&#10;` to preserve blank lines.
fn escape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / 4);
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        // Replace \n\n with &#10;&#10;
        if ch == '\n' && chars.peek() == Some(&'\n') {
            out.push_str("&#10;&#10;");
            let _ = chars.next();
            continue;
        }
        match ch {
            '\\' | '`' | '*' | '_' | '~' | '[' | ']' | '|' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }
    out
}

/// Strips excessive indentation (4+ spaces/tabs) from lines that start HTML blocks.
///
/// CommonMark treats 4+ spaces of indentation as code blocks. When LLMs generate
/// indented HTML, this causes the HTML to render as a code block instead of being
/// parsed as HTML. This function strips that indentation.
pub fn normalize_html_indentation(text: &str) -> Cow<'_, str> {
    if text.is_empty() {
        return Cow::Borrowed(text);
    }

    // Quick check: does the content start with optional whitespace then an HTML-like tag?
    if !starts_with_html_block(text) {
        return Cow::Borrowed(text);
    }

    let mut result = String::new();
    let mut changed = false;
    let mut first = true;

    for line in text.split('\n') {
        if !first {
            result.push('\n');
        }
        first = false;

        // Count leading whitespace in columns (spaces=1, tabs advance to next multiple of 4).
        let trimmed = line.trim_start_matches([' ', '\t']);
        let indent_cols = {
            let mut col = 0usize;
            for ch in line[..line.len() - trimmed.len()].chars() {
                match ch {
                    ' ' => col += 1,
                    '\t' => col = (col / 4 + 1) * 4,
                    _ => break,
                }
            }
            col
        };

        // If 4+ columns of indentation and the rest starts with an HTML tag, strip it.
        if indent_cols >= 4 && starts_with_html_tag_char(trimmed) {
            result.push_str(trimmed);
            changed = true;
        } else {
            result.push_str(line);
        }
    }

    if changed {
        Cow::Owned(result)
    } else {
        Cow::Borrowed(text)
    }
}

/// Returns true if text starts with optional whitespace followed by `<` and a tag char.
fn starts_with_html_block(text: &str) -> bool {
    let trimmed = text.trim_start_matches([' ', '\t']);
    starts_with_html_tag_char(trimmed)
}

/// Returns true if text starts with `<` followed by a word char, `!`, `/`, or `?`.
fn starts_with_html_tag_char(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'<' {
        return false;
    }
    let next = bytes[1];
    next.is_ascii_alphanumeric() || matches!(next, b'!' | b'/' | b'?' | b'-')
}

/// Case-insensitive search for an opening tag like `<tagname` followed by whitespace, `/`, or `>`.
/// Avoids allocating lowercased copies by comparing bytes directly.
fn find_tag_open(haystack: &str, tag_name: &str) -> Option<usize> {
    let needle = format!("<{}", tag_name);
    let needle_bytes = needle.as_bytes();
    let hay_bytes = haystack.as_bytes();
    let needle_len = needle_bytes.len();
    if hay_bytes.len() < needle_len {
        return None;
    }
    for start in 0..=hay_bytes.len() - needle_len {
        if hay_bytes[start..start + needle_len].eq_ignore_ascii_case(needle_bytes) {
            let after = start + needle_len;
            if after >= hay_bytes.len()
                || matches!(hay_bytes[after], b' ' | b'\t' | b'\n' | b'>' | b'/')
            {
                return Some(start);
            }
        }
    }
    None
}

/// Case-insensitive search for a string.
/// Avoids allocating lowercased copies by comparing bytes directly.
fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let needle_bytes = needle.as_bytes();
    let hay_bytes = haystack.as_bytes();
    let needle_len = needle_bytes.len();
    if needle_len == 0 {
        return Some(0);
    }
    if hay_bytes.len() < needle_len {
        return None;
    }
    for start in 0..=hay_bytes.len() - needle_len {
        if hay_bytes[start..start + needle_len].eq_ignore_ascii_case(needle_bytes) {
            // For closing tags, check that next char is whitespace or >
            let after = start + needle_len;
            if after >= hay_bytes.len() || matches!(hay_bytes[after], b' ' | b'\t' | b'\n' | b'>') {
                return Some(start);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        escape_markdown, normalize_html_indentation, preprocess_custom_tags,
        preprocess_literal_tag_content,
    };
    use std::borrow::Cow;

    #[test]
    fn custom_tags_replaces_blank_lines() {
        let input = "<custom>\nfoo\n\nbar\n</custom>";
        let result = preprocess_custom_tags(input, &["custom"]);
        assert!(result.contains("<!---->")); // blank line replaced
        assert!(!result.contains("\n\n</custom>")); // no blank line before close
    }

    #[test]
    fn custom_tags_no_blank_lines_unchanged() {
        let input = "<custom>\nfoo\nbar\n</custom>";
        let result = preprocess_custom_tags(input, &["custom"]);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn custom_tags_empty_list() {
        let input = "<custom>\nfoo\n\nbar\n</custom>";
        assert!(matches!(
            preprocess_custom_tags(input, &[]),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn literal_tags_escapes_markdown() {
        let input = "<literal>**bold** and `code`</literal>";
        let result = preprocess_literal_tag_content(input, &["literal"]);
        assert!(result.contains("\\*\\*bold\\*\\*"));
        assert!(result.contains("\\`code\\`"));
    }

    #[test]
    fn literal_tags_preserves_blank_lines() {
        let input = "<literal>foo\n\nbar</literal>";
        let result = preprocess_literal_tag_content(input, &["literal"]);
        assert!(result.contains("&#10;&#10;"));
    }

    #[test]
    fn literal_tags_no_special_chars_unchanged() {
        let input = "<literal>plain text</literal>";
        let result = preprocess_literal_tag_content(input, &["literal"]);
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn escape_markdown_all_chars() {
        assert_eq!(escape_markdown("\\`*_~[]|"), "\\\\\\`\\*\\_\\~\\[\\]\\|");
    }

    #[test]
    fn escape_markdown_non_ascii() {
        assert_eq!(
            escape_markdown("héllo 世界 **bold**"),
            "héllo 世界 \\*\\*bold\\*\\*"
        );
    }

    #[test]
    fn literal_tags_sibling_tags() {
        let input = "<literal>**a**</literal> text <literal>`b`</literal>";
        let result = preprocess_literal_tag_content(input, &["literal"]);
        assert!(result.contains("\\*\\*a\\*\\*"));
        assert!(result.contains("\\`b\\`"));
    }

    #[test]
    fn literal_tags_multiple_different_tags() {
        let input = "<literal>**x**</literal>\n<code-block>_y_</code-block>";
        let result = preprocess_literal_tag_content(input, &["literal", "code-block"]);
        assert!(result.contains("\\*\\*x\\*\\*"));
        assert!(result.contains("\\_y\\_"));
    }

    #[test]
    fn literal_tags_content_with_angle_brackets() {
        let input = "<literal>a <b> c</literal>";
        let result = preprocess_literal_tag_content(input, &["literal"]);
        assert!(result.contains("a <b> c"));
    }

    // --- normalize_html_indentation tests ---

    #[test]
    fn normalize_html_strips_4_space_indent() {
        let input = "    <div>\n        <p>text</p>\n    </div>";
        let result = normalize_html_indentation(input);
        assert_eq!(result.as_ref(), "<div>\n<p>text</p>\n</div>");
    }

    #[test]
    fn normalize_html_preserves_non_html_indent() {
        let input = "    regular text\n    more text";
        let result = normalize_html_indentation(input);
        // Doesn't start with HTML tag, so unchanged.
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn normalize_html_preserves_small_indent() {
        let input = "   <div>ok</div>";
        let result = normalize_html_indentation(input);
        // Only 3 spaces — below threshold.
        assert!(matches!(result, Cow::Borrowed(_)));
    }

    #[test]
    fn normalize_html_empty_string() {
        assert!(matches!(normalize_html_indentation(""), Cow::Borrowed(_)));
    }

    #[test]
    fn normalize_html_mixed_lines() {
        let input = "    <div>\nsome text\n      </div>";
        let result = normalize_html_indentation(input);
        assert_eq!(result.as_ref(), "<div>\nsome text\n</div>");
    }

    #[test]
    fn normalize_html_tab_indent() {
        let input = "\t\t\t\t<section>\n\t\t\t\t\t<p>hi</p>\n\t\t\t\t</section>";
        let result = normalize_html_indentation(input);
        assert_eq!(result.as_ref(), "<section>\n<p>hi</p>\n</section>");
    }

    #[test]
    fn normalize_html_closing_tag() {
        let input = "    </div>";
        let result = normalize_html_indentation(input);
        assert_eq!(result.as_ref(), "</div>");
    }

    #[test]
    fn normalize_html_comment() {
        let input = "    <!-- comment -->";
        let result = normalize_html_indentation(input);
        assert_eq!(result.as_ref(), "<!-- comment -->");
    }
}
