//! HIG Pop-up Button -- dropdown selector showing mutually-exclusive options.
//!
//! A stateless `RenderOnce` component that renders a trigger button displaying the
//! current selection plus a chevron. When open, an absolute-positioned dropdown
//! list appears below the trigger with a checkmark on the selected item.
//!
//! ## Keyboard auto-focus (HIG)
//!
//! HIG says the open dropdown should place keyboard focus on the currently
//! selected row so arrow-key navigation starts from the right place. Because
//! `PopupButton` is `RenderOnce` (stateless), the parent *must* call
//! `window.focus(&popup_focus_handle)` in the same update that flips
//! `is_open` from `false` to `true`. Callers that want the focus nudge to
//! happen automatically should use the stateful `PopupButtonController`
//! helper (future work; tracked via #142 F31). The `dropdown` element id
//! remains stable across renders so the focus handle can be stored
//! alongside it by the parent.

use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, SharedString, Window, deferred, div,
    px,
};

use std::rc::Rc;

use crate::callback_types::{OnSharedStringRefChange, OnToggle, rc_wrap};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{apply_standard_control_styling, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// Callback invoked when keyboard highlight changes in a [`PopupButton`] dropdown.
pub type OnHighlight = Option<Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>>;

/// A single option in a [`PopupButton`].
pub struct PopupItem {
    /// Display text shown in the trigger and option list.
    pub label: SharedString,
    /// Unique value used for selection matching and `on_change` callbacks.
    pub value: SharedString,
}

impl PopupItem {
    /// Create a new popup item with the given label and value.
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// HIG Pop-up Button -- dropdown selector for mutually-exclusive options.
///
/// Closed state shows the selected item's label (or the first item if none
/// selected) plus a chevron icon. Open state adds an absolute-positioned
/// dropdown list below the trigger. Selection state and open/closed state are
/// owned by the parent.
#[derive(IntoElement)]
pub struct PopupButton {
    id: ElementId,
    items: Vec<PopupItem>,
    selected: Option<SharedString>,
    is_open: bool,
    disabled: bool,
    focused: bool,
    /// Optional focus handle; when set, the popup tracks GPUI's focus
    /// graph and lights the ring reactively. Takes precedence over
    /// [`PopupButton::focused`].
    focus_handle: Option<FocusHandle>,
    compact: bool,
    highlighted_index: Option<usize>,
    on_change: OnSharedStringRefChange,
    on_toggle: OnToggle,
    on_highlight: OnHighlight,
}

impl PopupButton {
    /// Create a new pop-up button with the given element id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            selected: None,
            is_open: false,
            disabled: false,
            focused: false,
            focus_handle: None,
            compact: false,
            highlighted_index: None,
            on_change: None,
            on_toggle: None,
            on_highlight: None,
        }
    }

    /// Use a compact 22pt dropdown row height matching native `NSPopUpButton`
    /// mini-controls. The trigger keeps the standard target size so it
    /// remains click-target compliant.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Set the list of selectable items.
    pub fn items(mut self, items: Vec<PopupItem>) -> Self {
        self.items = items;
        self
    }

    /// Set the currently selected value.
    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    /// Set the open/closed state of the dropdown.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set the disabled state. When disabled, the button is visually dimmed
    /// and click/keyboard handlers are suppressed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the focused state for rendering a focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Wire the pop-up into GPUI's focus graph. When set, the focus ring
    /// renders based on `handle.is_focused(window)` — takes precedence
    /// over [`PopupButton::focused`].
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Set the handler called when the user selects an option.
    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Set the handler called when the dropdown opens or closes.
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Set the currently keyboard-highlighted item index in the dropdown list.
    pub fn highlighted_index(mut self, index: Option<usize>) -> Self {
        self.highlighted_index = index;
        self
    }

    /// Set the handler called when keyboard navigation moves the highlight.
    pub fn on_highlight(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_highlight = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PopupButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        // Resolve the display label: selected item's label, or first item, or empty.
        let trigger_label: SharedString = self
            .selected
            .as_ref()
            .and_then(|sel| self.items.iter().find(|i| &i.value == sel))
            .map(|item| item.label.clone())
            .or_else(|| self.items.first().map(|i| i.label.clone()))
            .unwrap_or_else(|| SharedString::from(""));

        // Wrap callbacks in Rc so they can be shared across multiple closures.
        let on_toggle = rc_wrap(self.on_toggle);
        let on_change = rc_wrap(self.on_change);

        // ── Trigger button ──────────────────────────────────────────────────
        let toggle_for_trigger = on_toggle.clone();
        let trigger_key_toggle = on_toggle.clone();
        let is_open = self.is_open;

        let trigger_content = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child(trigger_label),
            )
            .child(
                // HIG Pop-up buttons: trailing glyph is the up/down double
                // chevron (`chevron.up.chevron.down`), distinct from the
                // single `chevron.down` used by pull-down buttons.
                Icon::new(IconName::ChevronsUpDown)
                    .size(px(12.0))
                    .color(theme.text_muted),
            );

        let disabled = self.disabled;

        let mut trigger = div()
            .id(self.id.clone())
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .px(theme.spacing_md);

        if !disabled {
            trigger = trigger.cursor_pointer().focusable();
            if let Some(handle) = self.focus_handle.as_ref() {
                trigger = trigger.track_focus(handle);
            }
        }

        // Glass-styled trigger surface.
        trigger = apply_standard_control_styling(trigger, theme, GlassSize::Small, focused);

        if disabled {
            trigger = trigger.opacity(0.5).cursor_default();
        } else {
            trigger = trigger.hover(|style| style.cursor_pointer());
        }

        trigger = trigger.child(trigger_content);

        if !disabled && let Some(handler) = toggle_for_trigger {
            trigger = trigger.on_click(move |_event, window, cx| {
                handler(!is_open, window, cx);
            });
        }

        // Trigger keyboard activation: Enter/Space opens the dropdown.
        if !disabled && let Some(handler) = trigger_key_toggle {
            trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_activation_key(event) && !is_open {
                    cx.stop_propagation();
                    handler(true, window, cx);
                }
            });
        }

        // ── Container (trigger + optional dropdown) ─────────────────────────
        let mut container = div().relative().child(trigger);

        if self.is_open {
            // ── Dropdown list ───────────────────────────────────────────────
            let item_count = self.items.len();
            let highlighted = self.highlighted_index;
            let values: Vec<SharedString> =
                self.items.iter().map(|item| item.value.clone()).collect();
            let labels_lower: Vec<String> = self
                .items
                .iter()
                .map(|item| item.label.to_lowercase())
                .collect();
            let on_highlight = self.on_highlight.map(Rc::new);

            let mut list = glass_surface(
                div()
                    .absolute()
                    .left_0()
                    .top(theme.dropdown_top())
                    .w_full()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .max_h(px(DROPDOWN_MAX_HEIGHT)),
                theme,
                GlassSize::Medium,
            )
            .id(ElementId::from((self.id.clone(), "dropdown")))
            .focusable();

            // Keyboard navigation: arrow keys, Home/End, Enter, Escape, type-ahead.
            let key_toggle = on_toggle.clone();
            let key_change = on_change.clone();
            let key_highlight = on_highlight.clone();
            list = list.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_escape_key(event) {
                    if let Some(handler) = &key_toggle {
                        handler(false, window, cx);
                    }
                    return;
                }
                if item_count == 0 {
                    return;
                }
                let key = event.keystroke.key.as_str();
                match key {
                    "down" => {
                        cx.stop_propagation();
                        let next = match highlighted {
                            Some(i) if i + 1 < item_count => i + 1,
                            Some(_) => 0,
                            None => 0,
                        };
                        if let Some(handler) = &key_highlight {
                            handler(Some(next), window, cx);
                        }
                    }
                    "up" => {
                        cx.stop_propagation();
                        let prev = match highlighted {
                            Some(0) | None => item_count - 1,
                            Some(i) => i - 1,
                        };
                        if let Some(handler) = &key_highlight {
                            handler(Some(prev), window, cx);
                        }
                    }
                    "home" => {
                        cx.stop_propagation();
                        if let Some(handler) = &key_highlight {
                            handler(Some(0), window, cx);
                        }
                    }
                    "end" => {
                        cx.stop_propagation();
                        if let Some(handler) = &key_highlight {
                            handler(Some(item_count - 1), window, cx);
                        }
                    }
                    "enter" => {
                        cx.stop_propagation();
                        if let Some(idx) = highlighted
                            && let Some(value) = values.get(idx)
                        {
                            if let Some(handler) = &key_change {
                                handler(value, window, cx);
                            }
                            if let Some(handler) = &key_toggle {
                                handler(false, window, cx);
                            }
                        }
                    }
                    _ => {
                        // Type-ahead: match first character of a printable key against label.
                        let typed = event
                            .keystroke
                            .key_char
                            .as_deref()
                            .or(Some(key))
                            .filter(|s| s.chars().count() == 1);
                        if let Some(ch) = typed {
                            let ch_lower = ch.to_lowercase();
                            let start = highlighted.map(|i| (i + 1) % item_count).unwrap_or(0);
                            let mut found = None;
                            for offset in 0..item_count {
                                let idx = (start + offset) % item_count;
                                if labels_lower[idx].starts_with(&ch_lower) {
                                    found = Some(idx);
                                    break;
                                }
                            }
                            if let Some(idx) = found {
                                cx.stop_propagation();
                                if let Some(handler) = &key_highlight {
                                    handler(Some(idx), window, cx);
                                }
                            }
                        }
                    }
                }
            });

            // Close dropdown on click outside.
            let mouse_out_toggle = on_toggle.clone();
            if let Some(handler) = mouse_out_toggle {
                list = list.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(false, window, cx);
                });
            }

            let compact = self.compact;
            let row_height = if compact {
                px(22.0)
            } else {
                px(theme.target_size())
            };
            for (idx, item) in self.items.into_iter().enumerate() {
                let is_selected = self.selected.as_ref() == Some(&item.value);
                let is_highlighted = highlighted == Some(idx);
                let on_change = on_change.clone();
                let on_toggle = on_toggle.clone();
                let item_value = item.value.clone();

                // HIG: pop-up dropdown rows use the standard label color —
                // the selection indicator is the leading `Check` glyph in
                // accent, not a tinted label. Previously the label was
                // re-colored to `theme.accent` which read as "link" instead
                // of "selected".
                let text_color = theme.text;

                let mut row = div()
                    .id(ElementId::from(SharedString::from(format!(
                        "popup-item-{}",
                        item.value
                    ))))
                    .min_h(row_height)
                    .flex()
                    .items_center()
                    .px(theme.spacing_md)
                    .gap(theme.spacing_sm)
                    .cursor_pointer()
                    .hover(|style| style.bg(theme.hover));

                if is_highlighted {
                    row = row.bg(theme.hover);
                }

                // Check icon for the selected item.
                if is_selected {
                    row = row.child(
                        Icon::new(IconName::Check)
                            .size(theme.icon_size_inline)
                            .color(theme.accent),
                    );
                } else {
                    // Reserve space so labels stay aligned.
                    row = row.child(div().w(px(14.0)));
                }

                row = row.child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(text_color)
                        .child(item.label),
                );

                row = row.on_click(move |_event, window, cx| {
                    if let Some(handler) = &on_change {
                        handler(&item_value, window, cx);
                    }
                    if let Some(handler) = &on_toggle {
                        handler(false, window, cx);
                    }
                });

                list = list.child(row);
            }

            container = container.child(deferred(list).with_priority(1));
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::SharedString;

    use crate::components::menus_and_actions::popup_button::{PopupButton, PopupItem};

    #[test]
    fn popup_button_defaults() {
        let pb = PopupButton::new("test");
        assert!(pb.items.is_empty());
        assert!(pb.selected.is_none());
        assert!(!pb.is_open);
        assert!(!pb.disabled);
        assert!(!pb.focused);
        assert!(!pb.compact);
        assert!(pb.on_change.is_none());
        assert!(pb.on_toggle.is_none());
    }

    #[test]
    fn popup_button_compact_builder_sets_flag() {
        let pb = PopupButton::new("test").compact(true);
        assert!(pb.compact);
    }

    #[test]
    fn popup_button_focus_handle_none_by_default() {
        let pb = PopupButton::new("test");
        assert!(pb.focus_handle.is_none());
    }

    #[test]
    fn popup_button_items_builder() {
        let pb = PopupButton::new("test").items(vec![
            PopupItem::new("Small", "sm"),
            PopupItem::new("Large", "lg"),
        ]);
        assert_eq!(pb.items.len(), 2);
        assert_eq!(pb.items[0].label.as_ref(), "Small");
        assert_eq!(pb.items[1].value.as_ref(), "lg");
    }

    #[test]
    fn popup_button_selected_builder() {
        let pb = PopupButton::new("test").selected("sm");
        assert_eq!(pb.selected.unwrap().as_ref(), "sm");
    }

    #[test]
    fn popup_button_open_builder() {
        let pb = PopupButton::new("test").open(true);
        assert!(pb.is_open);
    }

    #[test]
    fn popup_button_on_change_is_some() {
        let pb = PopupButton::new("test").on_change(|_, _, _| {});
        assert!(pb.on_change.is_some());
    }

    #[test]
    fn popup_button_on_toggle_is_some() {
        let pb = PopupButton::new("test").on_toggle(|_, _, _| {});
        assert!(pb.on_toggle.is_some());
    }

    #[test]
    fn popup_item_new() {
        let item = PopupItem::new("Label", "value");
        assert_eq!(item.label.as_ref(), "Label");
        assert_eq!(item.value.as_ref(), "value");
    }

    #[test]
    fn popup_item_accepts_shared_string() {
        let label = SharedString::from("Shared");
        let value = SharedString::from("v");
        let item = PopupItem::new(label, value);
        assert_eq!(item.label.as_ref(), "Shared");
        assert_eq!(item.value.as_ref(), "v");
    }
}
