//! Selectable text element with cross-paragraph drag-select, link
//! click, and multi-click selection modes.
//!
//! Wraps [`StyledText`] into a custom [`gpui::Element`] that paints a
//! selection background before delegating the text paint, then
//! registers mouse handlers that talk to a shared
//! [`SelectionCoordinator`] so the selection can span multiple
//! paragraphs within the same host entity.
//!
//! Keyboard shortcuts (Cmd/Ctrl+C copy, Cmd/Ctrl+A select all) are
//! handled by the parent entity's `FocusHandle` rather than per-
//! paragraph handlers.
//!
//! # Supported gestures
//!
//! - **Single click** — anchor selection at the hit index.
//! - **Double click** — select the surrounding word.
//! - **Triple click** — select the entire paragraph.
//! - **Quadruple+ click** — select every registered paragraph.
//! - **Drag** — extend the selection by character, by word (after a
//!   double-click), or by paragraph (after a triple-click). Dragging
//!   across paragraph boundaries extends the selection across them
//!   (cross-paragraph selection is mediated by the coordinator).
//! - **Shift + click** — extend the current selection without
//!   moving the anchor.
//! - **Click on a link** — opens the URL via `cx.open_url` when the
//!   gesture is a click (no drag).
//!
//! # Pattern
//!
//! Mirrors Zed's [`InteractiveText`](gpui::InteractiveText) plus the
//! `paint_selection` quad pass in Zed's markdown
//! (`crates/markdown/src/markdown.rs:1022`). The cross-paragraph
//! coordination lives behind the [`SelectionCoordinator`] trait so
//! each paragraph can remain an independent GPUI element while still
//! participating in a host-wide selection. The markdown pipeline
//! implements this trait via [`crate::markdown::MarkdownSelection`];
//! [`crate::components::content::text_view::TextView`] implements it
//! via its own single-paragraph coordinator.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    App, Bounds, CursorStyle, DispatchPhase, ElementId, GlobalElementId, Hitbox, HitboxBehavior,
    Hsla, InspectorElementId, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, SharedString, StyledText, Window, fill, point,
};

/// Handler invoked when a reader clicks a `#fragment` link inside a
/// [`SelectableText`]. The first argument is the fragment string with
/// the leading `#` stripped and any percent-encoding already decoded.
/// Installed via [`SelectableText::with_anchor_click_handler`].
pub type AnchorClickHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;

/// Contract between [`SelectableText`] and the host entity that owns
/// the cross-paragraph selection state.
///
/// Each frame, every [`SelectableText`] paragraph calls
/// [`Self::register`] during paint so the coordinator knows the order
/// and identity of paragraphs participating in the current selection.
/// Mouse handlers then call [`Self::mouse_down`] / [`Self::drag_to`] /
/// [`Self::end_drag`] and the paint pass queries
/// [`Self::range_for_element`] to decide how much of each paragraph to
/// highlight.
///
/// Implementations must be cheap to clone — the trait object is
/// cloned into every mouse handler captured by every paragraph on
/// every frame.
pub trait SelectionCoordinator: Clone + 'static {
    /// Register a paragraph as part of the current frame. `element_id`
    /// must match the `Element::id()` of the [`SelectableText`] that
    /// owns the paragraph; `text` is the flat text (same index space
    /// as the paragraph's [`StyledText`]).
    fn register(&self, element_id: ElementId, text: SharedString);

    /// Return the portion of the selection that falls inside the
    /// paragraph identified by `element_id`, as a byte-index range
    /// into that paragraph's text. Returns `None` when the paragraph
    /// has no selection painted on it.
    fn range_for_element(&self, element_id: &ElementId, text_len: usize) -> Option<Range<usize>>;

    /// Dispatch a mouse-down at `(element_id, char_index)` within
    /// `text`. `click_count` and `shift` come from GPUI's
    /// `MouseDownEvent`.
    fn mouse_down(
        &self,
        element_id: ElementId,
        text: &str,
        char_index: usize,
        click_count: usize,
        shift: bool,
    );

    /// Extend the selection to the given position during a drag.
    /// No-op when no drag is in progress.
    fn drag_to(&self, element_id: ElementId, text: &str, char_index: usize);

    /// Called on `MouseUp` — finalises the drag so subsequent
    /// `MouseMove`s do not extend the selection.
    fn end_drag(&self);

    /// `true` while a drag is in progress (left button held after a
    /// `mouse_down`). Mouse-move handlers consult this before
    /// extending the selection.
    fn is_pending(&self) -> bool;

    /// Called once per frame *before* any paragraph re-registers via
    /// [`Self::register`]. Multi-paragraph coordinators
    /// ([`crate::markdown::MarkdownSelection`]) override this to flush
    /// the per-frame registration list so paragraphs that no longer
    /// exist in the new frame drop out of the selection rect.
    ///
    /// Single-paragraph coordinators (like
    /// [`crate::components::content::text_view::TextViewSelection`])
    /// deliberately keep the empty default: they own exactly one
    /// paragraph for their entire lifetime, so there is no list to
    /// flush and no stale registration to clear — the one `register`
    /// call per frame just re-pins the same `(element_id, text)`
    /// binding. Overriding to a non-empty body would be a no-op at
    /// best and risk dropping the only registration at worst.
    fn begin_frame(&self) {}
}

/// A [`StyledText`] wrapper that participates in a shared
/// [`SelectionCoordinator`]. See the module docs for gestures and
/// cross-paragraph semantics.
pub struct SelectableText<S: SelectionCoordinator> {
    element_id: ElementId,
    text: StyledText,
    text_string: SharedString,
    clickable_ranges: Vec<Range<usize>>,
    link_urls: Vec<SharedString>,
    selection_bg: Hsla,
    selection: S,
    anchor_click: Option<AnchorClickHandler>,
}

/// Per-element state persisted across frames. Only tracks whether the
/// mouse button is held down on this paragraph (used to disambiguate
/// drag vs. click on `MouseUp`); the actual selection range lives on
/// the shared [`SelectionCoordinator`].
#[derive(Default, Clone)]
struct SelectableTextState {
    mouse_down_index: Rc<Cell<Option<usize>>>,
}

impl<S: SelectionCoordinator> SelectableText<S> {
    /// Wraps a pre-built [`StyledText`] into a selectable element.
    ///
    /// `text` is the flat text behind the styled runs, used for
    /// clipboard copy and word-boundary detection. It must match the
    /// string the `StyledText` was constructed from; the element
    /// cannot read that back from `StyledText` directly at this GPUI
    /// version.
    ///
    /// `selection` is the shared coordinator — clone the same
    /// [`SelectionCoordinator`] into every paragraph that should
    /// participate in a single cross-paragraph selection.
    pub fn new(
        element_id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        styled: StyledText,
        selection_bg: Hsla,
        selection: S,
    ) -> Self {
        Self {
            element_id: element_id.into(),
            text: styled,
            text_string: text.into(),
            clickable_ranges: Vec::new(),
            link_urls: Vec::new(),
            selection_bg,
            selection,
            anchor_click: None,
        }
    }

    /// Attach clickable link ranges. `ranges[i]` is the byte-offset
    /// span in the text for the link; `urls[i]` is the destination
    /// passed to `cx.open_url` when the gesture is a click (no drag).
    /// Vectors must have equal length (debug-asserted).
    pub fn with_links(mut self, ranges: Vec<Range<usize>>, urls: Vec<SharedString>) -> Self {
        debug_assert_eq!(
            ranges.len(),
            urls.len(),
            "clickable_ranges and link_urls must match"
        );
        self.clickable_ranges = ranges;
        self.link_urls = urls;
        self
    }

    /// Install a handler for `#fragment` link clicks. When set, clicking a
    /// fragment URL invokes the handler instead of `cx.open_url` — the
    /// latter treats `#section` as an HTTP URL and silently fails. The
    /// handler receives the fragment string with the leading `#` stripped
    /// and any percent-encoding decoded.
    pub fn with_anchor_click_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.anchor_click = Some(Rc::new(handler));
        self
    }

    /// Variant of [`Self::with_anchor_click_handler`] that takes an
    /// already-wrapped handler. Used when forwarding a shared handler
    /// across many `SelectableText` elements rendered in one frame.
    pub fn with_anchor_click_handler_rc(mut self, handler: AnchorClickHandler) -> Self {
        self.anchor_click = Some(handler);
        self
    }
}

impl<S: SelectionCoordinator> Element for SelectableText<S> {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        Some(self.element_id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let (layout_id, _) = self.text.request_layout(None, inspector_id, window, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        state: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Hitbox {
        self.text
            .prepaint(None, inspector_id, bounds, state, window, cx);
        window.insert_hitbox(bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        hitbox: &mut Hitbox,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Register with the shared coordinator so cross-paragraph
        // logic can find this paragraph's text by element id.
        self.selection
            .register(self.element_id.clone(), self.text_string.clone());

        let text_layout = self.text.layout().clone();
        let text_string = self.text_string.clone();
        let selection = self.selection.clone();
        let selection_bg = self.selection_bg;
        // `std::mem::take` is safe here because the element is
        // consumed after this `paint()` — GPUI discards the element
        // tree once the frame finishes, so the moved-out `Vec`s would
        // be dropped regardless. Taking them lets the mouse closures
        // own the data outright instead of holding a second Rc/Arc
        // layer for the one-frame lifetime.
        let clickable_ranges = std::mem::take(&mut self.clickable_ranges);
        let link_urls = std::mem::take(&mut self.link_urls);
        let anchor_click = self.anchor_click.take();
        let element_id = self.element_id.clone();

        debug_assert!(
            global_id.is_some(),
            "SelectableText::paint: global_id is None despite id() returning Some"
        );
        // TODO(test): the global_id = None fallback is untested —
        // requires a GPUI window harness to call paint() directly.
        let Some(global_id) = global_id else {
            self.text
                .paint(None, inspector_id, bounds, &mut (), &mut (), window, cx);
            warn_global_id_missing_once();
            return;
        };

        window.with_element_state::<SelectableTextState, _>(global_id, |state, window| {
            let state: SelectableTextState = state.unwrap_or_default();

            // Cursor: PointingHand over clickable ranges, IBeam
            // over body text (matches NSTextView semantics).
            if hitbox.is_hovered(window) {
                let over_link = text_layout
                    .index_for_position(window.mouse_position())
                    .ok()
                    .is_some_and(|ix| clickable_ranges.iter().any(|r| r.contains(&ix)));
                let cursor = if over_link {
                    CursorStyle::PointingHand
                } else {
                    CursorStyle::IBeam
                };
                window.set_cursor_style(cursor, hitbox);
            }

            // Paint selection background BEFORE the text so the
            // text strokes sit on top of the tint (NSTextView
            // order).
            if let Some(range) = selection.range_for_element(&element_id, text_string.len()) {
                paint_selection_quads(
                    &text_layout,
                    range.start,
                    range.end,
                    bounds,
                    selection_bg,
                    window,
                );
            }

            // Delegate the text paint.
            self.text
                .paint(None, inspector_id, bounds, &mut (), &mut (), window, cx);

            // Each paragraph installs its own MouseDown / MouseMove /
            // MouseUp handlers at paint time. GPUI's `on_mouse_event`
            // already dispatches every event to every registered
            // handler, so each paragraph can hitbox-test itself in
            // isolation — the alternative (one coordinator-level
            // handler) would have to track every paragraph's
            // `text_layout` + `element_id` + hitbox across frames,
            // duplicating state the coordinator already normalises
            // via `SelectionCoordinator`. Per-paragraph wiring keeps
            // the layout / hitbox references local to the paint call
            // that produced them.

            // MouseDown — select, word-select, paragraph-select,
            // shift-extend, or arm a link click, depending on
            // click_count / modifiers.
            {
                let mouse_down_index = state.mouse_down_index.clone();
                let hitbox = hitbox.clone();
                let text_layout = text_layout.clone();
                let text_string = text_string.clone();
                let selection = selection.clone();
                let element_id = element_id.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, _cx| {
                    if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                        return;
                    }
                    if !hitbox.is_hovered(window) {
                        return;
                    }
                    let Ok(ix) = text_layout.index_for_position(event.position) else {
                        return;
                    };
                    // Only the single-click path can become a link
                    // navigation: setting `mouse_down_index` for a
                    // second click would mean the matching mouse-up
                    // (which still reports `down_ix == up_ix` for an
                    // in-place double-click) opens the URL while the
                    // coordinator simultaneously selects the word.
                    // Double / triple / quad clicks are gesture-only —
                    // they drive selection and must never navigate.
                    if event.click_count == 1 {
                        mouse_down_index.set(Some(ix));
                    } else {
                        mouse_down_index.set(None);
                    }
                    selection.mouse_down(
                        element_id.clone(),
                        &text_string,
                        ix,
                        event.click_count,
                        event.modifiers.shift,
                    );
                    window.refresh();
                });
            }

            // MouseMove while any paragraph's left button is
            // held (coordinator knows if a drag is pending, and
            // only handlers whose hitbox the cursor is over
            // contribute an index). Extends the selection
            // across paragraph boundaries naturally.
            {
                let hitbox = hitbox.clone();
                let text_layout = text_layout.clone();
                let text_string = text_string.clone();
                let selection = selection.clone();
                let element_id = element_id.clone();
                window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, _cx| {
                    if phase != DispatchPhase::Bubble {
                        return;
                    }
                    if !selection.is_pending() {
                        return;
                    }
                    if !hitbox.is_hovered(window) {
                        return;
                    }
                    let Ok(ix) = text_layout.index_for_position(event.position) else {
                        return;
                    };
                    selection.drag_to(element_id.clone(), &text_string, ix);
                    window.refresh();
                });
            }

            // MouseUp — finalise the drag or dispatch a link
            // click when the gesture was a click (no drag). Moves
            // `clickable_ranges` and `link_urls` straight into the
            // closure — MouseUp is their only remaining consumer, so
            // a defensive inner `.clone()` would just burn an alloc
            // per paint.
            {
                let mouse_down_index = state.mouse_down_index.clone();
                let hitbox = hitbox.clone();
                let text_layout = text_layout.clone();
                let selection = selection.clone();
                let anchor_click = anchor_click.clone();
                window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                    if phase != DispatchPhase::Bubble || event.button != MouseButton::Left {
                        return;
                    }
                    let Some(down_ix) = mouse_down_index.get() else {
                        return;
                    };
                    mouse_down_index.set(None);
                    selection.end_drag();
                    if !hitbox.is_hovered(window) {
                        window.refresh();
                        return;
                    }
                    let Ok(up_ix) = text_layout.index_for_position(event.position) else {
                        return;
                    };
                    if down_ix == up_ix
                        && let Some(url) = clickable_ranges
                            .iter()
                            .zip(link_urls.iter())
                            .find(|(range, _)| range.contains(&down_ix))
                            .map(|(_, url)| url.clone())
                    {
                        // In-document `#fragment` links never reach the
                        // OS URL handler — routing them through
                        // `cx.open_url` opens a broken HTTP URL. Invoke
                        // the consumer's anchor-click handler instead.
                        match fragment_of(url.as_ref()) {
                            Some(fragment) => {
                                // Defence in depth: fragment links
                                // are only registered as clickable
                                // when a handler is installed (the
                                // render-side gate lives in the
                                // markdown renderer). If we reach
                                // this arm without one the gate was
                                // bypassed.
                                debug_assert!(
                                    anchor_click.is_some(),
                                    "fragment link click reached without anchor_click handler"
                                );
                                if let Some(handler) = &anchor_click {
                                    handler(&fragment, window, cx);
                                }
                            }
                            None => {
                                cx.open_url(url.as_ref());
                            }
                        }
                    }
                    window.refresh();
                });
            }

            ((), state)
        })
    }
}

impl<S: SelectionCoordinator> IntoElement for SelectableText<S> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

/// Classify a link URL as either an in-document fragment or an external
/// link. Returns `Some(fragment)` with the leading `#` stripped and any
/// percent-encoding decoded — so `#my%20section` round-trips to the same
/// slug an auto-generated heading carries. Returns `None` for URLs that
/// do not begin with `#` (absolute URLs with a trailing `#frag` are
/// external — the OS URL handler resolves them correctly).
pub fn fragment_of(url: &str) -> Option<String> {
    url.strip_prefix('#').map(percent_decode_fragment)
}

/// Kind used by [`word_range_at`] to identify consecutive runs of
/// same-kind characters. `Word` covers Unicode alphanumerics and
/// underscores; `Whitespace` is `char::is_whitespace`; everything else
/// is `Punctuation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CharKind {
    Whitespace,
    Punctuation,
    Word,
}

impl CharKind {
    fn of(c: char) -> Self {
        if c.is_whitespace() {
            Self::Whitespace
        } else if c.is_alphanumeric() || c == '_' {
            Self::Word
        } else {
            Self::Punctuation
        }
    }

    /// Priority for picking the kind at a boundary: Word beats
    /// Punctuation which beats Whitespace — matches Zed's
    /// `CharClassifier` and macOS NSTextView double-click behaviour
    /// (prefer the word when hovering the boundary).
    fn priority(self) -> u8 {
        match self {
            Self::Whitespace => 0,
            Self::Punctuation => 1,
            Self::Word => 2,
        }
    }
}

/// Return the byte range of the "word" surrounding `index` in `text`.
/// A word is a maximal run of consecutive same-kind characters
/// (Word > Punctuation > Whitespace, with priority at boundaries).
///
/// `index` is clamped to `[0, text.len()]` and must lie on a UTF-8 char
/// boundary (as GPUI's `TextLayout::index_for_position` guarantees).
///
/// Shared by every in-crate [`SelectionCoordinator`] so double-click
/// gesture semantics stay identical across markdown, text views, and
/// any future selectable-text surface.
pub fn word_range_at(text: &str, index: usize) -> Range<usize> {
    let index = index.min(text.len());
    if text.is_empty() {
        return 0..0;
    }
    if !text.is_char_boundary(index) {
        // Defensive: the caller should never pass a non-boundary index,
        // but if they do, snap to the nearest previous boundary.
        let mut snapped = index;
        while snapped > 0 && !text.is_char_boundary(snapped) {
            snapped -= 1;
        }
        return word_range_at(text, snapped);
    }

    let before = &text[..index];
    let after = &text[index..];

    let prev_kind = before.chars().next_back().map(CharKind::of);
    let next_kind = after.chars().next().map(CharKind::of);
    let kind = match (prev_kind, next_kind) {
        (Some(p), Some(n)) => {
            if p.priority() >= n.priority() {
                p
            } else {
                n
            }
        }
        (Some(k), None) | (None, Some(k)) => k,
        (None, None) => return 0..0,
    };

    let mut start = index;
    for (i, c) in before.char_indices().rev() {
        if CharKind::of(c) == kind {
            start = i;
        } else {
            break;
        }
    }

    let mut end = index;
    for (i, c) in after.char_indices() {
        if CharKind::of(c) == kind {
            end = index + i + c.len_utf8();
        } else {
            break;
        }
    }

    start..end
}

/// Minimal, dependency-free percent decoder. `%XX` is folded to a byte
/// when the two digits are valid hex. Malformed escapes or non-UTF-8
/// decoded output fall back to the input unchanged.
///
/// Pulling in `percent-encoding` would be cleaner, but fragments are
/// typically <100 bytes of ASCII — this inline walker is adequate.
fn percent_decode_fragment(input: &str) -> String {
    if !input.contains('%') {
        return input.to_string();
    }
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = (bytes[i + 1] as char).to_digit(16);
            let lo = (bytes[i + 2] as char).to_digit(16);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h * 16 + l) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

/// Paints the selection background as up to three quads covering the
/// range `[lo, hi)` within `bounds`. Mirrors Zed's `paint_selection`
/// geometry (`crates/markdown/src/markdown.rs:1022`): single-line
/// selections emit one rect from the start position to the end
/// position; multi-line selections emit a start-line partial, an
/// optional middle-rows full-width block, and an end-line partial.
fn paint_selection_quads(
    layout: &gpui::TextLayout,
    lo: usize,
    hi: usize,
    bounds: Bounds<Pixels>,
    color: Hsla,
    window: &mut Window,
) {
    let Some(start) = layout.position_for_index(lo) else {
        return;
    };
    let Some(end) = layout.position_for_index(hi) else {
        return;
    };
    let line_height = layout.line_height();

    if (end.y - start.y).abs() < line_height / 2.0 {
        let rect = Bounds::from_corners(start, point(end.x, end.y + line_height));
        window.paint_quad(fill(rect, color));
        return;
    }

    let first = Bounds::from_corners(start, point(bounds.right(), start.y + line_height));
    window.paint_quad(fill(first, color));

    if end.y > start.y + line_height {
        let mid = Bounds::from_corners(
            point(bounds.left(), start.y + line_height),
            point(bounds.right(), end.y),
        );
        window.paint_quad(fill(mid, color));
    }

    let last = Bounds::from_corners(
        point(bounds.left(), end.y),
        point(end.x, end.y + line_height),
    );
    window.paint_quad(fill(last, color));
}

/// One-shot warning when `global_id` is `None` at paint time — indicates a
/// GPUI framework contract violation (should never happen since `id()` always
/// returns `Some`). `OnceLock` prevents log spam across frames.
/// `#[cfg(debug_assertions)]` keeps this out of release builds.
#[cfg(debug_assertions)]
fn warn_global_id_missing_once() {
    use std::sync::OnceLock;

    static WARNED: OnceLock<()> = OnceLock::new();
    WARNED.get_or_init(|| {
        tracing::warn!(
            "SelectableText::paint: global_id was None despite id() returning Some. \
             Selection interaction disabled for this element."
        );
    });
}

#[cfg(not(debug_assertions))]
fn warn_global_id_missing_once() {}

#[cfg(test)]
mod tests {
    use super::{fragment_of, percent_decode_fragment, word_range_at};
    use core::prelude::v1::test;

    fn normalize(a: usize, b: usize) -> (usize, usize) {
        if a <= b { (a, b) } else { (b, a) }
    }

    // ── word_range_at ────────────────────────────────────────────────
    // `word_range_at` is the core of double-click word selection: both
    // `TextViewSelection` and `MarkdownSelection` delegate to it. Cover
    // the shapes that once regressed: empty text, trailing boundary,
    // punctuation runs, whitespace-only hits, and multi-byte codepoints
    // (so a non-boundary index can't silently corrupt the range).

    #[test]
    fn word_range_at_empty_text_is_empty_range() {
        assert_eq!(word_range_at("", 0), 0..0);
    }

    #[test]
    fn word_range_at_clamps_past_end() {
        // Index past `text.len()` should clamp rather than panic.
        let text = "hello";
        assert_eq!(word_range_at(text, 99), 0..5);
    }

    #[test]
    fn word_range_at_selects_alphanumeric_word() {
        let text = "alpha beta gamma";
        // Index inside "beta".
        assert_eq!(word_range_at(text, 7), 6..10);
    }

    #[test]
    fn word_range_at_on_boundary_prefers_previous_word() {
        // Cursor sitting on the space between "alpha" and "beta" should
        // snap to the word on the preferred side — the matching priority
        // rule means the previous alphabetic run wins over the following
        // whitespace.
        let text = "alpha beta";
        assert_eq!(word_range_at(text, 5), 0..5);
    }

    #[test]
    fn word_range_at_punctuation_groups_together() {
        // A run of punctuation is its own "word" for double-click.
        let text = "hi!!! there";
        assert_eq!(word_range_at(text, 3), 2..5);
    }

    #[test]
    fn word_range_at_multibyte_codepoint_midpoint_snaps_back() {
        // 'é' is 2 bytes in UTF-8. An index in the middle of the
        // codepoint must snap back to the preceding char boundary
        // instead of panicking inside the `&text[..index]` split.
        let text = "café latte";
        assert_eq!(word_range_at(text, 4), 0..5);
    }

    #[test]
    fn normalize_swaps_when_reversed() {
        assert_eq!(normalize(5, 2), (2, 5));
        assert_eq!(normalize(2, 5), (2, 5));
        assert_eq!(normalize(3, 3), (3, 3));
    }

    #[test]
    fn copy_slice_respects_reversed_selection() {
        let text = "hello world";
        let (lo, hi) = normalize(7, 2);
        assert_eq!(&text[lo..hi], "llo w");
        let (lo, hi) = normalize(2, 7);
        assert_eq!(&text[lo..hi], "llo w");
    }

    #[test]
    fn fragment_of_strips_leading_hash() {
        assert_eq!(fragment_of("#section"), Some("section".to_string()));
    }

    #[test]
    fn fragment_of_empty_fragment_is_empty_string() {
        // `[home](#)` is a valid markdown link that points nowhere in
        // particular. The handler receives the empty string and decides
        // the policy (e.g. scroll to top, or no-op).
        assert_eq!(fragment_of("#"), Some(String::new()));
    }

    #[test]
    fn fragment_of_decodes_percent_encoding() {
        assert_eq!(fragment_of("#my%20section"), Some("my section".to_string()));
        assert_eq!(fragment_of("#caf%C3%A9"), Some("café".to_string()));
    }

    #[test]
    fn fragment_of_rejects_external_urls_with_fragment() {
        // Absolute URLs with a fragment are still external links — the
        // OS URL handler resolves the fragment server-side. `fragment_of`
        // only matches URLs that *start* with `#`.
        assert_eq!(fragment_of("https://example.com#frag"), None);
    }

    #[test]
    fn fragment_of_rejects_plain_url() {
        assert_eq!(fragment_of("https://example.com"), None);
        assert_eq!(fragment_of("mailto:foo@example.com"), None);
    }

    #[test]
    fn percent_decode_passes_plain_input_through() {
        assert_eq!(percent_decode_fragment("plain-slug"), "plain-slug");
    }

    #[test]
    fn percent_decode_handles_malformed_escape() {
        // `%Z` is invalid hex — the sequence is kept verbatim rather
        // than panicking.
        assert_eq!(percent_decode_fragment("%Znot-hex"), "%Znot-hex");
    }

    #[test]
    fn percent_decode_handles_trailing_percent() {
        assert_eq!(percent_decode_fragment("trailing%"), "trailing%");
        assert_eq!(percent_decode_fragment("trailing%2"), "trailing%2");
    }

    #[test]
    fn percent_decode_invalid_utf8_falls_back_to_input() {
        // `%FF` alone is not valid UTF-8 — we return the input unchanged
        // rather than corrupt the string.
        assert_eq!(percent_decode_fragment("%FF"), "%FF");
    }
}
