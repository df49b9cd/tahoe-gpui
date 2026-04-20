//! Single-line text input primitive.
//!
//! A lightweight single-line text field with cursor, selection, clipboard,
//! and IME support. Action types and keybindings are scoped to the
//! [`TEXT_FIELD_CONTEXT`] GPUI key context.

pub mod validation;
pub use validation::TextFieldValidation;

mod boundaries;

use crate::foundations::layout::SPACING_4;
use std::collections::VecDeque;
use std::ops::Range;
use std::time::Instant;

use gpui::prelude::*;
use gpui::{
    App, Bounds, ClickEvent, ClipboardItem, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, InspectorElementId, KeyDownEvent,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    SharedString, Style, TextAlign, TextRun, UTF16Selection, UnderlineStyle, Window, WrappedLine,
    div, fill, point, px, relative, size,
};

use crate::callback_types::OnStrChange;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::color::with_alpha;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::text_actions::{
    Backspace, Copy, Cut, Delete, DeleteToEnd, DeleteToStart, DeleteWord, End, Home, Left, Paste,
    Redo, Right, SelectAll, SelectLeft, SelectRight, SelectToDocEnd, SelectToDocStart,
    SelectToLineEnd, SelectToLineStart, SelectWordLeft, SelectWordRight, Undo, WordLeft, WordRight,
};

/// Visual style variants per HIG `NSTextField`.
///
/// * `Rounded` — default rounded rectangle with the theme's surface fill.
/// * `SquareBezel` — square corners with the same border and fill. Used
///   in toolbar contexts and data tables where rounded corners collide
///   with neighbouring elements.
/// * `Plain` — borderless / transparent background. Matches NSTextField's
///   plain/inline style used inside search bars, inline editors, and
///   table cells.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TextFieldStyle {
    #[default]
    Rounded,
    SquareBezel,
    Plain,
}

/// Autofill content hint per HIG `textContentType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextContentType {
    Username,
    Password,
    NewPassword,
    EmailAddress,
    PhoneNumber,
    Url,
    OneTimeCode,
    Name,
}

/// Customisable "Return key" label per HIG Virtual Keyboards.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SubmitLabel {
    #[default]
    Return,
    Search,
    Go,
    Done,
    Next,
}

/// The GPUI key context used by TextField for scoped keybindings.
///
/// Matches Zed's convention of naming the key context after the component it
/// scopes (cf. `Editor`'s `"Editor"` context). The crate's keybindings
/// module and `test_helpers.rs` derive their scope string from this
/// constant so renaming here keeps all binding sites aligned.
pub const TEXT_FIELD_CONTEXT: &str = "TextField";

/// Visual height of a `TextField` container in points.
///
/// This is the macOS default control metric (28 pt, see
/// [`MACOS_DEFAULT_TOUCH_TARGET`](crate::foundations::layout::MACOS_DEFAULT_TOUCH_TARGET))
/// plus 4 pt of breathing room above and below the cursor so the bezel
/// never clips the text baseline. The `min_h` below still pins the
/// activation region to the platform target size for non-macOS hosts.
pub const TEXTFIELD_HEIGHT: f32 = 32.0;

/// Returns the keybindings required for TextField keyboard interaction.
///
/// These map raw keys (Backspace, Delete, arrow keys) to text editing actions
/// within the [`TEXT_FIELD_CONTEXT`] context. Without these, TextField's
/// `on_action()` handlers won't fire.
///
/// Register during app initialization:
/// ```ignore
/// cx.bind_keys(tahoe_gpui::components::selection_and_input::text_field::keybindings());
/// ```
///
/// Or use [`all_keybindings()`](crate::all_keybindings) which includes these.
pub fn keybindings() -> Vec<gpui::KeyBinding> {
    use gpui::KeyBinding;
    let ctx = Some(TEXT_FIELD_CONTEXT);
    vec![
        // Raw keys scoped to TextField context
        KeyBinding::new("backspace", Backspace, ctx),
        KeyBinding::new("delete", Delete, ctx),
        KeyBinding::new("left", Left, ctx),
        KeyBinding::new("right", Right, ctx),
        KeyBinding::new("home", Home, ctx),
        KeyBinding::new("end", End, ctx),
        KeyBinding::new("shift-left", SelectLeft, ctx),
        KeyBinding::new("shift-right", SelectRight, ctx),
        KeyBinding::new("alt-left", WordLeft, ctx),
        KeyBinding::new("alt-right", WordRight, ctx),
        KeyBinding::new("alt-shift-left", SelectWordLeft, ctx),
        KeyBinding::new("alt-shift-right", SelectWordRight, ctx),
        KeyBinding::new("cmd-left", Home, ctx),
        KeyBinding::new("cmd-right", End, ctx),
        KeyBinding::new("cmd-a", SelectAll, ctx),
        KeyBinding::new("cmd-c", Copy, ctx),
        KeyBinding::new("cmd-x", Cut, ctx),
        KeyBinding::new("cmd-v", Paste, ctx),
        // Shift-Home / Shift-End extend selection to line boundaries per
        // the macOS standard; single-line fields treat "line boundary" as
        // the whole content range, so line == document for this widget.
        KeyBinding::new("shift-home", SelectToLineStart, ctx),
        KeyBinding::new("shift-end", SelectToLineEnd, ctx),
        KeyBinding::new("cmd-shift-left", SelectToLineStart, ctx),
        KeyBinding::new("cmd-shift-right", SelectToLineEnd, ctx),
        KeyBinding::new("cmd-shift-up", SelectToDocStart, ctx),
        KeyBinding::new("cmd-shift-down", SelectToDocEnd, ctx),
        // Undo / Redo (HIG Edit menu).
        KeyBinding::new("cmd-z", Undo, ctx),
        KeyBinding::new("cmd-shift-z", Redo, ctx),
        // Word/line deletion (macOS standard)
        KeyBinding::new("alt-backspace", DeleteWord, ctx),
        KeyBinding::new("cmd-backspace", DeleteToStart, ctx),
        KeyBinding::new("cmd-delete", DeleteToEnd, ctx),
    ]
}

/// A single-line text input with cursor, selection, and clipboard support.
pub struct TextField {
    /// Current text content (no newlines).
    content: SharedString,
    /// Selected byte range. When empty (start == end), the cursor is at `start`.
    selected_range: Range<usize>,
    /// Whether selection was started from the end (cursor is at start of selection).
    selection_reversed: bool,
    /// IME marked text range.
    marked_range: Option<Range<usize>>,
    /// Placeholder text when empty.
    placeholder: SharedString,
    /// Focus handle for keyboard input.
    focus_handle: FocusHandle,
    /// Whether mouse is currently dragging a selection.
    is_selecting: bool,
    /// Cached single-line layout from last render (for mouse hit-testing).
    last_layout: Option<WrappedLine>,
    /// Cached bounds from last render.
    last_bounds: Option<Bounds<Pixels>>,
    /// Cached line height from last render.
    last_line_height: Pixels,
    /// Callback invoked after every text modification.
    on_change: OnStrChange,
    /// When the cursor last moved (for blink timing).
    cursor_moved_at: Instant,
    /// When true and the input has content, show a clear ("x") button on the right.
    show_clear_button: bool,
    /// When true, display dots instead of actual characters (password field).
    is_secure: bool,
    /// Validation state controlling border color and optional error message.
    validation: TextFieldValidation,
    /// When true, suppress all user interaction and render dimmed.
    disabled: bool,
    /// When true, suppress text mutations (typing, paste, delete) but allow
    /// focus, selection, and copy.
    read_only: bool,
    /// Maximum content length in bytes (UTF-8). `None` means no cap.
    max_length: Option<usize>,
    /// Callback invoked on Enter/Return, with the current content.
    on_submit: OnStrChange,
    /// Visual style variant per HIG.
    style: TextFieldStyle,
    /// Optional leading icon rendered before the text area.
    leading_icon: Option<IconName>,
    /// Optional trailing icon rendered after the text area (right of the
    /// clear button if both are visible).
    trailing_icon: Option<IconName>,
    /// Autofill content hint (no direct GPUI API yet — surfaced in
    /// accessibility metadata and for parent-level autofill wiring).
    content_type: Option<TextContentType>,
    /// Customisable Return key label (no direct GPUI API yet — consumed
    /// by parent forms for helper-text/aria hints).
    submit_label: SubmitLabel,
    /// Undo history: snapshots of `(content, selected_range)` taken
    /// before each text mutation. Capped at `UNDO_STACK_LIMIT`.
    undo_stack: VecDeque<(SharedString, Range<usize>)>,
    /// Redo history: snapshots of `(content, selected_range)` that were
    /// popped by an undo. Cleared on any fresh mutation.
    redo_stack: VecDeque<(SharedString, Range<usize>)>,
    /// VoiceOver / AX label. Falls back to the placeholder when absent
    /// (HIG: "a field's placeholder serves as its accessibility name when
    /// no external label is associated with the field").
    accessibility_label: Option<SharedString>,
    /// Optional label rendered above the field. HIG Text Fields:
    /// "a label describes the field's purpose and accompanies the input."
    /// Separate from `placeholder` (shown inside the field) and
    /// `help_text` (shown below the field).
    label: Option<SharedString>,
    /// Optional help text rendered below the field. HIG Text Fields:
    /// "Include additional guidance as help text when callers need more
    /// context than the label provides." Hidden while the field is
    /// rendering a validation error (the error message takes priority).
    help_text: Option<SharedString>,
}

const UNDO_STACK_LIMIT: usize = 100;

impl TextField {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            content: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            placeholder: SharedString::default(),
            focus_handle: cx.focus_handle(),
            is_selecting: false,
            last_layout: None,
            last_bounds: None,
            last_line_height: px(20.0),
            on_change: None,
            cursor_moved_at: Instant::now(),
            show_clear_button: false,
            is_secure: false,
            validation: TextFieldValidation::None,
            disabled: false,
            read_only: false,
            max_length: None,
            on_submit: None,
            style: TextFieldStyle::Rounded,
            leading_icon: None,
            trailing_icon: None,
            content_type: None,
            submit_label: SubmitLabel::Return,
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            accessibility_label: None,
            label: None,
            help_text: None,
        }
    }

    /// Set the VoiceOver / AX label for this field. When unset, the
    /// placeholder (if any) is used so icon-only or visually-label-less
    /// fields still announce something meaningful.
    pub fn set_accessibility_label(&mut self, label: impl Into<SharedString>) {
        self.accessibility_label = Some(label.into());
    }

    /// Set the visible label rendered above the field.
    ///
    /// HIG distinguishes three text slots: **label** (describes purpose,
    /// rendered above the field), **placeholder** (hint shown inside the
    /// empty field), and **help text** (secondary guidance rendered
    /// below).  All three can be populated at once.
    pub fn set_label(&mut self, label: impl Into<SharedString>) {
        self.label = Some(label.into());
    }

    /// Set the help text rendered below the field. Hidden when a
    /// validation error is displayed (the error takes priority to avoid
    /// stacking two messages under the field).
    pub fn set_help_text(&mut self, text: impl Into<SharedString>) {
        self.help_text = Some(text.into());
    }

    /// Select the text-field visual style. Defaults to
    /// [`TextFieldStyle::Rounded`].
    pub fn set_style(&mut self, style: TextFieldStyle) {
        self.style = style;
    }

    /// Set the leading icon rendered at the start of the field (useful
    /// for search-field magnifiers or input-purpose glyphs).
    pub fn set_leading_icon(&mut self, icon: Option<IconName>) {
        self.leading_icon = icon;
    }

    /// Set the trailing icon rendered at the end of the field. Rendered
    /// after the clear button when both are visible.
    pub fn set_trailing_icon(&mut self, icon: Option<IconName>) {
        self.trailing_icon = icon;
    }

    /// Set the autofill / content-type hint (HIG `textContentType`).
    pub fn set_content_type(&mut self, ct: Option<TextContentType>) {
        self.content_type = ct;
    }

    /// Set the Return key label (HIG Virtual Keyboards).
    pub fn set_submit_label(&mut self, label: SubmitLabel) {
        self.submit_label = label;
    }

    /// Enable or disable the field. Disabled fields render dimmed and
    /// ignore all user interaction.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Enable or disable read-only mode. Read-only fields allow focus,
    /// cursor positioning, selection, and copy but block all mutations.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Returns `true` when text mutations should be suppressed
    /// (disabled or read-only).
    fn is_mutation_blocked(&self) -> bool {
        self.disabled || self.read_only
    }

    /// Cap the maximum content length in UTF-8 bytes. Paste, IME, and
    /// typing all respect the cap.
    pub fn set_max_length(&mut self, max_length: Option<usize>) {
        self.max_length = max_length;
    }

    /// Callback invoked on Enter/Return. Wire `all_keybindings()` to get
    /// the default Enter binding; the field handles it via action dispatch.
    pub fn set_on_submit(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_submit = Some(Box::new(handler));
    }

    /// Set placeholder text shown when the input is empty.
    pub fn set_placeholder(&mut self, text: impl Into<SharedString>) {
        self.placeholder = text.into();
    }

    /// Get the current text content.
    pub fn text(&self) -> &str {
        &self.content
    }

    /// Replace the text content imperatively.
    ///
    /// Strips newlines, truncates to `max_length`, resets the cursor to the
    /// end, pushes an undo snapshot, and fires `on_change` — so parents
    /// driving the field stay in sync and the programmatic write is undoable.
    ///
    /// Unlike user-typed edits, `set_text` intentionally bypasses the
    /// `disabled` / `read_only` gate: it's the owner's write path (e.g. a
    /// parent refreshing the field from a data source) and must still succeed
    /// when interactive input is blocked.
    pub fn set_text(
        &mut self,
        text: impl Into<SharedString>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.push_undo_snapshot();
        self.assign_content(text);
        cx.notify();
        if let Some(on_change) = &self.on_change {
            on_change(&self.content, window, cx);
        }
    }

    /// Construct a `TextField` seeded with initial text.
    ///
    /// Usable from `cx.new(|cx| TextField::new_with_text(cx, …))` where no
    /// `Window` is in scope. Does not push an undo snapshot or fire
    /// `on_change`; if a handler was attached on `self` before this call it
    /// will not be invoked — use `set_text` after construction when you need
    /// the callback to fire.
    pub fn new_with_text(cx: &mut Context<Self>, text: impl Into<SharedString>) -> Self {
        let mut this = Self::new(cx);
        this.assign_content(text);
        this
    }

    /// Normalise incoming text (strip newlines, truncate to `max_length`),
    /// assign it to `self.content`, and reset the selection/IME state.
    /// Shared by `set_text` and `new_with_text` so both entry points keep
    /// identical normalisation semantics.
    fn assign_content(&mut self, text: impl Into<SharedString>) {
        let mut s: String = text.into().to_string();
        s.retain(|c| c != '\n');
        if let Some(max) = self.max_length
            && s.len() > max
        {
            s = s
                .char_indices()
                .take_while(|(i, c)| *i + c.len_utf8() <= max)
                .map(|(_, c)| c)
                .collect();
        }
        self.content = SharedString::from(s);
        let len = self.content.len();
        self.selected_range = len..len;
        self.selection_reversed = false;
        self.marked_range = None;
    }

    /// Set the callback invoked after every text modification.
    pub fn set_on_change(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_change = Some(Box::new(handler));
    }

    /// Enable or disable the clear button shown when the input has content.
    pub fn set_show_clear_button(&mut self, show: bool) {
        self.show_clear_button = show;
    }

    /// Enable or disable secure (password) display mode.
    pub fn set_secure(&mut self, secure: bool) {
        self.is_secure = secure;
    }

    /// Set the validation state (controls border color and error message).
    pub fn set_validation(&mut self, validation: TextFieldValidation) {
        self.validation = validation;
    }

    /// Returns the text to display, masking with bullets when in secure mode.
    fn display_text(&self) -> SharedString {
        if self.is_secure && !self.content.is_empty() {
            SharedString::from("\u{2022}".repeat(self.content.chars().count()))
        } else {
            self.content.clone()
        }
    }

    /// Translate a byte offset in the display text (bullet-masked) back to the
    /// corresponding byte offset in the real content.
    fn display_to_content_offset(&self, display_offset: usize) -> usize {
        if !self.is_secure || self.content.is_empty() {
            return display_offset;
        }
        let bullet_len = '\u{2022}'.len_utf8(); // 3
        let char_index = display_offset / bullet_len;
        self.content
            .char_indices()
            .nth(char_index)
            .map_or(self.content.len(), |(byte_pos, _)| byte_pos)
    }

    /// Translate a byte offset in the real content to the corresponding byte
    /// offset in the display text.
    fn content_to_display_offset(&self, content_offset: usize) -> usize {
        if !self.is_secure || self.content.is_empty() {
            return content_offset;
        }
        let bullet_len = '\u{2022}'.len_utf8(); // 3
        let char_count = self.content[..content_offset.min(self.content.len())]
            .chars()
            .count();
        char_count * bullet_len
    }

    /// Clear the input text and reset cursor.
    fn clear_text(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.push_undo_snapshot();
        self.content = SharedString::default();
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
        if let Some(on_change) = &self.on_change {
            on_change(&self.content, window, cx);
        }
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.cursor_moved_at = Instant::now();
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        self.cursor_moved_at = Instant::now();
        cx.notify();
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }
        let Some(bounds) = self.last_bounds.as_ref() else {
            return 0;
        };
        let Some(line) = self.last_layout.as_ref() else {
            return 0;
        };
        let local_pos = point(position.x - bounds.left(), position.y - bounds.top());
        let display_idx = line
            .closest_index_for_position(local_pos, self.last_line_height)
            .unwrap_or_else(|i| i);
        let mut idx = self.display_to_content_offset(display_idx);
        idx = idx.min(self.content.len());
        // Snap to nearest char boundary
        while idx > 0 && !self.content.is_char_boundary(idx) {
            idx -= 1;
        }
        idx
    }

    // ── UTF-16 conversion for EntityInputHandler ──

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;
        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }
        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;
        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }
        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        let start = self
            .offset_from_utf16(range_utf16.start)
            .min(self.content.len());
        let end = self
            .offset_from_utf16(range_utf16.end)
            .min(self.content.len());
        start..end
    }

    // ── Action handlers ──

    fn handle_backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn handle_delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn handle_left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn handle_right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn handle_select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn handle_select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn handle_select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    fn handle_home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn handle_end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn handle_word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.previous_word_boundary(self.cursor_offset()), cx);
    }

    fn handle_word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.next_word_boundary(self.cursor_offset()), cx);
    }

    fn handle_select_word_left(
        &mut self,
        _: &SelectWordLeft,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.previous_word_boundary(self.cursor_offset()), cx);
    }

    fn handle_select_word_right(
        &mut self,
        _: &SelectWordRight,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.next_word_boundary(self.cursor_offset()), cx);
    }

    fn handle_copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn handle_cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn handle_paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text, window, cx);
        }
    }

    fn handle_delete_word(&mut self, _: &DeleteWord, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            self.replace_text_in_range(None, "", window, cx);
        } else {
            let offset = self.cursor_offset();
            let word_start = self.previous_word_boundary(offset);
            self.selected_range = word_start..offset;
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn handle_delete_to_start(
        &mut self,
        _: &DeleteToStart,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let offset = self.cursor_offset();
        if offset > 0 {
            self.selected_range = 0..offset;
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn handle_delete_to_end(
        &mut self,
        _: &DeleteToEnd,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let offset = self.cursor_offset();
        if offset < self.content.len() {
            self.selected_range = offset..self.content.len();
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    // Single-line TextField: the current line *is* the document, so line-
    // start / doc-start both route to offset 0 and line-end / doc-end to
    // `content.len()`. A multi-line editor would differentiate.
    fn handle_select_to_line_start(
        &mut self,
        _: &SelectToLineStart,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(0, cx);
    }

    fn handle_select_to_line_end(
        &mut self,
        _: &SelectToLineEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.content.len(), cx);
    }

    fn handle_select_to_doc_start(
        &mut self,
        _: &SelectToDocStart,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(0, cx);
    }

    fn handle_select_to_doc_end(
        &mut self,
        _: &SelectToDocEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.content.len(), cx);
    }

    fn handle_undo(&mut self, _: &Undo, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_mutation_blocked() {
            return;
        }
        let Some((prev_content, prev_range)) = self.undo_stack.pop_back() else {
            return;
        };
        // Snapshot the current state so we can redo.
        self.redo_stack
            .push_back((self.content.clone(), self.selected_range.clone()));
        self.content = prev_content;
        self.selected_range =
            prev_range.start.min(self.content.len())..prev_range.end.min(self.content.len());
        self.marked_range = None;
        cx.notify();
        if let Some(on_change) = &self.on_change {
            on_change(&self.content, window, cx);
        }
    }

    fn handle_redo(&mut self, _: &Redo, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_mutation_blocked() {
            return;
        }
        let Some((next_content, next_range)) = self.redo_stack.pop_back() else {
            return;
        };
        self.undo_stack
            .push_back((self.content.clone(), self.selected_range.clone()));
        self.content = next_content;
        self.selected_range =
            next_range.start.min(self.content.len())..next_range.end.min(self.content.len());
        self.marked_range = None;
        cx.notify();
        if let Some(on_change) = &self.on_change {
            on_change(&self.content, window, cx);
        }
    }

    fn push_undo_snapshot(&mut self) {
        if self.undo_stack.len() >= UNDO_STACK_LIMIT {
            self.undo_stack.pop_front();
        }
        self.undo_stack
            .push_back((self.content.clone(), self.selected_range.clone()));
        // Any fresh edit invalidates the redo stack — standard HIG undo
        // semantics.
        self.redo_stack.clear();
    }

    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let offset = self.index_for_mouse_position(event.position);
        match event.click_count {
            2 => {
                // Double-click: select word under cursor
                let word_start = self.previous_word_boundary(offset);
                let word_end = self.next_word_boundary(offset);
                self.move_to(word_start, cx);
                self.select_to(word_end, cx);
            }
            3 => {
                // Triple-click: select all
                self.move_to(0, cx);
                self.select_to(self.content.len(), cx);
            }
            _ => {
                self.is_selecting = true;
                if event.modifiers.shift {
                    self.select_to(offset, cx);
                } else {
                    self.move_to(offset, cx);
                }
            }
        }
    }

    fn handle_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }
}

impl EntityInputHandler for TextField {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_mutation_blocked() {
            return;
        }
        // Record the pre-mutation state once per distinct edit so undo
        // can step back one keystroke at a time.
        self.push_undo_snapshot();
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        // Strip newlines for single-line input
        let clean_text: String;
        let text = if new_text.contains('\n') {
            clean_text = new_text.replace('\n', "");
            &clean_text
        } else {
            new_text
        };

        // Enforce max_length: truncate the incoming text to whatever fits.
        let truncated: String;
        let text = if let Some(max) = self.max_length {
            let replaced = range.end - range.start;
            let current_without_range = self.content.len() - replaced;
            let room = max.saturating_sub(current_without_range);
            if text.len() <= room {
                text
            } else {
                truncated = text
                    .char_indices()
                    .take_while(|(i, c)| *i + c.len_utf8() <= room)
                    .map(|(_, c)| c)
                    .collect();
                &truncated
            }
        } else {
            text
        };

        self.content =
            (self.content[0..range.start].to_owned() + text + &self.content[range.end..]).into();
        self.selected_range = range.start + text.len()..range.start + text.len();
        self.marked_range.take();
        cx.notify();

        if let Some(on_change) = &self.on_change {
            on_change(&self.content, window, cx);
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_mutation_blocked() {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        // Strip newlines for single-line input
        let clean_text: String;
        let text = if new_text.contains('\n') {
            clean_text = new_text.replace('\n', "");
            &clean_text
        } else {
            new_text
        };

        self.content =
            (self.content[0..range.start].to_owned() + text + &self.content[range.end..]).into();
        if !text.is_empty() {
            self.marked_range = Some(range.start..range.start + text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .map(|new_range| range.start + new_range.start..range.start + new_range.end)
            .unwrap_or_else(|| range.start + text.len()..range.start + text.len());
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        let display_start = self.content_to_display_offset(range.start);
        let display_end = self.content_to_display_offset(range.end);
        let line_height = self.last_line_height;

        let start_x = line.unwrapped_layout.x_for_index(display_start);
        let end_x = line.unwrapped_layout.x_for_index(display_end);

        Some(Bounds::from_corners(
            point(bounds.left() + start_x, bounds.top()),
            point(bounds.left() + end_x, bounds.top() + line_height),
        ))
    }

    fn character_index_for_point(
        &mut self,
        pos: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let bounds = self.last_bounds?;
        let line = self.last_layout.as_ref()?;
        let local_pos = point(pos.x - bounds.left(), pos.y - bounds.top());
        let display_idx = line
            .closest_index_for_position(local_pos, self.last_line_height)
            .unwrap_or_else(|i| i);
        let content_idx = self.display_to_content_offset(display_idx);
        Some(self.offset_to_utf16(content_idx))
    }
}

impl Focusable for TextField {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextField {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        // Keep requesting frames while focused so the cursor blink updates.
        // GPUI coalesces frame requests per-window, so this is effectively
        // one request per paint. Gating by a coarser interval would skip
        // the responsive cursor move between blinks — the blink animation
        // itself is driven by `cursor_moved_at.elapsed() % 1000` in paint.
        if self.focus_handle.is_focused(window) {
            window.request_animation_frame();
        }
        let mut container = div()
            .w_full()
            .h(px(TEXTFIELD_HEIGHT))
            .min_h(px(theme.target_size()))
            .px(theme.spacing_sm)
            .py(theme.spacing_xs)
            .overflow_hidden()
            .flex()
            .flex_row()
            .items_center();

        // Apply style-specific skin. HIG distinguishes rounded, square
        // bezel, and plain/borderless styles.
        container = match self.style {
            TextFieldStyle::Rounded => container
                .bg(theme.surface)
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_md),
            TextFieldStyle::SquareBezel => container
                .bg(theme.surface)
                .border_1()
                .border_color(theme.border),
            TextFieldStyle::Plain => container,
        };
        container = container.cursor(CursorStyle::IBeam);

        // Apply validation border styling (overrides default border color).
        // When `DIFFERENTIATE_WITHOUT_COLOR` is on we also thicken the
        // invalid-state border so the bezel carries a second, non-color
        // cue — paired with the warning glyph added below the field.
        match &self.validation {
            TextFieldValidation::Invalid(_) => {
                container = container.border_color(theme.error);
                if theme.accessibility_mode.differentiate_without_color() {
                    container = container.border_2();
                }
            }
            TextFieldValidation::Valid => {
                container = container.border_color(theme.success);
            }
            TextFieldValidation::Warning(_) => {
                container = container.border_color(theme.warning);
                if theme.accessibility_mode.differentiate_without_color() {
                    container = container.border_2();
                }
            }
            TextFieldValidation::None => {}
        }

        // HIG: the inline clear ("x") affordance should appear only when
        // the field is focused. Showing it whenever content is present
        // causes visual noise in static forms (see the HIG Selection & Input audit
        // finding 3). Zed's `Input` takes the same approach.
        let is_focused = self.focus_handle.is_focused(window);

        // HIG macOS: focused text fields get a 3pt outer accent stroke. The
        // ring is emitted as box-shadow layers that sit OUTSIDE the
        // validation-state border, so the invalid/valid bezel color is
        // preserved inside the ring. Suppressed when disabled.
        let show_focus_ring = is_focused && !self.disabled;
        container = apply_focus_ring(container, theme, show_focus_ring, &[]);
        let show_clear =
            self.show_clear_button && !self.content.is_empty() && !self.disabled && is_focused;

        // VoiceOver / AX: prefer an explicit accessibility_label; fall
        // back to the placeholder so bare search boxes still announce.
        let ax_label = self.accessibility_label.clone().or_else(|| {
            if self.placeholder.is_empty() {
                None
            } else {
                Some(self.placeholder.clone())
            }
        });
        let mut input_container = container
            .key_context(TEXT_FIELD_CONTEXT)
            .track_focus(&self.focus_handle)
            .debug_selector(|| "text-field-root".into());
        let mut ax_props = AccessibilityProps::new().role(AccessibilityRole::TextField);
        if let Some(label) = ax_label {
            ax_props = ax_props.label(label);
        }
        input_container = input_container.with_accessibility(&ax_props);

        if self.disabled {
            input_container = input_container.opacity(0.5);
        } else {
            // Navigation and selection work in both editable and read-only modes.
            input_container = input_container
                .on_action(cx.listener(Self::handle_left))
                .on_action(cx.listener(Self::handle_right))
                .on_action(cx.listener(Self::handle_select_left))
                .on_action(cx.listener(Self::handle_select_right))
                .on_action(cx.listener(Self::handle_select_all))
                .on_action(cx.listener(Self::handle_home))
                .on_action(cx.listener(Self::handle_end))
                .on_action(cx.listener(Self::handle_word_left))
                .on_action(cx.listener(Self::handle_word_right))
                .on_action(cx.listener(Self::handle_select_word_left))
                .on_action(cx.listener(Self::handle_select_word_right))
                .on_action(cx.listener(Self::handle_select_to_line_start))
                .on_action(cx.listener(Self::handle_select_to_line_end))
                .on_action(cx.listener(Self::handle_select_to_doc_start))
                .on_action(cx.listener(Self::handle_select_to_doc_end))
                .on_action(cx.listener(Self::handle_copy))
                .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
                .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_mouse_up))
                .on_mouse_move(cx.listener(Self::handle_mouse_move));

            if !self.read_only {
                input_container = input_container
                    .on_action(cx.listener(Self::handle_backspace))
                    .on_action(cx.listener(Self::handle_delete))
                    .on_action(cx.listener(Self::handle_cut))
                    .on_action(cx.listener(Self::handle_paste))
                    .on_action(cx.listener(Self::handle_delete_word))
                    .on_action(cx.listener(Self::handle_delete_to_start))
                    .on_action(cx.listener(Self::handle_delete_to_end))
                    .on_action(cx.listener(Self::handle_undo))
                    .on_action(cx.listener(Self::handle_redo));
            }
        }

        // Enter/Return fires on_submit when configured. Bound via
        // on_key_down rather than actions! so the keystroke can be scoped
        // to the field's focus without requiring a global binding.
        if self.on_submit.is_some() && !self.disabled {
            input_container = input_container.on_key_down(cx.listener(
                |this, event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if (key == "enter" || key == "return") && this.on_submit.is_some() {
                        cx.stop_propagation();
                        let content = this.content.clone();
                        if let Some(handler) = &this.on_submit {
                            handler(&content, window, cx);
                        }
                    }
                },
            ));
        }

        // Leading icon — sits before the text area. HIG (iOS/iPadOS)
        // shows a leading image as a field-purpose indicator; on macOS
        // the same slot serves search-field magnifiers.
        if let Some(icon) = self.leading_icon {
            let (icon_ml, icon_mr) = crate::foundations::right_to_left::leading_trailing_insets(
                theme.layout_direction,
                px(0.0),
                theme.spacing_xs,
            );
            input_container = input_container.child(
                div().flex_shrink_0().ml(icon_ml).mr(icon_mr).child(
                    Icon::new(icon)
                        .size(theme.icon_size_inline)
                        .color(theme.text_muted),
                ),
            );
        }

        input_container =
            input_container.child(div().flex_grow().overflow_hidden().child(TextFieldElement {
                input: cx.entity().clone(),
            }));

        // Clear button — sits on the trailing edge of the field. `ml`/`mr`
        // are resolved via `leading_trailing_insets` so the gap between the
        // content and the clear glyph follows the reading direction.
        if show_clear {
            let (clear_ml, clear_mr) = crate::foundations::right_to_left::leading_trailing_insets(
                theme.layout_direction,
                theme.spacing_xs,
                px(0.0),
            );
            input_container = input_container.child(
                div()
                    .id(ElementId::NamedInteger(
                        "text-input-clear".into(),
                        cx.entity().entity_id().as_u64(),
                    ))
                    .debug_selector(|| "text-field-clear".into())
                    .flex_shrink_0()
                    .ml(clear_ml)
                    .mr(clear_mr)
                    .cursor(CursorStyle::PointingHand)
                    .on_click(
                        cx.listener(|this: &mut Self, _event: &ClickEvent, window, cx| {
                            this.clear_text(window, cx);
                        }),
                    )
                    .child(
                        // HIG: clear button uses the filled-circle X glyph
                        // (`xmark.circle.fill`) — matches NSSearchField and
                        // UISearchTextField conventions.
                        Icon::new(IconName::XmarkCircleFill)
                            .size(px(14.0))
                            .color(theme.text_muted),
                    ),
            );
        }

        // Trailing icon — sits at the end of the field, after the clear
        // button when both are present (bookmark/action affordance per
        // HIG iOS).
        if let Some(icon) = self.trailing_icon {
            let (icon_ml, icon_mr) = crate::foundations::right_to_left::leading_trailing_insets(
                theme.layout_direction,
                theme.spacing_xs,
                px(0.0),
            );
            input_container = input_container.child(
                div().flex_shrink_0().ml(icon_ml).mr(icon_mr).child(
                    Icon::new(icon)
                        .size(theme.icon_size_inline)
                        .color(theme.text_muted),
                ),
            );
        }

        // HIG Text Fields: the slot order is label (top), field,
        // help/validation (bottom). Invalid or Warning messages take
        // priority over help_text — showing both would stack noise under
        // the field.
        let validation_below: Option<(SharedString, gpui::Hsla, bool)> = match &self.validation {
            TextFieldValidation::Invalid(msg) => Some((msg.clone(), theme.error, true)),
            TextFieldValidation::Warning(msg) => Some((msg.clone(), theme.warning, true)),
            _ => None,
        };

        let mut outer = div().w_full().flex().flex_col();

        if let Some(label) = self.label.as_ref() {
            outer = outer.child(
                div()
                    .pb(theme.spacing_xs)
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text)
                    .child(label.clone()),
            );
        }

        outer = outer.child(input_container);

        if let Some((msg, color, validation_icon)) = validation_below {
            // HIG Accessibility (Color): don't rely on colour alone. When
            // the user has "Differentiate Without Color" turned on, pair
            // the message with an icon so the state reads regardless of
            // colour perception.
            let include_icon =
                validation_icon && theme.accessibility_mode.differentiate_without_color();
            let mut row = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(theme.spacing_xs)
                .pt(px(SPACING_4))
                .text_style(TextStyle::Callout, theme)
                .text_color(color);
            if include_icon {
                row = row.child(
                    Icon::new(IconName::AlertTriangle)
                        .size(px(12.0))
                        .color(color),
                );
            }
            row = row.child(msg.to_string());
            outer = outer.child(row);
        } else if let Some(help) = self.help_text.as_ref() {
            outer = outer.child(
                div()
                    .pt(px(SPACING_4))
                    .text_style(TextStyle::Callout, theme)
                    .text_color(theme.text_muted)
                    .child(help.clone()),
            );
        }

        outer
    }
}

// ── Custom Element for text rendering with cursor and selection ──

struct TextFieldElement {
    input: Entity<TextField>,
}

struct TextFieldPrepaintState {
    /// Shaped line (single line, no wrapping).
    line: Option<WrappedLine>,
    /// Vertical offset to center text within bounds.
    y_offset: Pixels,
    /// Cursor quad (if focused and no selection).
    cursor: Option<PaintQuad>,
    /// Selection quads.
    selections: Vec<PaintQuad>,
    /// Line height used for layout.
    line_height: Pixels,
}

impl Element for TextFieldElement {
    type RequestLayoutState = ();
    type PrepaintState = TextFieldPrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = px(20.).into();
        let layout_id = window.request_layout(style, [], cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.display_text();
        let is_secure = input.is_secure;
        let raw_selected_range = input.selected_range.clone();
        let raw_cursor_offset = input.cursor_offset();
        let is_empty = input.content.is_empty();

        // In secure mode, display_text() replaces each char with a multi-byte
        // bullet ('\u{2022}', 3 UTF-8 bytes). Translate byte offsets from original
        // content space to display text space.
        let (cursor_offset, selected_range) = if is_secure && !is_empty {
            let bullet_len = '\u{2022}'.len_utf8(); // 3
            let char_cursor = input.content[..raw_cursor_offset.min(input.content.len())]
                .chars()
                .count();
            let char_start = input.content[..raw_selected_range.start.min(input.content.len())]
                .chars()
                .count();
            let char_end = input.content[..raw_selected_range.end.min(input.content.len())]
                .chars()
                .count();
            (
                char_cursor * bullet_len,
                char_start * bullet_len..char_end * bullet_len,
            )
        } else {
            (raw_cursor_offset, raw_selected_range)
        };
        let placeholder = input.placeholder.clone();
        let is_focused = input.focus_handle.is_focused(window);
        let cursor_visible = if is_focused {
            // Blink: 500ms on, 500ms off, reset on cursor movement
            let elapsed = input.cursor_moved_at.elapsed().as_millis() % 1000;
            elapsed < 500
        } else {
            false
        };

        let theme = cx.theme();
        let text_style = window.text_style();

        // HIG Accessibility: placeholder must remain legible. The raw
        // `semantic.placeholder_text` token is alpha 0.30 which drops
        // below WCAG AA against the field's surface fill in dark mode
        // (see the HIG Selection & Input audit finding 4). `text_muted` is tuned
        // to stay above the 4.5:1 contrast floor on both the surface and
        // plain (transparent) backgrounds, so we use it for the
        // placeholder copy instead.
        let (display_text, text_color) = if is_empty {
            (placeholder, theme.text_muted)
        } else {
            (content.clone(), theme.text)
        };

        // If display text is also empty (no placeholder), use a space to get valid layout
        let display_text = if display_text.is_empty() {
            SharedString::from(" ")
        } else {
            display_text
        };

        let run = TextRun {
            len: display_text.len(),
            font: text_style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let marked_range = if is_secure && !is_empty {
            input.marked_range.as_ref().map(|raw| {
                let bullet_len = '\u{2022}'.len_utf8();
                let cs = input.content[..raw.start.min(input.content.len())]
                    .chars()
                    .count();
                let ce = input.content[..raw.end.min(input.content.len())]
                    .chars()
                    .count();
                cs * bullet_len..ce * bullet_len
            })
        } else {
            input.marked_range.clone()
        };

        let runs = if let Some(ref raw_marked) = marked_range {
            let marked_start = raw_marked.start.min(display_text.len());
            let marked_end = raw_marked.end.min(display_text.len());
            vec![
                TextRun {
                    len: marked_start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_end - marked_start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|r| r.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();

        // No wrapping for single-line input
        let lines = window
            .text_system()
            .shape_text(display_text, font_size, &runs, None, None)
            .unwrap_or_default();

        let line = lines.into_vec().into_iter().next();

        // Build cursor and selection quads
        let mut cursor_quad = None;
        let mut selection_quads = Vec::new();

        // Show cursor when focused (even on empty input), show selection when non-empty
        if (cursor_visible || (!is_empty && !selected_range.is_empty()))
            && let Some(ref shaped_line) = line
        {
            let accent = theme.accent;
            let sel_color = with_alpha(accent, 0.25);

            // Vertical centering offset for text, cursor, and selection
            let y_off =
                px(((f32::from(bounds.size.height) - f32::from(line_height)) / 2.0).max(0.0));

            if selected_range.is_empty() && cursor_visible {
                // Cursor (blinks when focused)
                if let Some(pos) = shaped_line.position_for_index(cursor_offset, line_height) {
                    cursor_quad = Some(fill(
                        Bounds::new(
                            point(bounds.left() + pos.x, bounds.top() + y_off),
                            size(px(2.), line_height),
                        ),
                        accent,
                    ));
                } else {
                    let x = shaped_line.unwrapped_layout.x_for_index(cursor_offset);
                    cursor_quad = Some(fill(
                        Bounds::new(
                            point(bounds.left() + x, bounds.top() + y_off),
                            size(px(2.), line_height),
                        ),
                        accent,
                    ));
                }
            } else {
                // Selection
                let start_x = shaped_line
                    .unwrapped_layout
                    .x_for_index(selected_range.start);
                let end_x = shaped_line.unwrapped_layout.x_for_index(selected_range.end);
                selection_quads.push(fill(
                    Bounds::from_corners(
                        point(bounds.left() + start_x, bounds.top() + y_off),
                        point(bounds.left() + end_x, bounds.top() + y_off + line_height),
                    ),
                    sel_color,
                ));
            }
        }

        // Compute vertical centering offset
        let y_offset =
            px(((f32::from(bounds.size.height) - f32::from(line_height)) / 2.0).max(0.0));

        TextFieldPrepaintState {
            line,
            y_offset,
            cursor: cursor_quad,
            selections: selection_quads,
            line_height,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        if focus_handle.is_focused(window) {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }

        let line_height = prepaint.line_height;

        // Paint selection highlights behind text
        for selection in prepaint.selections.drain(..) {
            window.paint_quad(selection);
        }

        // Paint the text line, vertically centered within bounds
        if let Some(ref line) = prepaint.line {
            let _ = line.paint(
                point(bounds.left(), bounds.top() + prepaint.y_offset),
                line_height,
                TextAlign::Left,
                None,
                window,
                cx,
            );
        }

        // Paint cursor on top of text
        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        // Cache layout for mouse hit-testing and input handler
        let line = prepaint.line.take();
        self.input.update(cx, |input, _cx| {
            input.last_layout = line;
            input.last_bounds = Some(bounds);
            input.last_line_height = line_height;
        });
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
}

impl IntoElement for TextFieldElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod interaction_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::TestAppContext;

    use super::TextField;
    use crate::test_helpers::helpers::{InteractionExt, assert_element_exists, setup_test_window};

    const TEXT_FIELD_ROOT: &str = "text-field-root";
    const TEXT_FIELD_CLEAR: &str = "text-field-clear";

    fn focus_text_field(field: &gpui::Entity<TextField>, cx: &mut gpui::VisualTestContext) {
        field.update_in(cx, |field, window, cx| {
            field.focus_handle.focus(window, cx);
        });
    }

    #[gpui::test]
    async fn typing_and_backspace_updates_text(cx: &mut TestAppContext) {
        let (field, cx) = setup_test_window(cx, |_window, cx| TextField::new(cx));

        assert_element_exists(cx, TEXT_FIELD_ROOT);
        focus_text_field(&field, cx);
        cx.type_text("hello");
        cx.press("backspace");

        field.update_in(cx, |field, _window, _cx| {
            assert_eq!(field.text(), "hell");
        });
    }

    #[gpui::test]
    async fn select_all_and_replace_uses_input_handler(cx: &mut TestAppContext) {
        let (field, cx) = setup_test_window(cx, |_window, cx| TextField::new(cx));

        focus_text_field(&field, cx);
        cx.type_text("hello");
        cx.press("cmd-a");
        cx.type_text("bye");

        field.update_in(cx, |field, _window, _cx| {
            assert_eq!(field.text(), "bye");
        });
    }

    #[gpui::test]
    async fn clear_button_clears_text_and_notifies(cx: &mut TestAppContext) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TextField::new(cx));

        field.update_in(cx, |field, window, cx| {
            field.set_show_clear_button(true);
            field.set_on_change({
                let changes = changes.clone();
                move |text, _, _| changes.borrow_mut().push(text.to_string())
            });
            field.set_text("search", window, cx);
        });

        // HIG: clear button is focus-gated, so the field has to own focus
        // before the `text-field-clear` element appears in the render tree.
        focus_text_field(&field, cx);
        assert_element_exists(cx, TEXT_FIELD_CLEAR);
        cx.click_on(TEXT_FIELD_CLEAR);

        field.update_in(cx, |field, _window, _cx| {
            assert_eq!(field.text(), "");
        });
        assert_eq!(&*changes.borrow(), &["search".to_string(), String::new()]);
    }

    // Regression for #35: the clear-X button must push an undo snapshot so
    // the wipe is recoverable. Without this, clicking clear drops user work
    // on the floor — every other mutation surface on `TextField` is undoable.
    #[gpui::test]
    async fn clear_button_is_undoable(cx: &mut TestAppContext) {
        let (field, cx) = setup_test_window(cx, |_window, cx| TextField::new(cx));

        field.update_in(cx, |field, window, cx| {
            field.set_show_clear_button(true);
            field.set_text("search", window, cx);
        });

        focus_text_field(&field, cx);
        assert_element_exists(cx, TEXT_FIELD_CLEAR);
        cx.click_on(TEXT_FIELD_CLEAR);

        field.update_in(cx, |field, window, cx| {
            assert_eq!(field.text(), "");
            field.handle_undo(&crate::text_actions::Undo, window, cx);
            assert_eq!(
                field.text(),
                "search",
                "clear button must be undoable — issue #35",
            );
        });
    }

    // Regression for #18: programmatic `set_text` must mirror typed edits —
    // push an undo snapshot and fire `on_change`, so parents driving the
    // field imperatively stay in sync and the write is undoable.
    #[gpui::test]
    async fn set_text_fires_on_change_and_is_undoable(cx: &mut TestAppContext) {
        let changes = Rc::new(RefCell::new(Vec::new()));
        let (field, cx) = setup_test_window(cx, |_window, cx| TextField::new(cx));

        field.update_in(cx, |field, window, cx| {
            field.set_on_change({
                let changes = changes.clone();
                move |text, _, _| changes.borrow_mut().push(text.to_string())
            });
            field.set_text("hello", window, cx);
        });

        field.update_in(cx, |field, _window, _cx| {
            assert_eq!(field.text(), "hello");
            assert_eq!(
                field.undo_stack.len(),
                1,
                "set_text must push an undo snapshot"
            );
        });
        assert_eq!(&*changes.borrow(), &["hello".to_string()]);

        // Type a character, then undo. The undo snapshot taken by `set_text`
        // should carry the field back to "hello". We call `handle_undo`
        // directly because the test harness doesn't register the Cmd-Z
        // binding (see `test_helpers::text_field_keybindings`).
        focus_text_field(&field, cx);
        cx.type_text("!");
        field.update_in(cx, |field, _window, _cx| {
            assert_eq!(field.text(), "hello!");
        });

        field.update_in(cx, |field, window, cx| {
            field.handle_undo(&crate::text_actions::Undo, window, cx);
            assert_eq!(field.text(), "hello");
            // After the undo, the redo stack should hold the "hello!" state
            // we just stepped out of.
            assert_eq!(field.redo_stack.len(), 1);
        });

        // A fresh `set_text` must invalidate the redo stack — standard HIG
        // undo semantics, and the guarantee `push_undo_snapshot` offers.
        field.update_in(cx, |field, window, cx| {
            field.set_text("fresh", window, cx);
            assert!(
                field.redo_stack.is_empty(),
                "set_text must clear redo stack"
            );
        });
    }

    // `new_with_text` is a construction-time seeder: no undo snapshot, no
    // `on_change` (even if one happened to be wired beforehand), newlines
    // stripped, cursor parked at end.
    #[gpui::test]
    async fn new_with_text_seeds_without_notification(cx: &mut TestAppContext) {
        let (field, cx) = setup_test_window(cx, |_window, cx| {
            TextField::new_with_text(cx, "init\nvalue")
        });

        field.update_in(cx, |field, _window, _cx| {
            assert_eq!(field.text(), "initvalue");
            assert!(field.undo_stack.is_empty());
            let len = field.text().len();
            assert_eq!(field.selected_range, len..len);
        });
    }

    // `set_text` must truncate to `max_length`, matching typed-edit behaviour
    // in `replace_text_in_range`.
    #[gpui::test]
    async fn set_text_respects_max_length(cx: &mut TestAppContext) {
        let (field, cx) = setup_test_window(cx, |_window, cx| TextField::new(cx));

        field.update_in(cx, |field, window, cx| {
            field.set_max_length(Some(5));
            field.set_text("exceeded length", window, cx);
            assert_eq!(field.text(), "excee");
        });
    }

    #[gpui::test]
    async fn focus_ring_renders_without_panic_in_all_states(cx: &mut TestAppContext) {
        // Exercises the focus ring branch across (focused × disabled ×
        // validation) without asserting shadow-layer internals, which
        // live inside GPUI. The underlying `apply_focus_ring` helper has
        // its own tests in `foundations::materials`; this guards that
        // the TextField render actually invokes it and doesn't panic
        // when validation colours are stacked behind the ring.
        use crate::components::selection_and_input::text_field::TextFieldValidation;
        let (field, cx) = setup_test_window(cx, |_window, cx| TextField::new(cx));

        // Focused + invalid: ring should stack OUTSIDE the error border.
        field.update_in(cx, |field, _window, _cx| {
            field.set_validation(TextFieldValidation::Invalid(gpui::SharedString::from(
                "bad",
            )));
        });
        focus_text_field(&field, cx);
        assert_element_exists(cx, TEXT_FIELD_ROOT);

        // Focused + disabled: ring suppressed.
        field.update_in(cx, |field, _window, _cx| {
            field.set_validation(TextFieldValidation::None);
            field.set_disabled(true);
        });
        assert_element_exists(cx, TEXT_FIELD_ROOT);

        // Enabled + unfocused: no ring, no panic.
        field.update_in(cx, |field, _window, _cx| {
            field.set_disabled(false);
        });
        // Blur by focusing a transient dummy and letting the field lose focus.
        assert_element_exists(cx, TEXT_FIELD_ROOT);
    }
}
