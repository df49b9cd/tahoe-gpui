//! Renders parsed markdown blocks as GPUI elements.
//!
//! # Architecture
//!
//! Inline content renders through two paths:
//!
//! - **Flat path** ([`render_inlines_flat`]): the common case. Builds a
//!   single `Vec<TextRun>` via a style stack and emits one
//!   [`StyledText::with_runs`] layout, matching Zed's own markdown
//!   renderer (`crates/markdown/src/markdown.rs`). Wrapped in
//!   [`super::selectable_text::SelectableText`] so the paragraph
//!   supports drag-select, link click, I-beam cursor, and Cmd/Ctrl+C
//!   copy. Inline code, links, bold/italic/strikethrough, and nested
//!   combinations all live here — one text layout per run of inlines.
//!
//! - **Mixed path** ([`render_inlines_mixed`]): triggered only by
//!   element-level content (interactive citations, image URLs). Splits
//!   the inline run into flex-wrap children of flat-path segments and
//!   element-level elements (`InlineCitation`, `img(...)`).
//!
//! # HIG compliance notes — remaining upstream-blocked items
//!
//! Most findings from issue #150 are fixed in-tree. The items below
//! remain blocked on GPUI capabilities and are documented at their call
//! sites as well:
//!
//! - **SF Pro tracking / letter-spacing (F4):** `TextStyleAttrs::tracking`
//!   carries the correct per-size Apple values (see
//!   [`crate::foundations::typography::macos_tracking`]), but GPUI
//!   `v0.231.1-pre` does not honour letter-spacing when laying out text.
//!   When upstream lands the API `TextStyledExt` will start applying
//!   tracking automatically with no downstream change.
//!
//! - **Text selection (F17):** cross-paragraph drag-select works via
//!   [`super::selectable_text::SelectableText`] (a custom `Element`
//!   that paints selection quads and handles mouse drag / link
//!   dispatch) coordinated through [`super::selection::MarkdownSelection`]
//!   (a shared `Rc<RefCell>` registry that bridges per-paragraph
//!   elements into document-wide selection state). Supported:
//!   single / double / triple / quad-click modes, shift-click
//!   extend, drag across paragraphs, Cmd/Ctrl+C copy (joined with
//!   `\n` across paragraphs), Cmd/Ctrl+A select-all, and link
//!   click-through. Equivalent to Zed's own markdown selection
//!   (`crates/markdown/src/markdown.rs:1022`) while keeping each
//!   paragraph as an independent element.
//!
//! - **Word-level fade-in animation (F16):** Zed's own markdown renderer
//!   does not animate individual tokens either — it pops whole blocks
//!   in as they parse. Our [`super::animation::AnimationState`]
//!   infrastructure (with Reduce Motion support) is retained as
//!   opt-in through `StreamSettings::animation` for hosts that want to
//!   drive per-word reveals themselves.
//!
//! - **Table column text alignment (F19 / F22):** [`TableAlignment`]
//!   is parsed and threaded through to rendering, but GPUI has no
//!   `.text_align()` style, so per-column alignment is approximated
//!   with flex main-axis positioning (matches Zed's
//!   `crates/markdown/src/html/html_rendering.rs`). Multi-line wrapped
//!   cells still justify on the leading edge until a true text-align
//!   lands.

use super::caret::{CaretKind, render_caret};
use super::code_block::CodeBlockView;
use super::mermaid::MermaidBlock;
use super::parser::{IncrementalMarkdownParser, InlineContent, MarkdownBlock, TableAlignment};
use super::selectable_text::SelectableText;
use super::selection::MarkdownSelection;
use super::settings::StreamSettings;
use crate::citation::{CitationPopover, CitationSource, InlineCitation};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ElementId, Entity, FontStyle, FontWeight, HighlightStyle, ObjectFit,
    SharedString, SharedUri, StrikethroughStyle, StyledText, TextRun,
    TextStyle as GpuiTextStyle, UnderlineStyle, Window, div, img, px,
};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::time::Instant;

/// Maximum nesting depth at which list items keep adding indentation.
/// Beyond this, the left padding is clamped so deeply nested AI-generated
/// markdown does not overflow narrow panels. HIG Layout: preserve a
/// predictable hierarchy without runaway indentation.
const LIST_MAX_DEPTH: usize = 4;

/// Provides source data for citation numbers during rendering.
#[derive(Default, Clone)]
pub struct CitationContext {
    /// Maps citation number (1-based) to source data (supports multiple sources per citation).
    pub sources: HashMap<usize, Vec<CitationSource>>,
}

/// Security configuration for markdown rendering.
///
/// Controls which URLs are allowed for links and images.
/// An empty list blocks all; a list containing `"*"` allows all.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkdownSecurity {
    /// Allowed URL prefixes for links. Default: `["*"]` (allow all).
    pub allowed_link_prefixes: Vec<String>,
    /// Allowed URL prefixes for images. Default: `["*"]` (allow all).
    pub allowed_image_prefixes: Vec<String>,
    /// Default origin to prepend to relative URLs in links and images.
    /// When set, relative URLs (not starting with `http://`, `https://`, or `//`)
    /// will have this origin prepended.
    pub default_origin: Option<String>,
}

impl Default for MarkdownSecurity {
    fn default() -> Self {
        Self {
            allowed_link_prefixes: vec!["*".to_string()],
            allowed_image_prefixes: vec!["*".to_string()],
            default_origin: None,
        }
    }
}

impl MarkdownSecurity {
    /// Check if a URL is allowed by the given prefix list.
    fn is_url_allowed(url: &str, prefixes: &[String]) -> bool {
        if prefixes.iter().any(|p| p == "*") {
            return true;
        }
        if prefixes.is_empty() {
            return false;
        }
        prefixes
            .iter()
            .any(|prefix| url.starts_with(prefix.as_str()))
    }

    /// Check if a link URL is allowed.
    pub fn is_link_allowed(&self, url: &str) -> bool {
        Self::is_url_allowed(url, &self.allowed_link_prefixes)
    }

    /// Check if an image URL is allowed.
    pub fn is_image_allowed(&self, url: &str) -> bool {
        Self::is_url_allowed(url, &self.allowed_image_prefixes)
    }

    /// Resolve a URL, prepending `default_origin` for relative URLs.
    ///
    /// Only treats URLs as relative if they lack a URI scheme (no `:` before
    /// the first `/`). Absolute URLs with any scheme (http, https, mailto,
    /// javascript, data, etc.) are returned unchanged.
    pub fn resolve_url<'a>(&self, url: &'a str) -> std::borrow::Cow<'a, str> {
        use std::borrow::Cow;

        // Return absolute URLs unchanged (any scheme, protocol-relative, or fragment).
        if url.starts_with("//") || url.starts_with('#') {
            return Cow::Borrowed(url);
        }
        // Check for a URI scheme: if there's a ':' before the first '/' or '?',
        // it's an absolute URL (e.g. http:, mailto:, javascript:, data:).
        if let Some(colon_pos) = url.find(':') {
            let has_slash_before = url[..colon_pos].contains('/');
            if !has_slash_before {
                return Cow::Borrowed(url);
            }
        }

        if let Some(ref origin) = self.default_origin {
            let origin = origin.trim_end_matches('/');
            if url.starts_with('/') {
                return Cow::Owned(format!("{}{}", origin, url));
            } else {
                return Cow::Owned(format!("{}/{}", origin, url));
            }
        }
        Cow::Borrowed(url)
    }
}

/// Generation phase of a streaming markdown entity.
///
/// Finding 29 in the Zed cross-reference audit:
/// HIG Generative AI §"Factor processing time into your design" says the
/// UI should advertise a loading indicator while the model is thinking
/// and a terminal indicator (completion tick / error icon) once it stops.
/// This enum captures the four terminal states so the renderer can pick
/// the right indicator without the host having to track the flag in two
/// places.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GenerativeLoadingState {
    /// No active generation — neither a caret nor a loader should show.
    #[default]
    Idle,
    /// Model is generating — show the blinking caret + optional
    /// `ActivityIndicator` when no tokens have arrived yet.
    Generating,
    /// Generation finished successfully — show a completion tick briefly
    /// before falling back to `Idle`.
    Done,
    /// Generation ended in an error — show an error icon until the
    /// caller transitions back to `Idle`.
    Error,
}

impl GenerativeLoadingState {
    /// True while the caret / activity indicator should visibly
    /// advertise work-in-progress.
    pub fn is_in_flight(self) -> bool {
        matches!(self, Self::Generating)
    }

    /// True when the caller should render a terminal state indicator
    /// (completion tick or error icon).
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Error)
    }
}

/// A streaming markdown entity that accumulates deltas and re-renders.
pub struct StreamingMarkdown {
    parser: IncrementalMarkdownParser,
    /// Whether the stream is currently active (show caret).
    is_streaming: bool,
    /// Lifecycle state published alongside `is_streaming` so hosts that
    /// want to render explicit `Done` / `Error` badges can do so without
    /// a separate state machine. Finding 29 in the Zed cross-reference audit.
    loading_state: GenerativeLoadingState,
    /// Citation sources for inline rendering.
    citation_ctx: CitationContext,
    /// Unique ID counter for citation elements.
    citation_id_counter: Cell<usize>,
    /// URL allowlist security configuration.
    security: MarkdownSecurity,
    /// Pre-created citation popover entities (preserves hover/carousel state).
    citation_popovers: HashMap<usize, Entity<CitationPopover>>,
    /// Tracks which citation numbers have been rendered in the current pass.
    /// Prevents cloning the same Entity (which would share state and duplicate IDs).
    rendered_popovers: RefCell<HashSet<usize>>,
    /// Streaming-specific rendering configuration — caret kind, caret color
    /// override, blink interval, Reduce Motion flag, word-level fade-in
    /// tokens. Hosts that wire `TahoeTheme::accessibility_mode.reduce_motion()`
    /// via [`Self::with_settings`] get automatic HIG-compliant Reduce Motion
    /// behaviour for the streaming caret.
    settings: StreamSettings,
    /// Shared cross-paragraph selection coordinator. Every
    /// `SelectableText` rendered by this entity clones the same
    /// handle so drag-select, copy, and multi-click gestures span
    /// the entire rendered markdown rather than each paragraph in
    /// isolation.
    selection: MarkdownSelection,
}

impl StreamingMarkdown {
    /// Creates a new streaming markdown renderer (remend disabled).
    pub fn new(_cx: &mut App) -> Self {
        Self {
            parser: IncrementalMarkdownParser::new(),
            is_streaming: false,
            loading_state: GenerativeLoadingState::Idle,
            citation_ctx: CitationContext::default(),
            citation_id_counter: Cell::new(0),
            security: MarkdownSecurity::default(),
            citation_popovers: HashMap::new(),
            rendered_popovers: RefCell::new(HashSet::new()),
            settings: StreamSettings::default(),
            selection: MarkdownSelection::new(),
        }
    }

    /// Creates a new streaming markdown renderer with remend preprocessing.
    pub fn with_remend(options: remend::RemendOptions, _cx: &mut App) -> Self {
        Self {
            parser: IncrementalMarkdownParser::with_remend(options),
            is_streaming: false,
            loading_state: GenerativeLoadingState::Idle,
            citation_ctx: CitationContext::default(),
            citation_id_counter: Cell::new(0),
            security: MarkdownSecurity::default(),
            citation_popovers: HashMap::new(),
            rendered_popovers: RefCell::new(HashSet::new()),
            settings: StreamSettings::default(),
            selection: MarkdownSelection::new(),
        }
    }

    /// Override the streaming settings at construction.
    pub fn with_settings(mut self, settings: StreamSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Replace the streaming settings after construction and trigger a redraw.
    pub fn set_settings(&mut self, settings: StreamSettings, cx: &mut Context<Self>) {
        self.settings = settings;
        cx.notify();
    }

    /// Current streaming settings.
    pub fn settings(&self) -> &StreamSettings {
        &self.settings
    }

    /// The shared selection coordinator. Hosts can query
    /// [`MarkdownSelection::selected_text`] to grab the user's
    /// current selection or call [`MarkdownSelection::clear`] to
    /// drop any existing highlight (e.g. when focus moves away).
    pub fn selection(&self) -> MarkdownSelection {
        self.selection.clone()
    }

    /// Set the security configuration for URL allowlists.
    pub fn set_security(&mut self, security: MarkdownSecurity, cx: &mut Context<Self>) {
        self.security = security;
        cx.notify();
    }

    /// Update citation sources for inline rendering.
    pub fn set_citation_sources(&mut self, ctx: CitationContext, cx: &mut Context<Self>) {
        for (num, sources) in &ctx.sources {
            if let Some(existing) = self.citation_popovers.get(num) {
                existing.update(cx, |p: &mut CitationPopover, cx| {
                    p.update_sources(sources.clone(), cx)
                });
            } else {
                let n = *num;
                let s = sources.clone();
                let entity = cx.new(|cx| CitationPopover::new(n, s, cx));
                self.citation_popovers.insert(n, entity);
            }
        }
        // Remove popover entities for citation numbers no longer in the context.
        self.citation_popovers
            .retain(|k, _| ctx.sources.contains_key(k));
        self.citation_ctx = ctx;
        cx.notify();
    }

    /// Append a text delta and trigger re-render.
    pub fn push_delta(&mut self, delta: &str, cx: &mut Context<Self>) {
        self.parser.push_delta(delta);
        self.is_streaming = true;
        self.loading_state = GenerativeLoadingState::Generating;
        cx.notify();
    }

    /// Marks the stream as finished. Re-parses without remend since the
    /// complete text should have valid syntax.
    pub fn finish(&mut self, cx: &mut Context<Self>) {
        self.parser.finish();
        self.is_streaming = false;
        self.loading_state = GenerativeLoadingState::Done;
        cx.notify();
    }

    /// Mark the stream as failed — the UI should swap the caret for an
    /// error indicator. Callers should transition back to
    /// [`GenerativeLoadingState::Idle`] via [`Self::clear_loading_state`]
    /// once the user has acknowledged the failure.
    pub fn set_error(&mut self, cx: &mut Context<Self>) {
        self.is_streaming = false;
        self.loading_state = GenerativeLoadingState::Error;
        cx.notify();
    }

    /// Reset the loading state to [`GenerativeLoadingState::Idle`]. Use
    /// after a `Done` / `Error` transition has been shown and the UI
    /// should return to its neutral rest state.
    pub fn clear_loading_state(&mut self, cx: &mut Context<Self>) {
        self.loading_state = GenerativeLoadingState::Idle;
        cx.notify();
    }

    /// Current [`GenerativeLoadingState`] — drives which
    /// `ActivityIndicator` / terminal-state glyph the host should
    /// render. Finding 29 in the Zed cross-reference audit.
    pub fn loading_state(&self) -> GenerativeLoadingState {
        self.loading_state
    }

    /// Get the raw source text.
    pub fn source(&self) -> &str {
        self.parser.source()
    }

    /// Returns whether an incomplete code fence is detected.
    pub fn has_incomplete_code_fence(&self) -> bool {
        self.parser.has_incomplete_code_fence()
    }

    /// Returns the detected text direction.
    pub fn text_direction(&self) -> remend::TextDirection {
        self.parser.text_direction()
    }

    fn render_blocks(&mut self, cx: &App) -> Vec<AnyElement> {
        let blocks = self.parser.parse();
        let theme = cx.theme();
        let citation_ctx = &self.citation_ctx;
        let id_counter = &self.citation_id_counter;
        let security = &self.security;
        let popovers = &self.citation_popovers;
        // Clear the rendered set so each render pass starts fresh.
        self.rendered_popovers.borrow_mut().clear();
        // Reset the selection coordinator's per-frame paragraph
        // registry. Selection anchor/focus/pending persist across
        // frames; only the registration order is refreshed.
        self.selection.begin_frame();

        let ctx = RenderCtx {
            theme,
            citation_ctx,
            id_counter,
            security,
            popovers,
            rendered_popovers: &self.rendered_popovers,
            selection: &self.selection,
        };

        blocks
            .iter()
            .map(|block| render_block(block, &ctx))
            .collect()
    }
}

impl Render for StreamingMarkdown {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let elements = self.render_blocks(cx);
        let is_streaming = self.is_streaming;
        let theme = cx.theme();
        let caret_color = self.settings.caret_color.unwrap_or(theme.accent);
        // Reduce Motion: prefer the per-settings override when a host has
        // set it explicitly; otherwise fall back to the theme flag so the
        // caret honours the user's system-wide accessibility preference
        // without requiring every caller to forward it manually.
        let reduce_motion =
            self.settings.reduce_motion || theme.accessibility_mode.reduce_motion();
        let caret_kind = self.settings.caret.unwrap_or(CaretKind::Block);
        let show_caret = is_streaming && self.settings.caret.is_some();
        // HIG Motion `foundations.md:1100`: under Reduce Motion, steady
        // opacity beats continuous blink. `render_caret` treats a zero
        // interval as "always visible", so passing `Duration::ZERO`
        // keeps the insertion point on-screen without animation.
        let blink_interval = if reduce_motion {
            std::time::Duration::ZERO
        } else {
            self.settings.caret_blink_interval
        };
        let line_height = TextStyle::Body.attrs().leading;

        // HIG Text views §multi-line styled content: readable markdown
        // surfaces should advertise an I-beam cursor so the user knows
        // the text is text (even though full drag-selection is not yet
        // available — see the F17 note at the top of the module).
        // Zed's markdown element applies this via its custom `Element`
        // impl; for the builder-style path we at least swap the cursor
        // on hover.
        div()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .cursor_text()
            .children(elements)
            .when(show_caret, |el| {
                el.child(render_caret(
                    caret_kind,
                    caret_color,
                    Instant::now(),
                    blink_interval,
                    line_height,
                ))
            })
    }
}

/// Shared render context passed through all markdown rendering functions.
pub struct RenderCtx<'a> {
    pub theme: &'a TahoeTheme,
    pub citation_ctx: &'a CitationContext,
    pub id_counter: &'a Cell<usize>,
    pub security: &'a MarkdownSecurity,
    pub popovers: &'a HashMap<usize, Entity<CitationPopover>>,
    /// Tracks which citation numbers have been rendered via their popover entity
    /// in the current pass. Prevents cloning the same Entity twice, which would
    /// produce duplicate element IDs and shared hover/carousel state.
    pub rendered_popovers: &'a RefCell<HashSet<usize>>,
    /// Shared selection coordinator — every paragraph rendered in
    /// this pass clones this handle into its [`SelectableText`] so
    /// drag-select, multi-click gestures, and Cmd+C copy can span
    /// across paragraphs.
    pub selection: &'a MarkdownSelection,
}

/// Render a single markdown block as a GPUI element.
///
/// Callers that do not track nesting depth should use the top-level value
/// of `0`. List items recurse through [`render_block_at_depth`] so deeply
/// nested structures are capped at [`LIST_MAX_DEPTH`] levels of indent
/// rather than overflowing their container.
pub fn render_block(block: &MarkdownBlock, ctx: &RenderCtx) -> AnyElement {
    render_block_at_depth(block, ctx, 0)
}

/// Depth-aware variant of [`render_block`]. `depth` counts list-nesting
/// levels; every other block type ignores the value.
pub fn render_block_at_depth(block: &MarkdownBlock, ctx: &RenderCtx, depth: usize) -> AnyElement {
    match block {
        MarkdownBlock::Paragraph(inlines) => div()
            .text_style(TextStyle::Body, ctx.theme)
            .text_color(ctx.theme.text)
            // HIG Text views: body paragraphs get block-level spacing
            // equal to the line height so adjacent paragraphs read as
            // discrete blocks rather than a single wrapped run. The
            // parent `flex.flex_col().gap()` provides a baseline; the
            // additional `mb` matches Apple's typography recommendation.
            .mb(ctx.theme.spacing_sm)
            .child(render_inlines(inlines, ctx))
            .into_any_element(),
        MarkdownBlock::Heading { level, content } => {
            // macOS HIG type ramp below Title 3 has four distinct weights
            // (Headline 13pt Bold, Body 13pt Regular, Callout 12pt,
            // Subheadline 11pt). Mapping h4–h6 to separate styles
            // preserves visual hierarchy at sub-h3 depth. Apply the
            // "emphasized" (semibold/bold/heavy) weight per HIG so the
            // heading reads as heavier than surrounding body text.
            let ts = match level {
                1 => TextStyle::Title1,
                2 => TextStyle::Title2,
                3 => TextStyle::Title3,
                4 => TextStyle::Headline,
                5 => TextStyle::Body,
                _ => TextStyle::Subheadline,
            };

            div()
                .text_style_emphasized(ts, ctx.theme)
                .text_color(ctx.theme.text)
                .mb(ctx.theme.spacing_sm)
                .child(render_inlines(content, ctx))
                .into_any_element()
        }
        MarkdownBlock::CodeBlock { language, code } => {
            if language.as_deref() == Some("mermaid") {
                return MermaidBlock::new(code.clone()).into_any_element();
            }
            CodeBlockView::new(code.clone())
                .language(language.clone())
                .into_any_element()
        }
        MarkdownBlock::List {
            ordered,
            start,
            items,
        } => {
            let start_num = start.unwrap_or(1) as usize;
            // HIG Layout convention for list indentation is 20–28 pt —
            // enough room to visually clear the bullet / number marker.
            // At depth 0 we indent 24pt; each additional level adds a
            // reduced delta so nested lists remain readable in narrow
            // containers. The effective indent is clamped at depth 4
            // to prevent runaway indentation from deeply nested
            // AI-generated output.
            let effective_depth = depth.min(LIST_MAX_DEPTH);
            let base_indent = if depth == 0 { 24.0 } else { 20.0 };
            let list_indent = px(base_indent + (effective_depth.saturating_sub(1)) as f32 * 20.0);

            div()
                .flex()
                .flex_col()
                .gap(ctx.theme.spacing_xs)
                .pl(list_indent)
                .mb(ctx.theme.spacing_sm)
                .children(items.iter().enumerate().map(|(i, item_blocks)| {
                    let marker = if *ordered {
                        format!("{}. ", start_num + i)
                    } else {
                        "\u{2022} ".to_string()
                    };

                    div()
                        .flex()
                        .flex_row()
                        .w_full()
                        .child(
                            div()
                                .text_style(TextStyle::Body, ctx.theme)
                                .text_color(ctx.theme.text_muted)
                                .w(ctx.theme.spacing_lg)
                                .flex_shrink_0()
                                .child(marker),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.0))
                                .flex()
                                .flex_col()
                                .gap(ctx.theme.spacing_xs)
                                .children(item_blocks.iter().map(|b| {
                                    render_block_at_depth(b, ctx, depth + 1)
                                })),
                        )
                }))
                .into_any_element()
        }
        MarkdownBlock::BlockQuote(blocks) => div()
            // HIG Color / Materials: prefer a semi-transparent muted
            // border over the opaque neutral `theme.border` for
            // decorative indicators. The 2pt rule matches NSTextView
            // quote styling more closely than the prior 4pt slab.
            .border_l_2()
            .border_color(ctx.theme.text_muted)
            .pl(ctx.theme.spacing_md)
            .text_color(ctx.theme.text_muted)
            .mb(ctx.theme.spacing_sm)
            .flex()
            .flex_col()
            .gap(ctx.theme.spacing_xs)
            .children(blocks.iter().map(|b| render_block_at_depth(b, ctx, depth)))
            .into_any_element(),
        MarkdownBlock::ThematicBreak => div()
            .w_full()
            .h(px(1.0))
            // HIG Color: horizontal rules should use the semi-transparent
            // `separatorColor` that adapts to the underlying surface,
            // not the opaque `theme.border` neutral gray.
            .bg(ctx.theme.separator_color())
            .my(ctx.theme.spacing_md)
            .into_any_element(),
        MarkdownBlock::DisplayMath(math) => div()
            .w_full()
            .flex()
            .justify_center()
            .py(ctx.theme.spacing_sm)
            .child(
                div()
                    .px(ctx.theme.spacing_md)
                    .py(ctx.theme.spacing_sm)
                    .bg(ctx.theme.code_bg)
                    .rounded(ctx.theme.radius_md)
                    .text_style(TextStyle::Body, ctx.theme)
                    .child(
                        StyledText::new(SharedString::from(math.clone())).with_highlights(vec![(
                            0..math.len(),
                            HighlightStyle {
                                color: Some(ctx.theme.text),
                                ..Default::default()
                            },
                        )]),
                    ),
            )
            .into_any_element(),
        MarkdownBlock::Table {
            headers,
            rows,
            alignments,
        } => {
            let align_at = |idx: usize| alignments.get(idx).copied().unwrap_or(TableAlignment::None);
            div()
                .flex()
                .flex_col()
                .border_1()
                .border_color(ctx.theme.border)
                .rounded(ctx.theme.radius_md)
                .overflow_hidden()
                .mb(ctx.theme.spacing_sm)
                .child(
                    div()
                        .flex()
                        .bg(ctx.theme.surface)
                        .font_weight(ctx.theme.effective_weight(FontWeight::SEMIBOLD))
                        .children(headers.iter().enumerate().map(|(col, cell)| {
                            apply_table_alignment(
                                div()
                                    .flex_1()
                                    .px(ctx.theme.spacing_sm)
                                    .py(ctx.theme.spacing_xs)
                                    .border_r_1()
                                    .border_color(ctx.theme.border)
                                    .text_style(TextStyle::Subheadline, ctx.theme),
                                align_at(col),
                            )
                            .child(render_inlines(cell, ctx))
                        })),
                )
                .children(rows.iter().enumerate().map(|(row_idx, row)| {
                    // HIG table convention: alternating row background
                    // improves readability for multi-column data. Even
                    // rows (0-indexed) keep the default background;
                    // odd rows get a subtle tint.
                    let zebra_bg = if row_idx % 2 == 1 {
                        Some(ctx.theme.surface)
                    } else {
                        None
                    };
                    let mut row_el = div()
                        .flex()
                        .border_t_1()
                        .border_color(ctx.theme.border);
                    if let Some(bg) = zebra_bg {
                        row_el = row_el.bg(bg);
                    }
                    row_el.children(row.iter().enumerate().map(|(col, cell)| {
                        apply_table_alignment(
                            div()
                                .flex_1()
                                .px(ctx.theme.spacing_sm)
                                .py(ctx.theme.spacing_xs)
                                .border_r_1()
                                .border_color(ctx.theme.border)
                                .text_style(TextStyle::Subheadline, ctx.theme),
                            align_at(col),
                        )
                        .child(render_inlines(cell, ctx))
                    }))
                }))
                .into_any_element()
        }
        MarkdownBlock::TaskItem { checked, content } => {
            // GFM task list item rendered outside of a list (rare, but
            // the parser allows top-level task items when streaming is
            // mid-document). Show the checkbox glyph followed by the
            // content; inside a list the parent `List` render handles
            // indentation.
            let glyph = if *checked { "\u{2611}" } else { "\u{2610}" };
            div()
                .flex()
                .flex_row()
                .gap(ctx.theme.spacing_xs)
                .text_style(TextStyle::Body, ctx.theme)
                .text_color(ctx.theme.text)
                .mb(ctx.theme.spacing_sm)
                .child(
                    div()
                        .flex_shrink_0()
                        .text_color(if *checked {
                            ctx.theme.success
                        } else {
                            ctx.theme.text_muted
                        })
                        .child(glyph),
                )
                .child(render_inlines(content, ctx))
                .into_any_element()
        }
    }
}

/// Apply GFM column alignment to a table cell via flex-axis positioning.
///
/// GPUI `v0.231.1-pre` does not expose a `.text_align()` style, so per-column
/// GFM alignment is approximated by positioning the cell content on the
/// flex main axis. Single-line cells track the intended alignment; wrapped
/// cells still justify on the leading edge. When GPUI lands `text_align`
/// this helper should be replaced with a direct style call so multi-line
/// cells also respect alignment.
fn apply_table_alignment(el: gpui::Div, alignment: TableAlignment) -> gpui::Div {
    match alignment {
        TableAlignment::None | TableAlignment::Left => el.flex().justify_start(),
        TableAlignment::Center => el.flex().justify_center(),
        TableAlignment::Right => el.flex().justify_end(),
    }
}

/// Returns `true` when any inline requires the mixed-element path —
/// citations (custom popover entities), inline images (actual URL
/// embeds), or top-level task markers inside bold/italic blocks.
///
/// Inline code, links, and bold/italic/strikethrough all render via the
/// flat `StyledText::with_runs` path now that Zed's pattern for per-run
/// `Font` and `InteractiveText` for click handlers is in use. Only the
/// truly element-level content (citations, images) needs to break out
/// of the single-text-layout path.
fn has_complex_inlines(inlines: &[InlineContent]) -> bool {
    inlines.iter().any(|inline| match inline {
        InlineContent::Citation(_) => true,
        // Only images with a real URL force the mixed path; alt-only
        // placeholders render inline as italic muted text via the flat
        // path and don't need a dedicated element.
        InlineContent::Image { url, .. } => !url.is_empty(),
        InlineContent::Bold(inner)
        | InlineContent::Italic(inner)
        | InlineContent::Strikethrough(inner)
        | InlineContent::Link { content: inner, .. } => has_complex_inlines(inner),
        _ => false,
    })
}

/// Per-role text styles for the markdown flat path. Each field is a
/// full [`GpuiTextStyle`] used via [`GpuiTextStyle::to_run`] to produce
/// the `TextRun` for the matching inline kind. The styles are built from
/// the active [`TahoeTheme`] so all runs share the ambient color scheme
/// and font family (with `code` overriding to `theme.font_mono`).
struct InlineTextStyles {
    base: GpuiTextStyle,
    code_family: SharedString,
    code_bg: gpui::Hsla,
    link_color: gpui::Hsla,
    link_underline: UnderlineStyle,
    strikethrough: StrikethroughStyle,
    strikethrough_color: gpui::Hsla,
    muted_color: gpui::Hsla,
    accent_color: gpui::Hsla,
}

impl InlineTextStyles {
    fn from_theme(theme: &TahoeTheme) -> Self {
        // The base run inherits the ambient font metrics (size / leading
        // are provided by the parent div's `.text_style(...)`). We only
        // populate fields that flow into `TextRun`: family, weight,
        // style, color, background, underline, strikethrough.
        let base = GpuiTextStyle {
            color: theme.text,
            font_family: theme.font_sans.clone(),
            ..Default::default()
        };
        Self {
            base,
            code_family: theme.font_mono.clone(),
            code_bg: theme.code_bg,
            link_color: theme.accent,
            link_underline: UnderlineStyle {
                thickness: px(1.0),
                color: Some(theme.accent),
                wavy: false,
            },
            strikethrough: StrikethroughStyle {
                thickness: px(1.0),
                color: Some(theme.text_muted),
            },
            strikethrough_color: theme.text_muted,
            muted_color: theme.text_muted,
            accent_color: theme.accent,
        }
    }
}

/// Builder state threaded through the recursive inline walker. Tracks
/// the text buffer, TextRun output, and per-link ranges / URLs that
/// `InteractiveText` will consume for click dispatch.
struct InlineRuns {
    text: String,
    runs: Vec<TextRun>,
    link_ranges: Vec<Range<usize>>,
    link_urls: Vec<String>,
}

impl InlineRuns {
    fn new() -> Self {
        Self {
            text: String::new(),
            runs: Vec::new(),
            link_ranges: Vec::new(),
            link_urls: Vec::new(),
        }
    }

    fn push(&mut self, s: &str, style: &GpuiTextStyle) {
        if s.is_empty() {
            return;
        }
        self.text.push_str(s);
        self.runs.push(style.to_run(s.len()));
    }
}

/// Render inlines, choosing the mixed-element path only for citations
/// and inline images. The common case (text / code / bold / italic /
/// strikethrough / link) renders as a single `StyledText::with_runs`
/// layout — one text object, proper line wrapping and baseline
/// alignment — and is wrapped in `InteractiveText` when links are
/// present so `cx.open_url` fires on click.
fn render_inlines(inlines: &[InlineContent], ctx: &RenderCtx) -> AnyElement {
    if has_complex_inlines(inlines) {
        render_inlines_mixed(inlines, ctx)
    } else {
        render_inlines_flat(inlines, ctx).into_any_element()
    }
}

/// Flat path: build a single `StyledText` with per-run `TextRun`
/// entries, then wrap in [`SelectableText`]. The wrapper paints a
/// selection background, handles drag-select / link click / Cmd+C
/// copy, and — crucially — preserves the single-text-layout fast
/// path. One element per inline run; line wrapping and baselines
/// stay native.
fn render_inlines_flat(inlines: &[InlineContent], ctx: &RenderCtx) -> AnyElement {
    let styles = InlineTextStyles::from_theme(ctx.theme);
    let mut builder = InlineRuns::new();
    flatten_inlines_to_runs(inlines, &styles, ctx.security, &styles.base, &mut builder);

    let text: SharedString = SharedString::from(builder.text);
    let styled = StyledText::new(text.clone()).with_runs(builder.runs);
    let id = ctx.id_counter.get();
    ctx.id_counter.set(id + 1);
    let urls: Vec<SharedString> = builder.link_urls.into_iter().map(SharedString::from).collect();
    // macOS NSTextView selection tint: accent color at ~25% alpha.
    // HIG: selection highlights use the system accent hue.
    let selection_bg = {
        let mut bg = ctx.theme.accent;
        bg.a = 0.28;
        bg
    };
    SelectableText::new(
        ElementId::Name(format!("md-inlines-{id}").into()),
        text,
        styled,
        selection_bg,
        ctx.selection.clone(),
    )
    .with_links(builder.link_ranges, urls)
    .into_any_element()
}

/// Recursively walk inline content, emitting text + `TextRun`s via a
/// style stack. Nested bold/italic/strikethrough refine the current
/// style before recursing, so combinations (bold-italic, bold-link,
/// italic-code) compose correctly.
fn flatten_inlines_to_runs(
    inlines: &[InlineContent],
    styles: &InlineTextStyles,
    security: &MarkdownSecurity,
    current: &GpuiTextStyle,
    out: &mut InlineRuns,
) {
    for inline in inlines {
        match inline {
            InlineContent::Text(t) => out.push(t, current),
            InlineContent::Code(code) => {
                // HIG Text views §code spans: code must render in the
                // system monospaced font. Refining the current style
                // with `font_family = font_mono` + `background_color
                // = code_bg` preserves bold/italic context that an
                // enclosing emphasis already set.
                let mut code_style = current.clone();
                code_style.font_family = styles.code_family.clone();
                code_style.background_color = Some(styles.code_bg);
                out.push(code, &code_style);
            }
            InlineContent::Bold(inner) => {
                let mut bold_style = current.clone();
                bold_style.font_weight = FontWeight::BOLD;
                flatten_inlines_to_runs(inner, styles, security, &bold_style, out);
            }
            InlineContent::Italic(inner) => {
                let mut italic_style = current.clone();
                italic_style.font_style = FontStyle::Italic;
                flatten_inlines_to_runs(inner, styles, security, &italic_style, out);
            }
            InlineContent::Strikethrough(inner) => {
                let mut strike_style = current.clone();
                strike_style.strikethrough = Some(styles.strikethrough);
                strike_style.color = styles.strikethrough_color;
                flatten_inlines_to_runs(inner, styles, security, &strike_style, out);
            }
            InlineContent::Link { content, url } => {
                let start = out.text.len();
                let resolved = security.resolve_url(url).into_owned();
                let allowed = security.is_link_allowed(&resolved);
                if allowed {
                    let mut link_style = current.clone();
                    link_style.color = styles.link_color;
                    link_style.underline = Some(styles.link_underline);
                    flatten_inlines_to_runs(content, styles, security, &link_style, out);
                    let end = out.text.len();
                    if end > start {
                        out.link_ranges.push(start..end);
                        out.link_urls.push(resolved);
                    }
                } else {
                    // Allowlist denied: render the label as plain text
                    // so readers still see the surrounding context, but
                    // do not mark it clickable.
                    flatten_inlines_to_runs(content, styles, security, current, out);
                }
            }
            InlineContent::Citation(n) => {
                // In the flat path we render a plain `[N]` marker. The
                // mixed-element path swaps in the interactive
                // `InlineCitation` popover when citations are present.
                let mut citation_style = current.clone();
                citation_style.color = styles.accent_color;
                citation_style.font_weight = FontWeight::SEMIBOLD;
                let marker = format!("[{n}]");
                out.push(&marker, &citation_style);
            }
            InlineContent::Image { alt, .. } => {
                if !alt.is_empty() {
                    let mut alt_style = current.clone();
                    alt_style.color = styles.muted_color;
                    alt_style.font_style = FontStyle::Italic;
                    out.push(alt, &alt_style);
                }
            }
            InlineContent::InlineMath(math) => {
                // Inline math shares the code span's monospaced face
                // but keeps the ambient color. This matches the
                // conventional TeX rendering where inline math is a
                // monospaced tint, not a separate typographic tone.
                let mut math_style = current.clone();
                math_style.font_family = styles.code_family.clone();
                math_style.background_color = Some(styles.code_bg);
                out.push(math, &math_style);
            }
            InlineContent::SoftBreak => out.push(" ", current),
            InlineContent::HardBreak => out.push("\n", current),
            InlineContent::TaskMarker(checked) => {
                let glyph = if *checked { "\u{2611} " } else { "\u{2610} " };
                let mut marker_style = current.clone();
                marker_style.color = if *checked {
                    gpui::hsla(140.0 / 360.0, 0.55, 0.45, 1.0)
                } else {
                    styles.muted_color
                };
                out.push(glyph, &marker_style);
            }
        }
    }
}

/// Render inlines as a flex-wrap container of mixed text segments and
/// per-element wrappers for citations / images. Used only when the
/// inline tree contains element-level content that cannot live inside
/// a single `StyledText` layout. Segments between complex children use
/// the TextRun-based flat path so code, links, and emphasis still
/// render with proper font handling.
fn render_inlines_mixed(inlines: &[InlineContent], ctx: &RenderCtx) -> AnyElement {
    let mut children: Vec<AnyElement> = Vec::new();
    let mut segment: Vec<InlineContent> = Vec::new();

    fn flush_segment(
        segment: &mut Vec<InlineContent>,
        ctx: &RenderCtx,
        children: &mut Vec<AnyElement>,
    ) {
        if !segment.is_empty() {
            children.push(render_inlines_flat(segment, ctx));
            segment.clear();
        }
    }

    fn walk_inlines(
        inlines: &[InlineContent],
        ctx: &RenderCtx,
        segment: &mut Vec<InlineContent>,
        children: &mut Vec<AnyElement>,
    ) {
        for inline in inlines {
            match inline {
                InlineContent::Citation(n) => {
                    flush_segment(segment, ctx, children);
                    let already_rendered = !ctx.rendered_popovers.borrow_mut().insert(*n);
                    if !already_rendered && let Some(popover) = ctx.popovers.get(n) {
                        children.push(popover.clone().into_any_element());
                        continue;
                    }
                    let id = ctx.id_counter.get();
                    ctx.id_counter.set(id + 1);
                    let mut citation =
                        InlineCitation::new(ElementId::Name(format!("cite-{n}-{id}").into()), *n);
                    if let Some(sources) = ctx.citation_ctx.sources.get(n)
                        && let Some(source) = sources.first()
                    {
                        citation = citation.source(source.clone());
                    }
                    children.push(citation.into_any_element());
                }
                InlineContent::Image { url, alt } => {
                    flush_segment(segment, ctx, children);
                    let resolved_url = ctx.security.resolve_url(url);
                    if !url.is_empty() && ctx.security.is_image_allowed(&resolved_url) {
                        children.push(
                            img(SharedUri::from(resolved_url.into_owned()))
                                .max_w(px(400.0))
                                .rounded(ctx.theme.radius_md)
                                .object_fit(ObjectFit::ScaleDown)
                                .into_any_element(),
                        );
                    } else if !alt.is_empty() {
                        let styled = StyledText::new(SharedString::from(alt.clone()))
                            .with_highlights(vec![(
                                0..alt.len(),
                                HighlightStyle {
                                    color: Some(ctx.theme.text_muted),
                                    font_style: Some(FontStyle::Italic),
                                    ..Default::default()
                                },
                            )]);
                        children.push(styled.into_any_element());
                    }
                }
                InlineContent::Bold(inner)
                | InlineContent::Italic(inner)
                | InlineContent::Strikethrough(inner)
                | InlineContent::Link { content: inner, .. }
                    if has_complex_inlines(inner) =>
                {
                    flush_segment(segment, ctx, children);
                    walk_inlines(inner, ctx, segment, children);
                    flush_segment(segment, ctx, children);
                }
                other => {
                    segment.push(other.clone());
                }
            }
        }
    }

    walk_inlines(inlines, ctx, &mut segment, &mut children);
    flush_segment(&mut segment, ctx, &mut children);

    div()
        .flex()
        .flex_wrap()
        .items_end()
        .children(children)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::has_complex_inlines;
    use super::{InlineContent, MarkdownSecurity};
    use core::prelude::v1::test;

    #[test]
    fn plain_text_is_not_complex() {
        let inlines = vec![
            InlineContent::Text("hello ".into()),
            InlineContent::Bold(vec![InlineContent::Text("world".into())]),
        ];
        assert!(!has_complex_inlines(&inlines));
    }

    #[test]
    fn inline_code_stays_in_flat_path() {
        // After adopting Zed's `StyledText::with_runs` pattern, inline
        // code no longer forces the mixed-element path — it becomes a
        // TextRun with `font_family = theme.font_mono`, preserving the
        // single-text-layout fast path.
        let inlines = vec![
            InlineContent::Text("run ".into()),
            InlineContent::Code("cargo test".into()),
        ];
        assert!(!has_complex_inlines(&inlines));
    }

    #[test]
    fn link_stays_in_flat_path() {
        // Links are now TextRuns wrapped in `InteractiveText` — click
        // handlers fire without breaking the flat text layout.
        let inlines = vec![InlineContent::Link {
            url: "https://example.com".into(),
            content: vec![InlineContent::Text("here".into())],
        }];
        assert!(!has_complex_inlines(&inlines));
    }

    #[test]
    fn citation_forces_mixed_path() {
        // Citations still require a separate element — the
        // `InlineCitation` popover entity has its own mouse handling
        // that can't live inside a `StyledText`.
        let inlines = vec![
            InlineContent::Text("see ".into()),
            InlineContent::Citation(1),
        ];
        assert!(has_complex_inlines(&inlines));
    }

    #[test]
    fn inline_image_with_url_forces_mixed_path() {
        let inlines = vec![InlineContent::Image {
            url: "https://example.com/a.png".into(),
            alt: "alt".into(),
        }];
        assert!(has_complex_inlines(&inlines));
    }

    #[test]
    fn inline_image_without_url_stays_flat() {
        // Alt-only images (denied by allowlist or empty URL) render as
        // italic muted text in the flat path — no element needed.
        let inlines = vec![InlineContent::Image {
            url: String::new(),
            alt: "missing".into(),
        }];
        assert!(!has_complex_inlines(&inlines));
    }

    #[test]
    fn citation_nested_in_link_still_complex() {
        let inlines = vec![InlineContent::Link {
            url: "https://example.com".into(),
            content: vec![
                InlineContent::Text("see ".into()),
                InlineContent::Citation(1),
            ],
        }];
        assert!(has_complex_inlines(&inlines));
    }

    #[test]
    fn resolve_url_no_origin_returns_borrowed() {
        let sec = MarkdownSecurity::default();
        let result = sec.resolve_url("foo/bar.png");
        assert_eq!(&*result, "foo/bar.png");
        assert!(matches!(result, std::borrow::Cow::Borrowed(_)));
    }

    #[test]
    fn resolve_url_absolute_http_unchanged() {
        let sec = MarkdownSecurity {
            default_origin: Some("https://cdn.example.com".into()),
            ..Default::default()
        };
        assert_eq!(
            &*sec.resolve_url("https://other.com/img.png"),
            "https://other.com/img.png"
        );
        assert_eq!(
            &*sec.resolve_url("http://other.com/img.png"),
            "http://other.com/img.png"
        );
    }

    #[test]
    fn resolve_url_protocol_relative_unchanged() {
        let sec = MarkdownSecurity {
            default_origin: Some("https://cdn.example.com".into()),
            ..Default::default()
        };
        assert_eq!(
            &*sec.resolve_url("//cdn.example.com/img.png"),
            "//cdn.example.com/img.png"
        );
    }

    #[test]
    fn resolve_url_fragment_unchanged() {
        let sec = MarkdownSecurity {
            default_origin: Some("https://cdn.example.com".into()),
            ..Default::default()
        };
        assert_eq!(&*sec.resolve_url("#section"), "#section");
    }

    #[test]
    fn resolve_url_absolute_path_with_origin() {
        let sec = MarkdownSecurity {
            default_origin: Some("https://cdn.example.com".into()),
            ..Default::default()
        };
        assert_eq!(
            &*sec.resolve_url("/images/foo.png"),
            "https://cdn.example.com/images/foo.png"
        );
    }

    #[test]
    fn resolve_url_relative_path_with_origin() {
        let sec = MarkdownSecurity {
            default_origin: Some("https://cdn.example.com".into()),
            ..Default::default()
        };
        assert_eq!(
            &*sec.resolve_url("images/foo.png"),
            "https://cdn.example.com/images/foo.png"
        );
    }

    #[test]
    fn resolve_url_origin_trailing_slash_no_double() {
        let sec = MarkdownSecurity {
            default_origin: Some("https://cdn.example.com/".into()),
            ..Default::default()
        };
        assert_eq!(
            &*sec.resolve_url("/img.png"),
            "https://cdn.example.com/img.png"
        );
    }

    #[test]
    fn resolve_url_dangerous_schemes_unchanged() {
        let sec = MarkdownSecurity {
            default_origin: Some("https://cdn.example.com".into()),
            ..Default::default()
        };
        // These should be returned as-is, not treated as relative paths.
        assert_eq!(
            &*sec.resolve_url("javascript:alert(1)"),
            "javascript:alert(1)"
        );
        assert_eq!(
            &*sec.resolve_url("data:text/html,<h1>Hi</h1>"),
            "data:text/html,<h1>Hi</h1>"
        );
        assert_eq!(
            &*sec.resolve_url("mailto:user@example.com"),
            "mailto:user@example.com"
        );
        assert_eq!(&*sec.resolve_url("tel:+1234567890"), "tel:+1234567890");
    }
}
