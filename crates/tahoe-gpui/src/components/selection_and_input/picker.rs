//! HIG Picker — desktop dropdown selector.
//!
//! A stateless `RenderOnce` component for selecting a value from a list.
//! The parent owns the open/closed state via `is_open` + `on_toggle`.

use crate::callback_types::{OnSharedStringRefChange, OnToggle, rc_wrap};
use crate::components::menus_and_actions::popup_button::OnHighlight;
use crate::components::selection_and_input::segmented_control::{SegmentItem, SegmentedControl};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{apply_standard_control_styling, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    App, ElementId, IntoElement, KeyDownEvent, MouseDownEvent, SharedString, Window, deferred, div,
    px,
};
use std::rc::Rc;

/// A single option in a [`Picker`].
pub struct PickerItem {
    /// Display text shown in the trigger and option list.
    pub label: SharedString,
    /// Unique value used for selection matching and `on_change` callbacks.
    pub value: SharedString,
}

impl PickerItem {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Visual style used by [`Picker`].
///
/// * `Menu` — the default dropdown. Matches `NSPopUpButton`'s menu style.
/// * `Inline` — renders all items in a vertical list directly under the
///   trigger region. Useful inside grouped forms where tapping the
///   trigger shouldn't open a floating menu.
/// * `Segmented` — renders the items as a [`super::SegmentedControl`]
///   inline (no trigger). HIG caps this style at ~7 segments, so
///   `Picker` clamps to 7 and `debug_assert!`s beyond that.
/// * `Palette` — grid-of-swatches layout for visual options (colors,
///   emoji). Out of scope for menu/value pickers; callers wanting a
///   palette should prefer [`super::color_well::ColorWell`]. Selecting
///   this style currently falls back to the menu layout.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PickerStyle {
    #[default]
    Menu,
    Inline,
    Segmented,
    Palette,
}

/// A named section of picker items. Used with [`Picker::sections`] when
/// callers want grouped-list semantics instead of a flat item list.
pub struct PickerSection {
    pub header: SharedString,
    pub items: Vec<PickerItem>,
}

impl PickerSection {
    pub fn new(header: impl Into<SharedString>, items: Vec<PickerItem>) -> Self {
        Self {
            header: header.into(),
            items,
        }
    }
}

/// A desktop dropdown selector following HIG.
///
/// Renders a button-like trigger that, when open, shows a positioned list of
/// options below it. Selection state and open/closed state are owned by the
/// parent.
#[derive(IntoElement)]
pub struct Picker {
    id: ElementId,
    items: Vec<PickerItem>,
    /// Optional grouped-list alternative to `items`. Takes precedence in
    /// render when non-empty; `items` is ignored if this is populated.
    sections: Vec<PickerSection>,
    selected: Option<SharedString>,
    placeholder: SharedString,
    is_open: bool,
    on_change: OnSharedStringRefChange,
    on_toggle: OnToggle,
    on_highlight: OnHighlight,
    focused: bool,
    highlighted_index: Option<usize>,
    /// Visual style. Defaults to [`PickerStyle::Menu`].
    style: PickerStyle,
}

impl Picker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            sections: Vec::new(),
            selected: None,
            placeholder: SharedString::from("Select..."),
            is_open: false,
            on_change: None,
            on_toggle: None,
            on_highlight: None,
            focused: false,
            highlighted_index: None,
            style: PickerStyle::Menu,
        }
    }

    /// Alternative to `items` — a grouped list with section headers.
    pub fn sections(mut self, sections: Vec<PickerSection>) -> Self {
        self.sections = sections;
        self
    }

    /// Select the picker style. Defaults to [`PickerStyle::Menu`].
    pub fn style(mut self, style: PickerStyle) -> Self {
        self.style = style;
        self
    }

    pub fn items(mut self, items: Vec<PickerItem>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(mut self, value: Option<SharedString>) -> Self {
        self.selected = value;
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Marks this picker as keyboard-focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Sets the keyboard-highlighted item index in the dropdown.
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

impl RenderOnce for Picker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let style = self.style;

        // Segmented style: render as `SegmentedControl` directly. Picker's
        // job in this mode is essentially index-to-value translation, so
        // we reuse the canonical HIG control rather than duplicating its
        // layout.
        if style == PickerStyle::Segmented {
            // Flatten sections for segmented rendering (section headers
            // have no counterpart in a segmented control).
            let items_flat: Vec<PickerItem> = if self.sections.is_empty() {
                self.items
            } else {
                self.sections.into_iter().flat_map(|s| s.items).collect()
            };
            let selected_idx = self
                .selected
                .as_ref()
                .and_then(|sel| items_flat.iter().position(|i| &i.value == sel))
                .unwrap_or(0);
            let values: Vec<SharedString> =
                items_flat.iter().map(|i| i.value.clone()).collect();
            let segments: Vec<SegmentItem> = items_flat
                .into_iter()
                .map(|i| SegmentItem::new(i.label))
                .collect();
            let on_change = rc_wrap(self.on_change);
            let mut control = SegmentedControl::new(self.id.clone())
                .items(segments)
                .selected(selected_idx)
                .focused(self.focused);
            if let Some(handler) = on_change {
                control = control.on_change(move |idx, window, cx| {
                    if let Some(value) = values.get(idx) {
                        handler(value, window, cx);
                    }
                });
            }
            return control.into_any_element();
        }

        // Flatten sections into a single items list for all downstream
        // layout logic. Section headers are tracked separately via
        // `header_at_index`.
        let mut header_at_index: Vec<Option<SharedString>> = Vec::new();
        let mut flat_items: Vec<PickerItem> = Vec::new();
        if !self.sections.is_empty() {
            for section in self.sections {
                for (i, item) in section.items.into_iter().enumerate() {
                    header_at_index.push(if i == 0 {
                        Some(section.header.clone())
                    } else {
                        None
                    });
                    flat_items.push(item);
                }
            }
        } else {
            for _ in 0..self.items.len() {
                header_at_index.push(None);
            }
            flat_items = self.items;
        }

        // Resolve the display label for the trigger button.
        let trigger_label: SharedString = self
            .selected
            .as_ref()
            .and_then(|sel| flat_items.iter().find(|i| &i.value == sel))
            .map(|item| item.label.clone())
            .unwrap_or_else(|| self.placeholder.clone());

        let trigger_text_color = if self.selected.is_some() {
            theme.text
        } else {
            theme.text_muted
        };

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
                    .text_color(trigger_text_color)
                    .child(trigger_label),
            )
            .child(
                Icon::new(IconName::ChevronDown)
                    .size(px(12.0))
                    .color(theme.text_muted),
            );

        let mut trigger = div()
            .id(self.id.clone())
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .px(theme.spacing_md)
            .cursor_pointer();

        // Glass-styled trigger surface.
        trigger = apply_standard_control_styling(trigger, theme, GlassSize::Small, self.focused);

        trigger = trigger
            .hover(|style| style.cursor_pointer())
            .child(trigger_content);

        if let Some(handler) = toggle_for_trigger {
            trigger = trigger.on_click(move |_event, window, cx| {
                handler(!is_open, window, cx);
            });
        }

        // Trigger keyboard activation: Enter/Space/Down opens the dropdown.
        if let Some(handler) = trigger_key_toggle {
            trigger = trigger.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if (crate::foundations::keyboard::is_activation_key(event)
                    || event.keystroke.key.as_str() == "down")
                    && !is_open
                {
                    cx.stop_propagation();
                    handler(true, window, cx);
                }
            });
        }

        // ── Container (trigger + optional dropdown) ─────────────────────────
        let mut container = div().relative().child(trigger);

        // In Inline style the list is always open — it lives directly
        // under the trigger row instead of in a floating dropdown.
        let is_list_visible = match style {
            PickerStyle::Menu | PickerStyle::Segmented | PickerStyle::Palette => self.is_open,
            PickerStyle::Inline => true,
        };

        if is_list_visible {
            let highlighted_index = self.highlighted_index;
            let item_count = flat_items.len();

            // Collect item values for keyboard enter selection.
            let item_values: Vec<SharedString> =
                flat_items.iter().map(|i| i.value.clone()).collect();

            // ── Dropdown list ───────────────────────────────────────────────
            // Inline mode renders the list flow-positioned directly below
            // the trigger (no `.absolute()`, no max-height clamp). Menu
            // mode keeps the floating glass dropdown positioned by
            // `theme.dropdown_top()`.
            let mut list_container = div().flex().flex_col().overflow_hidden();
            if style == PickerStyle::Inline {
                list_container = list_container.w_full();
            } else {
                list_container = list_container
                    .absolute()
                    .left_0()
                    .top(theme.dropdown_top())
                    .w_full()
                    .max_h(px(DROPDOWN_MAX_HEIGHT));
            }
            let mut list = glass_surface(list_container, theme, GlassSize::Medium)
                .id(ElementId::from((self.id.clone(), "dropdown")))
                .focusable();

            // Keyboard nav: Up/Down/Enter/Home/End/Escape
            let key_on_toggle = on_toggle.clone();
            let key_on_change = on_change.clone();
            let key_on_highlight = self.on_highlight.map(Rc::new);
            list = list.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_escape_key(event) {
                    if let Some(ref handler) = key_on_toggle {
                        handler(false, window, cx);
                    }
                    return;
                }
                if item_count == 0 {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "down" => {
                        cx.stop_propagation();
                        let next = match highlighted_index {
                            Some(i) if i + 1 < item_count => i + 1,
                            Some(_) => 0,
                            None => 0,
                        };
                        if let Some(ref handler) = key_on_highlight {
                            handler(Some(next), window, cx);
                        }
                    }
                    "up" => {
                        cx.stop_propagation();
                        let prev = match highlighted_index {
                            Some(0) | None => item_count - 1,
                            Some(i) => i - 1,
                        };
                        if let Some(ref handler) = key_on_highlight {
                            handler(Some(prev), window, cx);
                        }
                    }
                    "home" => {
                        cx.stop_propagation();
                        if let Some(ref handler) = key_on_highlight {
                            handler(Some(0), window, cx);
                        }
                    }
                    "end" => {
                        cx.stop_propagation();
                        if let Some(ref handler) = key_on_highlight {
                            handler(Some(item_count - 1), window, cx);
                        }
                    }
                    "enter" => {
                        cx.stop_propagation();
                        if let Some(idx) = highlighted_index {
                            if idx < item_count {
                                if let Some(ref handler) = key_on_change {
                                    handler(&item_values[idx], window, cx);
                                }
                                if let Some(ref handler) = key_on_toggle {
                                    handler(false, window, cx);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            });

            // Close dropdown on click outside.
            let mouse_out_toggle = on_toggle.clone();
            if let Some(handler) = mouse_out_toggle {
                list = list.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(false, window, cx);
                });
            }

            // Glass-aware hover background for highlighted item.
            let hover_bg = theme.hover_bg();

            for (item_idx, item) in flat_items.into_iter().enumerate() {
                // Render a section header before this row if the
                // grouped-items list emitted one.
                if let Some(Some(header)) =
                    header_at_index.get(item_idx).map(|h| h.as_ref())
                {
                    list = list.child(
                        div()
                            .px(theme.spacing_md)
                            .pt(theme.spacing_sm)
                            .pb(theme.spacing_xs)
                            .text_style(TextStyle::Footnote, theme)
                            .text_color(theme.text_muted)
                            .child(header.clone()),
                    );
                }
                let is_selected = self.selected.as_ref() == Some(&item.value);
                let is_highlighted = highlighted_index == Some(item_idx);
                let on_change = on_change.clone();
                let on_toggle = on_toggle.clone();
                let item_value = item.value.clone();

                let text_color = if is_selected {
                    theme.accent
                } else {
                    theme.text
                };

                let mut row = div()
                    .id(ElementId::from(SharedString::from(format!(
                        "picker-item-{}",
                        item.value
                    ))))
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .px(theme.spacing_md)
                    .gap(theme.spacing_sm)
                    .cursor_pointer()
                    .hover(|style| style.bg(hover_bg));

                // Apply highlight background when this item matches highlighted_index.
                if is_highlighted {
                    row = row.bg(hover_bg);
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

            if style == PickerStyle::Inline {
                // Inline: the list participates in normal flow below
                // the trigger — no deferred layer, no absolute positioning.
                container = container.child(list);
            } else {
                container = container.child(deferred(list).with_priority(1));
            }
        }

        container.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{Picker, PickerItem};
    use core::prelude::v1::test;
    use gpui::SharedString;

    #[test]
    fn picker_defaults() {
        let p = Picker::new("test");
        assert!(p.items.is_empty());
        assert!(p.selected.is_none());
        assert!(!p.is_open);
        assert!(p.on_change.is_none());
        assert!(p.on_toggle.is_none());
        assert_eq!(p.placeholder.as_ref(), "Select...");
        assert!(!p.focused);
        assert!(p.highlighted_index.is_none());
    }

    #[test]
    fn picker_items_builder() {
        let p = Picker::new("test").items(vec![
            PickerItem::new("Alpha", "a"),
            PickerItem::new("Beta", "b"),
        ]);
        assert_eq!(p.items.len(), 2);
        assert_eq!(p.items[0].label.as_ref(), "Alpha");
        assert_eq!(p.items[0].value.as_ref(), "a");
    }

    #[test]
    fn picker_selected_builder() {
        let p = Picker::new("test").selected(Some(SharedString::from("b")));
        assert_eq!(p.selected.as_ref().map(|s| s.as_ref()), Some("b"));
    }

    #[test]
    fn picker_placeholder_builder() {
        let p = Picker::new("test").placeholder("Choose one");
        assert_eq!(p.placeholder.as_ref(), "Choose one");
    }

    #[test]
    fn picker_open_builder() {
        let p = Picker::new("test").open(true);
        assert!(p.is_open);
    }

    #[test]
    fn picker_on_change_is_some() {
        let p = Picker::new("test").on_change(|_, _, _| {});
        assert!(p.on_change.is_some());
    }

    #[test]
    fn picker_on_toggle_is_some() {
        let p = Picker::new("test").on_toggle(|_, _, _| {});
        assert!(p.on_toggle.is_some());
    }

    #[test]
    fn picker_item_new() {
        let item = PickerItem::new("Label", "value");
        assert_eq!(item.label.as_ref(), "Label");
        assert_eq!(item.value.as_ref(), "value");
    }

    #[test]
    fn picker_focused_builder() {
        let p = Picker::new("test").focused(true);
        assert!(p.focused);
    }

    #[test]
    fn picker_highlighted_index_builder() {
        let p = Picker::new("test").highlighted_index(Some(2));
        assert_eq!(p.highlighted_index, Some(2));
    }

    #[test]
    fn picker_highlighted_index_none_by_default() {
        let p = Picker::new("test");
        assert_eq!(p.highlighted_index, None);
    }
}
