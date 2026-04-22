//! Incomplete code fence and table detection for streaming markdown.
//!
//! Detects unclosed fences (CommonMark §4.5) and GFM table delimiter rows so
//! the downstream parser can keep or release code-block styling mid-stream.

use crate::fence::FenceScanner;

/// Returns `true` if the markdown text has an unclosed code fence.
///
/// Walks line-by-line per CommonMark §4.5: a closing fence must use the same
/// character and be at least as long as the opening fence.
pub fn has_incomplete_code_fence(markdown: &str) -> bool {
    let mut scanner = FenceScanner::new();
    let bytes = markdown.as_bytes();
    let mut line_start = 0usize;

    for i in 0..=bytes.len() {
        if i == bytes.len() || bytes[i] == b'\n' {
            scanner.consume_fence_at_line_start(bytes, line_start);
            line_start = i + 1;
        }
    }

    scanner.in_code_block()
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
    fn no_fence_for_mid_line_backticks() {
        // Issue #50: mid-line ``` is not a fence, so no block opens.
        assert!(!has_incomplete_code_fence("hello ```\ncode"));
    }

    #[test]
    fn no_fence_for_mid_line_tildes() {
        assert!(!has_incomplete_code_fence("hello ~~~\ncode"));
    }

    #[test]
    fn mid_line_fence_on_only_line_is_not_a_fence() {
        // Single-line with mid-line run also must not toggle fence state.
        assert!(!has_incomplete_code_fence("a ```inline fence``` b"));
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
