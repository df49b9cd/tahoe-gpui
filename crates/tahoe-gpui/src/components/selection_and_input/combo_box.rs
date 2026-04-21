//! HIG Combo Box — text input with filterable dropdown.
//!
//! A stateless `RenderOnce` component that combines a text input area with a
//! filterable dropdown list. The parent owns all state (value, open/closed,
//! items) and provides callbacks for changes.

use crate::callback_types::{OnSharedStringChange, OnSharedStringRefChange, OnToggle, rc_wrap};
use crate::components::menus_and_actions::popup_button::OnHighlight;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{apply_standard_control_styling, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, SharedString, Window, deferred, div,
    px,
};
use std::rc::Rc;

/// A combo box component: text input with filterable dropdown.
///
/// Stateless `RenderOnce` — the parent owns the value, open state, and item
/// list, providing callbacks for mutations.
#[derive(IntoElement)]
pub struct ComboBox {
    id: ElementId,
    value: SharedString,
    items: Vec<SharedString>,
    /// Optional recent-items list surfaced at the top of the dropdown
    /// under a "Recent" header when the filter is blank. HIG: "Populate
    /// the list with the most useful choices … such as … recently used
    /// values."
    recent_items: Vec<SharedString>,
    /// When true, the combo box offers inline autocomplete: the best
    /// prefix match is concatenated after the typed text and displayed
    /// to hint at the completion.
    autocomplete: bool,
    is_open: bool,
    placeholder: SharedString,
    highlighted_index: Option<usize>,
    focused: bool,
    focus_handle: Option<FocusHandle>,
    disabled: bool,
    on_toggle: OnToggle,
    on_select: OnSharedStringRefChange,
    on_input: OnSharedStringChange,
    on_highlight: OnHighlight,
}

impl ComboBox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: SharedString::default(),
            items: Vec::new(),
            recent_items: Vec::new(),
            autocomplete: false,
            is_open: false,
            placeholder: SharedString::from("Search..."),
            highlighted_index: None,
            focused: false,
            focus_handle: None,
            disabled: false,
            on_toggle: None,
            on_select: None,
            on_input: None,
            on_highlight: None,
        }
    }

    /// Register a list of recently used items. When the filter is blank,
    /// these appear at the top of the dropdown under a "Recent" header.
    pub fn recent_items(mut self, items: Vec<SharedString>) -> Self {
        self.recent_items = items;
        self
    }

    /// Enable inline prefix autocomplete. When the typed text matches an
    /// item prefix, the remainder is shown inline (theme text-muted)
    /// after the typed text so callers can accept with Enter.
    pub fn autocomplete(mut self, autocomplete: bool) -> Self {
        self.autocomplete = autocomplete;
        self
    }

    pub fn value(mut self, text: impl Into<SharedString>) -> Self {
        self.value = text.into();
        self
    }

    pub fn items(mut self, items: Vec<SharedString>) -> Self {
        self.items = items;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn highlighted_index(mut self, index: Option<usize>) -> Self {
        self.highlighted_index = index;
        self
    }

    /// Set the focus ring manually. Ignored when a
    /// [`focus_handle`](Self::focus_handle) is supplied — the handle's
    /// reactive state takes precedence.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the combo box participates in GPUI's
    /// focus graph. When present, the focus ring is driven by
    /// `handle.is_focused(window)` and the trigger threads `track_focus`.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Set the callback fired when the user types into the open dropdown to filter items.
    /// Receives the updated filter text (value with typed character appended or backspaced).
    pub fn on_input(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_input = Some(Box::new(handler));
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

    /// Returns filtered items: those whose label contains `value` (case-insensitive).
    /// When value is empty or exactly matches an item, returns all items.
    fn filtered_items(&self) -> Vec<&SharedString> {
        if self.value.is_empty() {
            return self.items.iter().collect();
        }
        // If the value exactly matches an item, show all items (user is browsing)
        let exact_match = self
            .items
            .iter()
            .any(|item| item.as_ref() == self.value.as_ref());
        if exact_match {
            return self.items.iter().collect();
        }
        // Otherwise filter by typed text
        let needle = self.value.to_lowercase();
        self.items
            .iter()
            .filter(|item| item.to_lowercase().contains(&needle))
            .collect()
    }
}

impl RenderOnce for ComboBox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // FocusHandle (when supplied) drives the ring reactively via
        // GPUI's focus graph; falls back to the manual `focused: bool`
        // flag otherwise. Mirrors the `ButtonLike` pattern.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        let text_color = if self.value.is_empty() {
            theme.text_muted
        } else {
            theme.text
        };

        // Inline prefix autocomplete hint: when enabled and the typed
        // value prefixes an item (case-insensitive), we display the
        // remainder appended in muted text. Parents can wire Tab/Enter
        // to accept via `on_select`.
        let autocomplete_suffix: Option<SharedString> =
            if self.autocomplete && !self.value.is_empty() {
                let needle = self.value.to_lowercase();
                self.items
                    .iter()
                    .find(|item| {
                        let lower = item.to_lowercase();
                        lower.starts_with(&needle) && lower != needle
                    })
                    .map(|m| SharedString::from(m[self.value.len()..].to_string()))
            } else {
                None
            };

        let display_text: SharedString = if self.value.is_empty() {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };

        // Recent items are surfaced when the filter is blank. Past that
        // point the filtered list takes over so typing still narrows.
        let show_recents = self.value.is_empty() && !self.recent_items.is_empty();
        let recents = self.recent_items.clone();

        // Compute filtered items before moving callbacks out of self.
        let filtered: Vec<SharedString> = self.filtered_items().into_iter().cloned().collect();

        // Extract fields before moving callbacks out of self.
        let is_open = self.is_open;
        let id = self.id.clone();
        let highlighted_index = self.highlighted_index;

        // Wrap callbacks in Rc for sharing across closures.
        let on_toggle = rc_wrap(self.on_toggle);
        let on_select = rc_wrap(self.on_select);
        let on_input = rc_wrap(self.on_input);

        // ── Input trigger area ─────────────────────────────────────────────
        let toggle_for_trigger = on_toggle.clone();
        let trigger_key_toggle = on_toggle.clone();

        let mut trigger_label = div()
            .flex()
            .flex_row()
            .items_center()
            .flex_1()
            .text_style(TextStyle::Body, theme)
            .text_color(text_color)
            .child(display_text);
        if let Some(ref suffix) = autocomplete_suffix {
            trigger_label =
                trigger_label.child(div().text_color(theme.text_muted).child(suffix.clone()));
        }
        let input_content = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .gap(theme.spacing_sm)
            .child(trigger_label)
            .child(
                // HIG: combo box uses the double-chevron indicator
                // (`chevron.up.chevron.down`) — same as the pop-up button.
                Icon::new(IconName::ChevronsUpDown)
                    .size(px(12.0))
                    .color(theme.text_muted),
            );

        let disabled = self.disabled;

        let mut trigger = div()
            .id(id.clone())
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

        // Glass-styled trigger surface (matches TextField / Picker styling).
        trigger = apply_standard_control_styling(trigger, theme, GlassSize::Small, focused);

        if disabled {
            trigger = trigger.opacity(0.5).cursor_default();
        } else {
            trigger = trigger.hover(|style| style.cursor_pointer());
        }

        trigger = trigger.child(input_content);

        if !disabled {
            if let Some(handler) = toggle_for_trigger {
                trigger = trigger.on_click(move |_event, window, cx| {
                    handler(!is_open, window, cx);
                });
            }

            // Trigger keyboard activation: Enter/Space opens the dropdown.
            if let Some(handler) = trigger_key_toggle {
                trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if crate::foundations::keyboard::is_activation_key(event) && !is_open {
                        cx.stop_propagation();
                        handler(true, window, cx);
                    }
                });
            }
        }

        // ── Container (trigger + optional dropdown) ────────────────────────
        let mut container = div().relative().child(trigger);

        if is_open {
            let key_toggle = on_toggle.clone();
            let key_select = on_select.clone();
            let key_filtered = filtered.clone();
            let key_input = on_input.clone();
            let key_highlight = self.on_highlight.map(Rc::new);
            let current_value = self.value.clone();
            let mouse_out_toggle = on_toggle.clone();

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

            // Keyboard navigation + type-to-filter: up/down/enter/escape + printable chars.
            list = list.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_escape_key(event) {
                    if let Some(ref handler) = key_toggle {
                        handler(false, window, cx);
                    }
                    return;
                }
                let item_count = key_filtered.len();
                let key = event.keystroke.key.as_str();
                match key {
                    "down" if item_count > 0 => {
                        cx.stop_propagation();
                        let next = match highlighted_index {
                            Some(i) if i + 1 < item_count => i + 1,
                            Some(_) => 0,
                            None => 0,
                        };
                        if let Some(ref handler) = key_highlight {
                            handler(Some(next), window, cx);
                        }
                    }
                    "up" if item_count > 0 => {
                        cx.stop_propagation();
                        let prev = match highlighted_index {
                            Some(0) | None => item_count - 1,
                            Some(i) => i - 1,
                        };
                        if let Some(ref handler) = key_highlight {
                            handler(Some(prev), window, cx);
                        }
                    }
                    "home" if item_count > 0 => {
                        cx.stop_propagation();
                        if let Some(ref handler) = key_highlight {
                            handler(Some(0), window, cx);
                        }
                    }
                    "end" if item_count > 0 => {
                        cx.stop_propagation();
                        if let Some(ref handler) = key_highlight {
                            handler(Some(item_count - 1), window, cx);
                        }
                    }
                    "enter" => {
                        if let Some(idx) = highlighted_index
                            && idx < item_count
                        {
                            cx.stop_propagation();
                            if let Some(ref handler) = key_select {
                                handler(&key_filtered[idx], window, cx);
                            }
                            if let Some(ref handler) = key_toggle {
                                handler(false, window, cx);
                            }
                        }
                    }
                    "backspace" => {
                        if let Some(ref handler) = key_input {
                            // UTF-8-safe pop: remove the last grapheme cluster
                            // so we don't split a multi-byte char (e.g. `ä`,
                            // an emoji, or a CJK glyph) and produce invalid
                            // UTF-8. `String::pop` would drop a single `char`,
                            // which is still wrong for emoji ZWJ sequences.
                            use unicode_segmentation::UnicodeSegmentation;
                            let mut text = current_value.to_string();
                            if let Some((idx, _)) = text.grapheme_indices(true).next_back() {
                                text.truncate(idx);
                            }
                            handler(SharedString::from(text), window, cx);
                        }
                    }
                    _ => {
                        // Type-to-filter: accept any non-control printable character
                        // (including umlauts, accented chars, emoji) via `key_char`.
                        let typed = event
                            .keystroke
                            .key_char
                            .as_deref()
                            .filter(|s| !s.is_empty() && !s.chars().any(|c| c.is_control()));
                        if let Some(text) = typed
                            && let Some(ref handler) = key_input
                        {
                            let mut buf = current_value.to_string();
                            buf.push_str(text);
                            handler(SharedString::from(buf), window, cx);
                        }
                    }
                }
            });

            // Close dropdown on click outside.
            if let Some(handler) = mouse_out_toggle {
                list = list.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(false, window, cx);
                });
            }

            // Glass-aware hover background for highlighted item.
            let hover_bg = theme.hover_bg();

            // "Recent" section — rendered before the main filtered list
            // when the filter is blank and the caller supplied recents.
            if show_recents {
                list = list.child(
                    div()
                        .px(theme.spacing_md)
                        .pt(theme.spacing_sm)
                        .pb(theme.spacing_xs)
                        .text_style(TextStyle::Footnote, theme)
                        .text_color(theme.text_muted)
                        .child(SharedString::from("Recent")),
                );
                for (ridx, item) in recents.iter().enumerate() {
                    let on_select_r = on_select.clone();
                    let on_toggle_r = on_toggle.clone();
                    let item_value = item.clone();
                    let row = div()
                        .id(ElementId::NamedInteger("combo-recent".into(), ridx as u64))
                        .min_h(px(theme.target_size()))
                        .flex()
                        .items_center()
                        .px(theme.spacing_md)
                        .cursor_pointer()
                        .hover(|style| style.bg(hover_bg))
                        .child(
                            div()
                                .text_style(TextStyle::Body, theme)
                                .text_color(theme.text)
                                .child(item.clone()),
                        )
                        .on_click(move |_event, window, cx| {
                            if let Some(handler) = &on_select_r {
                                handler(&item_value, window, cx);
                            }
                            if let Some(handler) = &on_toggle_r {
                                handler(false, window, cx);
                            }
                        });
                    list = list.child(row);
                }
                // Divider between Recent and the full list.
                list = list.child(
                    div()
                        .h(theme.separator_thickness)
                        .w_full()
                        .bg(theme.border)
                        .my(theme.spacing_xs),
                );
            }

            for (idx, item) in filtered.into_iter().enumerate() {
                let on_select = on_select.clone();
                let on_toggle = on_toggle.clone();
                let item_value = item.clone();
                let item_label = item.clone();
                let is_highlighted = highlighted_index == Some(idx);

                let mut row = div()
                    .id(ElementId::NamedInteger("combo-item".into(), idx as u64))
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .px(theme.spacing_md)
                    .cursor_pointer()
                    .hover(|style| style.bg(hover_bg));

                // Apply highlight background when this item matches highlighted_index.
                if is_highlighted {
                    row = row.bg(hover_bg);
                }

                row = row.child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child(item_label),
                );

                row = row.on_click(move |_event, window, cx| {
                    if let Some(handler) = &on_select {
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
    use super::ComboBox;
    use core::prelude::v1::test;
    use gpui::SharedString;

    #[test]
    fn combo_box_defaults() {
        let cb = ComboBox::new("test");
        assert!(cb.value.is_empty());
        assert!(cb.items.is_empty());
        assert!(!cb.is_open);
        assert_eq!(cb.placeholder.as_ref(), "Search...");
        assert!(cb.highlighted_index.is_none());
        assert!(!cb.focused);
        assert!(!cb.disabled);
        assert!(cb.on_toggle.is_none());
        assert!(cb.on_select.is_none());
    }

    #[test]
    fn combo_box_value_builder() {
        let cb = ComboBox::new("test").value("hello");
        assert_eq!(cb.value.as_ref(), "hello");
    }

    #[test]
    fn combo_box_items_builder() {
        let items = vec![SharedString::from("Alpha"), SharedString::from("Beta")];
        let cb = ComboBox::new("test").items(items);
        assert_eq!(cb.items.len(), 2);
        assert_eq!(cb.items[0].as_ref(), "Alpha");
    }

    #[test]
    fn combo_box_filter_case_insensitive() {
        let items = vec![
            SharedString::from("Apple"),
            SharedString::from("Banana"),
            SharedString::from("Apricot"),
        ];
        let cb = ComboBox::new("test").items(items).value("ap");
        let filtered = cb.filtered_items();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].as_ref(), "Apple");
        assert_eq!(filtered[1].as_ref(), "Apricot");
    }

    #[test]
    fn combo_box_filter_empty_value_shows_all() {
        let items = vec![
            SharedString::from("One"),
            SharedString::from("Two"),
            SharedString::from("Three"),
        ];
        let cb = ComboBox::new("test").items(items);
        let filtered = cb.filtered_items();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn combo_box_filter_no_matches() {
        let items = vec![SharedString::from("Apple"), SharedString::from("Banana")];
        let cb = ComboBox::new("test").items(items).value("xyz");
        let filtered = cb.filtered_items();
        assert!(filtered.is_empty());
    }

    #[test]
    fn combo_box_callbacks_are_some() {
        let cb = ComboBox::new("test")
            .on_toggle(|_, _, _| {})
            .on_select(|_, _, _| {});
        assert!(cb.on_toggle.is_some());
        assert!(cb.on_select.is_some());
    }

    #[test]
    fn combo_box_highlighted_index_builder() {
        let cb = ComboBox::new("test").highlighted_index(Some(2));
        assert_eq!(cb.highlighted_index, Some(2));
    }

    #[test]
    fn combo_box_highlighted_index_none_by_default() {
        let cb = ComboBox::new("test");
        assert_eq!(cb.highlighted_index, None);
    }

    #[test]
    fn combo_box_focused_builder() {
        let cb = ComboBox::new("test").focused(true);
        assert!(cb.focused);
    }

    #[test]
    fn combo_box_focused_false_by_default() {
        let cb = ComboBox::new("test");
        assert!(!cb.focused);
    }

    #[test]
    fn combo_box_focus_handle_none_by_default() {
        let cb = ComboBox::new("test");
        assert!(cb.focus_handle.is_none());
    }

    #[test]
    fn combo_box_filter_mid_word_match() {
        let items = vec![
            SharedString::from("Apple"),
            SharedString::from("Banana"),
            SharedString::from("Mango"),
        ];
        let cb = ComboBox::new("test").items(items).value("nan");
        let filtered = cb.filtered_items();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].as_ref(), "Banana");
    }
}
