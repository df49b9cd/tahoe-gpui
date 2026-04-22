//! Incremental markdown parser that accumulates deltas.

mod citations;
mod slug;
#[cfg(test)]
mod tests;
mod types;

use citations::split_citations;
pub use types::{InlineContent, MarkdownBlock, TableAlignment};

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use std::rc::Rc;

/// Incremental markdown parser that accumulates text deltas and produces blocks.
///
/// Optionally uses [`mdstitch`] to auto-complete incomplete markdown syntax
/// during streaming, ensuring content renders correctly token-by-token.
///
/// Inline citation splitting (`[N]` → [`InlineContent::Citation`]) is enabled
/// by default because AI-generated content commonly uses bracketed numerals
/// as citation markers. Callers rendering arbitrary human-written Markdown
/// should pass `false` to [`Self::with_citations`] so literal brackets
/// like `"item [5] of 10"` round-trip unchanged.
pub struct IncrementalMarkdownParser {
    /// Accumulated raw markdown text.
    source: String,
    /// Stitch options for preprocessing (None = disabled).
    stitch_options: Option<mdstitch::StitchOptions>,
    /// When `true`, bracketed digit sequences in inline text are split into
    /// [`InlineContent::Citation`] variants. Mirrored by the accessor
    /// [`Self::citations_enabled`].
    citations_enabled: bool,
    /// Whether the current text has an unclosed code fence.
    has_incomplete_code_fence: bool,
    /// Whether the current text contains a table.
    has_table: bool,
    /// Detected text direction.
    text_direction: mdstitch::TextDirection,
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
            stitch_options: None,
            citations_enabled: true,
            has_incomplete_code_fence: false,
            has_table: false,
            text_direction: mdstitch::TextDirection::default(),
            is_streaming: false,
            cached_blocks: Rc::new(Vec::new()),
            blocks_dirty: true,
            direction_detected: false,
        }
    }

    /// Creates a parser with mdstitch preprocessing enabled.
    pub fn with_stitch(options: mdstitch::StitchOptions) -> Self {
        let mut s = Self::new();
        s.stitch_options = Some(options);
        s
    }

    /// Enable or disable inline citation splitting.
    ///
    /// When `true` (the default), bracketed digit sequences in paragraph text
    /// are parsed as [`InlineContent::Citation`] markers. When `false`,
    /// `"item [5] of 10"` stays as a single `Text` inline — pass `false`
    /// when rendering non-AI Markdown where bracketed numbers are literal.
    ///
    /// Toggling post-construction invalidates the block cache so the next
    /// [`Self::parse`] call reflects the new flag.
    pub fn with_citations(mut self, enabled: bool) -> Self {
        self.citations_enabled = enabled;
        self.blocks_dirty = true;
        self
    }

    /// Returns whether inline citation splitting is enabled.
    pub fn citations_enabled(&self) -> bool {
        self.citations_enabled
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
            self.has_incomplete_code_fence = mdstitch::has_incomplete_code_fence(&self.source);
        }
        if delta.contains('|') {
            self.has_table = mdstitch::has_table(&self.source);
        }

        // Detect text direction once the first alphabetic character arrives
        // (or immediately on RTL), then cache. This avoids re-running
        // detect_text_direction on every push_delta for purely numeric or
        // punctuation-only prefixes.
        if !self.direction_detected {
            let dir_end = self
                .source
                .char_indices()
                .nth(200)
                .map_or(self.source.len(), |(idx, _)| idx);
            let dir = mdstitch::detect_text_direction(&self.source[..dir_end]);
            self.text_direction = dir;
            if dir == mdstitch::TextDirection::Rtl
                || self.source[..dir_end].chars().any(|c| c.is_alphabetic())
            {
                self.direction_detected = true;
            }
        }
    }

    /// Marks the stream as finished.
    ///
    /// Performs one final parse without mdstitch preprocessing so the cache
    /// reflects a clean, authoritative tree even when `push_delta` arrives
    /// in the same frame as `finish` (in which case the cache is still
    /// dirty from the pending delta and must be rebuilt from the raw
    /// source before it is dropped).
    pub fn finish(&mut self) {
        self.is_streaming = false;
        // Force a final reparse without mdstitch so any incomplete-syntax
        // repairs from the last streaming parse are dropped in favour of
        // the raw final source.
        self.blocks_dirty = true;
        let _blocks = self.parse();
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
    pub fn text_direction(&self) -> mdstitch::TextDirection {
        self.text_direction
    }

    /// Parse the full source into blocks. Call after `push_delta`.
    ///
    /// Returns cached blocks if the source hasn't changed since the last parse.
    /// When mdstitch is enabled and the stream is active, the source text is
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
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_HEADING_ATTRIBUTES;

        // Apply mdstitch preprocessing if enabled and still streaming.
        let preprocessed: std::borrow::Cow<'_, str> = if self.is_streaming {
            if let Some(ref stitch_opts) = self.stitch_options {
                mdstitch::stitch(&self.source, stitch_opts)
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

        dedupe_heading_anchors(&mut blocks);

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
                    let math_block =
                        MarkdownBlock::DisplayMath(gpui::SharedString::from(math.to_string()));
                    // Skip: Start(Paragraph), DisplayMath, End(Paragraph)
                    let end = start + 3;
                    return (Some(math_block), end);
                }
                let (inlines, end) = self.collect_inlines(events, start + 1, TagEnd::Paragraph);
                (Some(MarkdownBlock::Paragraph(inlines)), end)
            }
            Event::Start(Tag::Heading { level, id, .. }) => {
                let level_num = *level as u8;
                let end_tag = TagEnd::Heading(*level);
                let (inlines, end) = self.collect_inlines(events, start + 1, end_tag);
                // Both explicit `{#id}` attributes and auto-derived
                // heading text go through `slugify` so the resulting
                // anchor is always URL-fragment safe. Explicit ids like
                // `{#My Custom ID}` become `my-custom-id`; this matches
                // GFM's treatment and means consumers don't have to
                // second-guess whether an id contains whitespace or
                // other characters that would break fragment matching.
                let explicit = id.as_ref().map(|s| s.as_ref()).unwrap_or("").trim();
                let candidate = if !explicit.is_empty() {
                    slug::slugify(explicit)
                } else {
                    slug::slugify(&inline_plain_text(&inlines))
                };
                let anchor_id = if candidate.is_empty() {
                    None
                } else {
                    Some(candidate)
                };
                (
                    Some(MarkdownBlock::Heading {
                        level: level_num,
                        content: inlines,
                        anchor_id,
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
                Some(MarkdownBlock::DisplayMath(gpui::SharedString::from(
                    math.to_string(),
                ))),
                start + 1,
            ),
            Event::Html(html) => (
                Some(MarkdownBlock::Paragraph(vec![InlineContent::Text(
                    html.to_string(),
                )])),
                start + 1,
            ),
            Event::Start(Tag::FootnoteDefinition(label)) => {
                let label_str = label.to_string();
                let mut inner_blocks = Vec::new();
                let mut idx = start + 1;
                while idx < events.len() {
                    match &events[idx] {
                        Event::End(TagEnd::FootnoteDefinition) => {
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
                (
                    Some(MarkdownBlock::FootnoteDefinition {
                        label: label_str,
                        content: inner_blocks,
                    }),
                    idx,
                )
            }
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

        // Flush accumulated text. When citation splitting is enabled, run
        // the splitter; otherwise move the buffer through as a single Text
        // inline so bracketed digits stay literal.
        let citations_enabled = self.citations_enabled;
        let flush_text = |buf: &mut String, out: &mut Vec<InlineContent>| {
            if buf.is_empty() {
                return;
            }
            if citations_enabled {
                split_citations(buf, out);
                buf.clear();
            } else {
                out.push(InlineContent::Text(std::mem::take(buf)));
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
                Event::InlineHtml(html) => {
                    text_buf.push_str(html);
                    idx += 1;
                }
                Event::FootnoteReference(label) => {
                    flush_text(&mut text_buf, &mut inlines);
                    inlines.push(InlineContent::FootnoteReference(label.to_string()));
                    idx += 1;
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

/// Flatten inline content to plain text for slug computation. Concatenates
/// text runs, recurses through nesting (bold/italic/strikethrough/link),
/// maps line breaks to a single space, and includes image alt text so
/// `## ![Logo](logo.png) Project` slugs as `logo-project` (matching
/// GitHub). Non-textual inlines (citations, math, task markers) emit a
/// single space rather than vanishing so adjacent text doesn't merge —
/// `## foo[^1]bar` slugs as `foo-bar`, not `foobar`. The extra spaces
/// collapse to single `-` separators in `slugify`.
fn inline_plain_text(inlines: &[InlineContent]) -> String {
    fn walk(inlines: &[InlineContent], out: &mut String) {
        for inline in inlines {
            match inline {
                InlineContent::Text(t) => out.push_str(t),
                InlineContent::Code(c) => out.push_str(c),
                InlineContent::Bold(inner)
                | InlineContent::Italic(inner)
                | InlineContent::Strikethrough(inner) => walk(inner, out),
                InlineContent::Link { content, .. } => walk(content, out),
                InlineContent::Image { alt, .. } => out.push_str(alt),
                InlineContent::SoftBreak | InlineContent::HardBreak => out.push(' '),
                InlineContent::Citation(_)
                | InlineContent::FootnoteReference(_)
                | InlineContent::InlineMath(_)
                | InlineContent::TaskMarker(_) => out.push(' '),
            }
        }
    }
    let mut out = String::with_capacity(inlines.len() * 16);
    walk(inlines, &mut out);
    out
}

/// Append `-2`, `-3`, … to duplicate heading anchors so each slug is unique
/// within the document (matches GitHub's anchor-generation rules). Skips
/// any numeric suffix that is already taken by an explicit id or a prior
/// generated one so explicit and generated ids never collide.
fn dedupe_heading_anchors(blocks: &mut [MarkdownBlock]) {
    let mut seen: HashMap<String, u32> = HashMap::new();
    for block in blocks.iter_mut() {
        if let MarkdownBlock::Heading {
            anchor_id: Some(id),
            ..
        } = block
        {
            match seen.get(id.as_str()).copied() {
                None => {
                    // First occurrence — register the id as seen with
                    // counter 1. We clone rather than `mem::take`
                    // because the block must retain its slug as the
                    // block's `anchor_id`.
                    seen.insert(id.clone(), 1);
                }
                Some(last) => {
                    // Collision — scan `{id}-{n}` candidates starting
                    // from `last + 1` until we find one not already in
                    // `seen` (covers prior generated AND prior explicit
                    // ids). Update both the base counter and the new
                    // unique id's counter so further collisions resume
                    // past this suffix.
                    let mut n = last + 1;
                    let unique = loop {
                        let candidate = format!("{id}-{n}");
                        if !seen.contains_key(&candidate) {
                            break candidate;
                        }
                        n += 1;
                    };
                    let base = id.clone();
                    seen.insert(base, n);
                    seen.insert(unique.clone(), 1);
                    *id = unique;
                }
            }
        }
    }
}
