use std::borrow::Cow;

use super::utils::is_inside_code_block;

/// Escapes `>` characters that appear as comparison operators inside list items.
/// E.g. `- > 25: expensive` → `- \> 25: expensive`.
pub fn handle(text: &str) -> Cow<'_, str> {
    if !text.contains('>') {
        return Cow::Borrowed(text);
    }

    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut result: Option<String> = None;
    let mut last_copy = 0;

    // Process line by line.
    let mut line_start = 0;
    while line_start < len {
        let line_end = bytes[line_start..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|p| line_start + p)
            .unwrap_or(len);

        let line = &bytes[line_start..line_end];

        // Check if this line is a list item with a comparison operator.
        if let Some(gt_offset) = find_list_comparison(line) {
            let gt_pos = line_start + gt_offset;
            if !is_inside_code_block(text, gt_pos) {
                let buf = result.get_or_insert_with(|| String::with_capacity(len + 8));
                buf.push_str(&text[last_copy..gt_pos]);
                buf.push_str("\\>");
                last_copy = gt_pos + 1;
            }
        }

        line_start = line_end + 1;
    }

    match result {
        Some(mut buf) => {
            buf.push_str(&text[last_copy..]);
            Cow::Owned(buf)
        }
        None => Cow::Borrowed(text),
    }
}

/// Checks if a line matches the list comparison pattern:
/// `^\s*[-*+]\s+>` or `^\s*\d+[.)]\s+>` followed by optional `=` and a digit.
/// Returns the offset of `>` within the line if found.
fn find_list_comparison(line: &[u8]) -> Option<usize> {
    let len = line.len();
    let mut i = 0;

    // Skip leading whitespace.
    while i < len && matches!(line[i], b' ' | b'\t') {
        i += 1;
    }

    // Check for list marker.
    if i >= len {
        return None;
    }

    if matches!(line[i], b'-' | b'*' | b'+') {
        i += 1;
    } else if line[i].is_ascii_digit() {
        // Ordered list: digits followed by `.` or `)`.
        while i < len && line[i].is_ascii_digit() {
            i += 1;
        }
        if i >= len || !matches!(line[i], b'.' | b')') {
            return None;
        }
        i += 1;
    } else {
        return None;
    }

    // Must have at least one space.
    if i >= len || line[i] != b' ' {
        return None;
    }
    while i < len && line[i] == b' ' {
        i += 1;
    }

    // Expect `>`.
    if i >= len || line[i] != b'>' {
        return None;
    }
    let gt_offset = i;
    i += 1;

    // Optional `=`.
    if i < len && line[i] == b'=' {
        i += 1;
    }

    // Optional space and `$`.
    if i < len && line[i] == b' ' {
        i += 1;
    }
    if i < len && line[i] == b'$' {
        i += 1;
    }

    // Must be followed by a digit.
    if i >= len || !line[i].is_ascii_digit() {
        return None;
    }

    Some(gt_offset)
}

#[cfg(test)]
mod tests {
    use super::handle;
    use std::borrow::Cow;

    #[test]
    fn escapes_comparison_in_list() {
        assert_eq!(handle("- > 25").as_ref(), "- \\> 25");
    }

    #[test]
    fn escapes_gte_in_list() {
        assert_eq!(handle("- >= 25").as_ref(), "- \\>= 25");
    }

    #[test]
    fn leaves_blockquote() {
        // Not followed by digit, so not a comparison.
        assert!(matches!(handle("- > text"), Cow::Borrowed(_)));
    }

    #[test]
    fn leaves_non_list() {
        assert!(matches!(handle("> 25"), Cow::Borrowed(_)));
    }

    #[test]
    fn ordered_list() {
        assert_eq!(handle("1. > 25").as_ref(), "1. \\> 25");
    }

    #[test]
    fn no_angle_bracket() {
        assert!(matches!(handle("- hello"), Cow::Borrowed(_)));
    }
}
