//! Selectable text element with cross-paragraph drag-select, link
//! click, multi-click selection modes, and Cmd/Ctrl+C copy.
//!
//! Wraps [`StyledText`] into a custom [`gpui::Element`] that paints a
//! selection background before delegating the text paint, then
//! registers mouse + key handlers that talk to a shared
//! [`MarkdownSelection`] coordinator so the selection can span
//! multiple paragraphs within the same
//! [`crate::markdown::StreamingMarkdown`] entity.
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
//! - **Cmd / Ctrl + C** — copy the selected text (empty string if
//!   no selection). Cross-paragraph selections are joined with `\n`.
//! - **Cmd / Ctrl + A** — select every registered paragraph.
//! - **Click on a link** — opens the URL via `cx.open_url` when the
//!   gesture is a click (no drag).
//!
//! # Pattern
//!
//! Mirrors Zed's [`InteractiveText`](gpui::InteractiveText) plus the
//! `paint_selection` quad pass in Zed's markdown
//! (`crates/markdown/src/markdown.rs:1022`). The cross-paragraph
//! coordination lives in [`MarkdownSelection`] so each paragraph can
//! remain an independent GPUI element instead of requiring a
//! monolithic Markdown element like Zed's.

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    App, Bounds, CursorStyle, DispatchPhase, ElementId, GlobalElementId, Hitbox, HitboxBehavior,
    Hsla, InspectorElementId, KeyDownEvent, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, SharedString, StyledText, Window, fill, point,
};

use super::AnchorClickHandler;
use super::selection::MarkdownSelection;

/// A [`StyledText`] wrapper that participates in a shared
/// [`MarkdownSelection`]. See the module docs for gestures and
/// cross-paragraph semantics.
pub struct SelectableText {
    element_id: ElementId,
    text: StyledText,
    text_string: SharedString,
    clickable_ranges: Vec<Range<usize>>,
    link_urls: Vec<SharedString>,
    selection_bg: Hsla,
    selection: MarkdownSelection,
    anchor_click: Option<AnchorClickHandler>,
}

/// Per-element state persisted across frames. Only tracks whether the
/// mouse button is held down on this paragraph (used to disambiguate
/// drag vs. click on `MouseUp`); the actual selection range lives on
/// the shared [`MarkdownSelection`] coordinator.
#[derive(Default, Clone)]
struct SelectableTextState {
    mouse_down_index: Rc<Cell<Option<usize>>>,
}

impl SelectableText {
    /// Wraps a pre-built [`StyledText`] into a selectable element.
    ///
    /// `text` is the flat text behind the styled runs, used for
    /// clipboard copy and word-boundary detection. It must match the
    /// string the `StyledText` was constructed from; the element
    /// cannot read that back from `StyledText` directly at this GPUI
    /// version.
    ///
    /// `selection` is the shared coordinator — clone the same
    /// [`MarkdownSelection`] into every paragraph that should
    /// participate in a single cross-paragraph selection.
    pub fn new(
        element_id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        styled: StyledText,
        selection_bg: Hsla,
        selection: MarkdownSelection,
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
    /// and any percent-encoding decoded so it matches the slug on
    /// [`crate::markdown::MarkdownBlock::Heading::anchor_id`].
    pub fn with_anchor_click_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.anchor_click = Some(Rc::new(handler));
        self
    }

    /// Variant of [`Self::with_anchor_click_handler`] that takes an
    /// already-wrapped handler. Used internally by the renderer when
    /// forwarding a shared handler from [`crate::markdown::StreamingMarkdown`]
    /// across many `SelectableText` elements on one frame.
    pub fn with_anchor_click_handler_rc(mut self, handler: AnchorClickHandler) -> Self {
        self.anchor_click = Some(handler);
        self
    }
}

impl Element for SelectableText {
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
        let clickable_ranges = std::mem::take(&mut self.clickable_ranges);
        let link_urls = std::mem::take(&mut self.link_urls);
        let anchor_click = self.anchor_click.take();
        let element_id = self.element_id.clone();

        window.with_element_state::<SelectableTextState, _>(
            global_id.expect("SelectableText requires an element id"),
            |state, window| {
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
                        mouse_down_index.set(Some(ix));
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
                // click when the gesture was a click (no drag).
                {
                    let mouse_down_index = state.mouse_down_index.clone();
                    let hitbox = hitbox.clone();
                    let text_layout = text_layout.clone();
                    let link_urls = link_urls.clone();
                    let clickable_ranges = clickable_ranges.clone();
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
                            // Fragment URLs are only registered as clickable
                            // when a handler is installed (see the gate in
                            // `render_inlines_flat`), so this arm always
                            // has a handler in practice — the `if let`
                            // survives as defence in depth.
                            match fragment_of(url.as_ref()) {
                                Some(fragment) => {
                                    // Defence in depth: fragment links
                                    // are only registered as clickable
                                    // when a handler is installed (see
                                    // `render_inlines_flat`), so this
                                    // arm should always have a handler.
                                    // If we reach it without one the
                                    // render-side gate was bypassed.
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

                // Cmd/Ctrl+C and Cmd/Ctrl+A — register a key handler
                // that only acts when the mouse is over this
                // paragraph (a rough proxy for "the selection is
                // mine"; a proper focus-scoped implementation would
                // use `FocusHandle`). Multiple paragraphs may have
                // the same handler registered; the shared coordinator
                // guarantees they all act on the same selection
                // state, so duplicated handlers are idempotent.
                {
                    let hitbox = hitbox.clone();
                    let selection = selection.clone();
                    window.on_key_event(move |event: &KeyDownEvent, phase, window, cx| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }
                        if !hitbox.is_hovered(window) {
                            return;
                        }
                        let keystroke = &event.keystroke;
                        let cmd_or_ctrl =
                            keystroke.modifiers.platform || keystroke.modifiers.control;
                        if !cmd_or_ctrl {
                            return;
                        }
                        match keystroke.key.as_str() {
                            "c" => selection.copy_to_clipboard(cx),
                            "a" => {
                                selection.select_all();
                                window.refresh();
                            }
                            _ => {}
                        }
                    });
                }

                ((), state)
            },
        )
    }
}

impl IntoElement for SelectableText {
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
pub(super) fn fragment_of(url: &str) -> Option<String> {
    url.strip_prefix('#').map(percent_decode_fragment)
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

#[cfg(test)]
mod tests {
    use super::{fragment_of, percent_decode_fragment};
    use core::prelude::v1::test;

    fn normalize(a: usize, b: usize) -> (usize, usize) {
        if a <= b { (a, b) } else { (b, a) }
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
