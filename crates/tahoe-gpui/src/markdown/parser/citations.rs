//! Citation splitting for inline content.

use super::InlineContent;

/// Split text on citation markers `[N]` into alternating `Text` and `Citation` variants.
pub fn split_citations(text: &str, out: &mut Vec<InlineContent>) {
    let mut last = 0;
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            // Try to parse a citation number
            let start = i;
            i += 1;
            let num_start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i > num_start
                && i < bytes.len()
                && bytes[i] == b']'
                && let Ok(n) = text[num_start..i].parse::<usize>()
            {
                // Emit any text before the citation
                if start > last {
                    out.push(InlineContent::Text(text[last..start].to_string()));
                }
                out.push(InlineContent::Citation(n));
                i += 1; // skip ']'
                last = i;
                continue;
            }
            // Not a valid citation, continue scanning from after '['
            i = start + 1;
        } else {
            i += 1;
        }
    }
    // Emit remaining text
    if last < text.len() {
        out.push(InlineContent::Text(text[last..].to_string()));
    }
}
