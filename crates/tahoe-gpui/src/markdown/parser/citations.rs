//! Citation splitting for inline content.

use super::InlineContent;

/// Split text on citation markers `[N]` into alternating `Text` and `Citation` variants.
///
/// The splitter treats **any** bracketed ASCII-digit sequence as an inline
/// citation, which is the convention for AI-generated content. The tradeoff
/// is that legitimate prose like `"item [5] of 10"` is claimed as a citation
/// marker too, and downstream renderers will treat the number as a footnote
/// reference.
///
/// Callers that render non-AI Markdown should disable splitting via
/// [`IncrementalMarkdownParser::with_citations(false)`][with_citations] (or
/// [`StreamingMarkdown::with_citations(false)`][sm_with_citations]) so the
/// bracketed number is preserved verbatim.
///
/// [with_citations]: super::IncrementalMarkdownParser::with_citations
/// [sm_with_citations]: crate::markdown::StreamingMarkdown::with_citations
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
