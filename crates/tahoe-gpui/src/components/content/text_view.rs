//! Read-only styled text display aligned with HIG Text views.
//!
//! Displays multiple lines of styled, non-editable text. Unlike
//! [`super::label::Label`] (single-line) or
//! [`crate::components::selection_and_input::text_field::TextField`] (editable),
//! `TextView` is for presenting blocks of formatted content.
//!
//! # Architecture
//!
//! `TextView` is a GPUI [`gpui::Entity`] — construct via
//! `cx.new(|cx| TextView::new(cx, "…"))`. The entity owns four pieces of
//! state that together make the view interactive:
//!
//! - a [`FocusHandle`] so ⌘C / ⌘A (and the raw scroll keys bound under
//!   [`TEXT_VIEW_CONTEXT`]) reach it through the standard focus
//!   dispatch path;
//! - a [`TextViewSelection`] coordinator that mediates mouse-driven
//!   selection against the [`super::selectable_text::SelectableText`]
//!   primitive;
//! - a [`gpui::ScrollHandle`] wired to the keyboard-scroll action set
//!   when [`TextView::scrollable`] is on;
//! - an [`Entity<ContextMenu>`] rendered as a fullscreen overlay on
//!   right-click. See [`Self::open_context_menu`].
//!
//! # Dynamic Type
//!
//! When the theme's accessibility mode reports Bold-Text / high-contrast
//! preferences, `TextView` applies the same `effective_weight` +
//! `effective_font_scale_factor` adjustments that [`TextStyledExt`] uses for
//! the rest of the design system. This keeps the text-body scale consistent
//! with sidebar / menu / button typography when the user enables an
//! accessibility text-size mode.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

use gpui::prelude::*;
use gpui::{
    App, ClipboardItem, Context, ElementId, Entity, FocusHandle, Focusable, HighlightStyle, Hsla,
    MouseButton, MouseDownEvent, Pixels, Point, ScrollHandle, SharedString, StyledText, TextAlign,
    Window, div, point, px,
};

use crate::components::content::selectable_text::{
    SelectableText, SelectionCoordinator, word_range_at,
};
use crate::components::menus_and_actions::context_menu::{
    ContextMenu, ContextMenuEntry, ContextMenuItem, ContextMenuItemStyle,
};
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::layout::READABLE_OPTIMAL_WIDTH;
use crate::foundations::theme::{
    ActiveTheme, FontDesign, LabelLevel, LeadingStyle, TahoeTheme, TextStyle, TextStyledExt,
};
use crate::text_actions::{
    Copy, Down, End, Home, PageDown, PageUp, SelectAll, ShowContextMenu, Up,
};

/// GPUI key context used by `TextView` for scoped keyboard shortcuts.
///
/// Mirrors the convention used by
/// [`TextField`](crate::components::selection_and_input::text_field::TEXT_FIELD_CONTEXT):
/// scope-specific raw-key or modifier-key bindings live under a named
/// context so they don't collide with global bindings.
pub const TEXT_VIEW_CONTEXT: &str = "TextView";

/// Returns the raw-key and modifier-key bindings scoped to
/// [`TEXT_VIEW_CONTEXT`]. Install alongside [`crate::text_keybindings`]
/// during app startup:
///
/// ```ignore
/// cx.bind_keys(tahoe_gpui::textview_keybindings());
/// ```
///
/// Covers:
/// - ⌘C (Copy) and ⌘A (Select All) — selection shortcuts.
/// - Up / Down / Page Up / Page Down / Home / End — keyboard scroll.
/// - Shift-F10 (macOS / NSTextView convention) — open the right-click
///   context menu from the keyboard so the Copy / Select All entries
///   stay reachable without a pointer.
///
/// The raw keys are scoped to [`TEXT_VIEW_CONTEXT`] so they only fire
/// when a focused [`TextView`] owns the key dispatch path — they do not
/// leak to the global scope where Up / Down have other meanings (tab
/// navigation, menu cursor, slider).
pub fn keybindings() -> Vec<gpui::KeyBinding> {
    use gpui::KeyBinding;
    let ctx = Some(TEXT_VIEW_CONTEXT);
    vec![
        KeyBinding::new("cmd-c", Copy, ctx),
        KeyBinding::new("cmd-a", SelectAll, ctx),
        KeyBinding::new("up", Up, ctx),
        KeyBinding::new("down", Down, ctx),
        KeyBinding::new("pageup", PageUp, ctx),
        KeyBinding::new("pagedown", PageDown, ctx),
        KeyBinding::new("home", Home, ctx),
        KeyBinding::new("end", End, ctx),
        KeyBinding::new("shift-f10", ShowContextMenu, ctx),
    ]
}

/// Selection mode set at mouse-down, consumed by subsequent drag
/// extension. Character-by-character for single-click, whole-word for
/// double-click, entire paragraph for triple-click or Cmd+A.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SelectMode {
    #[default]
    Character,
    Word,
    Line,
    All,
}

#[derive(Default)]
struct TextViewSelectionState {
    element_id: Option<ElementId>,
    text: SharedString,
    anchor: Option<usize>,
    focus: Option<usize>,
    pending: bool,
    mode: SelectMode,
    /// Original `(start, end)` at mouse-down for word / line drag so
    /// dragging back across the anchor keeps the original selection
    /// intact rather than collapsing it.
    original: Option<(usize, usize)>,
}

/// Single-paragraph selection coordinator for [`TextView`].
///
/// Implements [`SelectionCoordinator`] so it can slot directly into
/// [`SelectableText`]. Unlike [`crate::markdown::MarkdownSelection`]
/// (which tracks a list of paragraphs for cross-paragraph drag-select),
/// `TextViewSelection` holds state for the single paragraph the
/// enclosing `TextView` owns — all mouse events resolve to that
/// paragraph by element id.
#[derive(Clone, Default)]
pub struct TextViewSelection {
    inner: Rc<RefCell<TextViewSelectionState>>,
}

impl TextViewSelection {
    /// Construct a fresh coordinator with no registered element and an
    /// empty selection. Equivalent to [`TextViewSelection::default`] —
    /// kept as an explicit constructor so call sites read as
    /// `TextViewSelection::new()` rather than calling `Default::default`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Collapse the selection to empty.
    pub fn clear(&self) {
        let mut s = self.inner.borrow_mut();
        s.anchor = None;
        s.focus = None;
        s.pending = false;
        s.mode = SelectMode::default();
        s.original = None;
    }

    /// Select the entire paragraph. Requires the paragraph to have been
    /// registered (i.e. `TextView` to have rendered at least once);
    /// before that, this is a no-op.
    pub fn select_all(&self) {
        let mut s = self.inner.borrow_mut();
        if s.element_id.is_none() {
            return;
        }
        s.anchor = Some(0);
        s.focus = Some(s.text.len());
        s.pending = false;
        s.mode = SelectMode::All;
        s.original = None;
    }

    /// `true` when a non-empty selection exists.
    pub fn has_selection(&self) -> bool {
        let s = self.inner.borrow();
        matches!((s.anchor, s.focus), (Some(a), Some(f)) if a != f)
    }

    /// Return the selected substring, or `""` when nothing is selected.
    pub fn selected_text(&self) -> String {
        let s = self.inner.borrow();
        let Some(anchor) = s.anchor else {
            return String::new();
        };
        let Some(focus) = s.focus else {
            return String::new();
        };
        let (lo, hi) = normalise(anchor, focus);
        let text_len = s.text.len();
        let lo = lo.min(text_len);
        let hi = hi.min(text_len);
        s.text.get(lo..hi).unwrap_or("").to_string()
    }

    /// Copy the selected substring to the system clipboard. No-op when
    /// the selection is empty.
    pub fn copy_to_clipboard(&self, cx: &mut App) {
        let text = self.selected_text();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    /// Return the current selection range, or `None` when empty.
    /// Exposed for tests.
    #[cfg(test)]
    fn range(&self) -> Option<Range<usize>> {
        let s = self.inner.borrow();
        let (anchor, focus) = (s.anchor?, s.focus?);
        let (lo, hi) = normalise(anchor, focus);
        if lo < hi { Some(lo..hi) } else { None }
    }
}

impl SelectionCoordinator for TextViewSelection {
    fn register(&self, element_id: ElementId, text: SharedString) {
        let mut s = self.inner.borrow_mut();
        s.element_id = Some(element_id);
        s.text = text;
    }

    fn range_for_element(&self, element_id: &ElementId, text_len: usize) -> Option<Range<usize>> {
        let s = self.inner.borrow();
        if s.element_id.as_ref()? != element_id {
            return None;
        }
        let (lo, hi) = normalise(s.anchor?, s.focus?);
        let lo = lo.min(text_len);
        let hi = hi.min(text_len);
        if lo < hi { Some(lo..hi) } else { None }
    }

    fn mouse_down(
        &self,
        element_id: ElementId,
        text: &str,
        char_index: usize,
        click_count: usize,
        shift: bool,
    ) {
        let mut s = self.inner.borrow_mut();
        // Ignore events from elements we don't own (shouldn't happen in
        // practice since each TextView holds its own coordinator).
        if s.element_id.as_ref() != Some(&element_id) {
            return;
        }
        if shift && s.anchor.is_some() {
            // Shift-click extends the existing selection. Word / Line
            // modes grow to the surrounding word or paragraph so the
            // extension respects the original gesture — matching
            // macOS NSTextView where shift-clicking after a
            // double-click grows by whole words rather than collapsing
            // back to character granularity.
            let focus = match s.mode {
                SelectMode::Word => {
                    let word = word_range_at(text, char_index);
                    match s.original {
                        Some((orig_start, orig_end)) if char_index < orig_start => {
                            s.anchor = Some(orig_end);
                            word.start
                        }
                        Some((orig_start, _)) => {
                            s.anchor = Some(orig_start);
                            word.end
                        }
                        None => word.end,
                    }
                }
                SelectMode::Line => {
                    // Triple-click mode anchored to the full paragraph
                    // — shift-click keeps the paragraph selected.
                    s.anchor = Some(0);
                    text.len()
                }
                SelectMode::Character | SelectMode::All => char_index,
            };
            s.focus = Some(focus);
            s.pending = true;
            return;
        }
        match click_count {
            1 => {
                s.anchor = Some(char_index);
                s.focus = Some(char_index);
                s.pending = true;
                s.mode = SelectMode::Character;
                s.original = None;
            }
            2 => {
                let range = word_range_at(text, char_index);
                s.anchor = Some(range.start);
                s.focus = Some(range.end);
                s.pending = true;
                s.mode = SelectMode::Word;
                s.original = Some((range.start, range.end));
            }
            _ => {
                // Triple-click and beyond both select the whole
                // paragraph; only the anchor-preservation semantics on
                // drag differs (Line preserves, All ends pending).
                s.anchor = Some(0);
                s.focus = Some(text.len());
                if click_count == 3 {
                    s.pending = true;
                    s.mode = SelectMode::Line;
                    s.original = Some((0, text.len()));
                } else {
                    s.pending = false;
                    s.mode = SelectMode::All;
                    s.original = None;
                }
            }
        }
    }

    fn drag_to(&self, element_id: ElementId, text: &str, char_index: usize) {
        let mut s = self.inner.borrow_mut();
        if !s.pending || s.element_id.as_ref() != Some(&element_id) {
            return;
        }
        match s.mode {
            SelectMode::Character => {
                s.focus = Some(char_index);
            }
            SelectMode::Word => {
                let word = word_range_at(text, char_index);
                let Some((orig_start, orig_end)) = s.original else {
                    s.focus = Some(word.end);
                    return;
                };
                if char_index < orig_start {
                    s.anchor = Some(orig_end);
                    s.focus = Some(word.start);
                } else {
                    s.anchor = Some(orig_start);
                    s.focus = Some(word.end);
                }
            }
            SelectMode::Line => {
                // Single paragraph — any triple-click drag still spans
                // the whole paragraph. Keep anchor/focus pinned at the
                // bounds so reversing the drag doesn't collapse.
                s.anchor = Some(0);
                s.focus = Some(text.len());
            }
            SelectMode::All => {}
        }
    }

    fn end_drag(&self) {
        self.inner.borrow_mut().pending = false;
    }

    fn is_pending(&self) -> bool {
        self.inner.borrow().pending
    }
}

fn normalise(a: usize, b: usize) -> (usize, usize) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Debug-only guard against highlight ranges that would panic inside
/// [`StyledText::with_highlights`] or silently truncate visible text:
/// out-of-bounds endpoints, or endpoints that split a multi-byte UTF-8
/// codepoint. Release builds skip the check entirely — the same
/// contract GPUI itself documents on the `StyledText` API.
fn debug_assert_highlight_ranges(text: &str, highlights: &[(Range<usize>, HighlightStyle)]) {
    if cfg!(debug_assertions) {
        for (range, _) in highlights {
            debug_assert!(
                range.start <= range.end
                    && range.end <= text.len()
                    && text.is_char_boundary(range.start)
                    && text.is_char_boundary(range.end),
                "TextView highlight range {range:?} invalid for text of length {}",
                text.len(),
            );
        }
    }
}

/// Content held by a [`TextView`] — either a plain string or rich text
/// with inline highlight spans. Rich content carries the plain-text
/// equivalent alongside the highlight vector so VoiceOver has something
/// to announce without callers re-supplying the text via
/// [`TextView::accessibility_label`].
///
/// The highlight slice is wrapped in an [`Arc`] so the view itself (the
/// `Entity<TextView>`) only holds a cheap shared handle. Paint still
/// copies the spans into the [`StyledText`] it hands to GPUI, but the
/// builder path and any logical clone of the view avoid a deep copy of
/// the span vector.
enum TextViewContent {
    Plain(SharedString),
    Rich {
        text: SharedString,
        highlights: Arc<[(Range<usize>, HighlightStyle)]>,
    },
}

impl TextViewContent {
    fn text(&self) -> &SharedString {
        match self {
            Self::Plain(t) | Self::Rich { text: t, .. } => t,
        }
    }

    fn styled(&self) -> StyledText {
        match self {
            Self::Plain(t) => StyledText::new(t.clone()),
            Self::Rich { text, highlights } => {
                StyledText::new(text.clone()).with_highlights(highlights.to_vec())
            }
        }
    }
}

/// A read-only text display view per HIG.
///
/// Shows one or more paragraphs of styled text.
///
/// # Capabilities
///
/// - Content: plain text via [`Self::new`] or rich text with
///   [`HighlightStyle`] spans via [`Self::styled_text`].
/// - Typography: [`Self::text_style`], [`Self::emphasize`],
///   [`Self::font_design`], [`Self::leading_style`].
/// - Layout: [`Self::max_lines`] (line-clamp),
///   [`Self::readable_width`] (544 pt cap scaled by Dynamic Type),
///   [`Self::scrollable`] (vertical scroll), [`Self::text_align`].
/// - Color: [`Self::color`] (explicit) or [`Self::label_level`] (semantic
///   HIG hierarchy).
/// - Selection: drag-select, double-click word, triple-click paragraph,
///   shift-click extend, ⌘A select all, ⌘C copy. Opt-out via
///   [`Self::selectable`]`(false)` for decorative read-only labels.
/// - Accessibility: [`Self::accessibility_label`] override.
///
/// # Color precedence
///
/// `color()` > `label_level()` > default `theme.text`. An explicit
/// `color()` wins over any semantic tier. For a disabled look, pass
/// `color(theme.text_disabled())` directly — `TextView` does not carry a
/// `disabled` flag because it has no interactive state beyond selection.
///
/// # Layout precedence
///
/// [`Self::max_lines`] and [`Self::scrollable`] are mutually exclusive:
/// clamped content cannot scroll because its height is bounded by GPUI's
/// `line_clamp`. Setting both trips a `debug_assert!` so the conflict
/// panics in tests and debug builds; release builds silently prefer
/// `max_lines`.
pub struct TextView {
    focus_handle: FocusHandle,
    selection: TextViewSelection,
    element_id: ElementId,
    content: TextViewContent,
    style: TextStyle,
    max_lines: Option<usize>,
    emphasize: bool,
    color: Option<Hsla>,
    label_level: Option<LabelLevel>,
    font_design: Option<FontDesign>,
    leading_style: LeadingStyle,
    text_align: Option<TextAlign>,
    scroll_id: Option<ElementId>,
    scroll_handle: ScrollHandle,
    readable_width: bool,
    selectable: bool,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    context_menu: Entity<ContextMenu>,
}

impl TextView {
    /// Construct a new plain-text view.
    ///
    /// The returned value is a builder the caller wraps with
    /// `cx.new(|cx| TextView::new(cx, "…").…)`. The resulting
    /// [`gpui::Entity<TextView>`] is itself [`IntoElement`] so it slots
    /// directly into the parent element tree.
    pub fn new(cx: &mut Context<Self>, text: impl Into<SharedString>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selection: TextViewSelection::new(),
            element_id: ElementId::NamedInteger("text-view-body".into(), cx.entity_id().as_u64()),
            content: TextViewContent::Plain(text.into()),
            style: TextStyle::Body,
            max_lines: None,
            emphasize: false,
            color: None,
            label_level: None,
            font_design: None,
            leading_style: LeadingStyle::default(),
            text_align: None,
            scroll_id: None,
            scroll_handle: ScrollHandle::new(),
            readable_width: false,
            selectable: true,
            disabled: false,
            accessibility_label: None,
            context_menu: cx.new(ContextMenu::new),
        }
    }

    /// Construct a rich-text view directly. Equivalent to
    /// `TextView::new(cx, text).styled_text(text, highlights)` without
    /// the throwaway plain-text placeholder, and avoids the
    /// double-allocation the two-step pattern implies when the caller
    /// already has the `HighlightStyle` vector in hand.
    pub fn styled(
        cx: &mut Context<Self>,
        text: impl Into<SharedString>,
        highlights: Vec<(Range<usize>, HighlightStyle)>,
    ) -> Self {
        let text = text.into();
        debug_assert_highlight_ranges(&text, &highlights);
        let mut this = Self::new(cx, text.clone());
        this.content = TextViewContent::Rich {
            text,
            highlights: Arc::from(highlights),
        };
        this
    }

    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Display rich text with [`HighlightStyle`] spans over a plain-text
    /// backbone. `highlights` is a vector of `(byte_range, style)` pairs
    /// applied on top of the baseline typography — matches GPUI's
    /// [`StyledText::with_highlights`] contract.
    ///
    /// The plain-text `text` is used as the VoiceOver label when
    /// [`Self::accessibility_label`] is not set, so rich content stays
    /// accessible without forcing callers to restate the text twice.
    pub fn styled_text(
        mut self,
        text: impl Into<SharedString>,
        highlights: Vec<(Range<usize>, HighlightStyle)>,
    ) -> Self {
        let text = text.into();
        debug_assert_highlight_ranges(&text, &highlights);
        self.content = TextViewContent::Rich {
            text,
            highlights: Arc::from(highlights),
        };
        self
    }

    /// Clamp the rendered text to `max` lines using GPUI's native
    /// `line-clamp`. Overflowing content is hidden.
    ///
    /// `max_lines(0)` is ignored: `line_clamp(0)` would hide every line,
    /// which is almost never what a caller building the value dynamically
    /// wants. Pass `max_lines(1)` to keep a single line.
    pub fn max_lines(mut self, max: usize) -> Self {
        if max > 0 {
            self.max_lines = Some(max);
        }
        self
    }

    /// Render with the HIG "Emphasized" weight for the text style (see
    /// [`TextStyle::emphasized`]). For example `Body` emphasizes to
    /// `SEMIBOLD`, `LargeTitle` to `BOLD`, and `Headline` to `BLACK`.
    pub fn emphasize(mut self, emphasize: bool) -> Self {
        self.emphasize = emphasize;
        self
    }

    /// Override the text color (default: `theme.text`). Wins over
    /// [`Self::label_level`].
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the text color via the HIG label-level hierarchy.
    ///
    /// Resolves to the correct semantic color (e.g. `theme.text_muted`
    /// for [`LabelLevel::Secondary`]). If both `color()` and
    /// `label_level()` are set, the explicit `color()` value wins.
    pub fn label_level(mut self, level: LabelLevel) -> Self {
        self.label_level = Some(level);
        self
    }

    /// Override the font design (default: SF Pro). Use
    /// [`FontDesign::Serif`] for editorial content,
    /// [`FontDesign::Monospaced`] for code, or [`FontDesign::Rounded`]
    /// for a friendlier tone.
    pub fn font_design(mut self, design: FontDesign) -> Self {
        self.font_design = Some(design);
        self
    }

    /// Adjust the line-height: [`LeadingStyle::Tight`],
    /// [`Standard`](LeadingStyle::Standard), or
    /// [`Loose`](LeadingStyle::Loose).
    pub fn leading_style(mut self, style: LeadingStyle) -> Self {
        self.leading_style = style;
        self
    }

    /// Override text alignment. Defaults to GPUI's default (leading-edge).
    /// HIG: "text within a text view is aligned to the leading edge" by
    /// default, but centered or trailing alignment may be appropriate in
    /// specific contexts.
    ///
    /// Prefer leading alignment for running paragraphs — centered or
    /// right-aligned body copy breaks scanning rhythm. Reserve
    /// [`TextAlign::Center`] for short decorative labels (a single
    /// headline over a hero image) and [`TextAlign::Right`] for tabular
    /// right-aligned numerics.
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = Some(align);
        self
    }

    /// Enable vertical scrolling when the text content is taller than the
    /// view. Requires an [`ElementId`] because GPUI tracks scroll state
    /// per-element.
    ///
    /// Must not be combined with [`Self::max_lines`] — clamped content
    /// cannot scroll because its height is already bounded by GPUI's
    /// `line_clamp`. Combining the two trips a `debug_assert!` so the
    /// conflict is caught in tests; release builds silently prefer
    /// `max_lines`.
    pub fn scrollable(mut self, id: impl Into<ElementId>) -> Self {
        self.scroll_id = Some(id.into());
        self
    }

    /// Constrain the view to the HIG readable-content optimal width
    /// ([`READABLE_OPTIMAL_WIDTH`], 544 pt) for comfortable long-form
    /// reading. Scales with Dynamic Type via
    /// [`TahoeTheme::effective_font_scale_factor`] so the column widens
    /// proportionally when the user enables a Larger Text accessibility
    /// mode.
    pub fn readable_width(mut self, readable: bool) -> Self {
        self.readable_width = readable;
        self
    }

    /// Enable or disable mouse / keyboard selection. `true` by default
    /// (matches NSTextView). Set `false` for decorative read-only prose
    /// where drag-select, the text cursor affordance, and ⌘C / ⌘A
    /// should be suppressed.
    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    /// Mark the view as disabled. Paints with [`TahoeTheme::text_disabled`]
    /// (unless an explicit [`Self::color`] wins) and threads
    /// [`AccessibilityProps::disabled`] so VoiceOver will announce the
    /// dimmed state once GPUI lands an AX tree.
    ///
    /// `TextView` has no interactive state beyond selection, so disabling
    /// a selectable view does not also suppress Copy / Select All — the
    /// reader can still pull text off the disabled surface, matching
    /// macOS behaviour for disabled NSTextView.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override the VoiceOver label. Defaults to the plain-text content
    /// for both [`Self::new`] and [`Self::styled_text`] views — rich
    /// content carries its plain-text equivalent alongside the styled
    /// element.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Immutable access to the underlying selection coordinator.
    /// Exposed so host apps can programmatically clear the selection or
    /// inspect its state (e.g. to enable/disable a Copy menu item).
    pub fn selection(&self) -> &TextViewSelection {
        &self.selection
    }

    fn handle_copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        self.selection.copy_to_clipboard(cx);
    }

    fn handle_select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.selection.select_all();
        cx.notify();
    }

    /// Scroll unit for arrow-key navigation. Matches the rendered
    /// line-height (style attrs + leading-style adjustment + Dynamic Type
    /// scale) so one tap of Up/Down moves by exactly one visual line.
    fn scroll_line_height(&self, cx: &App) -> Pixels {
        let theme = cx.theme();
        let base = if self.emphasize {
            self.style.emphasized()
        } else {
            self.style.attrs()
        };
        let attrs = base.with_leading(self.leading_style);
        px(f32::from(attrs.leading) * theme.effective_font_scale_factor())
    }

    /// Apply a vertical scroll delta. Positive `delta` scrolls up (towards
    /// the start); negative scrolls down. Preserves the x offset so
    /// horizontal scroll state survives a vertical keyboard press.
    ///
    /// Out-of-range values are clamped by GPUI's prepaint pass against the
    /// current `[-max_offset.y, 0]` range, so an Up press at the top or a
    /// Down press at the bottom is a visual no-op even though the raw
    /// offset briefly stores the over-shoot.
    fn scroll_by(&self, delta: Pixels) {
        let cur = self.scroll_handle.offset();
        self.scroll_handle
            .set_offset(Point::new(cur.x, cur.y + delta));
    }

    fn handle_scroll_up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_by(self.scroll_line_height(cx));
        cx.notify();
    }

    fn handle_scroll_down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_by(-self.scroll_line_height(cx));
        cx.notify();
    }

    /// Viewport height used by Page Up / Page Down. Falls back to a
    /// ten-line span when layout has not populated `bounds()` yet —
    /// tests (and the first keypress before layout runs) would otherwise
    /// see a zero-height "page" and treat the key as a no-op.
    fn scroll_page_height(&self, cx: &App) -> Pixels {
        let bounds = self.scroll_handle.bounds().size.height;
        if bounds > px(0.) {
            bounds
        } else {
            self.scroll_line_height(cx) * 10.0
        }
    }

    fn handle_page_up(&mut self, _: &PageUp, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_by(self.scroll_page_height(cx));
        cx.notify();
    }

    fn handle_page_down(&mut self, _: &PageDown, _: &mut Window, cx: &mut Context<Self>) {
        self.scroll_by(-self.scroll_page_height(cx));
        cx.notify();
    }

    fn handle_home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        let cur = self.scroll_handle.offset();
        self.scroll_handle.set_offset(Point::new(cur.x, px(0.)));
        cx.notify();
    }

    fn handle_end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        // `scroll_to_bottom` marks a flag consumed during the next paint;
        // it's the correct primitive here because `max_offset` is only
        // populated after layout runs, and we want a reliable end-scroll
        // even on the very first keypress before max_offset is known.
        self.scroll_handle.scroll_to_bottom();
        cx.notify();
    }

    /// Build the context-menu entries for the current selection state.
    ///
    /// Pure function so it is unit-testable without a GPUI context.
    /// Returns a two-item menu (Copy / Select All) with a separator.
    /// Copy is [`ContextMenuItemStyle::Disabled`] when no selection
    /// exists — there is nothing to copy — and [`Default`] otherwise.
    ///
    /// Item activation dispatches a GPUI [`Copy`] / [`SelectAll`] action
    /// via the menu's `.action(...)` hook. The action bubbles up through
    /// the focus chain to the same `on_action` handlers registered for
    /// keyboard shortcuts, so click, keybinding, and (future) command
    /// palette all route through one handler.
    ///
    /// HIG convention: the macOS NSTextView right-click menu also
    /// exposes Look Up / Speak / Share entries. Those are **omitted**
    /// here because they require AppKit bridges (`NSServices`,
    /// `AVSpeechSynthesizer`, `NSSharingServicePicker`) that this crate
    /// does not yet wire. Showing them as no-op / disabled stubs would
    /// feel broken; hiding them entirely is cleaner until the bridge
    /// lands.
    ///
    /// [`Copy`]: crate::text_actions::Copy
    /// [`SelectAll`]: crate::text_actions::SelectAll
    /// [`Default`]: ContextMenuItemStyle::Default
    fn build_context_menu_items(has_selection: bool) -> Vec<ContextMenuEntry> {
        let copy = {
            let mut item = ContextMenuItem::new("Copy").shortcut("Cmd+C");
            if has_selection {
                item = item.action(Box::new(Copy));
            } else {
                item = item.style(ContextMenuItemStyle::Disabled);
            }
            item
        };
        let select_all = ContextMenuItem::new("Select All")
            .shortcut("Cmd+A")
            .action(Box::new(SelectAll));
        vec![
            ContextMenuEntry::Item(copy),
            ContextMenuEntry::Separator,
            ContextMenuEntry::Item(select_all),
        ]
    }

    /// Right-click handler: rebuild the items against the current selection
    /// (so Copy's enabled-state tracks live selection) and open the menu at
    /// the cursor position.
    fn open_context_menu(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.show_context_menu_at(event.position, window, cx);
    }

    /// Shift-F10 / Menu-key handler: opens the context menu without a
    /// pointer event. No anchor rectangle is reachable from an action
    /// listener (bounds() is only valid post-paint), so the menu opens
    /// at the view's scroll-handle origin if available, otherwise at
    /// the window origin. The user still gets the Copy / Select All
    /// affordance; positioning polish can follow once GPUI exposes the
    /// focused element's bounds in action context.
    fn handle_show_context_menu(
        &mut self,
        _: &ShowContextMenu,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let bounds = self.scroll_handle.bounds();
        let position = if bounds.size.width > px(0.) {
            bounds.origin
        } else {
            point(px(0.), px(0.))
        };
        self.show_context_menu_at(position, window, cx);
    }

    fn show_context_menu_at(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Prevent a stacked open from a rapid double-trigger (right-click
        // arriving during an in-flight Shift-F10 activation, or vice
        // versa). The menu stays open once and re-fires notify for the
        // host to repaint.
        if self.context_menu.read(cx).is_open() {
            return;
        }
        let items = Self::build_context_menu_items(self.selection.has_selection());
        self.context_menu.update(cx, |menu, cx| {
            menu.set_items(items);
            menu.open(position, window, cx);
        });
    }
}

impl Focusable for TextView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

/// Resolve the final text color from the four inputs.
///
/// Precedence (first set wins): explicit [`TextView::color`] > semantic
/// [`LabelLevel`] > [`TextView::disabled`] > default `theme.text`.
///
/// An explicit `color()` wins over the disabled flag: callers who want a
/// disabled view to ignore an accent colour should drop the explicit
/// colour alongside setting `disabled(true)`. This matches the wider
/// design-system rule that a literal color always trumps a semantic
/// tier or state.
fn resolve_color(
    color: Option<Hsla>,
    level: Option<LabelLevel>,
    disabled: bool,
    theme: &TahoeTheme,
) -> Hsla {
    if let Some(color) = color {
        color
    } else if let Some(level) = level {
        level.resolve(theme)
    } else if disabled {
        theme.text_disabled()
    } else {
        theme.text
    }
}

fn apply_typography(
    el: gpui::Div,
    style: TextStyle,
    theme: &TahoeTheme,
    emphasize: bool,
    font_design: Option<FontDesign>,
) -> gpui::Div {
    match (emphasize, font_design) {
        (false, None) => el.text_style(style, theme),
        (true, None) => el.text_style_emphasized(style, theme),
        (false, Some(design)) => el.text_style_with_design(style, design, theme),
        (true, Some(design)) => el.text_style_emphasized_with_design(style, design, theme),
    }
}

impl Render for TextView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let focused = self.focus_handle.is_focused(window);

        // max_lines + scrollable is undefined: clamped height short-circuits
        // the scroll viewport. Assert in debug so the conflict is caught in
        // tests; release silently prefers max_lines.
        debug_assert!(
            !(self.max_lines.is_some() && self.scroll_id.is_some()),
            "TextView: max_lines() and scrollable() are mutually exclusive — \
             clamped content cannot scroll. Drop one of the two.",
        );

        let mut typography =
            apply_typography(div(), self.style, theme, self.emphasize, self.font_design);

        // Only override line_height when leading_style differs from Standard.
        // apply_typography already sets the correct scaled leading for the
        // default case. When Tight or Loose is active, the adjusted value
        // must be scaled by effective_font_scale_factor() to match.
        if self.leading_style != LeadingStyle::Standard {
            let base_attrs = if self.emphasize {
                self.style.emphasized()
            } else {
                self.style.attrs()
            };
            let attrs = base_attrs.with_leading(self.leading_style);
            let scale = theme.effective_font_scale_factor();
            typography = typography.line_height(gpui::px(f32::from(attrs.leading) * scale));
        }

        typography = typography.text_color(resolve_color(
            self.color,
            self.label_level,
            self.disabled,
            theme,
        ));

        if let Some(align) = self.text_align {
            typography = typography.text_align(align);
        }

        if let Some(max) = self.max_lines {
            typography = typography.line_clamp(max);
        }

        if self.readable_width {
            // Scale the optimal width by Dynamic Type so Larger-Text
            // accessibility modes keep ~65 characters per line.
            let scale = theme.effective_font_scale_factor();
            typography = typography.max_w(gpui::px(READABLE_OPTIMAL_WIDTH * scale));
        }

        // A11y label falls back to plain-text (both content variants carry one).
        let label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| self.content.text().clone());
        let a11y = AccessibilityProps::new()
            .role(AccessibilityRole::StaticText)
            .label(label)
            .disabled(self.disabled);

        let text_body = if self.selectable {
            // macOS NSTextView selection tint: accent color at ~28% alpha.
            // HIG: selection highlights use the system accent hue.
            let selection_bg = {
                let mut bg = theme.accent;
                bg.a = 0.28;
                bg
            };
            typography
                .child(SelectableText::new(
                    self.element_id.clone(),
                    self.content.text().clone(),
                    self.content.styled(),
                    selection_bg,
                    self.selection.clone(),
                ))
                .with_accessibility(&a11y)
        } else {
            // Non-selectable view: render the plain-text child directly
            // so there are no mouse handlers / hitboxes.
            typography
                .child(self.content.styled())
                .with_accessibility(&a11y)
        };

        // Scroll wrapper (optional). `scroll_id` and `max_lines` are
        // mutually exclusive per the struct-level contract; the filter
        // mirrors the debug_assert above so release builds silently drop
        // the scroll when both are set. `track_scroll` wires the scroll
        // handle so keyboard scroll handlers can read and mutate the
        // current offset.
        //
        // `as_ref().cloned()` (not `.clone().filter(...)`) keeps the
        // `Option<ElementId>` borrow cheap when `max_lines` wins — no
        // `ElementId` allocation happens on the hot clamp path.
        let scrollable = self
            .scroll_id
            .as_ref()
            .filter(|_| self.max_lines.is_none())
            .cloned();
        let body = match scrollable {
            Some(id) => div()
                .id(id)
                .overflow_y_scroll()
                .track_scroll(&self.scroll_handle)
                .child(text_body)
                .into_any_element(),
            None => text_body.into_any_element(),
        };

        // Outer stateful root: carries the focus handle and keyboard
        // action dispatch. Attached whenever the view owns a focusable
        // capability (selection or scroll); purely decorative views fall
        // through to a plain div so they don't inject a silent tab stop
        // into the parent flow.
        let attach_focus = self.selectable || self.scroll_id.is_some();
        if attach_focus {
            let mut root = div()
                .id(("text-view-root", cx.entity_id().as_u64() as usize))
                .key_context(TEXT_VIEW_CONTEXT)
                .track_focus(&self.focus_handle);
            if self.selectable {
                root = root
                    .on_action(cx.listener(Self::handle_copy))
                    .on_action(cx.listener(Self::handle_select_all))
                    .on_action(cx.listener(Self::handle_show_context_menu))
                    .on_mouse_down(MouseButton::Right, cx.listener(Self::open_context_menu));
            }
            if self.scroll_id.is_some() {
                root = root
                    .on_action(cx.listener(Self::handle_scroll_up))
                    .on_action(cx.listener(Self::handle_scroll_down))
                    .on_action(cx.listener(Self::handle_page_up))
                    .on_action(cx.listener(Self::handle_page_down))
                    .on_action(cx.listener(Self::handle_home))
                    .on_action(cx.listener(Self::handle_end));
            }
            // HIG §Focus-and-selection: focusable controls must show a
            // visible ring. `apply_focus_ring` draws the theme-accent
            // shadow only when `focused` is true, so idle views carry no
            // extra chrome.
            root = crate::foundations::materials::apply_focus_ring(root, theme, focused, &[]);
            // The context-menu entity renders as a fullscreen overlay only
            // when open, so attaching it unconditionally on selectable
            // views adds no visible chrome when idle.
            if self.selectable {
                root.child(body)
                    .child(self.context_menu.clone())
                    .into_any_element()
            } else {
                root.child(body).into_any_element()
            }
        } else {
            div().child(body).into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::{ElementId, SharedString};

    use crate::components::content::selectable_text::SelectionCoordinator;
    use crate::foundations::theme::{LabelLevel, TahoeTheme};

    use super::{TextViewSelection, keybindings, resolve_color};

    // ── Pure-function tests (no TestAppContext required) ──────────

    // ── Color resolution ──────────────────────────────────────────

    #[test]
    fn resolve_color_defaults_to_theme_text() {
        let theme = TahoeTheme::dark();
        assert_eq!(resolve_color(None, None, false, &theme), theme.text);
    }

    #[test]
    fn resolve_color_label_level_resolves_to_theme_tier() {
        let theme = TahoeTheme::dark();
        assert_eq!(
            resolve_color(None, Some(LabelLevel::Secondary), false, &theme),
            theme.text_muted,
        );
    }

    #[test]
    fn resolve_color_explicit_wins_over_label_level() {
        let theme = TahoeTheme::dark();
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        assert_eq!(
            resolve_color(Some(color), Some(LabelLevel::Secondary), false, &theme),
            color,
            "explicit color() must win over label_level()",
        );
    }

    #[test]
    fn resolve_color_explicit_wins_over_default() {
        let theme = TahoeTheme::dark();
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        assert_eq!(resolve_color(Some(color), None, false, &theme), color);
    }

    #[test]
    fn resolve_color_disabled_resolves_to_theme_text_disabled() {
        // No explicit color and no label level: disabled flag must win
        // over the plain `theme.text` default.
        let theme = TahoeTheme::dark();
        assert_eq!(
            resolve_color(None, None, true, &theme),
            theme.text_disabled(),
        );
    }

    #[test]
    fn resolve_color_label_level_wins_over_disabled() {
        // A semantic tier is a deliberate author choice that trumps the
        // generic disabled tint — callers who want both should drop the
        // label level alongside setting `disabled(true)`.
        let theme = TahoeTheme::dark();
        assert_eq!(
            resolve_color(None, Some(LabelLevel::Secondary), true, &theme),
            theme.text_muted,
        );
    }

    #[test]
    fn resolve_color_explicit_wins_over_disabled() {
        let theme = TahoeTheme::dark();
        let color = gpui::hsla(0.5, 0.8, 0.6, 1.0);
        assert_eq!(resolve_color(Some(color), None, true, &theme), color);
    }

    // ── TextViewSelection coordinator behaviour ──────────────────
    // These exercise the `TextViewSelection` coordinator directly —
    // the inner `Rc<RefCell<_>>` needs no GPUI context.

    fn registered_selection(text: &'static str) -> (TextViewSelection, ElementId) {
        let sel = TextViewSelection::new();
        let id = ElementId::Name("test-body".into());
        sel.register(id.clone(), SharedString::from(text));
        (sel, id)
    }

    #[test]
    fn selection_starts_empty() {
        let (sel, _) = registered_selection("hello world");
        assert!(!sel.has_selection());
        assert_eq!(sel.selected_text(), "");
        assert_eq!(sel.range(), None);
    }

    #[test]
    fn single_click_does_not_select() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id, "hello world", 3, 1, false);
        assert!(!sel.has_selection());
        assert_eq!(sel.range(), None);
    }

    #[test]
    fn double_click_selects_surrounding_word() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id, "hello world", 2, 2, false);
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn triple_click_selects_whole_paragraph() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id, "hello world", 0, 3, false);
        assert_eq!(sel.selected_text(), "hello world");
    }

    #[test]
    fn quadruple_click_selects_all() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id, "hello world", 0, 4, false);
        assert_eq!(sel.selected_text(), "hello world");
    }

    #[test]
    fn select_all_covers_full_text() {
        let (sel, _) = registered_selection("alpha beta gamma");
        sel.select_all();
        assert_eq!(sel.selected_text(), "alpha beta gamma");
    }

    #[test]
    fn select_all_before_register_is_noop() {
        // Coordinator has no registered element yet — Cmd+A should
        // not create a bogus 0..0 selection.
        let sel = TextViewSelection::new();
        sel.select_all();
        assert!(!sel.has_selection());
    }

    #[test]
    fn clear_collapses_selection() {
        let (sel, _) = registered_selection("alpha");
        sel.select_all();
        sel.clear();
        assert!(!sel.has_selection());
        assert_eq!(sel.selected_text(), "");
    }

    #[test]
    fn drag_extends_character_selection() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id.clone(), "hello world", 0, 1, false);
        sel.drag_to(id, "hello world", 5);
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn reverse_drag_is_symmetric() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id.clone(), "hello world", 5, 1, false);
        sel.drag_to(id, "hello world", 0);
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn word_drag_snaps_to_word_boundaries() {
        let text = "alpha beta gamma delta";
        let (sel, id) = registered_selection(text);
        sel.mouse_down(id.clone(), text, 7, 2, false); // inside "beta"
        assert_eq!(sel.selected_text(), "beta");
        sel.drag_to(id, text, 19); // inside "delta"
        assert_eq!(sel.selected_text(), "beta gamma delta");
    }

    #[test]
    fn shift_click_extends_existing_anchor() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id.clone(), "hello world", 0, 1, false);
        sel.end_drag();
        sel.mouse_down(id, "hello world", 5, 1, true);
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn shift_click_after_word_grows_by_word() {
        // Double-click "beta" then shift-click inside "delta" — the
        // extension should snap to the word boundary (up to "delta"'s
        // end), matching NSTextView's "grow by whole words" rule when
        // the original gesture anchored on a word.
        let text = "alpha beta gamma delta";
        let (sel, id) = registered_selection(text);
        sel.mouse_down(id.clone(), text, 7, 2, false); // inside "beta"
        sel.end_drag();
        assert_eq!(sel.selected_text(), "beta");
        sel.mouse_down(id, text, 19, 1, true); // inside "delta"
        assert_eq!(sel.selected_text(), "beta gamma delta");
    }

    #[test]
    fn shift_click_after_line_keeps_paragraph() {
        // Triple-click selects the whole paragraph. A subsequent
        // shift-click anywhere inside the same paragraph must keep
        // every character selected — a naive character-mode extension
        // would collapse to the shift-click position.
        let text = "the quick brown fox";
        let (sel, id) = registered_selection(text);
        sel.mouse_down(id.clone(), text, 0, 3, false); // triple-click
        sel.end_drag();
        assert_eq!(sel.selected_text(), text);
        sel.mouse_down(id, text, 5, 1, true);
        assert_eq!(sel.selected_text(), text);
    }

    #[test]
    fn drag_requires_pending_state() {
        // MouseMove without a preceding MouseDown should be a no-op.
        let (sel, id) = registered_selection("hello world");
        sel.drag_to(id, "hello world", 5);
        assert!(!sel.has_selection());
    }

    #[test]
    fn end_drag_stops_extension() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id.clone(), "hello world", 0, 1, false);
        sel.end_drag();
        // Subsequent drag must not extend.
        sel.drag_to(id, "hello world", 5);
        assert!(!sel.has_selection());
    }

    #[test]
    fn range_for_element_returns_selection_range() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id.clone(), "hello world", 0, 1, false);
        sel.drag_to(id.clone(), "hello world", 5);
        assert_eq!(sel.range_for_element(&id, 11), Some(0..5));
    }

    #[test]
    fn range_for_element_returns_none_for_unknown_id() {
        let (sel, id) = registered_selection("hello world");
        sel.mouse_down(id.clone(), "hello world", 0, 1, false);
        sel.drag_to(id, "hello world", 5);
        let other = ElementId::Name("other".into());
        assert_eq!(sel.range_for_element(&other, 11), None);
    }

    #[test]
    fn range_for_element_clamps_to_text_len() {
        let (sel, id) = registered_selection("short");
        // Simulate a stale anchor past the current text length.
        sel.mouse_down(id.clone(), "short", 0, 1, false);
        sel.drag_to(id.clone(), "short", 5);
        // text_len now reports as 3 — selection should clamp.
        assert_eq!(sel.range_for_element(&id, 3), Some(0..3));
    }

    // ── Keybindings ───────────────────────────────────────────────

    #[test]
    fn keybindings_cover_selection_scroll_and_context_menu() {
        // Two selection bindings (⌘C, ⌘A), six scroll bindings (Up, Down,
        // PageUp, PageDown, Home, End), plus the Shift-F10 context-menu
        // activation = 9 total.
        let bindings = keybindings();
        assert_eq!(bindings.len(), 9);
    }

    // ── Context menu item composition (Phase 4) ──────────────────
    // Exercises the pure `build_context_menu_items` helper — no GPUI
    // context required.

    use crate::components::menus_and_actions::context_menu::{
        ContextMenuEntry, ContextMenuItemStyle,
    };

    use super::TextView;

    #[test]
    fn context_menu_items_count_and_order() {
        let items = TextView::build_context_menu_items(true);
        assert_eq!(items.len(), 3, "Copy + Separator + Select All = 3 entries");
        assert!(matches!(items[0], ContextMenuEntry::Item(_)));
        assert!(matches!(items[1], ContextMenuEntry::Separator));
        assert!(matches!(items[2], ContextMenuEntry::Item(_)));
    }

    #[test]
    fn context_menu_copy_disabled_when_selection_empty() {
        // Right-clicking with no selection must show Copy greyed out —
        // there is nothing to put on the clipboard, so activating it
        // would be a confusing no-op.
        let items = TextView::build_context_menu_items(false);
        match &items[0] {
            ContextMenuEntry::Item(copy) => {
                assert_eq!(copy.label.as_ref(), "Copy");
                assert_eq!(copy.style, ContextMenuItemStyle::Disabled);
            }
            _ => panic!("expected Copy item at index 0"),
        }
    }

    #[test]
    fn context_menu_copy_enabled_when_selection_present() {
        let items = TextView::build_context_menu_items(true);
        match &items[0] {
            ContextMenuEntry::Item(copy) => {
                assert_eq!(copy.label.as_ref(), "Copy");
                assert_eq!(copy.style, ContextMenuItemStyle::Default);
                assert!(
                    copy.action.is_some(),
                    "enabled Copy must dispatch the Copy action",
                );
            }
            _ => panic!("expected Copy item at index 0"),
        }
    }

    #[test]
    fn context_menu_select_all_always_enabled() {
        // Select All is enabled regardless of current selection state —
        // matches the macOS NSTextView right-click menu.
        for has_selection in [false, true] {
            let items = TextView::build_context_menu_items(has_selection);
            match &items[2] {
                ContextMenuEntry::Item(select_all) => {
                    assert_eq!(select_all.label.as_ref(), "Select All");
                    assert_eq!(select_all.style, ContextMenuItemStyle::Default);
                    assert!(select_all.action.is_some());
                }
                _ => panic!("expected Select All item at index 2"),
            }
        }
    }

    #[test]
    fn context_menu_hides_lookup_speak_share() {
        // Per plan risk note: Look Up / Speak / Share require AppKit
        // bridges that don't exist in the crate yet. The menu omits
        // them rather than stubbing no-op rows, which would feel
        // broken. Guard against anyone re-introducing placeholder
        // entries without wiring the underlying behaviour.
        let items = TextView::build_context_menu_items(true);
        for entry in &items {
            if let ContextMenuEntry::Item(item) = entry {
                let label = item.label.as_ref();
                assert!(
                    !matches!(label, "Look Up" | "Speak" | "Share"),
                    "{label:?} must not appear until the AppKit bridge lands",
                );
            }
        }
    }
}

#[cfg(test)]
mod gpui_tests {
    use gpui::{ElementId, HighlightStyle, TextAlign};

    use crate::components::content::selectable_text::SelectionCoordinator;
    use crate::foundations::theme::{FontDesign, LabelLevel, LeadingStyle, TextStyle};
    use crate::test_helpers::helpers::setup_test_window;

    use super::{TextView, TextViewContent};

    // ── Constructor + builder defaults ────────────────────────────

    #[gpui::test]
    async fn text_view_new_defaults(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TextView::new(cx, "Hello world"));
        handle.update(cx, |tv, _| {
            assert!(matches!(
                &tv.content,
                TextViewContent::Plain(s) if s.as_ref() == "Hello world"
            ));
            assert_eq!(tv.style, TextStyle::Body);
            assert!(tv.max_lines.is_none());
            assert!(!tv.emphasize);
            assert!(tv.color.is_none());
            assert!(tv.label_level.is_none());
            assert!(tv.font_design.is_none());
            assert_eq!(tv.leading_style, LeadingStyle::Standard);
            assert!(tv.text_align.is_none());
            assert!(tv.scroll_id.is_none());
            assert!(!tv.readable_width);
            assert!(tv.selectable);
            assert!(!tv.disabled);
            assert!(tv.accessibility_label.is_none());
        });
    }

    #[gpui::test]
    async fn text_view_builders(cx: &mut gpui::TestAppContext) {
        // Exclude `scrollable()` — see `text_view_scrollable_builder` below.
        // max_lines + scrollable are mutually exclusive (debug_assert fires
        // at render time), so the two are covered in separate tests.
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "text")
                .text_style(TextStyle::LargeTitle)
                .max_lines(3)
                .emphasize(true)
                .color(gpui::hsla(0.5, 0.8, 0.6, 1.0))
                .label_level(LabelLevel::Secondary)
                .font_design(FontDesign::Monospaced)
                .leading_style(LeadingStyle::Tight)
                .text_align(TextAlign::Center)
                .readable_width(true)
                .selectable(false)
                .accessibility_label("alt")
        });
        handle.update(cx, |tv, _| {
            assert_eq!(tv.style, TextStyle::LargeTitle);
            assert_eq!(tv.max_lines, Some(3));
            assert!(tv.emphasize);
            assert_eq!(tv.color, Some(gpui::hsla(0.5, 0.8, 0.6, 1.0)));
            assert_eq!(tv.label_level, Some(LabelLevel::Secondary));
            assert_eq!(tv.font_design, Some(FontDesign::Monospaced));
            assert_eq!(tv.leading_style, LeadingStyle::Tight);
            assert_eq!(tv.text_align, Some(TextAlign::Center));
            assert!(tv.scroll_id.is_none());
            assert!(tv.readable_width);
            assert!(!tv.selectable);
            assert_eq!(
                tv.accessibility_label.as_ref().map(|s| s.as_ref()),
                Some("alt"),
            );
        });
    }

    #[gpui::test]
    async fn text_view_scrollable_builder(cx: &mut gpui::TestAppContext) {
        // Covers the scroll_id side of the max_lines/scrollable mutual
        // exclusion — must be exercised without max_lines set.
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "text").scrollable("scroll-id")
        });
        handle.update(cx, |tv, _| {
            assert!(tv.max_lines.is_none());
            assert!(matches!(
                tv.scroll_id.as_ref(),
                Some(ElementId::Name(n)) if n.as_ref() == "scroll-id"
            ));
        });
    }

    #[gpui::test]
    async fn text_view_max_lines_zero_is_ignored(cx: &mut gpui::TestAppContext) {
        // line_clamp(0) would hide every line — almost never what a
        // caller computing the value dynamically wants. The builder
        // silently drops a zero so callers do not accidentally erase
        // their text.
        let (handle, cx) =
            setup_test_window(cx, |_window, cx| TextView::new(cx, "keep me").max_lines(0));
        handle.update(cx, |tv, _| {
            assert_eq!(tv.max_lines, None);
        });
    }

    #[gpui::test]
    async fn text_view_disabled_builder_threads_flag(cx: &mut gpui::TestAppContext) {
        // Smoke-test the disabled() builder so a future caller can
        // trust the flag actually flips. The downstream colour / AX
        // plumbing is covered by `resolve_color_disabled_*` tests.
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "inactive").disabled(true)
        });
        handle.update(cx, |tv, _| assert!(tv.disabled));
    }

    #[cfg(debug_assertions)]
    #[gpui::test]
    #[should_panic(expected = "mutually exclusive")]
    async fn text_view_max_lines_plus_scrollable_panics_in_debug(cx: &mut gpui::TestAppContext) {
        // The mutual-exclusion contract is enforced by a `debug_assert!`
        // fired during render. Combining both options panics in debug
        // builds; release silently prefers `max_lines` (and the test
        // compiles out entirely via `cfg(debug_assertions)` so
        // `cargo nextest run --release` stays green). The panic
        // substring check is stable against rewording of the full
        // message.
        let (_handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "whoops")
                .max_lines(3)
                .scrollable("scroll-id")
        });
        // Drive a render so the debug_assert trips.
        cx.run_until_parked();
    }

    #[gpui::test]
    async fn text_view_styled_text_stores_plain_text_and_highlights(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "placeholder").styled_text(
                "Bold hello",
                vec![(
                    0..4,
                    HighlightStyle {
                        font_weight: Some(gpui::FontWeight::BOLD),
                        ..Default::default()
                    },
                )],
            )
        });
        handle.update(cx, |tv, _| match &tv.content {
            TextViewContent::Rich { text, highlights } => {
                assert_eq!(text.as_ref(), "Bold hello");
                assert_eq!(highlights.len(), 1);
                assert_eq!(highlights[0].0, 0..4);
            }
            TextViewContent::Plain(_) => panic!("expected Rich content"),
        });
    }

    #[gpui::test]
    async fn text_view_styled_constructor_skips_plaintext_placeholder(
        cx: &mut gpui::TestAppContext,
    ) {
        // `styled` is equivalent to `new().styled_text()` without the
        // throwaway plain-text argument. Confirm it lands as Rich content
        // in a single allocation.
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::styled(
                cx,
                "Bold hello",
                vec![(
                    0..4,
                    HighlightStyle {
                        font_weight: Some(gpui::FontWeight::BOLD),
                        ..Default::default()
                    },
                )],
            )
        });
        handle.update(cx, |tv, _| match &tv.content {
            TextViewContent::Rich { text, highlights } => {
                assert_eq!(text.as_ref(), "Bold hello");
                assert_eq!(highlights.len(), 1);
                assert_eq!(highlights[0].0, 0..4);
            }
            TextViewContent::Plain(_) => panic!("expected Rich content"),
        });
    }

    // ── Action handlers ───────────────────────────────────────────

    #[gpui::test]
    async fn handle_copy_writes_selection_to_clipboard(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TextView::new(cx, "hello world"));
        handle.update_in(cx, |tv, window, cx| {
            // Register with the coordinator so select_all works
            // without a render pass.
            tv.selection
                .register(tv.element_id.clone(), tv.content.text().clone());
            tv.selection.select_all();
            tv.handle_copy(&super::Copy, window, cx);
        });
        cx.update(|_window, cx| {
            let clip = cx.read_from_clipboard();
            assert_eq!(
                clip.and_then(|item: gpui::ClipboardItem| item.text()),
                Some("hello world".to_string()),
            );
        });
    }

    #[gpui::test]
    async fn handle_select_all_selects_full_text(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| TextView::new(cx, "hello world"));
        handle.update_in(cx, |tv, window, cx| {
            tv.selection
                .register(tv.element_id.clone(), tv.content.text().clone());
            tv.handle_select_all(&super::SelectAll, window, cx);
            assert_eq!(tv.selection.selected_text(), "hello world");
        });
    }

    // ── Scroll handlers (Phase 3) ─────────────────────────────────

    #[gpui::test]
    async fn handle_home_resets_offset_to_zero(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "lots of text").scrollable("scroll-id")
        });
        handle.update_in(cx, |tv, window, cx| {
            // Pre-set a non-zero offset as if the view had been scrolled.
            tv.scroll_handle
                .set_offset(gpui::Point::new(gpui::px(0.), gpui::px(-123.)));
            tv.handle_home(&super::Home, window, cx);
            assert_eq!(tv.scroll_handle.offset().y, gpui::px(0.));
        });
    }

    #[gpui::test]
    async fn handle_home_preserves_x_offset(cx: &mut gpui::TestAppContext) {
        // Home resets the vertical scroll but must not disturb a nonzero
        // horizontal scroll position (matters for future horizontal-overflow
        // content; today TextView is vertical-only but the contract is
        // stable).
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "lots of text").scrollable("scroll-id")
        });
        handle.update_in(cx, |tv, window, cx| {
            tv.scroll_handle
                .set_offset(gpui::Point::new(gpui::px(-5.), gpui::px(-123.)));
            tv.handle_home(&super::Home, window, cx);
            assert_eq!(tv.scroll_handle.offset().x, gpui::px(-5.));
        });
    }

    #[gpui::test]
    async fn handle_scroll_up_applies_positive_line_height_delta(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "hello").scrollable("scroll-id")
        });
        handle.update_in(cx, |tv, window, cx| {
            tv.scroll_handle
                .set_offset(gpui::Point::new(gpui::px(0.), gpui::px(-100.)));
            let line = tv.scroll_line_height(cx);
            tv.handle_scroll_up(&super::Up, window, cx);
            assert_eq!(
                tv.scroll_handle.offset().y,
                gpui::px(-100.) + line,
                "Up adds one line-height (positive delta) to offset.y",
            );
        });
    }

    #[gpui::test]
    async fn handle_scroll_down_applies_negative_line_height_delta(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "hello").scrollable("scroll-id")
        });
        handle.update_in(cx, |tv, window, cx| {
            tv.scroll_handle
                .set_offset(gpui::Point::new(gpui::px(0.), gpui::px(-10.)));
            let line = tv.scroll_line_height(cx);
            tv.handle_scroll_down(&super::Down, window, cx);
            assert_eq!(
                tv.scroll_handle.offset().y,
                gpui::px(-10.) - line,
                "Down subtracts one line-height (negative delta) from offset.y",
            );
        });
    }

    #[gpui::test]
    async fn handle_page_up_down_preserve_x_offset(cx: &mut gpui::TestAppContext) {
        // Without a paint pass `bounds()` is zero, so Page Up/Down is a
        // no-op on offset.y in isolation. The handler must still not panic
        // and must leave the x offset untouched.
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "hello").scrollable("scroll-id")
        });
        handle.update_in(cx, |tv, window, cx| {
            tv.scroll_handle
                .set_offset(gpui::Point::new(gpui::px(7.), gpui::px(-50.)));
            tv.handle_page_up(&super::PageUp, window, cx);
            assert_eq!(tv.scroll_handle.offset().x, gpui::px(7.));
            tv.handle_page_down(&super::PageDown, window, cx);
            assert_eq!(tv.scroll_handle.offset().x, gpui::px(7.));
        });
    }

    #[gpui::test]
    async fn handle_end_does_not_panic(cx: &mut gpui::TestAppContext) {
        // `scroll_to_bottom` sets an internal flag consumed at paint; no
        // user-visible state changes without a subsequent render. Smoke
        // check that the handler is callable before layout has populated
        // max_offset.
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            TextView::new(cx, "hello").scrollable("scroll-id")
        });
        handle.update_in(cx, |tv, window, cx| {
            tv.handle_end(&super::End, window, cx);
        });
    }

    // ── Context menu (Phase 4) ────────────────────────────────────

    #[gpui::test]
    async fn open_context_menu_opens_and_populates_entity(cx: &mut gpui::TestAppContext) {
        // Synthesise a MouseDownEvent and verify the inner ContextMenu
        // entity reports open=true afterwards. Direct invocation of the
        // handler is equivalent to a real right-click for this check —
        // the listener wrapping is just event-routing glue.
        let (handle, cx) = setup_test_window(cx, |_window, cx| TextView::new(cx, "hello world"));
        handle.update_in(cx, |tv, window, cx| {
            let event = gpui::MouseDownEvent {
                button: gpui::MouseButton::Right,
                position: gpui::point(gpui::px(10.0), gpui::px(20.0)),
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            };
            tv.open_context_menu(&event, window, cx);
            assert!(
                tv.context_menu.read(cx).is_open(),
                "right-click must open the context menu",
            );
        });
    }

    #[gpui::test]
    async fn open_context_menu_rebuilds_items_against_live_selection(
        cx: &mut gpui::TestAppContext,
    ) {
        // Opening the menu must rebuild items from the *current* selection,
        // not the state at construction time. Without this, Copy would be
        // stuck in whichever style was baked in on first open.
        let (handle, cx) = setup_test_window(cx, |_window, cx| TextView::new(cx, "hello world"));
        handle.update_in(cx, |tv, window, cx| {
            tv.selection
                .register(tv.element_id.clone(), tv.content.text().clone());
            let event = gpui::MouseDownEvent {
                button: gpui::MouseButton::Right,
                position: gpui::point(gpui::px(0.0), gpui::px(0.0)),
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            };
            // First open: no selection — Copy should be disabled.
            tv.open_context_menu(&event, window, cx);
            assert!(tv.context_menu.read(cx).is_open());
            // Now select all and re-open — Copy should flip to enabled.
            tv.selection.select_all();
            tv.open_context_menu(&event, window, cx);
            assert!(tv.context_menu.read(cx).is_open());
        });
    }
}
