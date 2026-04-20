//! HIG Picker — desktop dropdown selector.
//!
//! A stateless `RenderOnce` component for selecting a value from a list.
//! The parent owns the open/closed state via `is_open` + `on_toggle`.

use crate::callback_types::{OnSharedStringRefChange, OnToggle, rc_wrap};
use crate::components::menus_and_actions::popup_button::OnHighlight;
use crate::components::selection_and_input::segmented_control::{SegmentItem, SegmentedControl};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{
    apply_focus_ring, apply_standard_control_styling, glass_surface,
};
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
    /// Optional SF Symbol rendered by visual styles such as
    /// [`PickerStyle::Palette`]. Ignored by list-style pickers.
    pub icon: Option<IconName>,
}

impl PickerItem {
    pub fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            icon: None,
        }
    }

    /// Attach an SF Symbol that visual styles (e.g. [`PickerStyle::Palette`])
    /// will render inside the tile. List-style pickers ignore this field.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
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
/// * `Radio` — vertical list of radio rows. The selected row renders a
///   filled accent circle, unselected rows render an empty circle.
///   Useful in preference panes where the options need to be visible at
///   a glance rather than collapsed into a trigger.
/// * `Wheel` — HIG wheel picker. A fixed-height vertical column snaps the
///   selected item to the centre, with the neighbouring items dimmed.
/// * `Palette` — horizontal grid of square tiles (max 10 per row). Each
///   tile shows the item's [`PickerItem::icon`] when present, otherwise
///   the first letter of the label. The selected tile receives an
///   accent-coloured border.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum PickerStyle {
    #[default]
    Menu,
    Inline,
    Segmented,
    Radio,
    Wheel,
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
            let values: Vec<SharedString> = items_flat.iter().map(|i| i.value.clone()).collect();
            let segments: Vec<SegmentItem> = items_flat
                .into_iter()
                .map(|i| SegmentItem::new(i.label))
                .collect();
            let on_change = rc_wrap(self.on_change);
            // Picker's `focused: bool` drives the focus ring on the
            // list/wheel/grid/trigger variants (below). The segmented
            // variant dropped `focused: bool` in favour of `focus_handle`
            // (#65 fix); a caller wanting a focus ring here should
            // construct a `SegmentedControl` directly and wire in a
            // FocusHandle. Fail loudly in debug so the silent drop is
            // caught at authoring time rather than quietly diverging from
            // the other variants.
            debug_assert!(
                !self.focused,
                "Picker::focused is ignored on PickerVariant::Segmented — \
                 construct a SegmentedControl directly with \
                 `.focus_handle(...)` to render a focus ring",
            );
            let mut control = SegmentedControl::new(self.id.clone())
                .items(segments)
                .selected(selected_idx);
            if let Some(handler) = on_change {
                control = control.on_change(move |idx, window, cx| {
                    if let Some(value) = values.get(idx) {
                        handler(value, window, cx);
                    }
                });
            }
            return control.into_any_element();
        }

        // Radio style — vertical list of radio rows. Always visible, no
        // trigger. Selected row gets a filled accent circle; unselected
        // rows get an empty circle.
        if style == PickerStyle::Radio {
            let items_flat: Vec<PickerItem> = if self.sections.is_empty() {
                self.items
            } else {
                self.sections.into_iter().flat_map(|s| s.items).collect()
            };
            let item_count = items_flat.len();
            let item_values: Vec<SharedString> =
                items_flat.iter().map(|i| i.value.clone()).collect();
            let highlighted_index = self.highlighted_index;
            let selected_value = self.selected.clone();
            let on_change = rc_wrap(self.on_change);
            let on_highlight = self.on_highlight.map(Rc::new);

            let mut list = div()
                .id(self.id.clone())
                .focusable()
                .flex()
                .flex_col()
                .w_full()
                .gap(theme.spacing_xs);

            // Keyboard: Up/Down move focus, Space/Enter pick.
            let key_change = on_change.clone();
            let key_highlight = on_highlight.clone();
            list = list.on_key_down(move |event: &KeyDownEvent, window, cx| {
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
                        if let Some(ref h) = key_highlight {
                            h(Some(next), window, cx);
                        }
                    }
                    "up" => {
                        cx.stop_propagation();
                        let prev = match highlighted_index {
                            Some(0) | None => item_count - 1,
                            Some(i) => i - 1,
                        };
                        if let Some(ref h) = key_highlight {
                            h(Some(prev), window, cx);
                        }
                    }
                    _ => {
                        if crate::foundations::keyboard::is_activation_key(event) {
                            cx.stop_propagation();
                            if let Some(idx) = highlighted_index
                                && idx < item_count
                                && let Some(ref h) = key_change
                            {
                                h(&item_values[idx], window, cx);
                            }
                        }
                    }
                }
            });

            for (idx, item) in items_flat.into_iter().enumerate() {
                let is_selected = selected_value.as_ref() == Some(&item.value);
                let is_highlighted = highlighted_index == Some(idx);
                let click_change = on_change.clone();
                let item_value = item.value.clone();

                // Radio glyph: outer 14pt border, inner 6pt accent fill
                // when selected. Kept as nested divs so we don't depend
                // on SF Symbol availability for this primitive.
                let mut outer = div()
                    .w(px(14.0))
                    .h(px(14.0))
                    .rounded_full()
                    .border_1()
                    .border_color(if is_selected {
                        theme.accent
                    } else {
                        theme.border
                    })
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0();
                if is_selected {
                    outer =
                        outer.child(div().w(px(6.0)).h(px(6.0)).rounded_full().bg(theme.accent));
                }

                let mut row = div()
                    .id(ElementId::from(SharedString::from(format!(
                        "picker-radio-{}",
                        item.value
                    ))))
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .px(theme.spacing_sm)
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover_bg()));
                if is_highlighted {
                    row = row.bg(theme.hover_bg());
                }
                row = row.child(outer).child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child(item.label),
                );
                row = row.on_click(move |_event, window, cx| {
                    if let Some(h) = &click_change {
                        h(&item_value, window, cx);
                    }
                });
                list = list.child(row);
            }

            list = apply_focus_ring(list, theme, self.focused, &[]);
            return list.into_any_element();
        }

        // Wheel style — fixed-height vertical column where the selected
        // item is centred and the items immediately above/below are
        // dimmed. Up/Down cycles through the list.
        if style == PickerStyle::Wheel {
            let items_flat: Vec<PickerItem> = if self.sections.is_empty() {
                self.items
            } else {
                self.sections.into_iter().flat_map(|s| s.items).collect()
            };
            let item_count = items_flat.len();
            let selected_value = self.selected.clone();
            let selected_idx = selected_value
                .as_ref()
                .and_then(|s| items_flat.iter().position(|i| &i.value == s));
            let item_values: Vec<SharedString> =
                items_flat.iter().map(|i| i.value.clone()).collect();
            let on_change = rc_wrap(self.on_change);

            // Row height tuned so three rows fit within the visible wheel.
            let row_h = px(32.0);
            let wheel_h = row_h * 3.0;

            let mut wheel = div()
                .id(self.id.clone())
                .focusable()
                .flex()
                .flex_col()
                .items_center()
                .w_full()
                .h(wheel_h)
                .overflow_hidden();
            wheel = wheel.bg(theme
                .glass
                .accessible_bg(GlassSize::Small, theme.accessibility_mode));
            wheel = wheel.rounded(theme.glass.radius(GlassSize::Small));

            // Up/Down cycles through the list.
            let key_change = on_change.clone();
            wheel = wheel.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if item_count == 0 {
                    return;
                }
                let current = selected_idx.unwrap_or(0);
                let new_idx = match event.keystroke.key.as_str() {
                    "down" => Some((current + 1) % item_count),
                    "up" => Some(if current == 0 {
                        item_count - 1
                    } else {
                        current - 1
                    }),
                    _ => None,
                };
                if let Some(idx) = new_idx {
                    cx.stop_propagation();
                    if let Some(ref h) = key_change {
                        h(&item_values[idx], window, cx);
                    }
                }
            });

            for (idx, item) in items_flat.into_iter().enumerate() {
                let is_selected = selected_idx == Some(idx);
                let is_neighbour = selected_idx
                    .map(|s| (idx as isize - s as isize).unsigned_abs() == 1)
                    .unwrap_or(false);
                let (color, style_kind) = if is_selected {
                    (theme.text, TextStyle::Body)
                } else if is_neighbour {
                    (theme.text_muted, TextStyle::Body)
                } else {
                    // Items outside the visible ±1 range stay muted; the
                    // `overflow_hidden` wheel still clips them to the
                    // three-row window.
                    (theme.text_muted, TextStyle::Body)
                };
                let click_change = on_change.clone();
                let item_value = item.value.clone();
                let mut row = div()
                    .id(ElementId::from(SharedString::from(format!(
                        "picker-wheel-{}",
                        item.value
                    ))))
                    .h(row_h)
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_style(style_kind, theme)
                    .text_color(color);
                if is_selected {
                    row = row.bg(theme.hover_bg());
                }
                row = row.child(item.label).on_click(move |_event, window, cx| {
                    if let Some(h) = &click_change {
                        h(&item_value, window, cx);
                    }
                });
                wheel = wheel.child(row);
            }

            wheel = apply_focus_ring(wheel, theme, self.focused, &[]);
            return wheel.into_any_element();
        }

        // Palette style — grid of square icon tiles, max 10 per row.
        // Ignores section headers because palettes are inherently flat.
        if style == PickerStyle::Palette {
            let items_flat: Vec<PickerItem> = if self.sections.is_empty() {
                self.items
            } else {
                self.sections.into_iter().flat_map(|s| s.items).collect()
            };
            let on_change = rc_wrap(self.on_change);
            let selected_value = self.selected.clone();

            const TILES_PER_ROW: usize = 10;
            const TILE_SIZE: f32 = 32.0;

            let mut grid = div()
                .id(self.id.clone())
                .flex()
                .flex_col()
                .gap(theme.spacing_xs);

            for chunk in items_flat.chunks(TILES_PER_ROW) {
                let mut row = div().flex().flex_row().gap(theme.spacing_xs);
                for item in chunk {
                    let is_selected = selected_value.as_ref() == Some(&item.value);
                    let click_change = on_change.clone();
                    let item_value = item.value.clone();
                    let mut tile = div()
                        .id(ElementId::from(SharedString::from(format!(
                            "picker-palette-{}",
                            item.value
                        ))))
                        .w(px(TILE_SIZE))
                        .h(px(TILE_SIZE))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.hover_bg()));
                    // 2pt accent border when selected; 1pt transparent
                    // otherwise so tile size stays constant.
                    if is_selected {
                        tile = tile.border_2().border_color(theme.accent);
                    } else {
                        tile = tile.border_1().border_color(theme.border);
                    }

                    if let Some(icon_name) = item.icon {
                        tile = tile.child(Icon::new(icon_name).size(px(18.0)).color(theme.text));
                    } else {
                        // First letter of label, uppercased, as a
                        // fallback glyph.
                        let glyph: SharedString = item
                            .label
                            .chars()
                            .next()
                            .map(|c| c.to_uppercase().to_string())
                            .unwrap_or_default()
                            .into();
                        tile = tile.child(
                            div()
                                .text_style(TextStyle::Body, theme)
                                .text_color(theme.text)
                                .child(glyph),
                        );
                    }

                    tile = tile.on_click(move |_event, window, cx| {
                        if let Some(h) = &click_change {
                            h(&item_value, window, cx);
                        }
                    });
                    row = row.child(tile);
                }
                grid = grid.child(row);
            }

            grid = apply_focus_ring(grid, theme, self.focused, &[]);
            return grid.into_any_element();
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
                // HIG macOS `NSPopUpButton` pop-up-style triggers use the
                // `chevron.up.chevron.down` glyph, not a single down
                // chevron (which denotes navigation/expansion). Picker's
                // default `Menu` style is a pop-up menu, so mirror the
                // system symbol.
                Icon::new(IconName::ChevronsUpDown)
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
        // Segmented / Radio / Wheel / Palette all return above, so the
        // only shapes that reach this match are Menu and Inline. The
        // fallback branch keeps the match exhaustive for any future
        // `#[non_exhaustive]` additions that fall through to the menu
        // layout (the documented fallback for non-trigger styles).
        let is_list_visible = match style {
            PickerStyle::Inline => true,
            _ => self.is_open,
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
                        if let Some(idx) = highlighted_index
                            && idx < item_count
                        {
                            if let Some(ref handler) = key_on_change {
                                handler(&item_values[idx], window, cx);
                            }
                            if let Some(ref handler) = key_on_toggle {
                                handler(false, window, cx);
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
                if let Some(Some(header)) = header_at_index.get(item_idx).map(|h| h.as_ref()) {
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
    use super::{Picker, PickerItem, PickerStyle};
    use crate::foundations::icons::IconName;
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

    #[test]
    fn picker_style_defaults_to_menu() {
        let p = Picker::new("test");
        assert_eq!(p.style, PickerStyle::Menu);
    }

    #[test]
    fn picker_radio_style_builder() {
        let p = Picker::new("radio").style(PickerStyle::Radio).items(vec![
            PickerItem::new("One", "1"),
            PickerItem::new("Two", "2"),
        ]);
        assert_eq!(p.style, PickerStyle::Radio);
        assert_eq!(p.items.len(), 2);
    }

    #[test]
    fn picker_wheel_style_builder() {
        let p = Picker::new("wheel")
            .style(PickerStyle::Wheel)
            .items(vec![PickerItem::new("A", "a"), PickerItem::new("B", "b")])
            .selected(Some(SharedString::from("b")));
        assert_eq!(p.style, PickerStyle::Wheel);
        assert_eq!(p.selected.as_ref().map(|s| s.as_ref()), Some("b"));
    }

    #[test]
    fn picker_palette_style_builder_with_icons() {
        let p = Picker::new("palette")
            .style(PickerStyle::Palette)
            .items(vec![
                PickerItem::new("Star", "s").icon(IconName::StarFill),
                PickerItem::new("Heart", "h"),
            ]);
        assert_eq!(p.style, PickerStyle::Palette);
        assert!(p.items[0].icon.is_some());
        assert!(p.items[1].icon.is_none());
    }

    #[test]
    fn picker_item_icon_builder() {
        let item = PickerItem::new("Label", "v").icon(IconName::CircleFilled);
        assert!(item.icon.is_some());
    }
}
