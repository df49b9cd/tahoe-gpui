//! Terminal output display component with compound sub-components.
//!
//! Provides a terminal view with ANSI color support, virtualized line rendering,
//! streaming indicators, and copy/clear actions. Supports both convenience API
//! and compound composition.
//!
//! # Convenience (default render)
//! ```ignore
//! let terminal = cx.new(|cx| {
//!     let mut t = TerminalView::new(cx);
//!     t.set_title("Build output", cx);
//!     t.push_output("\x1b[32mSuccess\x1b[0m\n", cx);
//!     t.set_streaming(true, cx);
//!     t
//! });
//! ```
//!
//! # Compound composition (custom layout)
//! ```ignore
//! terminal.update(cx, |t, cx| {
//!     let lines = t.parsed_lines(cx);
//!     let content = TerminalContent::new(t.list_id(), lines, t.scroll_handle());
//!     let header = TerminalHeader::new()
//!         .border(true)
//!         .child(TerminalTitle::new("Custom"))
//!         .child(
//!             TerminalActions::new()
//!                 .child(TerminalStatus::new("my-status", true))
//!                 .child(t.copy_button()),
//!         );
//!     // ... compose as needed
//! });
//! ```

use super::ansi_parser;
use crate::callback_types::{OnClick, OnMutCallbackArc};
use crate::components::layout_and_organization::FlexHeader;
use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::components::status::shimmer::Shimmer;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::SPACING_4;
use crate::foundations::theme::{ActiveTheme, AnsiColors, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use crate::markdown::caret::{CaretKind, render_caret};
use crate::markdown::selectable_text::SelectableText;
use crate::markdown::selection::MarkdownSelection;
use gpui::prelude::*;
use gpui::{
    App, ElementId, Entity, Font, FontStyle, FontWeight, Hsla, Pixels, SharedString,
    StrikethroughStyle, StyledText, TextRun, UnderlineStyle, UniformListScrollHandle, Window, div,
    px, uniform_list,
};
use std::ops::Range;
use std::sync::Arc;
use std::time::{Duration, Instant};

// -- TerminalHeader -----------------------------------------------------------

/// Header wrapper for the terminal. Flex row with border-bottom.
///
/// Type alias for [`FlexHeader`] with border enabled at the call site.
pub type TerminalHeader = FlexHeader;

// -- TerminalTitle ------------------------------------------------------------

/// Title element with an optional icon (defaults to Terminal icon).
#[derive(IntoElement)]
pub struct TerminalTitle {
    title: SharedString,
    icon: IconName,
}

impl TerminalTitle {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            icon: IconName::Terminal,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = icon;
        self
    }
}

impl RenderOnce for TerminalTitle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .child(
                Icon::new(self.icon)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child(self.title),
            )
    }
}

// -- TerminalStatus -----------------------------------------------------------

/// Streaming status indicator. Shows a shimmer cursor when streaming.
#[derive(IntoElement)]
pub struct TerminalStatus {
    element_id: ElementId,
    is_streaming: bool,
}

impl TerminalStatus {
    pub fn new(id: impl Into<ElementId>, is_streaming: bool) -> Self {
        Self {
            element_id: id.into(),
            is_streaming,
        }
    }
}

impl RenderOnce for TerminalStatus {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        if self.is_streaming {
            div().child(Shimmer::new(self.element_id).label("\u{2588}"))
        } else {
            div()
        }
    }
}

// -- TerminalActions ----------------------------------------------------------

/// Actions container for terminal header. Flex row with gap.
///
/// Type alias for [`crate::components::layout_and_organization::FlexActions`]
/// — a horizontal flex row with gap spacing.
pub type TerminalActions = crate::components::layout_and_organization::FlexActions;

// -- TerminalContent ----------------------------------------------------------

/// Virtualized terminal content area. Renders parsed ANSI lines via uniform_list.
#[derive(IntoElement)]
pub struct TerminalContent {
    list_id: ElementId,
    lines: Arc<Vec<Vec<ansi_parser::AnsiSpan>>>,
    scroll_handle: UniformListScrollHandle,
    max_height: Pixels,
    text_style: TextStyle,
    bold_is_bright: bool,
    /// Shared cross-line selection coordinator. When `None`, lines render
    /// without drag-select / Cmd-C / link-click support (the pre-selection
    /// behaviour).
    selection: Option<MarkdownSelection>,
    /// When `true`, a caret is rendered at the end of the last line —
    /// mirrors Terminal.app's "insertion point during output" affordance
    /// (finding #3 / the HIG Code-surface audit). The caret is static rather
    /// than blinking because the terminal only re-renders on
    /// `push_output`; a proper blink would require an animation driver.
    streaming: bool,
}

impl TerminalContent {
    pub fn new(
        list_id: ElementId,
        lines: Arc<Vec<Vec<ansi_parser::AnsiSpan>>>,
        scroll_handle: UniformListScrollHandle,
    ) -> Self {
        Self {
            list_id,
            lines,
            scroll_handle,
            max_height: px(384.0),
            // HIG Typography (macOS): default body size is 13 pt. Terminal.app
            // ships 13 pt Menlo; Xcode console uses 12 pt SF Mono. 11 pt was
            // below the conventional default and hurt multi-line scanning.
            text_style: TextStyle::Body,
            // Most CLIs (cargo, git, npm) emit bold+standard colour expecting
            // the bright variant. Terminal.app and Zed default this on.
            bold_is_bright: true,
            selection: None,
            streaming: false,
        }
    }

    pub fn max_height(mut self, height: Pixels) -> Self {
        self.max_height = height;
        self
    }

    /// Override the [`TextStyle`] used for terminal content. Defaults to
    /// [`TextStyle::Body`] (13 pt on macOS) to match Terminal.app / Xcode
    /// console conventions.
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self
    }

    /// Toggle bold-is-bright rendering. When enabled (the default), bold
    /// text using a standard 8-colour foreground is upgraded to the bright
    /// palette — matching Terminal.app, iTerm2, and Zed. Disable to render
    /// bold and non-bold standard colours identically.
    pub fn bold_is_bright(mut self, enabled: bool) -> Self {
        self.bold_is_bright = enabled;
        self
    }

    /// Attach a shared selection coordinator so lines participate in
    /// mouse-drag selection, Cmd/Ctrl+C copy, and OSC 8 link click.
    /// Clone the same [`MarkdownSelection`] into every frame to preserve
    /// selection state across renders. Without this, each line renders
    /// as plain styled text (no selection).
    pub fn selection(mut self, selection: MarkdownSelection) -> Self {
        self.selection = Some(selection);
        self
    }

    /// Mark the content as currently streaming so a caret appears at the
    /// end of the last line. HIG §Status: a visible insertion point signals
    /// ongoing activity and the current output position.
    pub fn streaming(mut self, streaming: bool) -> Self {
        self.streaming = streaming;
        self
    }
}

impl RenderOnce for TerminalContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let text_color = theme.text;
        let terminal_bg = theme.terminal_bg;
        let link_color = theme.accent;
        let selection_bg = theme.selected_bg;
        // Prototype `Font` carrying the theme's mono family + fallback list
        // so every ANSI span inherits fallbacks (finding #29).
        // Cloning `FontFallbacks` only bumps an `Arc`, so per-span clones are cheap.
        let font_mono = theme.font_mono();
        let font_mono_outer = font_mono.clone();
        let attrs = self.text_style.attrs();
        let font_size = attrs.size;
        let line_height = attrs.leading;
        let spacing_md = theme.spacing_md;
        let bold_is_bright = self.bold_is_bright;
        let ansi_colors = theme.ansi.clone();
        let bold_weight = theme.effective_weight(FontWeight::BOLD);
        let base_weight = theme.effective_weight(FontWeight::NORMAL);
        let streaming = self.streaming;

        // Begin a new frame on the shared selection coordinator so each
        // visible line re-registers before paint. Mirrors the markdown
        // pattern — see `MarkdownSelection::begin_frame`.
        let selection = self.selection.clone();
        if let Some(sel) = &selection {
            sel.begin_frame();
        }

        let line_count = self.lines.len();
        let lines_snapshot = self.lines;

        uniform_list(self.list_id, line_count, move |range, _window, _cx| {
            range
                .into_iter()
                .map(|ix| {
                    if ix >= lines_snapshot.len() {
                        return div().into_any_element();
                    }
                    let (flat_text, runs, link_ranges, link_urls) = build_line_runs(
                        &lines_snapshot[ix],
                        &font_mono,
                        base_weight,
                        bold_weight,
                        text_color,
                        terminal_bg,
                        link_color,
                        bold_is_bright,
                        &ansi_colors,
                    );
                    let shared_text = SharedString::from(flat_text);
                    let styled = StyledText::new(shared_text.clone()).with_runs(runs);
                    let is_last = ix + 1 == line_count;

                    let line_element: gpui::AnyElement = if let Some(sel) = &selection {
                        let id =
                            ElementId::Name(SharedString::from(format!("terminal-line-{}", ix)));
                        let mut selectable =
                            SelectableText::new(id, shared_text, styled, selection_bg, sel.clone());
                        if !link_ranges.is_empty() {
                            selectable = selectable.with_links(link_ranges, link_urls);
                        }
                        selectable.into_any_element()
                    } else {
                        styled.into_any_element()
                    };

                    if streaming && is_last {
                        // Static caret (Duration::ZERO) because the
                        // terminal only re-renders on `push_output`; a
                        // real blink would require `with_animation`.
                        // HIG Reduce Motion also prefers non-blinking.
                        let caret = render_caret(
                            CaretKind::Block,
                            text_color,
                            Instant::now(),
                            Duration::ZERO,
                            line_height,
                        );
                        div()
                            .flex()
                            .items_center()
                            .child(line_element)
                            .child(caret)
                            .into_any_element()
                    } else {
                        line_element
                    }
                })
                .collect()
        })
        .track_scroll(&self.scroll_handle)
        .px(spacing_md)
        .py(px(SPACING_4))
        .max_h(self.max_height)
        .font(font_mono_outer)
        .text_size(font_size)
    }
}

/// Build a flat string + per-span [`TextRun`]s + OSC 8 link ranges for a
/// single terminal line. Resolves fg/bg with SGR 7 reverse, bold-is-bright
/// substitution, and SGR 2 dim at span-level so the run output is
/// render-ready.
#[allow(clippy::too_many_arguments)]
fn build_line_runs(
    line: &[ansi_parser::AnsiSpan],
    font_mono: &Font,
    base_weight: FontWeight,
    bold_weight: FontWeight,
    text_color: Hsla,
    terminal_bg: Hsla,
    link_color: Hsla,
    bold_is_bright: bool,
    ansi_colors: &AnsiColors,
) -> (String, Vec<TextRun>, Vec<Range<usize>>, Vec<SharedString>) {
    let mut flat_text = String::new();
    let mut runs: Vec<TextRun> = Vec::with_capacity(line.len());
    let mut link_ranges = Vec::new();
    let mut link_urls = Vec::new();

    for span in line {
        if span.text.is_empty() {
            continue;
        }
        let start = flat_text.len();
        flat_text.push_str(&span.text);
        let end = flat_text.len();

        // SGR 7 (reverse) is resolved at render time because inversion
        // depends on the theme's default fg/bg which aren't available
        // during parsing.
        let resolved_fg = if span.style.reverse {
            span.style.bg.unwrap_or(terminal_bg)
        } else if let Some(fg) = span.style.fg {
            if bold_is_bright && span.style.bold {
                span.style.fg_bright.unwrap_or(fg)
            } else {
                fg
            }
        } else {
            text_color
        };
        let resolved_bg = if span.style.reverse {
            span.style.fg.unwrap_or(text_color)
        } else {
            span.style
                .bg
                .unwrap_or_else(|| gpui::hsla(0.0, 0.0, 0.0, 0.0))
        };

        let is_link = span.style.link.is_some();
        let mut final_fg = if is_link { link_color } else { resolved_fg };
        if span.style.dim {
            final_fg = ansi_colors.dim(final_fg);
        }

        if is_link {
            link_ranges.push(start..end);
            link_urls.push(
                span.style
                    .link
                    .clone()
                    .expect("is_link was computed from `span.style.link.is_some()` just above"),
            );
        }

        let weight = if span.style.bold {
            bold_weight
        } else {
            base_weight
        };
        let style = if span.style.italic {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };
        let mut font = font_mono.clone();
        font.weight = weight;
        font.style = style;

        let background_color = if resolved_bg.a > 0.0 {
            Some(resolved_bg)
        } else {
            None
        };
        let underline = if span.style.underline || is_link {
            Some(UnderlineStyle {
                thickness: px(1.0),
                color: Some(final_fg),
                wavy: false,
            })
        } else {
            None
        };
        let strikethrough = if span.style.strikethrough {
            Some(StrikethroughStyle {
                thickness: px(1.0),
                color: Some(final_fg),
            })
        } else {
            None
        };

        runs.push(TextRun {
            len: span.text.len(),
            font,
            color: final_fg,
            background_color,
            underline,
            strikethrough,
        });
    }

    (flat_text, runs, link_ranges, link_urls)
}

// -- TerminalClearButton ------------------------------------------------------

/// Clear button for terminal output. Renders a ghost button with trash icon.
#[derive(IntoElement)]
pub struct TerminalClearButton {
    element_id: ElementId,
    on_click: OnClick,
}

impl TerminalClearButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            element_id: id.into(),
            on_click: None,
        }
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TerminalClearButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut btn = Button::new(self.element_id)
            .icon(Icon::new(IconName::Trash).size(theme.icon_size_inline))
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSmall)
            .tooltip("Clear terminal");

        if let Some(handler) = self.on_click {
            btn = btn.on_click(handler);
        }

        btn
    }
}

// -- TerminalView (stateful entity) -------------------------------------------

/// Default scrollback length. Mirrors Zed's `DEFAULT_SCROLL_HISTORY_LINES`
/// (`crates/terminal/src/terminal.rs:342`) — 10 000 lines matches
/// Terminal.app / iTerm2 defaults and is the upper bound most users hit
/// before clearing manually. Per-view override via
/// [`TerminalView::set_scrollback_lines`].
pub const DEFAULT_SCROLLBACK_LINES: usize = 10_000;

/// A terminal output display with ANSI color support and virtualized line rendering.
pub struct TerminalView {
    element_id: ElementId,
    list_id: ElementId,
    output: String,
    /// Cached parsed lines — invalidated on push_output/clear.
    cached_lines: Arc<Vec<Vec<ansi_parser::AnsiSpan>>>,
    lines_dirty: bool,
    title: Option<SharedString>,
    is_streaming: bool,
    auto_scroll: bool,
    scroll_handle: UniformListScrollHandle,
    copy_button: Entity<CopyButton>,
    on_clear: OnMutCallbackArc,
    /// Maximum number of newline-separated lines retained in `output`.
    /// Older lines are dropped from the front when the limit is exceeded
    /// (Zed-style scrollback cap; finding #18 / the HIG Code-surface audit).
    /// `0` disables the cap — every line is retained.
    scrollback_lines: usize,
    /// Shared cross-line selection coordinator. Outlives re-renders so
    /// anchor / focus state persists across scrolls. Reuses the markdown
    /// infrastructure because the selection model is identical — see
    /// `crate::markdown::selection::MarkdownSelection`.
    selection: MarkdownSelection,
}

impl TerminalView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("terminal"),
            list_id: next_element_id("terminal-list"),
            output: String::new(),
            cached_lines: Arc::new(Vec::new()),
            lines_dirty: true,
            title: None,
            is_streaming: false,
            auto_scroll: true,
            scroll_handle: UniformListScrollHandle::new(),
            copy_button: CopyButton::new("", cx),
            on_clear: None,
            scrollback_lines: DEFAULT_SCROLLBACK_LINES,
            selection: MarkdownSelection::new(),
        }
    }

    /// Borrow the cross-line [`MarkdownSelection`] coordinator. Useful
    /// for parent views that want to query the selected text or react
    /// to selection-related events.
    pub fn selection(&self) -> MarkdownSelection {
        self.selection.clone()
    }

    pub fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.title = Some(title.into());
        cx.notify();
    }

    /// Set the maximum number of retained lines. `0` disables the cap;
    /// any larger value truncates older lines (from the front) once
    /// `output` exceeds the threshold on the next `push_output`.
    pub fn set_scrollback_lines(&mut self, lines: usize, cx: &mut Context<Self>) {
        self.scrollback_lines = lines;
        self.truncate_scrollback();
        self.lines_dirty = true;
        cx.notify();
    }

    /// Current scrollback cap. `0` means unlimited.
    pub fn scrollback_lines(&self) -> usize {
        self.scrollback_lines
    }

    pub fn push_output(&mut self, delta: &str, cx: &mut Context<Self>) {
        self.output.push_str(delta);
        self.truncate_scrollback();
        self.lines_dirty = true;
        if self.auto_scroll {
            self.scroll_handle.scroll_to_bottom();
        }
        let output = self.output.clone();
        self.copy_button
            .update(cx, |btn, _| btn.set_content(output));
        cx.notify();
    }

    /// Drop leading lines so that `output` has at most `scrollback_lines`
    /// newline-terminated lines. Preserves the trailing partial line (the
    /// one being streamed) by counting `\n` rather than splitting by it.
    fn truncate_scrollback(&mut self) {
        if self.scrollback_lines == 0 {
            return;
        }
        let newline_count = self.output.bytes().filter(|b| *b == b'\n').count();
        if newline_count <= self.scrollback_lines {
            return;
        }
        let drop = newline_count - self.scrollback_lines;
        // Find the byte index just after the `drop`-th newline and slice
        // from there. This preserves the trailing partial line without
        // allocating a separate Vec<Line>.
        let mut seen = 0;
        let mut cut = 0;
        for (idx, byte) in self.output.bytes().enumerate() {
            if byte == b'\n' {
                seen += 1;
                if seen == drop {
                    cut = idx + 1;
                    break;
                }
            }
        }
        if cut > 0 {
            self.output.drain(..cut);
        }
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.output.clear();
        Arc::make_mut(&mut self.cached_lines).clear();
        self.lines_dirty = true;
        self.copy_button
            .update(cx, |btn, _| btn.set_content(String::new()));
        cx.notify();
    }

    pub fn set_streaming(&mut self, streaming: bool, cx: &mut Context<Self>) {
        self.is_streaming = streaming;
        cx.notify();
    }

    pub fn set_auto_scroll(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.auto_scroll = enabled;
        cx.notify();
    }

    pub fn set_on_clear(&mut self, callback: Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>) {
        self.on_clear = Some(callback);
    }

    pub fn set_on_copy(
        &mut self,
        callback: Arc<dyn Fn() + Send + Sync + 'static>,
        cx: &mut Context<Self>,
    ) {
        self.copy_button
            .update(cx, |btn, _| btn.set_on_copy(callback));
    }

    // -- Accessors for compound composition -----------------------------------

    /// Get a snapshot of parsed lines for custom `TerminalContent` rendering.
    pub fn parsed_lines(&mut self, cx: &Context<Self>) -> Arc<Vec<Vec<ansi_parser::AnsiSpan>>> {
        self.ensure_lines_parsed(cx);
        Arc::clone(&self.cached_lines)
    }

    /// Get the scroll handle for custom `TerminalContent` rendering.
    pub fn scroll_handle(&self) -> UniformListScrollHandle {
        self.scroll_handle.clone()
    }

    /// Get the list element ID.
    pub fn list_id(&self) -> ElementId {
        self.list_id.clone()
    }

    /// Get the copy button entity for inclusion in custom layouts.
    pub fn copy_button(&self) -> Entity<CopyButton> {
        self.copy_button.clone()
    }

    // -- Internal -------------------------------------------------------------

    fn ensure_lines_parsed(&mut self, cx: &Context<Self>) {
        if self.lines_dirty {
            let theme = cx.theme();
            let ansi_colors = &theme.ansi;
            // Thread ANSI style state across line boundaries so colors
            // set on one line carry to subsequent lines.
            let mut style = ansi_parser::AnsiStyle::default();
            let mut lines = Vec::new();
            for line in self.output.split('\n') {
                let (spans, final_style) =
                    ansi_parser::parse_ansi_with_style(line, ansi_colors, style);
                style = final_style;
                lines.push(spans);
            }
            self.cached_lines = Arc::new(lines);
            self.lines_dirty = false;
        }
    }
}

impl Render for TerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Reparse ANSI only when output changed
        self.ensure_lines_parsed(cx);

        // Build header with sub-components
        let title_text = self.title.clone().unwrap_or_else(|| "Terminal".into());

        let mut actions = TerminalActions::new();
        if self.is_streaming {
            let streaming_id = SharedString::from(format!("{}-streaming", &self.element_id));
            actions = actions.child(TerminalStatus::new(streaming_id, true));
        }
        actions = actions.child(self.copy_button.clone());

        if self.on_clear.is_some() {
            let clear_id = SharedString::from(format!("{}-clear", &self.element_id));
            actions = actions.child(TerminalClearButton::new(clear_id).on_click(cx.listener(
                |this, _event, window, cx| {
                    this.clear(cx);
                    if let Some(on_clear) = &this.on_clear {
                        on_clear(window, cx);
                    }
                },
            )));
        }

        let header = TerminalHeader::new()
            .border(true)
            .child(TerminalTitle::new(title_text))
            .child(actions);

        let content = TerminalContent::new(
            self.list_id.clone(),
            Arc::clone(&self.cached_lines),
            self.scroll_handle.clone(),
        )
        .selection(self.selection.clone())
        .streaming(self.is_streaming);

        div()
            .flex()
            .flex_col()
            .bg(theme.terminal_bg)
            .rounded(theme.radius_lg)
            .border_1()
            .border_color(theme.border)
            .overflow_hidden()
            .child(header)
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::ansi_parser::{AnsiSpan, AnsiStyle};
    use super::{
        TerminalActions, TerminalClearButton, TerminalContent, TerminalHeader, TerminalStatus,
        TerminalTitle,
    };
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;
    use gpui::{ElementId, SharedString, UniformListScrollHandle, div, px};
    use std::sync::Arc;

    // -- TerminalTitle --------------------------------------------------------

    #[test]
    fn title_defaults_to_terminal_icon() {
        let title = TerminalTitle::new("Build");
        assert_eq!(title.title.as_ref(), "Build");
        assert_eq!(title.icon, IconName::Terminal);
    }

    #[test]
    fn title_custom_icon() {
        let title = TerminalTitle::new("Server").icon(IconName::Globe);
        assert_eq!(title.title.as_ref(), "Server");
        assert_eq!(title.icon, IconName::Globe);
    }

    // -- TerminalStatus -------------------------------------------------------

    #[test]
    fn status_streaming_flag() {
        let status = TerminalStatus::new("test-streaming", true);
        assert!(status.is_streaming);

        let status = TerminalStatus::new("test-streaming", false);
        assert!(!status.is_streaming);
    }

    // -- TerminalHeader -------------------------------------------------------

    #[test]
    fn header_accumulates_children() {
        let header = TerminalHeader::new().child(div()).child(div());
        assert_eq!(header.children.len(), 2);
    }

    // -- TerminalActions ------------------------------------------------------

    #[test]
    fn actions_accumulates_children() {
        let actions = TerminalActions::new()
            .child(div())
            .child(div())
            .child(div());
        assert_eq!(actions.children.len(), 3);
    }

    // -- TerminalContent ------------------------------------------------------

    #[test]
    fn content_default_max_height() {
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        );
        assert_eq!(content.max_height, px(384.0));
    }

    #[test]
    fn content_defaults_to_body_text_style() {
        // Finding N2: 13 pt `Body` matches Terminal.app / Xcode console.
        use crate::foundations::theme::TextStyle;
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        );
        assert!(matches!(content.text_style, TextStyle::Body));
    }

    #[test]
    fn content_bold_is_bright_enabled_by_default() {
        // Finding #5: Cargo/git/npm emit bold+standard expecting bright.
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        );
        assert!(content.bold_is_bright);
    }

    #[test]
    fn content_bold_is_bright_opt_out() {
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        )
        .bold_is_bright(false);
        assert!(!content.bold_is_bright);
    }

    #[test]
    fn content_streaming_flag_wiring() {
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        );
        assert!(!content.streaming);
        let content = content.streaming(true);
        assert!(content.streaming);
    }

    #[test]
    fn content_selection_slot_default_empty() {
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        );
        assert!(content.selection.is_none());
    }

    #[test]
    fn content_selection_slot_accepts_coordinator() {
        use crate::markdown::selection::MarkdownSelection;
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        )
        .selection(MarkdownSelection::new());
        assert!(content.selection.is_some());
    }

    fn test_font_mono() -> gpui::Font {
        gpui::Font {
            family: gpui::SharedString::from("SF Mono"),
            features: gpui::FontFeatures::default(),
            fallbacks: Some(gpui::FontFallbacks::from_fonts(vec!["Menlo".into()])),
            weight: gpui::FontWeight::default(),
            style: gpui::FontStyle::default(),
        }
    }

    #[test]
    fn build_line_runs_emits_one_run_per_span() {
        use super::ansi_parser::{AnsiSpan, AnsiStyle};
        use super::build_line_runs;
        use crate::foundations::theme::AnsiColors;
        use gpui::{FontWeight, hsla};

        let spans = vec![
            AnsiSpan {
                text: "foo".into(),
                style: AnsiStyle::default(),
            },
            AnsiSpan {
                text: "bar".into(),
                style: AnsiStyle {
                    bold: true,
                    ..AnsiStyle::default()
                },
            },
        ];
        let colors = AnsiColors::new(true);
        let mono = test_font_mono();
        let (text, runs, _, _) = build_line_runs(
            &spans,
            &mono,
            FontWeight::NORMAL,
            FontWeight::BOLD,
            hsla(0.0, 0.0, 0.9, 1.0),
            hsla(0.0, 0.0, 0.05, 1.0),
            hsla(0.6, 0.7, 0.6, 1.0),
            true,
            &colors,
        );
        assert_eq!(text, "foobar");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].len, 3);
        assert_eq!(runs[1].len, 3);
        assert_eq!(runs[1].font.weight, FontWeight::BOLD);
        // Finding #29: fallbacks from the prototype must flow into each TextRun
        // so terminal output stays monospaced on hosts without SF Mono.
        assert_eq!(
            runs[0]
                .font
                .fallbacks
                .as_ref()
                .expect("prototype fallbacks must propagate")
                .fallback_list(),
            ["Menlo"]
        );
    }

    #[test]
    fn build_line_runs_records_osc8_links() {
        use super::ansi_parser::{AnsiSpan, AnsiStyle};
        use super::build_line_runs;
        use crate::foundations::theme::AnsiColors;
        use gpui::{FontWeight, SharedString, hsla};

        let spans = vec![
            AnsiSpan {
                text: "click me".into(),
                style: AnsiStyle {
                    link: Some(SharedString::from("https://apple.com")),
                    ..AnsiStyle::default()
                },
            },
            AnsiSpan {
                text: " done".into(),
                style: AnsiStyle::default(),
            },
        ];
        let colors = AnsiColors::new(true);
        let mono = test_font_mono();
        let (text, _runs, link_ranges, link_urls) = build_line_runs(
            &spans,
            &mono,
            FontWeight::NORMAL,
            FontWeight::BOLD,
            hsla(0.0, 0.0, 0.9, 1.0),
            hsla(0.0, 0.0, 0.05, 1.0),
            hsla(0.6, 0.7, 0.6, 1.0),
            true,
            &colors,
        );
        assert_eq!(text, "click me done");
        assert_eq!(link_ranges.len(), 1);
        assert_eq!(link_ranges[0], 0..8);
        assert_eq!(link_urls[0].as_ref(), "https://apple.com");
    }

    #[test]
    fn build_line_runs_applies_bold_is_bright_when_enabled() {
        use super::ansi_parser::{AnsiSpan, AnsiStyle};
        use super::build_line_runs;
        use crate::foundations::theme::AnsiColors;
        use gpui::{FontWeight, hsla};

        let colors = AnsiColors::new(true);
        let red = colors.red;
        let bright_red = colors.bright_red;
        let spans = vec![AnsiSpan {
            text: "x".into(),
            style: AnsiStyle {
                fg: Some(red),
                fg_bright: Some(bright_red),
                bold: true,
                ..AnsiStyle::default()
            },
        }];
        let mono = test_font_mono();
        let default_text = hsla(0.0, 0.0, 0.9, 1.0);
        let default_bg = hsla(0.0, 0.0, 0.05, 1.0);
        let link_c = hsla(0.6, 0.7, 0.6, 1.0);

        let (_, runs_on, _, _) = build_line_runs(
            &spans,
            &mono,
            FontWeight::NORMAL,
            FontWeight::BOLD,
            default_text,
            default_bg,
            link_c,
            true,
            &colors,
        );
        assert_eq!(runs_on[0].color, bright_red);

        let (_, runs_off, _, _) = build_line_runs(
            &spans,
            &mono,
            FontWeight::NORMAL,
            FontWeight::BOLD,
            default_text,
            default_bg,
            link_c,
            false,
            &colors,
        );
        assert_eq!(runs_off[0].color, red);
    }

    #[test]
    fn content_custom_max_height() {
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::new(Vec::new()),
            UniformListScrollHandle::new(),
        )
        .max_height(px(200.0));
        assert_eq!(content.max_height, px(200.0));
    }

    #[test]
    fn content_stores_lines() {
        let spans = vec![vec![AnsiSpan {
            text: "hello".to_string(),
            style: AnsiStyle::default(),
        }]];
        let lines = Arc::new(spans);
        let content = TerminalContent::new(
            ElementId::from(SharedString::from("test-list")),
            Arc::clone(&lines),
            UniformListScrollHandle::new(),
        );
        assert_eq!(content.lines.len(), 1);
        assert_eq!(content.lines[0][0].text, "hello");
    }

    // -- TerminalClearButton --------------------------------------------------

    #[test]
    fn clear_button_no_handler_by_default() {
        let btn = TerminalClearButton::new("clear-test");
        assert!(btn.on_click.is_none());
    }

    #[test]
    fn clear_button_with_handler() {
        let btn = TerminalClearButton::new("clear-test").on_click(|_event, _window, _cx| {});
        assert!(btn.on_click.is_some());
    }

    // Integration tests for TerminalView accessor methods live in
    // `terminal_view_tests` below because they need `gpui::TestAppContext`.
}

#[cfg(test)]
mod terminal_view_tests {
    use super::TerminalView;
    use crate::test_helpers::helpers::setup_test_window;

    #[gpui::test]
    async fn parsed_lines_reflects_output(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.push_output("hello\nworld\n", cx);
            let lines = terminal.parsed_lines(cx);
            // `output.split('\n')` yields a trailing empty segment after the
            // final newline — kept deliberately so the caret has a final
            // line to anchor against while streaming.
            assert_eq!(lines.len(), 3);
            assert_eq!(lines[0][0].text, "hello");
            assert_eq!(lines[1][0].text, "world");
        });
    }

    #[gpui::test]
    async fn parsed_lines_reparses_on_ansi_delta(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.push_output("\x1b[31mred\x1b[0m ok\n", cx);
            let lines = terminal.parsed_lines(cx);
            assert!(lines[0].iter().any(|span| span.style.fg.is_some()));
        });
    }

    #[gpui::test]
    async fn clear_resets_parsed_lines(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.push_output("foo\nbar\n", cx);
            assert!(!terminal.parsed_lines(cx).is_empty());
            terminal.clear(cx);
            // After clear, parsing a fresh empty output yields one line
            // (the trailing empty segment).
            let lines = terminal.parsed_lines(cx);
            assert_eq!(lines.len(), 1);
            assert!(lines[0].is_empty());
        });
    }

    #[gpui::test]
    async fn scroll_handle_and_list_id_are_stable(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, _cx| {
            let first_id = terminal.list_id();
            let second_id = terminal.list_id();
            assert_eq!(first_id, second_id);
            // Handle is a cheap clone — still pointer-equivalent.
            let _h = terminal.scroll_handle();
        });
    }

    #[gpui::test]
    async fn copy_button_tracks_output(cx: &mut gpui::TestAppContext) {
        use crate::components::menus_and_actions::copy_button::CopyButton;
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.push_output("payload", cx);
            let button: gpui::Entity<CopyButton> = terminal.copy_button();
            button.update(cx, |btn, _cx| {
                assert_eq!(btn.content(), "payload");
            });
        });
    }

    #[gpui::test]
    async fn set_streaming_toggles_flag(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.set_streaming(true, cx);
            assert!(terminal.is_streaming);
            terminal.set_streaming(false, cx);
            assert!(!terminal.is_streaming);
        });
    }

    #[gpui::test]
    async fn selection_handle_is_shared_clone(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, _cx| {
            let s1 = terminal.selection();
            let s2 = terminal.selection();
            // Both clones reference the same inner Rc<RefCell<…>>; mutating
            // via one is observable via the other.
            s1.select_all();
            // Nothing registered yet, so select_all shouldn't panic and the
            // coordinator state stays consistent across clones.
            let _ = s2;
        });
    }
}

#[cfg(test)]
mod scrollback_tests {
    use super::{DEFAULT_SCROLLBACK_LINES, TerminalView};
    use crate::test_helpers::helpers::setup_test_window;

    #[gpui::test]
    async fn scrollback_defaults_to_10k(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, _cx| {
            assert_eq!(terminal.scrollback_lines(), DEFAULT_SCROLLBACK_LINES);
            assert_eq!(DEFAULT_SCROLLBACK_LINES, 10_000);
        });
    }

    #[gpui::test]
    async fn scrollback_truncates_head_when_exceeded(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.set_scrollback_lines(3, cx);
            for i in 0..6 {
                terminal.push_output(&format!("line {}\n", i), cx);
            }
            // After truncation, output keeps the most recent 3 newline-
            // terminated lines. The trailing empty partial line is
            // preserved by `truncate_scrollback`'s `\n` counter.
            assert_eq!(terminal.output, "line 3\nline 4\nline 5\n");
        });
    }

    #[gpui::test]
    async fn scrollback_zero_disables_cap(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.set_scrollback_lines(0, cx);
            for i in 0..50 {
                terminal.push_output(&format!("L{}\n", i), cx);
            }
            // No truncation when cap is 0.
            assert!(terminal.output.starts_with("L0\n"));
            assert!(terminal.output.contains("L49\n"));
        });
    }

    #[gpui::test]
    async fn scrollback_preserves_trailing_partial_line(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TerminalView::new(cx));
        handle.update(cx, |terminal, cx| {
            terminal.set_scrollback_lines(2, cx);
            terminal.push_output("a\nb\nc\npartial", cx);
            // `partial` has no trailing `\n`, so it's kept — the cap
            // applies only to fully-terminated lines.
            assert!(terminal.output.ends_with("partial"));
        });
    }
}
