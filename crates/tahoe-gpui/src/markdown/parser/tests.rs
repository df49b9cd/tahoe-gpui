//! Tests for the incremental markdown parser.

use super::{IncrementalMarkdownParser, InlineContent, MarkdownBlock};
use core::prelude::v1::test;
use std::rc::Rc;

fn parse(input: &str) -> Rc<Vec<MarkdownBlock>> {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta(input);
    p.parse()
}

fn first_paragraph_text(blocks: &[MarkdownBlock]) -> String {
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => inlines
            .iter()
            .map(|i| match i {
                InlineContent::Text(t) => t.clone(),
                _ => String::new(),
            })
            .collect(),
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_paragraph() {
    let blocks = parse("Hello world");
    assert_eq!(blocks.len(), 1);
    assert_eq!(first_paragraph_text(&blocks), "Hello world");
}

#[test]
fn parse_heading_levels() {
    for level in 1..=6u8 {
        let input = format!("{} Heading", "#".repeat(level as usize));
        let blocks = parse(&input);
        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            MarkdownBlock::Heading { level: l, content } => {
                assert_eq!(*l, level);
                assert!(!content.is_empty());
            }
            _ => panic!("expected heading for h{}", level),
        }
    }
}

#[test]
fn parse_code_block_fenced() {
    let blocks = parse("```rust\nfn main() {}\n```");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::CodeBlock { language, code } => {
            assert_eq!(language.as_deref(), Some("rust"));
            assert_eq!(code.trim(), "fn main() {}");
        }
        _ => panic!("expected code block"),
    }
}

#[test]
fn parse_code_block_no_language() {
    let blocks = parse("```\nhello\n```");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::CodeBlock { language, code } => {
            assert!(language.is_none());
            assert_eq!(code.trim(), "hello");
        }
        _ => panic!("expected code block"),
    }
}

#[test]
fn parse_unordered_list() {
    let blocks = parse("- a\n- b\n- c");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::List { ordered, items, .. } => {
            assert!(!ordered);
            assert_eq!(items.len(), 3);
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn parse_ordered_list() {
    let blocks = parse("1. a\n2. b");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::List {
            ordered,
            start,
            items,
        } => {
            assert!(ordered);
            assert_eq!(*start, Some(1));
            assert_eq!(items.len(), 2);
        }
        _ => panic!("expected ordered list"),
    }
}

#[test]
fn parse_blockquote() {
    let blocks = parse("> quoted text");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::BlockQuote(inner) => {
            assert!(!inner.is_empty());
        }
        _ => panic!("expected blockquote"),
    }
}

#[test]
fn parse_thematic_break() {
    let blocks = parse("---");
    assert_eq!(blocks.len(), 1);
    assert!(matches!(&blocks[0], MarkdownBlock::ThematicBreak));
}

#[test]
fn parse_table() {
    let blocks = parse("| a | b |\n|---|---|\n| 1 | 2 |");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Table {
            headers,
            rows,
            alignments,
        } => {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].len(), 2);
            // Plain `|---|` rule carries no explicit alignment.
            assert!(
                alignments.iter().all(|a| *a == super::TableAlignment::None),
                "expected no explicit alignments, got {alignments:?}"
            );
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn parse_table_alignment_columns() {
    use super::TableAlignment;
    let blocks = parse("| a | b | c |\n|:--|:-:|--:|\n| 1 | 2 | 3 |");
    match &blocks[0] {
        MarkdownBlock::Table { alignments, .. } => {
            assert_eq!(
                alignments.as_slice(),
                &[
                    TableAlignment::Left,
                    TableAlignment::Center,
                    TableAlignment::Right,
                ]
            );
        }
        _ => panic!("expected table"),
    }
}

#[test]
fn parse_bold() {
    let blocks = parse("**bold**");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => {
            assert!(matches!(&inlines[0], InlineContent::Bold(_)));
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_italic() {
    let blocks = parse("*italic*");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => {
            assert!(matches!(&inlines[0], InlineContent::Italic(_)));
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_inline_code() {
    let blocks = parse("`code`");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => {
            assert!(matches!(&inlines[0], InlineContent::Code(_)));
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_strikethrough() {
    let blocks = parse("~~struck~~");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => {
            assert!(matches!(&inlines[0], InlineContent::Strikethrough(_)));
        }
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_link() {
    let blocks = parse("[text](https://example.com)");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => match &inlines[0] {
            InlineContent::Link { url, content } => {
                assert_eq!(url, "https://example.com");
                assert!(!content.is_empty());
            }
            _ => panic!("expected link"),
        },
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_nested_inline() {
    let blocks = parse("**bold *and italic***");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => match &inlines[0] {
            InlineContent::Bold(inner) => {
                assert!(inner.iter().any(|i| matches!(i, InlineContent::Italic(_))));
            }
            _ => panic!("expected bold"),
        },
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_mixed_blocks() {
    let input = "# Title\n\nParagraph text.\n\n```\ncode\n```";
    let blocks = parse(input);
    assert_eq!(blocks.len(), 3);
    assert!(matches!(
        &blocks[0],
        MarkdownBlock::Heading { level: 1, .. }
    ));
    assert!(matches!(&blocks[1], MarkdownBlock::Paragraph(_)));
    assert!(matches!(&blocks[2], MarkdownBlock::CodeBlock { .. }));
}

#[test]
fn parse_streaming_delta() {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("Hel");
    p.push_delta("lo ");
    p.push_delta("world");
    let blocks = p.parse();
    assert_eq!(blocks.len(), 1);
    assert_eq!(first_paragraph_text(&blocks), "Hello world");
}

#[test]
fn parse_empty() {
    let blocks = parse("");
    assert!(blocks.is_empty());
}

#[test]
fn parse_with_remend() {
    let mut p = IncrementalMarkdownParser::with_remend(remend::RemendOptions::default());
    p.push_delta("```rust\nfn main() {");
    // With remend, incomplete code fence should still parse
    let blocks = p.parse();
    assert!(!blocks.is_empty());
}

#[test]
fn has_incomplete_code_fence_detection() {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("```\nsome code");
    assert!(p.has_incomplete_code_fence());

    p.push_delta("\n```\n");
    assert!(!p.has_incomplete_code_fence());
}

#[test]
fn has_table_detection() {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("| a | b |\n|---|---|");
    assert!(p.has_table());
}

#[test]
fn parse_whitespace_only() {
    let blocks = parse("   ");
    // Whitespace-only input produces no blocks
    assert!(blocks.is_empty());
}

#[test]
fn parse_newlines_only() {
    let blocks = parse("\n\n\n");
    assert!(blocks.is_empty());
}

#[test]
fn parse_tab_only() {
    let blocks = parse("\t");
    // A tab alone is whitespace, no blocks
    assert!(blocks.is_empty());
}

#[test]
fn parse_mixed_whitespace() {
    let blocks = parse("  \t\n  \n\t  ");
    assert!(blocks.is_empty());
}

#[test]
fn parse_single_word() {
    let blocks = parse("hello");
    assert_eq!(blocks.len(), 1);
    assert_eq!(first_paragraph_text(&blocks), "hello");
}

#[test]
fn parse_trailing_newlines() {
    let blocks = parse("hello\n\n\n");
    assert_eq!(blocks.len(), 1);
    assert_eq!(first_paragraph_text(&blocks), "hello");
}

#[test]
fn parse_leading_newlines() {
    let blocks = parse("\n\nhello");
    assert_eq!(blocks.len(), 1);
    assert_eq!(first_paragraph_text(&blocks), "hello");
}

#[test]
fn incremental_parser_source_accumulation() {
    let mut p = IncrementalMarkdownParser::new();
    assert_eq!(p.source(), "");
    p.push_delta("abc");
    assert_eq!(p.source(), "abc");
    p.push_delta("def");
    assert_eq!(p.source(), "abcdef");
}

#[test]
fn incremental_parser_finish_clears_streaming() {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("hello");
    assert!(p.is_streaming);
    p.finish();
    assert!(!p.is_streaming);
}

#[test]
fn incremental_parser_finish_preserves_source() {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("hello ");
    p.push_delta("world");
    p.finish();
    assert_eq!(p.source(), "hello world");
}

#[test]
fn incremental_parser_caching() {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("hello");
    let blocks1 = p.parse();
    // Second call should return cached (no dirty flag)
    let blocks2 = p.parse();
    assert_eq!(blocks1.len(), blocks2.len());
}

#[test]
fn incremental_parser_dirty_after_delta() {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("hello");
    let _ = p.parse(); // clears dirty
    p.push_delta(" world");
    // blocks_dirty should be set again
    let blocks = p.parse();
    assert_eq!(blocks.len(), 1);
    assert_eq!(first_paragraph_text(&blocks), "hello world");
}

#[test]
fn parse_nested_blockquote() {
    let blocks = parse("> > nested");
    assert_eq!(blocks.len(), 1);
    if let MarkdownBlock::BlockQuote(inner) = &blocks[0] {
        assert!(!inner.is_empty());
        // Inner should also be a blockquote
        assert!(matches!(&inner[0], MarkdownBlock::BlockQuote(_)));
    } else {
        panic!("expected blockquote");
    }
}

#[test]
fn parse_code_block_preserves_content() {
    let code = "fn main() {\n    println!(\"Hello\");\n}";
    let input = format!("```rust\n{}\n```", code);
    let blocks = parse(&input);
    if let MarkdownBlock::CodeBlock {
        code: parsed_code, ..
    } = &blocks[0]
    {
        assert!(parsed_code.contains("println!"));
        assert!(parsed_code.contains("Hello"));
    }
}

#[test]
fn parse_multiple_paragraphs() {
    let blocks = parse("First paragraph\n\nSecond paragraph\n\nThird paragraph");
    assert_eq!(blocks.len(), 3);
    for block in blocks.iter() {
        assert!(matches!(block, MarkdownBlock::Paragraph(_)));
    }
}

#[test]
fn parse_table_multiple_rows() {
    let input = "| a | b |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n| 5 | 6 |";
    let blocks = parse(input);
    if let MarkdownBlock::Table { headers, rows, .. } = &blocks[0] {
        assert_eq!(headers.len(), 2);
        assert_eq!(rows.len(), 3);
    }
}

#[test]
fn parse_task_list_item_marker() {
    let blocks = parse("- [x] done\n- [ ] todo\n");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::List { items, .. } => {
            assert_eq!(items.len(), 2);
            let first_marker = items[0]
                .iter()
                .flat_map(|b| match b {
                    MarkdownBlock::Paragraph(inlines) => inlines.iter().collect::<Vec<_>>(),
                    _ => Vec::new(),
                })
                .find(|i| matches!(i, InlineContent::TaskMarker(_)));
            assert!(matches!(
                first_marker,
                Some(InlineContent::TaskMarker(true))
            ));
            let second_marker = items[1]
                .iter()
                .flat_map(|b| match b {
                    MarkdownBlock::Paragraph(inlines) => inlines.iter().collect::<Vec<_>>(),
                    _ => Vec::new(),
                })
                .find(|i| matches!(i, InlineContent::TaskMarker(_)));
            assert!(matches!(
                second_marker,
                Some(InlineContent::TaskMarker(false))
            ));
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn default_impl() {
    let p = IncrementalMarkdownParser::default();
    assert_eq!(p.source(), "");
    assert!(!p.is_streaming);
}

// --- Citation parsing tests ---

fn get_paragraph_inlines(blocks: &[MarkdownBlock]) -> &[InlineContent] {
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => inlines,
        _ => panic!("expected paragraph"),
    }
}

#[test]
fn parse_citation_basic() {
    let blocks = parse("See [1] for details");
    let inlines = get_paragraph_inlines(&blocks);
    assert_eq!(inlines.len(), 3);
    assert!(matches!(&inlines[0], InlineContent::Text(t) if t == "See "));
    assert!(matches!(&inlines[1], InlineContent::Citation(1)));
    assert!(matches!(&inlines[2], InlineContent::Text(t) if t == " for details"));
}

#[test]
fn parse_citation_multiple() {
    let blocks = parse("[1] and [2]");
    let inlines = get_paragraph_inlines(&blocks);
    assert_eq!(inlines.len(), 3);
    assert!(matches!(&inlines[0], InlineContent::Citation(1)));
    assert!(matches!(&inlines[1], InlineContent::Text(t) if t == " and "));
    assert!(matches!(&inlines[2], InlineContent::Citation(2)));
}

#[test]
fn parse_citation_adjacent() {
    let blocks = parse("[1][2]");
    let inlines = get_paragraph_inlines(&blocks);
    assert_eq!(inlines.len(), 2);
    assert!(matches!(&inlines[0], InlineContent::Citation(1)));
    assert!(matches!(&inlines[1], InlineContent::Citation(2)));
}

#[test]
fn parse_citation_not_number() {
    let blocks = parse("[abc] stays text");
    let inlines = get_paragraph_inlines(&blocks);
    // [abc] is not a citation, should be plain text
    let text: String = inlines
        .iter()
        .filter_map(|i| match i {
            InlineContent::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert!(text.contains("[abc]"));
}

#[test]
fn parse_citation_large_number() {
    let blocks = parse("[42]");
    let inlines = get_paragraph_inlines(&blocks);
    assert_eq!(inlines.len(), 1);
    assert!(matches!(&inlines[0], InlineContent::Citation(42)));
}

#[test]
fn parse_citation_in_bold() {
    let blocks = parse("**see [1]**");
    let inlines = get_paragraph_inlines(&blocks);
    assert_eq!(inlines.len(), 1);
    match &inlines[0] {
        InlineContent::Bold(inner) => {
            assert!(
                inner
                    .iter()
                    .any(|i| matches!(i, InlineContent::Citation(1)))
            );
        }
        _ => panic!("expected bold"),
    }
}

#[test]
fn parse_citation_streaming_split() {
    // Citation marker split across deltas: "[" then "1]"
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("See [");
    p.push_delta("1] for details");
    let blocks = p.parse();
    let inlines = get_paragraph_inlines(&blocks);
    assert!(
        inlines
            .iter()
            .any(|i| matches!(i, InlineContent::Citation(1)))
    );
}

fn parse_without_citations(input: &str) -> Rc<Vec<MarkdownBlock>> {
    let mut p = IncrementalMarkdownParser::new().with_citations(false);
    p.push_delta(input);
    p.parse()
}

#[test]
fn parse_citation_default_is_on() {
    // Guard against silent default flips — new() with NO builder call must
    // still split citations. Also covers the citations_enabled() accessor.
    let mut p = IncrementalMarkdownParser::new();
    assert!(
        p.citations_enabled(),
        "default must keep citation splitting on"
    );
    p.push_delta("See [1] for details");
    let blocks = p.parse();
    let inlines = get_paragraph_inlines(&blocks);
    assert!(matches!(&inlines[1], InlineContent::Citation(1)));
}

#[test]
fn parse_citation_disabled_accessor_reflects_toggle() {
    let p = IncrementalMarkdownParser::new().with_citations(false);
    assert!(!p.citations_enabled());
}

#[test]
fn parse_citation_disabled_keeps_literal_brackets() {
    // "item [5] of 10" must round-trip unchanged when splitting is off —
    // the regression from the issue report.
    let blocks = parse_without_citations("item [5] of 10");
    let inlines = get_paragraph_inlines(&blocks);
    assert_eq!(inlines.len(), 1);
    assert!(matches!(&inlines[0], InlineContent::Text(t) if t == "item [5] of 10"));
    assert!(
        !inlines
            .iter()
            .any(|i| matches!(i, InlineContent::Citation(_)))
    );
}

#[test]
fn parse_citation_disabled_inside_bold() {
    // The flag lives on &self and collect_inlines recurses through &self, so
    // nested contexts inherit the opt-out automatically.
    let blocks = parse_without_citations("**item [5] of 10**");
    let inlines = get_paragraph_inlines(&blocks);
    assert_eq!(inlines.len(), 1);
    match &inlines[0] {
        InlineContent::Bold(inner) => {
            assert_eq!(inner.len(), 1);
            assert!(matches!(&inner[0], InlineContent::Text(t) if t == "item [5] of 10"));
            assert!(
                !inner
                    .iter()
                    .any(|i| matches!(i, InlineContent::Citation(_)))
            );
        }
        other => panic!("expected bold, got {other:?}"),
    }
}

#[test]
fn parse_citation_disabled_inside_heading() {
    // collect_inlines runs for headings via parse_block on TagEnd::Heading.
    let blocks = parse_without_citations("# Section [5] of 10");
    match &blocks[0] {
        MarkdownBlock::Heading { level, content } => {
            assert_eq!(*level, 1);
            assert_eq!(content.len(), 1);
            assert!(matches!(&content[0], InlineContent::Text(t) if t == "Section [5] of 10"));
        }
        other => panic!("expected heading, got {other:?}"),
    }
}

#[test]
fn parse_citation_disabled_inside_table_cell() {
    // Table cells run through collect_inlines on TagEnd::TableCell.
    let blocks = parse_without_citations("| a | b |\n|---|---|\n| item [5] of 10 | x |");
    match &blocks[0] {
        MarkdownBlock::Table { rows, .. } => {
            assert_eq!(rows.len(), 1);
            let cell = &rows[0][0];
            assert_eq!(cell.len(), 1);
            assert!(matches!(&cell[0], InlineContent::Text(t) if t == "item [5] of 10"));
        }
        other => panic!("expected table, got {other:?}"),
    }
}

#[test]
fn parse_citation_disabled_inside_image_alt() {
    // Image alt text is built by joining only InlineContent::Text fragments
    // from inner collect_inlines. When citations are on, a leaked Citation
    // would silently disappear from alt; when off, the literal [5] survives.
    let blocks = parse_without_citations("![item [5] of 10](https://example.com/x.png)");
    let inlines = get_paragraph_inlines(&blocks);
    match &inlines[0] {
        InlineContent::Image { alt, .. } => {
            assert!(
                alt.contains("[5]"),
                "expected literal [5] in alt, got {alt:?}"
            );
        }
        other => panic!("expected image, got {other:?}"),
    }
}

#[test]
fn parse_citation_disabled_streaming_split() {
    // Mirror of parse_citation_streaming_split with the flag off — guards
    // against the toggle leaking across buffered flushes.
    let mut p = IncrementalMarkdownParser::new().with_citations(false);
    p.push_delta("item [");
    p.push_delta("5] of 10");
    let blocks = p.parse();
    let inlines = get_paragraph_inlines(&blocks);
    let text: String = inlines
        .iter()
        .filter_map(|i| match i {
            InlineContent::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert!(
        text.contains("[5]"),
        "expected literal [5] across streaming split, got {text:?}"
    );
    assert!(
        !inlines
            .iter()
            .any(|i| matches!(i, InlineContent::Citation(_)))
    );
}

#[test]
fn parse_citation_toggle_invalidates_cache() {
    // with_citations promises to invalidate the block cache. Parse with the
    // default (on), then toggle off, parse again, and confirm the cache did
    // not serve a stale Citation tree.
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta("See [1] for details");
    let first = p.parse();
    assert!(matches!(
        &get_paragraph_inlines(&first)[1],
        InlineContent::Citation(1)
    ));

    let mut p = p.with_citations(false);
    let blocks = p.parse();
    let inlines = get_paragraph_inlines(&blocks);
    assert!(
        !inlines
            .iter()
            .any(|i| matches!(i, InlineContent::Citation(_))),
        "toggle should invalidate cache; got {inlines:?}"
    );
}

use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_markdown_deltas(deltas in prop::collection::vec("[# *`\\-\\[\\](){}\n a-zA-Z0-9]*", 0..15)) {
        let mut parser = IncrementalMarkdownParser::new();
        for delta in &deltas {
            parser.push_delta(delta);
            let _ = parser.parse(); // must not panic
        }
        parser.finish();
        let _ = parser.parse(); // must not panic after finish either
    }

    #[test]
    fn fuzz_markdown_deltas_toggle(
        deltas in prop::collection::vec("[# *`\\-\\[\\](){}\n a-zA-Z0-9]*", 0..15),
        enable in prop::bool::ANY,
    ) {
        let mut parser = IncrementalMarkdownParser::new().with_citations(enable);
        for delta in &deltas {
            parser.push_delta(delta);
            let _ = parser.parse();
        }
        parser.finish();
        let _ = parser.parse();
    }
}

/// Helper that parses non-streaming (finished) input.
fn parse_finished(input: &str) -> Rc<Vec<MarkdownBlock>> {
    let mut p = IncrementalMarkdownParser::new();
    p.push_delta(input);
    let blocks = p.parse();
    p.finish();
    blocks
}

#[test]
fn parse_display_math() {
    let blocks = parse_finished("$$\nx^2 + y^2 = z^2\n$$");
    assert_eq!(blocks.len(), 1, "blocks: {:?}", blocks);
    match &blocks[0] {
        MarkdownBlock::DisplayMath(math) => {
            assert!(math.contains("x^2 + y^2 = z^2"));
        }
        other => panic!("expected display math, got {:?}", other),
    }
}

#[test]
fn parse_inline_math() {
    let blocks = parse_finished("before $E=mc^2$ after");
    assert_eq!(blocks.len(), 1);
    match &blocks[0] {
        MarkdownBlock::Paragraph(inlines) => {
            assert!(
                inlines
                    .iter()
                    .any(|i| matches!(i, InlineContent::Text(t) if t.contains("before")))
            );
            assert!(
                inlines
                    .iter()
                    .any(|i| matches!(i, InlineContent::InlineMath(m) if m == "E=mc^2"))
            );
            assert!(
                inlines
                    .iter()
                    .any(|i| matches!(i, InlineContent::Text(t) if t.contains("after")))
            );
        }
        other => panic!("expected paragraph, got {:?}", other),
    }
}
