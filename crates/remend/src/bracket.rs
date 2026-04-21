//! Balanced `[` / `]` bracket matching that respects inline-code boundaries.
//!
//! Lives in its own module so `utils.rs` no longer depends on
//! `ranges::CodeBlockRanges`, breaking the utils ↔ ranges import cycle.

use super::ranges::CodeBlockRanges;

/// Finds the matching opening bracket `[` for a closing bracket at
/// `close_index`, handling nested brackets. Brackets inside inline code spans
/// are ignored per `CodeBlockRanges::is_inside_code`.
pub fn find_matching_opening_bracket(
    text: &str,
    close_index: usize,
    ranges: &CodeBlockRanges,
) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: i32 = 1;
    let mut i = close_index;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b']' | b'[' if ranges.is_inside_code(i) => continue,
            b']' => depth += 1,
            b'[' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Finds the matching closing bracket `]` for an opening bracket at
/// `open_index`, handling nested brackets. Brackets inside inline code spans
/// are ignored per `CodeBlockRanges::is_inside_code`.
pub fn find_matching_closing_bracket(
    text: &str,
    open_index: usize,
    ranges: &CodeBlockRanges,
) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth: i32 = 1;
    let mut i = open_index + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'[' | b']' if ranges.is_inside_code(i) => {}
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::super::ranges::CodeBlockRanges;
    use super::{find_matching_closing_bracket, find_matching_opening_bracket};

    fn ranges(text: &str) -> CodeBlockRanges {
        CodeBlockRanges::new(text)
    }

    #[test]
    fn test_find_matching_brackets() {
        let r = ranges("[hello]");
        assert_eq!(find_matching_opening_bracket("[hello]", 6, &r), Some(0));
        assert_eq!(find_matching_closing_bracket("[hello]", 0, &r), Some(6));
        let r = ranges("a[b[c]]");
        assert_eq!(find_matching_opening_bracket("a[b[c]]", 6, &r), Some(1));
        let r = ranges("hello]");
        assert_eq!(find_matching_opening_bracket("hello]", 5, &r), None);
    }

    #[test]
    fn test_brackets_skip_inline_code() {
        // ] inside inline code should not count as a bracket.
        let text = "[`code]text`]";
        let r = ranges(text);
        assert_eq!(find_matching_opening_bracket(text, 12, &r), Some(0));
        assert_eq!(find_matching_closing_bracket(text, 0, &r), Some(12));

        let text = "[`a]]b`]";
        let r = ranges(text);
        assert_eq!(find_matching_closing_bracket(text, 0, &r), Some(7));

        // [ inside inline code should not count either.
        let text = "`[text` ](url";
        let r = ranges(text);
        assert_eq!(find_matching_opening_bracket(text, 8, &r), None);
    }
}
