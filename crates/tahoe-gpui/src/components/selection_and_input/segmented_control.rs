//! HIG segmented control component.
//!
//! Distinct from tabs — used for switching between related views or filtering.
//! All segments have equal width with a sliding selection indicator.

use crate::callback_types::{OnUsizeChange, rc_wrap};
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::materials::{apply_focus_ring, apply_high_contrast_border};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, FocusHandle, KeyDownEvent, SharedString, Window, div, px};

/// Inner padding of the segmented-control track that creates the visual
/// inset between the track edge and the selected-segment indicator.
///
/// HIG / AppKit `NSSegmentedControl` uses a 2 pt inset so the
/// selected-segment chip sits concentric with the track bezel.
const SEGMENTED_INSET: f32 = 2.0;

/// A single segment item.
pub struct SegmentItem {
    /// Display label.
    pub label: SharedString,
    /// Optional icon rendered before the label.
    pub icon: Option<AnyElement>,
}

impl SegmentItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }
}

/// A segmented control per HIG.
///
/// Renders a horizontal row of equal-width segments with glass styling.
/// Exactly one segment is selected at a time, unless [`Self::momentary`]
/// is enabled — in that mode no segment persists selection state and
/// clicks fire `on_change` without visually marking a winner.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    items: Vec<SegmentItem>,
    selected: usize,
    on_change: OnUsizeChange,
    momentary: bool,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
}

impl SegmentedControl {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            selected: 0,
            on_change: None,
            momentary: false,
            disabled: false,
            focus_handle: None,
        }
    }

    pub fn items(mut self, items: Vec<SegmentItem>) -> Self {
        self.items = items;
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Clamp `selected` to a valid index for the current items list.
    ///
    /// Called before render so an out-of-range index set via the builder
    /// cannot silently produce a "no segment selected" state.
    fn clamp_selected(&mut self) {
        // HIG macOS: "Aim for no more than about five to seven segments."
        // Surface the violation loudly in debug builds; render keeps
        // going in release so consumers don't crash.
        debug_assert!(
            self.items.len() <= 7,
            "SegmentedControl: {} segments exceeds the HIG ceiling of 7",
            self.items.len()
        );
        if self.items.is_empty() {
            self.selected = 0;
        } else if self.selected >= self.items.len() {
            self.selected = self.items.len() - 1;
        }
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Toggle momentary mode. When enabled, clicks fire [`Self::on_change`]
    /// but no segment persists a selected visual state — matches AppKit's
    /// `NSSegmentStyleTexturedRounded` momentary trackingMode. Keyboard
    /// still dispatches `on_change` via Left/Right/Home/End.
    pub fn momentary(mut self, momentary: bool) -> Self {
        self.momentary = momentary;
        self
    }

    /// Disable all interaction. Segments stop receiving clicks, keyboard
    /// navigation is dropped, and the cursor becomes the default arrow.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Supply a host-owned focus handle so the segmented control
    /// participates in the parent's focus graph. The handle drives the
    /// focus-ring state — when no handle is supplied, the control never
    /// renders a focus ring.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.clamp_selected();
        let theme = cx.theme();

        // Footgun: a focus_handle implies keyboard interaction, but the key
        // handler only dispatches when `on_change` is also set. Without the
        // handler the control would Tab-focus, render a ring, and swallow
        // every arrow key silently. Surface loudly in debug.
        debug_assert!(
            !(self.focus_handle.is_some() && self.on_change.is_none()),
            "SegmentedControl: focus_handle is set but on_change is not — \
             arrow keys will be dropped silently",
        );

        // Empty items: short-circuit with an empty track so downstream logic
        // (selected-label lookup, separator placement) never has to reason
        // about the "no valid selection" state.
        if self.items.is_empty() {
            let glass = &theme.glass;
            let track_bg = glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
            let radius = glass.radius(GlassSize::Small);
            return div()
                .id(self.id)
                .bg(track_bg)
                .rounded(radius)
                .min_h(px(theme.target_size()))
                .into_any_element();
        }

        let on_change = rc_wrap(self.on_change);
        let focused = self
            .focus_handle
            .as_ref()
            .is_some_and(|h| h.is_focused(window));

        let glass = &theme.glass;
        let track_bg = glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
        // HIG Tahoe/Liquid Glass: selected segments use a distinct elevated
        // fill, not the hover background. Sharing `hover_bg` caused the
        // selection indicator to melt into hover state as soon as the
        // pointer entered another segment (see the HIG Selection & Input audit
        // finding 34).
        let selected_bg = theme.surface;
        let radius = glass.radius(GlassSize::Small);

        // Arrow key navigation: Left/Right move selection (mirrored in RTL).
        let arrow_handler = on_change.clone();
        let selected = self.selected;
        let item_count_for_keys = self.items.len();
        let is_rtl = theme.is_rtl();

        // Selected segment label for VoiceOver's value announcement — sampled
        // before the items vector is consumed by the render loop below.
        let selected_label = self.items[self.selected].label.clone();

        let mut track = div()
            .id(self.id)
            .flex()
            .items_center()
            .bg(track_bg)
            .rounded(radius)
            .overflow_hidden()
            .p(px(SEGMENTED_INSET))
            .min_h(px(theme.target_size()));

        // Disabled controls drop out of the Tab order entirely — WCAG 2.4.3
        // and HIG both prefer skipping non-interactive controls rather than
        // landing Tab on a dead stop. `.focusable()` + `.track_focus()` +
        // arrow handler are therefore gated together.
        if !self.disabled {
            track = track.focusable();
            if let Some(handle) = self.focus_handle.as_ref() {
                track = track.track_focus(handle);
            }
            if let Some(handler) = arrow_handler {
                track = track.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    if item_count_for_keys == 0 {
                        return;
                    }
                    // Visual-leading motion: in RTL the leading segment is
                    // on the right, so `Right` must decrement and `Left`
                    // must increment. Home/End stay absolute.
                    let decrement = if is_rtl { "right" } else { "left" };
                    let increment = if is_rtl { "left" } else { "right" };
                    let new_index = if key == decrement {
                        Some(if selected == 0 {
                            item_count_for_keys - 1
                        } else {
                            selected - 1
                        })
                    } else if key == increment {
                        Some((selected + 1) % item_count_for_keys)
                    } else if key == "home" {
                        Some(0)
                    } else if key == "end" {
                        Some(item_count_for_keys - 1)
                    } else {
                        None
                    };
                    if let Some(idx) = new_index {
                        handler(idx, window, cx);
                    }
                });
            }
        }

        // Apply glass shadows (with focus ring when focused and enabled).
        // Disabled controls never render a focus ring — matches
        // `text_field.rs` `show_focus_ring = is_focused && !self.disabled`.
        let base_shadows = glass.shadows(GlassSize::Small);
        let show_focus_ring = focused && !self.disabled;
        track = apply_focus_ring(track, theme, show_focus_ring, base_shadows);
        track = apply_high_contrast_border(track, theme);

        let item_count = self.items.len();
        let last_idx = item_count.saturating_sub(1);
        let momentary = self.momentary;
        let disabled = self.disabled;
        for (i, item) in self.items.into_iter().enumerate() {
            // Momentary controls never persist a selected chip — every
            // segment renders in its idle state regardless of
            // `self.selected`. The field stays around so the keyboard
            // handler can still compute relative nav targets.
            let is_selected = !momentary && i == self.selected;
            let handler = on_change.clone();
            // HIG: disabled tint is a fixed muted color, not a proportional
            // opacity — `opacity(0.5)` fails WCAG 4.5:1 on low-contrast
            // variants. Same pattern as `button.rs`/`toggle.rs` disabled
            // branches.
            let text_color = if disabled {
                theme.text_disabled()
            } else if is_selected {
                theme.text
            } else {
                theme.text_muted
            };

            let mut segment = div()
                .id(ElementId::from(SharedString::from(format!("seg-{i}"))))
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .gap(theme.spacing_xs)
                .min_h(px(theme.target_size() - SEGMENTED_INSET * 2.0)) // Inner height = track height minus the 2pt inset top-and-bottom
                .px(theme.spacing_sm)
                .text_color(text_color)
                .text_style(TextStyle::Subheadline, theme);

            // Misleading-cursor fix (#65): only offer the pointer cursor
            // when the segment is actually actionable. A disabled control
            // or one without an `on_change` handler reads as decorative
            // and must keep the default arrow.
            if disabled || handler.is_none() {
                segment = segment.cursor_default();
            } else {
                segment = segment.cursor_pointer();
            }

            if is_selected && !disabled {
                // HIG: only the leading/trailing outermost selected segments
                // round their outer corners; interior selected segments are
                // square along the edges shared with neighbours. The inner
                // radius keeps the chip concentric with the track bezel
                // minus the 2 pt `SEGMENTED_INSET`.
                let inner_radius = (radius - px(SEGMENTED_INSET)).max(px(0.0));
                segment = segment.bg(selected_bg);
                let is_leading = i == 0;
                let is_trailing = i == last_idx;
                match (is_leading, is_trailing) {
                    (true, true) => segment = segment.rounded(inner_radius),
                    (true, false) => {
                        segment = segment.rounded_tl(inner_radius).rounded_bl(inner_radius);
                    }
                    (false, true) => {
                        segment = segment.rounded_tr(inner_radius).rounded_br(inner_radius);
                    }
                    (false, false) => {
                        // Interior chip: stays square so it reads as a
                        // flush rectangle between its neighbours.
                    }
                }
                // Glass is always present; no opaque shadow fallback needed.
            } else if !is_selected && !disabled {
                segment = segment.hover(|style| style.bg(theme.hover));
            }
            // Disabled selected segment: no elevated fill, no hover — the
            // chip shares the track background so the control never reads
            // as interactive. Text remains `text_disabled()` from above.

            if !disabled && let Some(handler) = handler {
                segment = segment.on_click(move |_event, window, cx| {
                    handler(i, window, cx);
                });
            }

            if let Some(icon) = item.icon {
                segment = segment.child(icon);
            }
            segment = segment.child(item.label);

            track = track.child(segment);

            // Separator between non-selected adjacent segments. In
            // momentary mode no segment is selected, so a separator is
            // drawn between every pair. Disabled selected segments render
            // with no chip, so the chip can't absorb the separator either.
            let chip_visible = is_selected && !disabled;
            let next_chip_visible = !momentary && i + 1 == self.selected && !disabled;
            if i + 1 < item_count && !chip_visible && !next_chip_visible {
                track = track.child(
                    div()
                        .w(theme.separator_thickness)
                        .h(px(theme.row_height() * 0.55))
                        .bg(theme.border)
                        .flex_shrink_0(),
                );
            }
        }

        // VoiceOver scaffolding — GPUI v0.231.1-pre drops these props, but
        // wiring them here means the whole control lights up the AX tree
        // as soon as upstream lands `accessibility_role`/`value`. `Group`
        // is the closest role the crate exposes to an `NSAccessibilityRole`
        // tablist/segmented-control — each segment is a `Tab` conceptually,
        // but the crate renders segments as `div` children rather than
        // individually-labelled elements.
        let ax_props = AccessibilityProps::new()
            .role(AccessibilityRole::Group)
            .value(selected_label);
        track.with_accessibility(&ax_props).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{SegmentItem, SegmentedControl};
    use core::prelude::v1::test;

    #[test]
    fn default_selected_is_zero() {
        let sc = SegmentedControl::new("test");
        assert_eq!(sc.selected, 0);
    }

    #[test]
    fn items_builder() {
        let sc = SegmentedControl::new("test").items(vec![
            SegmentItem::new("One"),
            SegmentItem::new("Two"),
            SegmentItem::new("Three"),
        ]);
        assert_eq!(sc.items.len(), 3);
    }

    #[test]
    fn selected_builder() {
        let sc = SegmentedControl::new("test").selected(2);
        assert_eq!(sc.selected, 2);
    }

    #[test]
    fn on_change_builder() {
        let sc = SegmentedControl::new("test").on_change(|_, _, _| {});
        assert!(sc.on_change.is_some());
    }

    #[test]
    fn disabled_defaults_to_false() {
        let sc = SegmentedControl::new("test");
        assert!(!sc.disabled);
    }

    #[test]
    fn disabled_builder() {
        let sc = SegmentedControl::new("test").disabled(true);
        assert!(sc.disabled);
    }

    #[test]
    fn focus_handle_defaults_to_none() {
        let sc = SegmentedControl::new("test");
        assert!(sc.focus_handle.is_none());
    }

    #[test]
    fn segment_item_label() {
        let item = SegmentItem::new("Test");
        assert_eq!(item.label.as_ref(), "Test");
        assert!(item.icon.is_none());
    }

    #[test]
    fn momentary_defaults_to_false_and_builder_sets() {
        let sc = SegmentedControl::new("test");
        assert!(!sc.momentary);
        let sc = SegmentedControl::new("test").momentary(true);
        assert!(sc.momentary);
    }
}
