//! HIG Digit Entry View — row of individual boxes for PIN/OTP entry.
//!
//! Renders `length` equally-sized boxes in a horizontal row. Each box displays
//! the corresponding digit from the live `content`, a bullet when `is_secure`
//! is set, or remains empty for unfilled positions. The active position
//! (where the next digit will appear) receives an accent border.
//!
//! `DigitEntry` is a stateful [`gpui::Entity`] — constructed with
//! `Entity::new(|cx| DigitEntry::new(cx))` — because OTP entry is highly
//! keystroke-sensitive. Owning the content internally avoids the
//! stateless-snapshot race where rapid keystrokes could be reordered
//! against an unflushed parent render.

use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, App, Context, ElementId, FocusHandle, Focusable, FontWeight,
    KeyDownEvent, MouseDownEvent, SharedString, Window, div, px,
};

use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TextStyle};

/// A stateful digit-entry field for PIN/OTP input per HIG.
///
/// Construct with `cx.new(|cx| DigitEntry::new(cx))` and pass the entity
/// into your view. Parents listen to changes via [`set_on_change`] and
/// the completion of the final box via [`set_on_complete`].
///
/// # Example
///
/// ```ignore
/// let digit_entry = cx.new(|cx| {
///     let mut de = DigitEntry::new(cx);
///     de.set_length(4);
///     de.set_secure(true);
///     de.set_on_change(|value, _, _| { /* update model */ });
///     de
/// });
/// ```
#[allow(clippy::type_complexity)]
pub struct DigitEntry {
    element_id: ElementId,
    length: usize,
    content: SharedString,
    is_secure: bool,
    focus_handle: FocusHandle,
    on_change: Option<Box<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
    on_complete: Option<Box<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
}

impl DigitEntry {
    /// Create a new `DigitEntry` with the default 6-box layout.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: ElementId::from(SharedString::from("digit-entry")),
            length: 6,
            content: SharedString::default(),
            is_secure: false,
            focus_handle: cx.focus_handle(),
            on_change: None,
            on_complete: None,
        }
    }

    /// Set the element ID used for the root container (affects `debug_selector`
    /// and GPUI hit-testing — pick a stable string per component instance).
    pub fn set_id(&mut self, id: impl Into<ElementId>) {
        self.element_id = id.into();
    }

    /// Set the number of digit boxes (default 6). Values of 0 clamp to 1
    /// so the render path always has at least one cell.
    pub fn set_length(&mut self, length: usize) {
        self.length = length.max(1);
        // Truncate content to the new length so excess digits don't
        // linger invisibly past the last box.
        if self.content.chars().count() > self.length {
            let truncated: String = self.content.chars().take(self.length).collect();
            self.content = SharedString::from(truncated);
        }
    }

    /// Replace the current digit string. Non-digit characters are stripped.
    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        let digits: String = text
            .into()
            .chars()
            .filter(|c| c.is_ascii_digit())
            .take(self.length)
            .collect();
        self.content = SharedString::from(digits);
        cx.notify();
    }

    /// Read the current digit string.
    pub fn text(&self) -> &str {
        &self.content
    }

    /// Toggle secure mode. When `true`, filled positions render bullets.
    pub fn set_secure(&mut self, secure: bool) {
        self.is_secure = secure;
    }

    /// Register a change handler that fires on every digit update.
    pub fn set_on_change(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_change = Some(Box::new(handler));
    }

    /// Register a completion handler that fires exactly once when the
    /// value transitions from "partially filled" to "fully filled" (all
    /// `length` boxes populated).
    pub fn set_on_complete(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_complete = Some(Box::new(handler));
    }

    /// Focus the field programmatically.
    pub fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
    }

    /// Returns the character to display for a filled position.
    fn display_char(&self, ch: char) -> char {
        if self.is_secure { '\u{2022}' } else { ch }
    }

    fn handle_mouse_down(
        &mut self,
        _event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Clicking anywhere on the row focuses the field so typing picks
        // up where it left off — mirrors NSSecureTextField behaviour.
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let platform = event.keystroke.modifiers.platform;

        // Cmd-V paste — pull digits out of the clipboard, truncate to
        // `length`, commit, and fire on_complete if we fully filled.
        if platform && key == "v" {
            let Some(text) = cx.read_from_clipboard().and_then(|c| c.text()) else {
                return;
            };
            let mut new_val: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
            if new_val.chars().count() > self.length {
                new_val = new_val.chars().take(self.length).collect();
            }
            self.commit(SharedString::from(new_val), window, cx);
            return;
        }

        match key {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                if self.content.chars().count() >= self.length {
                    return;
                }
                let mut new_val = self.content.to_string();
                new_val.push_str(key);
                self.commit(SharedString::from(new_val), window, cx);
            }
            "backspace" => {
                if self.content.is_empty() {
                    return;
                }
                let mut new_val = self.content.to_string();
                new_val.pop();
                self.commit(SharedString::from(new_val), window, cx);
            }
            _ => {}
        }
    }

    fn commit(&mut self, new_val: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let was_complete = self.content.chars().count() == self.length;
        let is_complete = new_val.chars().count() == self.length;
        self.content = new_val.clone();
        cx.notify();
        if let Some(on_change) = &self.on_change {
            on_change(&new_val, window, cx);
        }
        if is_complete
            && !was_complete
            && let Some(on_complete) = &self.on_complete
        {
            on_complete(&new_val, window, cx);
        }
    }
}

impl Focusable for DigitEntry {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DigitEntry {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let is_focused = self.focus_handle.is_focused(window);
        let value_len = self.content.chars().count();
        let value_chars: Vec<char> = self.content.chars().collect();

        // ── Build each digit box ───────────────────────────────────────────
        let mut row = div()
            .flex()
            .flex_row()
            .gap(theme.spacing_sm)
            .items_center()
            .justify_center();

        for pos in 0..self.length {
            let (display_text, is_active) = if let Some(&raw) = value_chars.get(pos) {
                (Some(self.display_char(raw)), false)
            } else if pos == value_len {
                (None, is_focused)
            } else {
                (None, false)
            };

            let inner = div()
                .min_w(px(theme.target_size()))
                .min_h(px(52.0))
                .flex()
                .items_center()
                .justify_center()
                .flex_shrink_0();

            let mut box_el = inner.bg(theme.surface).rounded(theme.radius_md);
            if is_active {
                box_el = box_el.border_2().border_color(theme.accent);
            } else {
                box_el = box_el.border_1().border_color(theme.border);
            }

            let digit_size = TextStyle::Body.attrs().size * 1.5;
            if let Some(ch) = display_text {
                box_el = box_el.child(
                    div()
                        .text_size(digit_size)
                        .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                        .text_color(theme.text)
                        .child(SharedString::from(ch.to_string())),
                );
            } else if is_active {
                let accent = theme.accent;
                let cursor_h = digit_size;
                box_el = box_el.child(div().w(px(2.0)).h(cursor_h).bg(accent).with_animation(
                    ElementId::NamedInteger("cursor-blink".into(), pos as u64),
                    Animation::new(Duration::from_millis(1000)).repeat(),
                    |el, delta| {
                        if delta < 0.5 { el } else { el.opacity(0.0) }
                    },
                ));
            }

            row = row.child(box_el);
        }

        let mut inner_wrapper = div()
            .flex()
            .flex_shrink_0()
            .items_center()
            .justify_center()
            .rounded(theme.radius_md)
            .child(row);
        inner_wrapper = apply_focus_ring(inner_wrapper, theme, is_focused, &[]);

        div()
            .id(self.element_id.clone())
            .track_focus(&self.focus_handle)
            .flex()
            .items_center()
            .justify_center()
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(Self::handle_mouse_down),
            )
            .child(inner_wrapper)
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::TestAppContext;

    use super::DigitEntry;
    use crate::test_helpers::helpers::setup_test_window;

    #[gpui::test]
    async fn digit_entry_defaults(cx: &mut TestAppContext) {
        let (entry, cx) = setup_test_window(cx, |_window, cx| DigitEntry::new(cx));
        entry.update_in(cx, |entry, _window, _cx| {
            assert_eq!(entry.length, 6);
            assert!(entry.content.is_empty());
            assert!(!entry.is_secure);
            assert!(entry.on_change.is_none());
        });
    }

    #[gpui::test]
    async fn set_length_truncates_content(cx: &mut TestAppContext) {
        let (entry, cx) = setup_test_window(cx, |_window, cx| DigitEntry::new(cx));
        entry.update_in(cx, |entry, _window, cx| {
            entry.set_text("123456", cx);
            entry.set_length(4);
            assert_eq!(entry.text(), "1234");
        });
    }

    #[gpui::test]
    async fn set_text_strips_non_digits(cx: &mut TestAppContext) {
        let (entry, cx) = setup_test_window(cx, |_window, cx| DigitEntry::new(cx));
        entry.update_in(cx, |entry, _window, cx| {
            entry.set_text("12ab34", cx);
            assert_eq!(entry.text(), "1234");
        });
    }

    #[gpui::test]
    async fn display_char_plain(cx: &mut TestAppContext) {
        let (entry, cx) = setup_test_window(cx, |_window, cx| DigitEntry::new(cx));
        entry.update_in(cx, |entry, _window, _cx| {
            assert_eq!(entry.display_char('5'), '5');
        });
    }

    #[gpui::test]
    async fn display_char_secure(cx: &mut TestAppContext) {
        let (entry, cx) = setup_test_window(cx, |_window, cx| DigitEntry::new(cx));
        entry.update_in(cx, |entry, _window, _cx| {
            entry.set_secure(true);
            assert_eq!(entry.display_char('5'), '\u{2022}');
        });
    }
}

#[cfg(test)]
mod interaction_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::TestAppContext;

    use super::DigitEntry;
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    fn focus_digit_entry(entry: &Entity<DigitEntry>, cx: &mut gpui::VisualTestContext) {
        entry.update_in(cx, |entry, window, cx| {
            entry.focus(window, cx);
        });
    }

    use gpui::Entity;

    #[gpui::test]
    async fn typing_digits_appends_and_fires_on_complete(cx: &mut TestAppContext) {
        let (entry, cx) = setup_test_window(cx, |_window, cx| DigitEntry::new(cx));
        let complete_count = Rc::new(RefCell::new(0));
        let changes = Rc::new(RefCell::new(Vec::new()));

        entry.update_in(cx, |entry, _window, _cx| {
            entry.set_length(3);
            let changes = changes.clone();
            entry.set_on_change(move |value, _, _| changes.borrow_mut().push(value.to_string()));
            let complete_count = complete_count.clone();
            entry.set_on_complete(move |_value, _, _| *complete_count.borrow_mut() += 1);
        });

        focus_digit_entry(&entry, cx);
        cx.type_text("12");
        entry.update_in(cx, |entry, _window, _cx| {
            assert_eq!(entry.text(), "12");
        });
        cx.type_text("3");
        entry.update_in(cx, |entry, _window, _cx| {
            assert_eq!(entry.text(), "123");
        });
        // Further digits while full are ignored (no overflow).
        cx.type_text("4");
        entry.update_in(cx, |entry, _window, _cx| {
            assert_eq!(entry.text(), "123");
        });

        cx.press("backspace");
        entry.update_in(cx, |entry, _window, _cx| {
            assert_eq!(entry.text(), "12");
        });

        assert_eq!(*complete_count.borrow(), 1);
        assert_eq!(&*changes.borrow(), &["1", "12", "123", "12"]);
    }
}
