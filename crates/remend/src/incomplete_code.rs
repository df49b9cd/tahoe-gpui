//! Incomplete code fence and table detection for streaming markdown.
//!
//! Ported from Streamdown's `incomplete-code-utils.ts`.

/// Returns `true` if the markdown text has an unclosed code fence.
///
/// Walks line-by-line per CommonMark spec: a closing fence must use the same
/// character and be at least as long as the opening fence.
pub fn has_incomplete_code_fence(markdown: &str) -> bool {
    let mut open_fence_char: Option<u8> = None;
    let mut open_fence_length: usize = 0;

    for line in markdown.split('\n') {
        if let Some(fence) = parse_code_fence(line) {
            if let Some(open_char) = open_fence_char {
                // We're inside a fence — check if this closes it.
                if fence.char == open_char && fence.len >= open_fence_length {
                    open_fence_char = None;
                    open_fence_length = 0;
                }
            } else {
                // Not inside a fence — this opens one.
                open_fence_char = Some(fence.char);
                open_fence_length = fence.len;
            }
        }
    }

    open_fence_char.is_some()
}

struct FenceInfo {
    char: u8,
    len: usize,
}

/// Parses a code fence from a line: up to 3 leading spaces, then 3+ backticks or tildes.
fn parse_code_fence(line: &str) -> Option<FenceInfo> {
    let bytes = line.as_bytes();
    let mut i = 0;

    // Skip up to 3 leading spaces/tabs.
    let mut leading_spaces = 0;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        if bytes[i] == b' ' {
            leading_spaces += 1;
        } else {
            leading_spaces += 4; // tab counts as 4
        }
        if leading_spaces > 3 {
            return None;
        }
        i += 1;
    }

    if i >= bytes.len() {
        return None;
    }

    let fence_char = bytes[i];
    if fence_char != b'`' && fence_char != b'~' {
        return None;
    }

    let mut fence_len = 0;
    while i < bytes.len() && bytes[i] == fence_char {
        fence_len += 1;
        i += 1;
    }

    if fence_len >= 3 {
        Some(FenceInfo {
            char: fence_char,
            len: fence_len,
        })
    } else {
        None
    }
}

/// Returns `true` if the markdown text contains a table delimiter row.
pub fn has_table(markdown: &str) -> bool {
    // Pattern: optional |, then one or more columns of :?-+:? separated by |
    for line in markdown.split('\n') {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.contains('|') && is_table_delimiter(trimmed) {
            return true;
        }
    }
    false
}

/// Checks if a line is a table delimiter row like `| --- | :---: | ---: |`
fn is_table_delimiter(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut i = 0;
    let len = bytes.len();

    // Skip optional leading |
    if i < len && bytes[i] == b'|' {
        i += 1;
    }

    let mut found_column = false;

    loop {
        // Skip whitespace.
        while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= len {
            break;
        }
        // If we hit trailing |, that's fine.
        if bytes[i] == b'|' && !found_column {
            return false; // empty column before any content
        }
        if bytes[i] == b'|' {
            i += 1;
            if i >= len {
                break;
            }
            continue;
        }

        // Expect a column: optional :, then one or more -, then optional :
        if bytes[i] == b':' {
            i += 1;
        }
        let dash_start = i;
        while i < len && bytes[i] == b'-' {
            i += 1;
        }
        if i == dash_start {
            return false; // no dashes
        }
        if i < len && bytes[i] == b':' {
            i += 1;
        }
        // Skip trailing whitespace for this column.
        while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        found_column = true;

        // Must be followed by | or end of line.
        if i < len && bytes[i] != b'|' {
            return false;
        }
    }

    found_column
}

#[cfg(test)]
mod tests {
    use super::{has_incomplete_code_fence, has_table};

    #[test]
    fn detects_incomplete_backtick_fence() {
        assert!(has_incomplete_code_fence("```rust\nfn main() {"));
    }

    #[test]
    fn complete_fence_is_not_incomplete() {
        assert!(!has_incomplete_code_fence("```rust\nfn main() {}\n```"));
    }

    #[test]
    fn detects_incomplete_tilde_fence() {
        assert!(has_incomplete_code_fence("~~~\ncode here"));
    }

    #[test]
    fn closing_fence_must_match_char() {
        // Opened with ```, closing with ~~~ doesn't close it.
        assert!(has_incomplete_code_fence("```\ncode\n~~~"));
    }

    #[test]
    fn closing_fence_must_be_long_enough() {
        // Opened with ````, closing with ``` doesn't close it.
        assert!(has_incomplete_code_fence("````\ncode\n```"));
    }

    #[test]
    fn closing_fence_equal_length() {
        assert!(!has_incomplete_code_fence("```\ncode\n```"));
    }

    #[test]
    fn closing_fence_longer_ok() {
        assert!(!has_incomplete_code_fence("```\ncode\n`````"));
    }

    #[test]
    fn no_fence_at_all() {
        assert!(!has_incomplete_code_fence("just some text"));
    }

    #[test]
    fn indented_fence() {
        assert!(has_incomplete_code_fence("   ```\ncode"));
    }

    #[test]
    fn too_much_indent_is_not_fence() {
        assert!(!has_incomplete_code_fence("    ```\ncode"));
    }

    #[test]
    fn detects_simple_table() {
        assert!(has_table("| a | b |\n| --- | --- |\n| 1 | 2 |"));
    }

    #[test]
    fn detects_aligned_table() {
        assert!(has_table("| a | b |\n| :---: | ---: |"));
    }

    #[test]
    fn no_table() {
        assert!(!has_table("just some text with | pipes"));
    }

    #[test]
    fn minimal_table_delimiter() {
        assert!(has_table("|-|"));
    }
}
