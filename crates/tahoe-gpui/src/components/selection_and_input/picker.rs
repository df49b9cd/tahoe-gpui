//! HIG Picker — desktop dropdown selector.
//!
//! A stateless `RenderOnce` component for selecting a value from a list.
//! The parent owns the open/closed state via `is_open` + `on_toggle`.

use crate::callback_types::{OnSharedStringRefChange, OnToggle, rc_wrap};
use crate::components::menus_and_actions::popup_button::OnHighlight;
use crate::components::selection_and_input::segmented_control::{SegmentItem, SegmentedControl};
use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup, FocusGroupExt,
};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::DROPDOWN_MAX_HEIGHT;
use crate::foundations::materials::{
    LensEffect, apply_focus_ring, apply_standard_control_styling, glass_lens_surface,
};
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, IntoElement, KeyDownEvent, MouseDownEvent, SharedString, Window,
    div, px,
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
    /// Optional focus handle; when set, the picker tracks GPUI's focus
    /// graph and lights the ring reactively. Takes precedence over
    /// [`Picker::focused`]. Ignored on [`PickerStyle::Segmented`] — use
    /// [`super::SegmentedControl::focus_handle`] directly in that case.
    focus_handle: Option<FocusHandle>,
    highlighted_index: Option<usize>,
    /// Visual style. Defaults to [`PickerStyle::Menu`].
    style: PickerStyle,
    accessibility_label: Option<SharedString>,
    /// Host-owned focus group used by the `Palette` style when
    /// `AccessibilityMode::FULL_KEYBOARD_ACCESS` is active. Ignored by
    /// the other picker styles.
    palette_focus_group: Option<FocusGroup>,
    /// Host-owned per-tile focus handles for the `Palette` style. Expected
    /// to contain exactly one handle per palette item. Ignored unless the
    /// active theme reports FKA and the group is also supplied.
    palette_focus_handles: Vec<FocusHandle>,
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
            focus_handle: None,
            highlighted_index: None,
            style: PickerStyle::Menu,
            accessibility_label: None,
            palette_focus_group: None,
            palette_focus_handles: Vec::new(),
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

    /// Accessibility label for the picker's outer container, read as a
    /// VoiceOver landmark (e.g. "Theme", "Size").
    ///
    /// Consumed by every style: attached as the `Group` label on
    /// Radio/Menu/Inline/Wheel/Palette containers, and forwarded to
    /// [`SegmentedControl::accessibility_label`] for the Segmented style.
    ///
    /// Attached through [`AccessibleExt::with_accessibility`], which is
    /// a structural no-op today because GPUI v0.231.1-pre has no public
    /// `accessibility_label` API. When that upstream API lands, this
    /// label lights up for VoiceOver without per-caller changes.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
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

    /// Marks this picker as keyboard-focused, showing a visible focus
    /// ring. Ignored when a [`focus_handle`](Self::focus_handle) is
    /// supplied — the handle's reactive state
    /// (`handle.is_focused(window)`) takes precedence.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Wire the picker into GPUI's focus graph. When set, the focus ring
    /// renders based on `handle.is_focused(window)` — takes precedence
    /// over [`Picker::focused`]. Ignored on [`PickerStyle::Segmented`];
    /// build a [`super::SegmentedControl`] directly instead.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
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

    /// Attach a host-owned [`FocusGroup`] for per-tile arrow-nav and
    /// Tab-reachability under macOS Full Keyboard Access. Only honoured by
    /// the [`PickerStyle::Palette`] layout, which exposes each tile as a
    /// Tab stop when this is paired with
    /// [`Picker::palette_focus_handles`] and the active theme reports FKA.
    /// Use [`FocusGroup::open`] so Tab still exits the palette naturally.
    pub fn palette_focus_group(mut self, group: FocusGroup) -> Self {
        self.palette_focus_group = Some(group);
        self
    }

    /// Per-tile [`FocusHandle`]s for the [`PickerStyle::Palette`] layout.
    /// Expected to hold exactly one handle per palette item, in item
    /// order. Host-owned: stateless components cannot keep handles across
    /// renders.
    pub fn palette_focus_handles(mut self, handles: Vec<FocusHandle>) -> Self {
        self.palette_focus_handles = handles;
        self
    }
}

/// Index of the tile directly above `idx` in a `tiles_per_row` grid,
/// or `idx` itself when already on the top row (no-op clamp).
const fn palette_up_target(idx: usize, tiles_per_row: usize) -> usize {
    if idx < tiles_per_row {
        idx
    } else {
        idx - tiles_per_row
    }
}

/// Index of the tile directly below `idx` in a grid of `total` tiles /
/// `tiles_per_row`, preserving column. When the next row is ragged and
/// has no tile in the same column, clamps to the last tile. Returns
/// `idx` itself when already on the bottom row. Requires `total > 0`.
const fn palette_down_target(idx: usize, total: usize, tiles_per_row: usize) -> usize {
    let col = idx % tiles_per_row;
    let row_idx = idx / tiles_per_row;
    let last_row = total.saturating_sub(1) / tiles_per_row;
    if row_idx >= last_row {
        return idx;
    }
    let target = (row_idx + 1) * tiles_per_row + col;
    let max = total.saturating_sub(1);
    if target > max { max } else { target }
}

impl RenderOnce for Picker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let style = self.style;

        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

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
                "Picker::focused is ignored on PickerStyle::Segmented — \
                 construct a SegmentedControl directly with \
                 `.focus_handle(...)` to render a focus ring",
            );
            debug_assert!(
                self.focus_handle.is_none(),
                "Picker::focus_handle is ignored on PickerStyle::Segmented — \
                 construct a SegmentedControl directly with \
                 `.focus_handle(...)` to render a focus ring",
            );
            let mut control = SegmentedControl::new(self.id.clone())
                .items(segments)
                .selected(selected_idx);
            if let Some(label) = self.accessibility_label {
                control = control.accessibility_label(label);
            }
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
            // VoiceOver `value` for the group: the display label of the
            // currently-selected item (mapped from `selected.value` →
            // matching `item.label`). Sampled before `items_flat` is
            // consumed by the render loop below.
            let selected_label: Option<SharedString> = items_flat
                .iter()
                .find(|i| selected_value.as_ref() == Some(&i.value))
                .map(|i| i.label.clone());
            let on_change = rc_wrap(self.on_change);
            let on_highlight = self.on_highlight.map(Rc::new);

            let mut list = div()
                .id(self.id.clone())
                .focusable()
                .flex()
                .flex_col()
                .w_full()
                .gap(theme.spacing_xs);
            if let Some(handle) = self.focus_handle.as_ref() {
                list = list.track_focus(handle);
            }

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
                let item_label = item.label.clone();

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
                row = row.with_accessibility(
                    &AccessibilityProps::new()
                        .role(AccessibilityRole::RadioButton)
                        .label(item_label)
                        .value(if is_selected {
                            "Selected"
                        } else {
                            "Unselected"
                        })
                        .posinset(idx + 1)
                        .setsize(item_count),
                );
                list = list.child(row);
            }

            list = apply_focus_ring(list, theme, focused, &[]);
            let mut group_props = AccessibilityProps::new().role(AccessibilityRole::RadioGroup);
            if let Some(label) = self.accessibility_label {
                group_props = group_props.label(label);
            }
            if let Some(value) = selected_label {
                group_props = group_props.value(value);
            }
            list = list.with_accessibility(&group_props);
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
            // VoiceOver `value` for the group: display label of the
            // currently-selected item. Sampled before `items_flat` is
            // consumed by the render loop below.
            let selected_label: Option<SharedString> =
                selected_idx.and_then(|idx| items_flat.get(idx).map(|i| i.label.clone()));
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
            if let Some(handle) = self.focus_handle.as_ref() {
                wheel = wheel.track_focus(handle);
            }
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

            wheel = apply_focus_ring(wheel, theme, focused, &[]);
            let mut group_props = AccessibilityProps::new().role(AccessibilityRole::Group);
            if let Some(label) = self.accessibility_label {
                group_props = group_props.label(label);
            }
            if let Some(value) = selected_label {
                group_props = group_props.value(value);
            }
            wheel = wheel.with_accessibility(&group_props);
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
            // VoiceOver `value` for the group: display label of the
            // currently-selected tile. `items_flat.chunks(...)` only
            // borrows, so this lookup can happen after the render loop,
            // but computing it up front keeps the scan adjacent to
            // `selected_value` for clarity.
            let selected_label: Option<SharedString> = items_flat
                .iter()
                .find(|i| selected_value.as_ref() == Some(&i.value))
                .map(|i| i.label.clone());

            const TILES_PER_ROW: usize = 10;
            const TILE_SIZE: f32 = 32.0;

            // FKA: only attach per-tile focus when the flag is set AND the
            // host supplied both a FocusGroup and exactly one handle per
            // tile. Strict count matching avoids leaking stale tab indices
            // from a previous item layout.
            let fka_tiles = FocusGroup::bind_if_fka(
                theme.full_keyboard_access(),
                self.palette_focus_group,
                self.palette_focus_handles,
                items_flat.len(),
            );

            // Only make the grid a Tab stop when a focus handle is
            // supplied. The Palette variant has no arrow-key navigation
            // of its own (unlike Radio / Wheel), so an unconditional
            // `.focusable()` would create a dead Tab stop for hosts that
            // never asked for focus wiring.
            let mut grid = div()
                .id(self.id.clone())
                .flex()
                .flex_col()
                .gap(theme.spacing_xs);
            if let Some(handle) = self.focus_handle.as_ref() {
                grid = grid.focusable().track_focus(handle);
            }

            let mut global_idx = 0usize;
            let total_tiles = items_flat.len();
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

                    // FKA: attach per-tile focus + 2D arrow-nav keybindings
                    // + per-tile focus ring. Left/Right step ±1 via the
                    // FocusGroup; Up/Down step by a whole row of tiles
                    // while preserving the same column — important for a
                    // ragged last row where the naive `+TILES_PER_ROW` jump
                    // would slide the focus into a neighbouring column if
                    // clamped. End jumps to the last tile; Home to the
                    // first. Enter/Space activates the tile via its click
                    // handler.
                    if let Some((group, handles)) = fka_tiles.as_ref() {
                        let handle = &handles[global_idx];
                        let is_focused = handle.is_focused(window);
                        tile = tile.focus_group(group, handle);
                        tile = apply_focus_ring(tile, theme, is_focused, &[]);

                        // Precompute the two handles the Up/Down branches
                        // need. Doing this per-tile avoids cloning the
                        // full `Vec<FocusHandle>` into every on_key_down
                        // closure (O(N²) across the render); each branch
                        // now holds a single `FocusHandle` (Rc bump only).
                        let up_handle =
                            handles[palette_up_target(global_idx, TILES_PER_ROW)].clone();
                        let down_handle = handles
                            [palette_down_target(global_idx, total_tiles, TILES_PER_ROW)]
                        .clone();

                        let nav_group = group.clone();
                        let nav_change = on_change.clone();
                        let nav_value = item.value.clone();
                        tile = tile.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                            match ev.keystroke.key.as_str() {
                                "left" => {
                                    nav_group.focus_previous(window, cx);
                                    cx.stop_propagation();
                                }
                                "right" => {
                                    nav_group.focus_next(window, cx);
                                    cx.stop_propagation();
                                }
                                "up" => {
                                    up_handle.focus(window, cx);
                                    cx.stop_propagation();
                                }
                                "down" => {
                                    down_handle.focus(window, cx);
                                    cx.stop_propagation();
                                }
                                "home" => {
                                    nav_group.focus_first(window, cx);
                                    cx.stop_propagation();
                                }
                                "end" => {
                                    nav_group.focus_last(window, cx);
                                    cx.stop_propagation();
                                }
                                _ => {
                                    if crate::foundations::keyboard::is_activation_key(ev)
                                        && let Some(ref h) = nav_change
                                    {
                                        h(&nav_value, window, cx);
                                        cx.stop_propagation();
                                    }
                                }
                            }
                        });
                    }

                    row = row.child(tile);
                    global_idx += 1;
                }
                grid = grid.child(row);
            }

            grid = apply_focus_ring(grid, theme, focused, &[]);
            let mut group_props = AccessibilityProps::new().role(AccessibilityRole::Group);
            if let Some(label) = self.accessibility_label {
                group_props = group_props.label(label);
            }
            if let Some(value) = selected_label {
                group_props = group_props.value(value);
            }
            grid = grid.with_accessibility(&group_props);
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

        // Resolve the display label for the trigger button and the
        // group's VoiceOver `value` in one lookup. `selected_label` is
        // `None` when nothing is selected (so the group announces its
        // `label` alone); the trigger falls back to `placeholder`.
        let selected_label: Option<SharedString> = self
            .selected
            .as_ref()
            .and_then(|sel| flat_items.iter().find(|i| &i.value == sel))
            .map(|item| item.label.clone());
        let trigger_label: SharedString = selected_label
            .clone()
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
            .cursor_pointer()
            .focusable();
        if let Some(handle) = self.focus_handle.as_ref() {
            trigger = trigger.track_focus(handle);
        }

        // Glass-styled trigger surface.
        trigger = apply_standard_control_styling(trigger, theme, GlassSize::Small, focused);

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

        // Build the list element if visible; assembly into either a flow
        // container (Inline) or an `AnchoredOverlay` (Menu) happens after
        // the build block so the list-construction logic stays unchanged.
        let mut list_el: Option<gpui::AnyElement> = None;

        if is_list_visible {
            let highlighted_index = self.highlighted_index;
            let item_count = flat_items.len();

            // Collect item values for keyboard enter selection.
            let item_values: Vec<SharedString> =
                flat_items.iter().map(|i| i.value.clone()).collect();

            // ── Dropdown list (Liquid Glass lens) ───────────────────────────
            // Inline mode renders the list flow-positioned directly below
            // the trigger (no max-height clamp, full width of the
            // container). Menu mode is wrapped in `AnchoredOverlay` at
            // assembly time — the list body carries the max-height and
            // the lens surface; positioning is owned by the overlay.
            let dropdown_effect = LensEffect::liquid_glass(GlassSize::Medium, theme);
            let mut list = glass_lens_surface(theme, &dropdown_effect, GlassSize::Medium)
                .flex()
                .flex_col()
                .overflow_hidden();
            if style == PickerStyle::Inline {
                list = list.w_full();
            } else {
                list = list.max_h(px(DROPDOWN_MAX_HEIGHT));
            }
            let mut list = list
                .id(ElementId::from((self.id.clone(), "dropdown")))
                .debug_selector(|| "picker-dropdown".into())
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
                let item_label = item.label.clone();

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

                // MenuItem: role + label only. Selection state is carried
                // by the group's `value` (the selected item's label) and,
                // in the future, a native AX `selected` trait — mirroring
                // how AppKit announces `NSMenuItem`. Matching Checkbox's
                // per-row `Checked/Unchecked/Mixed` would double-announce.
                let row_props = AccessibilityProps::new()
                    .role(AccessibilityRole::MenuItem)
                    .label(item_label);
                row = row.with_accessibility(&row_props);

                list = list.child(row);
            }

            list_el = Some(list.into_any_element());
        }

        // Assemble trigger + (optional) list. Inline keeps the normal-
        // flow layout the legacy code had; Menu uses `AnchoredOverlay`
        // so the dropdown escapes parent `overflow_hidden()` clipping
        // and picks up the primitive's snap-margin + flip-on-overflow.
        let body: gpui::AnyElement = if style == PickerStyle::Inline {
            let mut c = div().relative().child(trigger);
            if let Some(list) = list_el {
                c = c.child(list);
            }
            c.into_any_element()
        } else {
            let overlay_id = ElementId::from((self.id.clone(), "overlay"));
            let mut overlay = AnchoredOverlay::new(overlay_id, trigger)
                .anchor(OverlayAnchor::BelowLeft)
                .gap(theme.dropdown_offset);
            if let Some(list) = list_el {
                overlay = overlay.content(list);
            }
            overlay.into_any_element()
        };

        // Group-level VoiceOver landmark for Menu + Inline styles. The
        // wrapper div carries the group role so VoiceOver announces it
        // before descending into the trigger + dropdown children. The
        // accessibility metadata can't live on `AnchoredOverlay` itself
        // (the primitive is a custom `Element`, not an `InteractiveElement`).
        let mut group_props = AccessibilityProps::new().role(AccessibilityRole::Group);
        if let Some(label) = self.accessibility_label {
            group_props = group_props.label(label);
        }
        if let Some(value) = selected_label {
            group_props = group_props.value(value);
        }
        div()
            .with_accessibility(&group_props)
            .child(body)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{Picker, PickerItem, PickerStyle};
    use crate::foundations::accessibility::{AccessibilityMode, FocusGroup};
    use crate::foundations::icons::IconName;
    use crate::foundations::theme::TahoeTheme;
    use crate::test_helpers::helpers::setup_test_window;
    use core::prelude::v1::test;
    use gpui::{Context, FocusHandle, IntoElement, Render, SharedString, TestAppContext, Window};

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
    fn picker_focus_handle_none_by_default() {
        let p = Picker::new("test");
        assert!(p.focus_handle.is_none());
    }

    #[gpui::test]
    async fn picker_focus_handle_builder_stores_handle(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let handle = cx.focus_handle();
            let p = Picker::new("test").focus_handle(&handle);
            assert!(
                p.focus_handle.is_some(),
                "focus_handle(..) must round-trip into the field"
            );
        });
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

    #[test]
    fn picker_palette_focus_fields_default_empty() {
        let p = Picker::new("test");
        assert!(p.palette_focus_group.is_none());
        assert!(p.palette_focus_handles.is_empty());
    }

    #[test]
    fn picker_accessibility_label_default_is_none() {
        let p = Picker::new("t");
        assert!(p.accessibility_label.is_none());
    }

    #[test]
    fn picker_accessibility_label_builder() {
        let p = Picker::new("t").accessibility_label("Theme");
        assert_eq!(
            p.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Theme"),
        );
    }

    #[test]
    fn picker_accessibility_label_survives_all_styles() {
        // The builder lives on `Picker` (shared by every style). This
        // test pins the contract that setting it for any style keeps the
        // label on the struct — render() then forwards it to the
        // style-specific container / SegmentedControl.
        for style in [
            PickerStyle::Menu,
            PickerStyle::Inline,
            PickerStyle::Segmented,
            PickerStyle::Radio,
            PickerStyle::Wheel,
            PickerStyle::Palette,
        ] {
            let p = Picker::new("t").style(style).accessibility_label("Size");
            assert_eq!(p.style, style);
            assert_eq!(
                p.accessibility_label.as_ref().map(|s| s.as_ref()),
                Some("Size"),
                "label dropped for style {style:?}",
            );
        }
    }

    // ─── HIG: Full Keyboard Access ───────────────────────────────────

    fn palette_items() -> Vec<PickerItem> {
        vec![
            PickerItem::new("Star", "s").icon(IconName::StarFill),
            PickerItem::new("Circle", "c").icon(IconName::CircleFilled),
            PickerItem::new("Bell", "b"),
        ]
    }

    struct PickerFkaHarness {
        handles: Vec<FocusHandle>,
        group: FocusGroup,
    }

    impl PickerFkaHarness {
        fn new(cx: &mut Context<Self>, count: usize) -> Self {
            Self {
                handles: (0..count).map(|_| cx.focus_handle()).collect(),
                group: FocusGroup::open(),
            }
        }
    }

    impl Render for PickerFkaHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            Picker::new("palette-test")
                .style(PickerStyle::Palette)
                .items(palette_items())
                .palette_focus_group(self.group.clone())
                .palette_focus_handles(self.handles.clone())
        }
    }

    #[gpui::test]
    async fn fka_off_does_not_register_palette_handles(cx: &mut TestAppContext) {
        let (host, _cx) = setup_test_window(cx, |_window, cx| PickerFkaHarness::new(cx, 3));
        host.update(cx, |host, _cx| {
            assert!(host.group.is_empty());
        });
    }

    #[gpui::test]
    async fn fka_on_registers_one_focus_per_tile(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PickerFkaHarness::new(cx, 3)
        });
        host.update(vcx, |host, _cx| {
            assert_eq!(host.group.len(), 3);
        });
    }

    #[gpui::test]
    async fn fka_on_preserves_registration_order(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PickerFkaHarness::new(cx, 3)
        });
        host.update(vcx, |host, _cx| {
            for (i, handle) in host.handles.iter().enumerate() {
                assert_eq!(host.group.register(handle), i);
            }
        });
    }

    #[gpui::test]
    async fn fka_on_mismatched_handle_count_skips_registration(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PickerFkaHarness {
                group: FocusGroup::open(),
                handles: vec![cx.focus_handle()], // 1 for 3 items
            }
        });
        host.update(vcx, |host, _cx| {
            assert!(host.group.is_empty());
        });
    }

    // ─── Palette 2D arrow-nav: 25 tiles = 2 full rows + 1 partial (5) ───
    //
    // Pure helpers `palette_up_target` / `palette_down_target` are the
    // single source of truth for the render path's Up/Down targets, so
    // verifying their arithmetic verifies the closure's navigation.

    const TILES_PER_ROW: usize = 10;
    const TOTAL_TILES: usize = 25; // rows: 0..10, 10..20, 20..25

    #[test]
    fn up_top_row_is_noop() {
        // Any tile on the top row stays put (Up from row 0 = no-op).
        for idx in 0..TILES_PER_ROW {
            assert_eq!(super::palette_up_target(idx, TILES_PER_ROW), idx);
        }
    }

    #[test]
    fn up_middle_row_moves_one_row_up() {
        // Tile 15 is row 1, col 5 → moves to tile 5 (row 0, col 5).
        assert_eq!(super::palette_up_target(15, TILES_PER_ROW), 5);
    }

    #[test]
    fn up_bottom_row_moves_one_row_up() {
        // Tile 22 is row 2, col 2 → moves to tile 12 (row 1, col 2).
        assert_eq!(super::palette_up_target(22, TILES_PER_ROW), 12);
    }

    #[test]
    fn down_middle_row_preserves_column_into_full_row() {
        // Tile 5 (row 0, col 5) → tile 15 (row 1, col 5). Both full rows.
        assert_eq!(
            super::palette_down_target(5, TOTAL_TILES, TILES_PER_ROW),
            15
        );
    }

    #[test]
    fn down_middle_row_into_ragged_row_clamps_to_last_tile() {
        // Tile 15 (row 1, col 5) → row 2 has no col 5 (last row is
        // tiles 20..25, cols 0..5 only; col 5 doesn't exist). Clamp
        // to last tile (24).
        assert_eq!(
            super::palette_down_target(15, TOTAL_TILES, TILES_PER_ROW),
            24
        );
    }

    #[test]
    fn down_middle_row_into_ragged_row_keeps_existing_column() {
        // Tile 13 (row 1, col 3) → tile 23 (row 2, col 3) exists, no clamp.
        assert_eq!(
            super::palette_down_target(13, TOTAL_TILES, TILES_PER_ROW),
            23
        );
    }

    #[test]
    fn down_bottom_row_is_noop() {
        // Any tile on the last row stays put.
        for idx in 20..TOTAL_TILES {
            assert_eq!(
                super::palette_down_target(idx, TOTAL_TILES, TILES_PER_ROW),
                idx
            );
        }
    }

    #[test]
    fn down_single_row_is_noop() {
        // With only 5 tiles (single ragged row), Down anywhere is a no-op.
        for idx in 0..5 {
            assert_eq!(super::palette_down_target(idx, 5, TILES_PER_ROW), idx);
        }
    }

    struct PaletteGridHarness {
        handles: Vec<FocusHandle>,
        group: FocusGroup,
    }

    impl Render for PaletteGridHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let items: Vec<PickerItem> = (0..TOTAL_TILES)
                .map(|i| PickerItem::new(format!("Item {i}"), format!("v{i}")))
                .collect();
            Picker::new("palette-grid-test")
                .style(PickerStyle::Palette)
                .items(items)
                .palette_focus_group(self.group.clone())
                .palette_focus_handles(self.handles.clone())
        }
    }

    #[gpui::test]
    async fn fka_on_registers_all_tiles_in_25_item_grid(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PaletteGridHarness {
                handles: (0..TOTAL_TILES).map(|_| cx.focus_handle()).collect(),
                group: FocusGroup::open(),
            }
        });
        host.update(vcx, |host, _cx| {
            assert_eq!(host.group.len(), TOTAL_TILES);
        });
    }

    #[gpui::test]
    async fn fka_on_25_item_grid_focus_next_walks_registration_order(cx: &mut TestAppContext) {
        // Proves Left/Right arrow-nav (which calls focus_next/previous)
        // walks the grid in flat registration order.
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PaletteGridHarness {
                handles: (0..TOTAL_TILES).map(|_| cx.focus_handle()).collect(),
                group: FocusGroup::open(),
            }
        });
        host.update_in(vcx, |host, window, cx| {
            host.handles[9].focus(window, cx); // end of row 0
            host.group.focus_next(window, cx); // → row 1, col 0
            assert!(host.handles[10].is_focused(window));
        });
    }

    #[gpui::test]
    async fn fka_on_25_item_grid_focus_last_jumps_to_final_tile(cx: &mut TestAppContext) {
        // End key → focus_last → last registered handle (index 24).
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PaletteGridHarness {
                handles: (0..TOTAL_TILES).map(|_| cx.focus_handle()).collect(),
                group: FocusGroup::open(),
            }
        });
        host.update_in(vcx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_last(window, cx);
            assert!(host.handles[TOTAL_TILES - 1].is_focused(window));
        });
    }
}

#[cfg(test)]
mod clip_escape_tests {
    use gpui::prelude::*;
    use gpui::{Context, IntoElement, Render, TestAppContext, div, px};

    use super::{Picker, PickerItem, PickerStyle};
    use crate::test_helpers::helpers::{LocatorExt, setup_test_window};

    /// Mirrors the PopupButton clip-escape pattern: the non-inline
    /// picker dropdown must anchor past the parent clip region.
    struct ClipEscapeHarness;

    impl Render for ClipEscapeHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div().pt(px(120.0)).pl(px(40.0)).child(
                div()
                    .debug_selector(|| "clip-region".into())
                    .w(px(160.0))
                    .h(px(32.0))
                    .overflow_hidden()
                    .child(
                        Picker::new("picker")
                            .items(vec![
                                PickerItem::new("Alpha", "a"),
                                PickerItem::new("Beta", "b"),
                                PickerItem::new("Gamma", "c"),
                            ])
                            .style(PickerStyle::Menu)
                            .open(true),
                    ),
            )
        }
    }

    #[gpui::test]
    async fn menu_style_dropdown_anchors_outside_parent_clip(cx: &mut TestAppContext) {
        let (_host, cx) = setup_test_window(cx, |_window, _cx| ClipEscapeHarness);

        let clip = cx.get_element("clip-region");
        let dropdown = cx.get_element("picker-dropdown");

        assert!(
            dropdown.bounds.top() >= clip.bounds.bottom(),
            "dropdown.top() {:?} should be at or below clip.bottom() {:?}",
            dropdown.bounds.top(),
            clip.bounds.bottom(),
        );
    }
}
