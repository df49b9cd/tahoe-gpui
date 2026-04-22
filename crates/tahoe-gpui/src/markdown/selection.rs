//! Cross-paragraph selection coordinator for
//! [`crate::markdown::StreamingMarkdown`].
//!
//! Each [`crate::markdown::selectable_text::SelectableText`] paragraph
//! registers itself with a shared [`MarkdownSelection`] during paint.
//! The coordinator stores anchor / focus positions as
//! `(ElementId, char_index)` pairs and exposes helpers the element
//! consults when painting selection quads, dispatching Cmd+C copy,
//! and handling shift-click extension.
//!
//! This is the per-frame registry + selection-state pattern Zed uses
//! inside its monolithic `MarkdownElement` (crates/markdown/src/
//! markdown.rs:2480 `RenderedText`). We split it out so each
//! paragraph can remain an independent GPUI element while still
//! participating in a document-wide selection.

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{App, ClipboardItem, ElementId, SharedString};

use crate::components::content::selectable_text::{SelectionCoordinator, word_range_at};

/// The selection gesture mode set at mouse-down time. Determines how
/// subsequent drag moves extend the selection — character-granular
/// for a single click, whole-word for double-click, whole-paragraph
/// for triple-click.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectMode {
    /// Single click: drag extends by character.
    #[default]
    Character,
    /// Double click: anchored at the word's range; drag extends by
    /// whole-word chunks.
    Word,
    /// Triple click: anchored at the paragraph; drag extends by
    /// whole-paragraph chunks.
    Line,
    /// Quad+ click or Cmd+A: all registered paragraphs. No drag
    /// extension.
    All,
}

/// A point within the rendered selection — the element id identifies
/// the paragraph, `char_index` is the byte offset into that
/// paragraph's flat text (same index space used by GPUI's
/// `TextLayout::index_for_position`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionPoint {
    pub element_id: ElementId,
    pub char_index: usize,
}

/// A paragraph's registration entry for the current frame. Cleared
/// at the start of each render via
/// [`MarkdownSelection::begin_frame`].
#[derive(Clone)]
struct ParagraphInfo {
    element_id: ElementId,
    text: SharedString,
}

#[derive(Default)]
struct MarkdownSelectionState {
    anchor: Option<SelectionPoint>,
    focus: Option<SelectionPoint>,
    /// `true` while the left mouse button is held mid-drag. Used by
    /// `MouseMove` handlers to decide whether to extend the
    /// selection.
    pending: bool,
    /// The per-gesture mode captured at mouse-down. Word-granular
    /// drag extends the selection to whole words rather than
    /// characters.
    mode: SelectMode,
    /// Original range anchored at mouse-down for word / line modes
    /// (so dragging back over the anchor keeps the original word /
    /// line selected instead of collapsing).
    original_range: Option<(SelectionPoint, SelectionPoint)>,
    /// Paragraphs registered during the current render, in the order
    /// paint was called on them. Populated by
    /// [`MarkdownSelection::register`]; cleared by
    /// [`MarkdownSelection::begin_frame`].
    paragraphs: Vec<ParagraphInfo>,
}

/// Shared selection coordinator. Cheap to clone — internally an
/// [`Rc`] around the mutable state.
#[derive(Clone, Default)]
pub struct MarkdownSelection {
    inner: Rc<RefCell<MarkdownSelectionState>>,
}

impl MarkdownSelection {
    /// Construct a fresh coordinator with no selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Called at the start of each render pass on the owning
    /// [`crate::markdown::StreamingMarkdown`]. Clears the per-frame
    /// paragraph registry so the new frame's paragraphs (which may
    /// have been added, removed, or reordered by an incremental
    /// parse) re-register in fresh order. Anchor / focus / pending
    /// state is preserved.
    pub fn begin_frame(&self) {
        self.inner.borrow_mut().paragraphs.clear();
    }

    /// Register a paragraph as part of the current frame. `element_id`
    /// must match the `Element::id()` of the [`crate::markdown::
    /// selectable_text::SelectableText`] that owns the paragraph;
    /// `text` is the flat text (same index space as the paragraph's
    /// `StyledText`).
    pub fn register(&self, element_id: ElementId, text: SharedString) {
        let mut state = self.inner.borrow_mut();
        // Guard against duplicate ids (shouldn't happen, but be
        // defensive so a caller bug doesn't produce confusing
        // selection coordinates).
        if state.paragraphs.iter().any(|p| p.element_id == element_id) {
            return;
        }
        state.paragraphs.push(ParagraphInfo { element_id, text });
    }

    /// Returns the portion of the selection that falls inside the
    /// given paragraph, as a `char_index` range into that paragraph's
    /// text. Returns `None` when the paragraph has no selection
    /// painted on it.
    ///
    /// For the paragraph containing the anchor: `[anchor_ix, text_len)`.
    /// For the paragraph containing the focus: `[0, focus_ix)`.
    /// For paragraphs strictly between anchor and focus: `[0, text_len)`.
    /// For a paragraph that contains both anchor and focus:
    /// `[min(a, f), max(a, f))`.
    pub fn range_for_element(
        &self,
        element_id: &ElementId,
        text_len: usize,
    ) -> Option<Range<usize>> {
        let state = self.inner.borrow();
        let anchor = state.anchor.as_ref()?;
        let focus = state.focus.as_ref()?;

        let anchor_pos = state
            .paragraphs
            .iter()
            .position(|p| p.element_id == anchor.element_id)?;
        let focus_pos = state
            .paragraphs
            .iter()
            .position(|p| p.element_id == focus.element_id)?;
        let my_pos = state
            .paragraphs
            .iter()
            .position(|p| &p.element_id == element_id)?;

        // Normalise anchor / focus into (start, end) by paragraph index
        // then character index within.
        let (start, end) = normalise(
            (anchor_pos, anchor.char_index),
            (focus_pos, focus.char_index),
        );

        if my_pos < start.0 || my_pos > end.0 {
            return None;
        }

        let lo = if my_pos == start.0 { start.1 } else { 0 };
        let hi = if my_pos == end.0 { end.1 } else { text_len };

        // Clamp to paragraph length so an out-of-date anchor from a
        // prior frame can't produce an out-of-bounds slice.
        let lo = lo.min(text_len);
        let hi = hi.min(text_len);
        if lo < hi { Some(lo..hi) } else { None }
    }

    /// Dispatch a mouse-down at `(element_id, char_index)` within
    /// `text`. `click_count` and `shift` come from GPUI's
    /// `MouseDownEvent`. Returns `true` when the event consumed the
    /// selection (i.e. started or extended one) — callers can use
    /// this to decide whether to treat the gesture as a link click.
    pub fn mouse_down(
        &self,
        element_id: ElementId,
        text: &str,
        char_index: usize,
        click_count: usize,
        shift: bool,
    ) {
        let mut state = self.inner.borrow_mut();
        if shift && state.anchor.is_some() {
            // Shift-click extends from the existing anchor.
            state.focus = Some(SelectionPoint {
                element_id,
                char_index,
            });
            state.pending = true;
            return;
        }

        match click_count {
            1 => {
                let point = SelectionPoint {
                    element_id,
                    char_index,
                };
                state.anchor = Some(point.clone());
                state.focus = Some(point);
                state.pending = true;
                state.mode = SelectMode::Character;
                state.original_range = None;
            }
            2 => {
                let range = word_range_at(text, char_index);
                let start = SelectionPoint {
                    element_id: element_id.clone(),
                    char_index: range.start,
                };
                let end = SelectionPoint {
                    element_id,
                    char_index: range.end,
                };
                state.anchor = Some(start.clone());
                state.focus = Some(end.clone());
                state.pending = true;
                state.mode = SelectMode::Word;
                state.original_range = Some((start, end));
            }
            3 => {
                let start = SelectionPoint {
                    element_id: element_id.clone(),
                    char_index: 0,
                };
                let end = SelectionPoint {
                    element_id,
                    char_index: text.len(),
                };
                state.anchor = Some(start.clone());
                state.focus = Some(end.clone());
                state.pending = true;
                state.mode = SelectMode::Line;
                state.original_range = Some((start, end));
            }
            _ => {
                if let (Some(first), Some(last)) = (
                    state.paragraphs.first().cloned(),
                    state.paragraphs.last().cloned(),
                ) {
                    state.anchor = Some(SelectionPoint {
                        element_id: first.element_id,
                        char_index: 0,
                    });
                    state.focus = Some(SelectionPoint {
                        element_id: last.element_id,
                        char_index: last.text.len(),
                    });
                    state.pending = false;
                    state.mode = SelectMode::All;
                    state.original_range = None;
                }
            }
        }
    }

    /// Extend the selection to the given position during a drag.
    /// No-op when no drag is in progress.
    pub fn drag_to(&self, element_id: ElementId, text: &str, char_index: usize) {
        let mut state = self.inner.borrow_mut();
        if !state.pending {
            return;
        }
        let head = SelectionPoint {
            element_id: element_id.clone(),
            char_index,
        };
        match state.mode {
            SelectMode::Word => {
                let word = word_range_at(text, char_index);
                let head_start = SelectionPoint {
                    element_id: element_id.clone(),
                    char_index: word.start,
                };
                let head_end = SelectionPoint {
                    element_id,
                    char_index: word.end,
                };
                // Compare head relative to anchor, but expand
                // outwards to include the full word under the cursor.
                extend_with_range(&mut state, head_start, head_end);
            }
            SelectMode::Line => {
                // Triple-click drag: select whole paragraphs.
                let head_start = SelectionPoint {
                    element_id: element_id.clone(),
                    char_index: 0,
                };
                let head_end = SelectionPoint {
                    element_id,
                    char_index: text.len(),
                };
                extend_with_range(&mut state, head_start, head_end);
            }
            SelectMode::All => {}
            SelectMode::Character => {
                state.focus = Some(head);
            }
        }
    }

    /// Called on `MouseUp` — finalises the drag so subsequent
    /// `MouseMove`s do not extend the selection.
    pub fn end_drag(&self) {
        self.inner.borrow_mut().pending = false;
    }

    /// Cmd+A / Ctrl+A — select every registered paragraph.
    pub fn select_all(&self) {
        let mut state = self.inner.borrow_mut();
        let (Some(first), Some(last)) = (
            state.paragraphs.first().cloned(),
            state.paragraphs.last().cloned(),
        ) else {
            return;
        };
        state.anchor = Some(SelectionPoint {
            element_id: first.element_id,
            char_index: 0,
        });
        state.focus = Some(SelectionPoint {
            element_id: last.element_id,
            char_index: last.text.len(),
        });
        state.pending = false;
        state.mode = SelectMode::All;
        state.original_range = None;
    }

    /// Copy the current selection to the clipboard. Multi-paragraph
    /// selections are joined with `\n`.
    pub fn copy_to_clipboard(&self, cx: &mut App) {
        let text = self.selected_text();
        if !text.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    /// Returns the concatenated text of the current selection, or an
    /// empty string when nothing is selected.
    pub fn selected_text(&self) -> String {
        let state = self.inner.borrow();
        let Some(anchor) = state.anchor.as_ref() else {
            return String::new();
        };
        let Some(focus) = state.focus.as_ref() else {
            return String::new();
        };

        let Some(anchor_pos) = state
            .paragraphs
            .iter()
            .position(|p| p.element_id == anchor.element_id)
        else {
            return String::new();
        };
        let Some(focus_pos) = state
            .paragraphs
            .iter()
            .position(|p| p.element_id == focus.element_id)
        else {
            return String::new();
        };

        let (start, end) = normalise(
            (anchor_pos, anchor.char_index),
            (focus_pos, focus.char_index),
        );

        let mut out = String::new();
        for (idx, para) in state.paragraphs.iter().enumerate() {
            if idx < start.0 || idx > end.0 {
                continue;
            }
            let text_len = para.text.len();
            let lo = if idx == start.0 {
                start.1.min(text_len)
            } else {
                0
            };
            let hi = if idx == end.0 {
                end.1.min(text_len)
            } else {
                text_len
            };
            if lo < hi
                && let Some(slice) = para.text.get(lo..hi)
            {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(slice);
            }
        }
        out
    }

    /// `true` when there's at least one selected character.
    pub fn has_selection(&self) -> bool {
        let state = self.inner.borrow();
        match (&state.anchor, &state.focus) {
            (Some(a), Some(f)) => a != f,
            _ => false,
        }
    }

    /// `true` while a drag is in progress.
    pub fn is_pending(&self) -> bool {
        self.inner.borrow().pending
    }

    /// Clear any existing selection. Called when focus moves away or
    /// the host wants to drop the highlight.
    pub fn clear(&self) {
        let mut state = self.inner.borrow_mut();
        state.anchor = None;
        state.focus = None;
        state.pending = false;
        state.mode = SelectMode::default();
        state.original_range = None;
    }
}

impl SelectionCoordinator for MarkdownSelection {
    fn register(&self, element_id: ElementId, text: SharedString) {
        MarkdownSelection::register(self, element_id, text);
    }

    fn range_for_element(&self, element_id: &ElementId, text_len: usize) -> Option<Range<usize>> {
        MarkdownSelection::range_for_element(self, element_id, text_len)
    }

    fn mouse_down(
        &self,
        element_id: ElementId,
        text: &str,
        char_index: usize,
        click_count: usize,
        shift: bool,
    ) {
        MarkdownSelection::mouse_down(self, element_id, text, char_index, click_count, shift);
    }

    fn drag_to(&self, element_id: ElementId, text: &str, char_index: usize) {
        MarkdownSelection::drag_to(self, element_id, text, char_index);
    }

    fn end_drag(&self) {
        MarkdownSelection::end_drag(self);
    }

    fn is_pending(&self) -> bool {
        MarkdownSelection::is_pending(self)
    }
}

fn normalise(a: (usize, usize), b: (usize, usize)) -> ((usize, usize), (usize, usize)) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Extend the current selection to include the head range `[lo, hi]`,
/// keeping the original anchor word/line intact. Used by word / line
/// drag-select so the selection snaps to whole-word or whole-line
/// boundaries as the cursor moves.
fn extend_with_range(
    state: &mut std::cell::RefMut<'_, MarkdownSelectionState>,
    head_start: SelectionPoint,
    head_end: SelectionPoint,
) {
    let Some((orig_start, orig_end)) = state.original_range.clone() else {
        // Fall back to character behaviour if no original range.
        state.focus = Some(head_end);
        return;
    };

    let Some(anchor_pos) = state
        .paragraphs
        .iter()
        .position(|p| p.element_id == orig_start.element_id)
    else {
        return;
    };
    let Some(head_pos) = state
        .paragraphs
        .iter()
        .position(|p| p.element_id == head_start.element_id)
    else {
        return;
    };

    if (head_pos, head_start.char_index) < (anchor_pos, orig_start.char_index) {
        // Dragged before the original anchor — reverse the selection.
        state.anchor = Some(orig_end);
        state.focus = Some(head_start);
    } else {
        state.anchor = Some(orig_start);
        state.focus = Some(head_end);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::SharedString;

    fn make_id(name: &'static str) -> ElementId {
        ElementId::Name(name.into())
    }

    #[test]
    fn word_range_inside_word() {
        let text = "hello world";
        assert_eq!(word_range_at(text, 2), 0..5); // inside "hello"
        assert_eq!(word_range_at(text, 7), 6..11); // inside "world"
    }

    #[test]
    fn word_range_at_boundary_prefers_word() {
        // At the boundary between "hello" and " ", prefer "hello".
        let text = "hello world";
        assert_eq!(word_range_at(text, 5), 0..5);
    }

    #[test]
    fn word_range_on_whitespace_only() {
        let text = "a   b";
        assert_eq!(word_range_at(text, 2), 1..4); // middle space
    }

    #[test]
    fn word_range_with_underscores() {
        let text = "foo_bar baz";
        assert_eq!(word_range_at(text, 3), 0..7); // _bar continues the word
    }

    #[test]
    fn word_range_with_punctuation() {
        let text = "foo, bar";
        // At index 4 (between ',' and ' ') the comma is the surrounding
        // punctuation run — priority ties favour the previous char.
        assert_eq!(word_range_at(text, 4), 3..4);
        // Index 3 sits between 'o' and ',' — Word beats Punctuation so
        // the surrounding word is "foo".
        assert_eq!(word_range_at(text, 3), 0..3);
        assert_eq!(word_range_at(text, 2), 0..3); // in "foo"
    }

    #[test]
    fn word_range_unicode() {
        let text = "λ café";
        // Inside "café" — 4 chars but é is multi-byte.
        let idx = text.find("café").unwrap() + 2; // inside "ca"
        let range = word_range_at(text, idx);
        assert_eq!(&text[range], "café");
    }

    #[test]
    fn word_range_empty_text() {
        assert_eq!(word_range_at("", 0), 0..0);
    }

    #[test]
    fn word_range_at_end_of_text() {
        let text = "hello";
        assert_eq!(word_range_at(text, 5), 0..5); // at EOF, word = "hello"
    }

    #[test]
    fn empty_coordinator_has_no_selection() {
        let sel = MarkdownSelection::new();
        assert!(!sel.has_selection());
        assert_eq!(sel.selected_text(), "");
    }

    #[test]
    fn single_click_collapses_to_zero_range() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        sel.mouse_down(make_id("p0"), "hello world", 3, 1, false);
        assert!(!sel.has_selection());
        assert_eq!(sel.selected_text(), "");
    }

    #[test]
    fn double_click_selects_surrounding_word() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        sel.mouse_down(make_id("p0"), "hello world", 2, 2, false);
        assert!(sel.has_selection());
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn triple_click_selects_whole_paragraph() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        sel.mouse_down(make_id("p0"), "hello world", 0, 3, false);
        assert_eq!(sel.selected_text(), "hello world");
    }

    #[test]
    fn quadruple_click_selects_all_paragraphs() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("alpha"));
        sel.register(make_id("p1"), SharedString::from("beta"));
        sel.mouse_down(make_id("p0"), "alpha", 0, 4, false);
        assert_eq!(sel.selected_text(), "alpha\nbeta");
    }

    #[test]
    fn select_all_covers_every_paragraph() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("alpha"));
        sel.register(make_id("p1"), SharedString::from("beta"));
        sel.register(make_id("p2"), SharedString::from("gamma"));
        sel.select_all();
        assert_eq!(sel.selected_text(), "alpha\nbeta\ngamma");
    }

    #[test]
    fn drag_across_paragraphs_selects_crosspara() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("alpha beta"));
        sel.register(make_id("p1"), SharedString::from("gamma delta"));
        // Click at the end of "alpha" in p0.
        sel.mouse_down(make_id("p0"), "alpha beta", 5, 1, false);
        // Drag to mid-"gamma" in p1.
        sel.drag_to(make_id("p1"), "gamma delta", 3);
        assert_eq!(sel.selected_text(), " beta\ngam");
    }

    #[test]
    fn shift_click_extends_existing_anchor() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        sel.mouse_down(make_id("p0"), "hello world", 0, 1, false);
        sel.end_drag();
        sel.mouse_down(make_id("p0"), "hello world", 5, 1, true); // shift-click
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn range_for_element_single_paragraph() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        sel.mouse_down(make_id("p0"), "hello world", 0, 1, false);
        sel.drag_to(make_id("p0"), "hello world", 5);
        assert_eq!(sel.range_for_element(&make_id("p0"), 11), Some(0..5));
    }

    #[test]
    fn range_for_element_middle_paragraph_is_full_text() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("alpha"));
        sel.register(make_id("p1"), SharedString::from("beta"));
        sel.register(make_id("p2"), SharedString::from("gamma"));
        sel.mouse_down(make_id("p0"), "alpha", 0, 1, false);
        sel.drag_to(make_id("p2"), "gamma", 3);
        assert_eq!(sel.range_for_element(&make_id("p0"), 5), Some(0..5));
        assert_eq!(sel.range_for_element(&make_id("p1"), 4), Some(0..4));
        assert_eq!(sel.range_for_element(&make_id("p2"), 5), Some(0..3));
    }

    #[test]
    fn range_for_element_outside_selection_is_none() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("alpha"));
        sel.register(make_id("p1"), SharedString::from("beta"));
        sel.register(make_id("p2"), SharedString::from("gamma"));
        sel.mouse_down(make_id("p0"), "alpha", 0, 1, false);
        sel.drag_to(make_id("p0"), "alpha", 3);
        assert_eq!(sel.range_for_element(&make_id("p1"), 4), None);
        assert_eq!(sel.range_for_element(&make_id("p2"), 5), None);
    }

    #[test]
    fn reverse_drag_is_symmetric() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        // Click at index 5, drag backwards to 0.
        sel.mouse_down(make_id("p0"), "hello world", 5, 1, false);
        sel.drag_to(make_id("p0"), "hello world", 0);
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn begin_frame_preserves_selection_across_re_register() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        sel.mouse_down(make_id("p0"), "hello world", 0, 1, false);
        sel.drag_to(make_id("p0"), "hello world", 5);
        // New render frame re-registers the same paragraph.
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello world"));
        assert_eq!(sel.selected_text(), "hello");
    }

    #[test]
    fn clear_removes_all_selection() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("hello"));
        sel.mouse_down(make_id("p0"), "hello", 0, 3, false);
        assert!(sel.has_selection());
        sel.clear();
        assert!(!sel.has_selection());
    }

    #[test]
    fn duplicate_register_ignored() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        sel.register(make_id("p0"), SharedString::from("first"));
        sel.register(make_id("p0"), SharedString::from("second"));
        sel.mouse_down(make_id("p0"), "first", 0, 3, false);
        assert_eq!(sel.selected_text(), "first");
    }

    #[test]
    fn word_drag_extends_by_whole_words() {
        let sel = MarkdownSelection::new();
        sel.begin_frame();
        let text = "alpha beta gamma delta";
        sel.register(make_id("p0"), SharedString::from(text));
        // Double-click on "beta" (index 7, inside "beta").
        sel.mouse_down(make_id("p0"), text, 7, 2, false);
        assert_eq!(sel.selected_text(), "beta");
        // Drag into "delta"; expect the selection to include whole
        // words.
        sel.drag_to(make_id("p0"), text, 19); // inside "delta"
        assert_eq!(sel.selected_text(), "beta gamma delta");
    }
}
