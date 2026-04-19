//! Incremental markdown parser that accumulates deltas.

mod citations;
#[cfg(test)]
mod tests;
mod types;

use citations::split_citations;
pub use types::{InlineContent, MarkdownBlock, TableAlignment};

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::rc::Rc;

/// Incremental markdown parser that accumulates text deltas and produces blocks.
///
/// Optionally uses [`remend`] to auto-complete incomplete markdown syntax
/// during streaming, ensuring content renders correctly token-by-token.
pub struct IncrementalMarkdownParser {
    /// Accumulated raw markdown text.
    source: String,
    /// Remend options for preprocessing (None = disabled).
    remend_options: Option<remend::RemendOptions>,
    /// Whether the current text has an unclosed code fence.
    has_incomplete_code_fence: bool,
    /// Whether the current text contains a table.
    has_table: bool,
    /// Detected text direction.
    text_direction: remend::TextDirection,
    /// Whether the stream is still active.
    is_streaming: bool,
    /// Cached parsed blocks to avoid re-parsing on every render frame.
    /// Wrapped in `Rc` so callers can share ownership without cloning the entire vec.
    cached_blocks: Rc<Vec<MarkdownBlock>>,
    /// Whether the source has changed since the last parse.
    blocks_dirty: bool,
    /// Whether text direction has been detected (only detect once).
    direction_detected: bool,
}

impl IncrementalMarkdownParser {
    pub fn new() -> Self {
        Self {
            source: String::new(),
            remend_options: None,
            has_incomplete_code_fence: false,
            has_table: false,
            text_direction: remend::TextDirection::default(),
            is_streaming: false,
            cached_blocks: Rc::new(Vec::new()),
            blocks_dirty: true,
            direction_detected: false,
        }
    }

    /// Creates a parser with remend preprocessing enabled.
    pub fn with_remend(options: remend::RemendOptions) -> Self {
        let mut s = Self::new();
        s.remend_options = Some(options);
        s
    }

    /// Append a text delta.
    pub fn push_delta(&mut self, delta: &str) {
        if !self.is_streaming {
            self.is_streaming = true;
        }
        self.source.push_str(delta);
        self.blocks_dirty = true;

        // Update code fence / table state incrementally:
        // Only re-scan if the delta could have changed the state.
        if delta.contains('`') || delta.contains('~') {
            self.has_incomplete_code_fence = remend::has_incomplete_code_fence(&self.source);
        }
        if delta.contains('|') {
            self.has_table = remend::has_table(&self.source);
        }

        // Detect text direction once from first ~200 chars, then cache.
        if !self.direction_detected {
            let dir_end = self
                .source
                .char_indices()
                .nth(200)
                .map_or(self.source.len(), |(idx, _)| idx);
            self.text_direction = remend::detect_text_direction(&self.source[..dir_end]);
            if self.source.len() >= 200 {
                self.direction_detected = true;
            }
        }
    }

    /// Marks the stream as finished.
    pub fn finish(&mut self) {
        self.is_streaming = false;
        self.blocks_dirty = false; // Cache is already up to date from the last push_delta + parse cycle.
        // Release raw source string — cached_blocks is now the authority.
        self.source = String::new();
    }

    /// Get the full source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns whether the current text has an unclosed code fence.
    pub fn has_incomplete_code_fence(&self) -> bool {
        self.has_incomplete_code_fence
    }

    /// Returns whether the current text contains a table.
    pub fn has_table(&self) -> bool {
        self.has_table
    }

    /// Returns the detected text direction.
    pub fn text_direction(&self) -> remend::TextDirection {
        self.text_direction
    }

    /// Parse the full source into blocks. Call after `push_delta`.
    ///
    /// Returns cached blocks if the source hasn't changed since the last parse.
    /// When remend is enabled and the stream is active, the source text is
    /// preprocessed to auto-complete incomplete markdown syntax before parsing.
    pub fn parse(&mut self) -> Rc<Vec<MarkdownBlock>> {
        if !self.blocks_dirty {
            return Rc::clone(&self.cached_blocks);
        }
        self.blocks_dirty = false;

        // GFM footnotes are common in AI-generated output, and pulldown-cmark
        // silently drops them unless the option is enabled — the issue #150
        // audit flagged this. Task lists keep the existing `TaskListMarker`
        // events so the renderer can draw checkbox glyphs.
        let options = Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS
            | Options::ENABLE_MATH
            | Options::ENABLE_FOOTNOTES;

        // Apply remend preprocessing if enabled and still streaming.
        let preprocessed: std::borrow::Cow<'_, str> = if self.is_streaming {
            if let Some(ref remend_opts) = self.remend_options {
                remend::remend(&self.source, remend_opts)
            } else {
                std::borrow::Cow::Borrowed(&self.source)
            }
        } else {
            std::borrow::Cow::Borrowed(&self.source)
        };

        let parser = Parser::new_ext(&preprocessed, options);
        let events: Vec<Event<'_>> = parser.collect();

        let mut blocks = Vec::new();
        let mut idx = 0;

        while idx < events.len() {
            let (block, next_idx) = self.parse_block(&events, idx);
            if let Some(block) = block {
                blocks.push(block);
            }
            if next_idx <= idx {
                idx += 1;
            } else {
                idx = next_idx;
            }
        }

        self.cached_blocks = Rc::new(blocks);
        Rc::clone(&self.cached_blocks)
    }

    fn parse_block<'a>(
        &self,
        events: &[Event<'a>],
        start: usize,
    ) -> (Option<MarkdownBlock>, usize) {
        if start >= events.len() {
            return (None, start + 1);
        }

        match &events[start] {
            Event::Start(Tag::Paragraph) => {
                // pulldown-cmark wraps DisplayMath inside a Paragraph;
                // detect and extract it as a block-level element.
                if let Some(Event::DisplayMath(math)) = events.get(start + 1) {
                    let math_block = MarkdownBlock::DisplayMath(math.to_string());
                    // Skip: Start(Paragraph), DisplayMath, End(Paragraph)
                    let end = start + 3;
                    return (Some(math_block), end);
                }
                let (inlines, end) = self.collect_inlines(events, start + 1, TagEnd::Paragraph);
                (Some(MarkdownBlock::Paragraph(inlines)), end)
            }
            Event::Start(Tag::Heading { level, .. }) => {
                let level_num = *level as u8;
                let end_tag = TagEnd::Heading(*level);
                let (inlines, end) = self.collect_inlines(events, start + 1, end_tag);
                (
                    Some(MarkdownBlock::Heading {
                        level: level_num,
                        content: inlines,
                    }),
                    end,
                )
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(lang) => {
                        let lang = lang.to_string();
                        if lang.is_empty() { None } else { Some(lang) }
                    }
                    CodeBlockKind::Indented => None,
                };
                let mut code = String::new();
                let mut idx = start + 1;
                while idx < events.len() {
                    match &events[idx] {
                        Event::Text(text) => code.push_str(text),
                        Event::End(TagEnd::CodeBlock) => {
                            idx += 1;
                            break;
                        }
                        _ => {}
                    }
                    idx += 1;
                }
                (Some(MarkdownBlock::CodeBlock { language, code }), idx)
            }
            Event::Start(Tag::List(start_num)) => {
                let ordered = start_num.is_some();
                let start_val = *start_num;
                let mut items = Vec::new();
                let mut idx = start + 1;

                while idx < events.len() {
                    match &events[idx] {
                        Event::Start(Tag::Item) => {
                            idx += 1;
                            let mut item_blocks: Vec<MarkdownBlock> = Vec::new();
                            // GFM task markers appear at the item level
                            // before the first paragraph. Capture the
                            // marker here and prepend an
                            // `InlineContent::TaskMarker` to the first
                            // inline run of the item so the renderer can
                            // draw a checkbox glyph. Without this the
                            // marker event would fall through `parse_block`
                            // and the checkbox state would be lost.
                            let mut pending_task_marker: Option<bool> = None;
                            while idx < events.len() {
                                match &events[idx] {
                                    Event::End(TagEnd::Item) => {
                                        idx += 1;
                                        break;
                                    }
                                    Event::TaskListMarker(checked) => {
                                        pending_task_marker = Some(*checked);
                                        idx += 1;
                                    }
                                    _ => {
                                        let (block, next) = self.parse_block(events, idx);
                                        if let Some(mut b) = block {
                                            if let Some(checked) = pending_task_marker.take()
                                                && let MarkdownBlock::Paragraph(inlines) = &mut b
                                            {
                                                inlines
                                                    .insert(0, InlineContent::TaskMarker(checked));
                                            }
                                            item_blocks.push(b);
                                        }
                                        idx = next;
                                    }
                                }
                            }
                            // Item ended without any block content (e.g. an
                            // empty checkbox). Emit a synthetic paragraph
                            // holding only the task marker so the renderer
                            // still draws the checkbox glyph.
                            if let Some(checked) = pending_task_marker.take() {
                                item_blocks.push(MarkdownBlock::Paragraph(vec![
                                    InlineContent::TaskMarker(checked),
                                ]));
                            }
                            items.push(item_blocks);
                        }
                        Event::End(TagEnd::List(_)) => {
                            idx += 1;
                            break;
                        }
                        _ => idx += 1,
                    }
                }

                (
                    Some(MarkdownBlock::List {
                        ordered,
                        start: start_val,
                        items,
                    }),
                    idx,
                )
            }
            Event::Start(Tag::BlockQuote(_)) => {
                let mut inner_blocks = Vec::new();
                let mut idx = start + 1;
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::BlockQuote(_)) => {
                            idx += 1;
                            break;
                        }
                        _ => {
                            let (block, next) = self.parse_block(events, idx);
                            if let Some(b) = block {
                                inner_blocks.push(b);
                            }
                            idx = next;
                        }
                    }
                }
                (Some(MarkdownBlock::BlockQuote(inner_blocks)), idx)
            }
            Event::Start(Tag::Table(alignments)) => {
                let alignments: Vec<TableAlignment> = alignments
                    .iter()
                    .copied()
                    .map(TableAlignment::from)
                    .collect();
                let mut headers: Vec<Vec<InlineContent>> = Vec::new();
                let mut rows: Vec<Vec<Vec<InlineContent>>> = Vec::new();
                let mut idx = start + 1;
                let mut in_head = false;
                let mut current_row: Vec<Vec<InlineContent>> = Vec::new();

                while idx < events.len() {
                    match &events[idx] {
                        Event::Start(Tag::TableHead) => {
                            in_head = true;
                            current_row.clear();
                            idx += 1;
                        }
                        Event::End(TagEnd::TableHead) => {
                            in_head = false;
                            headers = std::mem::take(&mut current_row);
                            idx += 1;
                        }
                        Event::Start(Tag::TableRow) => {
                            current_row.clear();
                            idx += 1;
                        }
                        Event::End(TagEnd::TableRow) => {
                            if !in_head {
                                rows.push(std::mem::take(&mut current_row));
                            }
                            idx += 1;
                        }
                        Event::Start(Tag::TableCell) => {
                            let (inlines, end) =
                                self.collect_inlines(events, idx + 1, TagEnd::TableCell);
                            current_row.push(inlines);
                            idx = end;
                        }
                        Event::End(TagEnd::Table) => {
                            idx += 1;
                            break;
                        }
                        _ => idx += 1,
                    }
                }

                (
                    Some(MarkdownBlock::Table {
                        headers,
                        rows,
                        alignments,
                    }),
                    idx,
                )
            }
            Event::Rule => (Some(MarkdownBlock::ThematicBreak), start + 1),
            Event::DisplayMath(math) => (
                Some(MarkdownBlock::DisplayMath(math.to_string())),
                start + 1,
            ),
            _ => (None, start + 1),
        }
    }

    fn collect_inlines<'a>(
        &self,
        events: &[Event<'a>],
        start: usize,
        end_tag: TagEnd,
    ) -> (Vec<InlineContent>, usize) {
        let mut inlines = Vec::new();
        let mut idx = start;
        // Buffer for accumulating consecutive text events (pulldown_cmark
        // splits `[1]` into `[`, `1`, `]` as separate Text events).
        let mut text_buf = String::new();

        // Flush accumulated text through citation splitting.
        let flush_text = |buf: &mut String, out: &mut Vec<InlineContent>| {
            if !buf.is_empty() {
                split_citations(buf, out);
                buf.clear();
            }
        };

        while idx < events.len() {
            match &events[idx] {
                Event::End(tag) if *tag == end_tag => {
                    flush_text(&mut text_buf, &mut inlines);
                    idx += 1;
                    break;
                }
                Event::Text(text) => {
                    text_buf.push_str(text);
                    idx += 1;
                }
                Event::Code(code) => {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(InlineContent::Code(code.to_string()));
                    idx += 1;
                }
                Event::InlineMath(math) => {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(InlineContent::InlineMath(math.to_string()));
                    idx += 1;
                }
                Event::SoftBreak => {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(InlineContent::SoftBreak);
                    idx += 1;
                }
                Event::HardBreak => {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(InlineContent::HardBreak);
                    idx += 1;
                }
                Event::TaskListMarker(checked) => {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(InlineContent::TaskMarker(*checked));
                    idx += 1;
                }
                Event::Start(Tag::Emphasis) => {
                    flush_text(&mut text_buf, &mut inlines);
                    let (inner, end) = self.collect_inlines(events, idx + 1, TagEnd::Emphasis);
                    inlines.push(InlineContent::Italic(inner));
                    idx = end;
                }
                Event::Start(Tag::Strong) => {
                    flush_text(&mut text_buf, &mut inlines);
                    let (inner, end) = self.collect_inlines(events, idx + 1, TagEnd::Strong);
                    inlines.push(InlineContent::Bold(inner));
                    idx = end;
                }
                Event::Start(Tag::Strikethrough) => {
                    flush_text(&mut text_buf, &mut inlines);
                    let (inner, end) = self.collect_inlines(events, idx + 1, TagEnd::Strikethrough);
                    inlines.push(InlineContent::Strikethrough(inner));
                    idx = end;
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    flush_text(&mut text_buf, &mut inlines);
                    let url = dest_url.to_string();
                    let (inner, end) = self.collect_inlines(events, idx + 1, TagEnd::Link);
                    inlines.push(InlineContent::Link {
                        url,
                        content: inner,
                    });
                    idx = end;
                }
                Event::Start(Tag::Image { dest_url, .. }) => {
                    flush_text(&mut text_buf, &mut inlines);
                    let url = dest_url.to_string();
                    // Collect alt text from the image's inner events
                    let (alt_inlines, end) = self.collect_inlines(events, idx + 1, TagEnd::Image);
                    let alt = alt_inlines
                        .iter()
                        .map(|i| match i {
                            InlineContent::Text(t) => t.as_str(),
                            _ => "",
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    inlines.push(InlineContent::Image { url, alt });
                    idx = end;
                }
                _ => {
                    flush_text(&mut text_buf, &mut inlines);
                    idx += 1;
                }
            }
        }

        flush_text(&mut text_buf, &mut inlines);
        (inlines, idx)
    }
}

impl Default for IncrementalMarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}
