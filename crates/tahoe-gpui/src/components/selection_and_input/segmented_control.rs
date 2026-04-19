//! HIG segmented control component.
//!
//! Distinct from tabs — used for switching between related views or filtering.
//! All segments have equal width with a sliding selection indicator.

use crate::callback_types::{OnUsizeChange, rc_wrap};
use crate::foundations::materials::{apply_focus_ring, apply_high_contrast_border};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, KeyDownEvent, SharedString, Window, div, px};

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
/// Exactly one segment is selected at a time.
#[derive(IntoElement)]
pub struct SegmentedControl {
    id: ElementId,
    items: Vec<SegmentItem>,
    selected: usize,
    on_change: OnUsizeChange,
    focused: bool,
}

impl SegmentedControl {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            selected: 0,
            on_change: None,
            focused: false,
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

    /// Marks this control as keyboard-focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl RenderOnce for SegmentedControl {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.clamp_selected();
        let theme = cx.theme();
        let on_change = rc_wrap(self.on_change);

        let glass = &theme.glass;
        let track_bg = glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
        // HIG Tahoe/Liquid Glass: selected segments use a distinct elevated
        // fill, not the hover background. Sharing `hover_bg` caused the
        // selection indicator to melt into hover state as soon as the
        // pointer entered another segment (see the HIG Selection & Input audit
        // finding 34).
        let selected_bg = theme.surface;
        let radius = glass.radius(GlassSize::Small);

        // Arrow key navigation: Left/Right move selection
        let arrow_handler = on_change.clone();
        let selected = self.selected;
        let item_count_for_keys = self.items.len();

        let mut track = div()
            .id(self.id)
            .focusable()
            .flex()
            .items_center()
            .bg(track_bg)
            .rounded(radius)
            .overflow_hidden()
            .p(px(SEGMENTED_INSET))
            .min_h(px(theme.target_size()));

        if let Some(handler) = arrow_handler {
            track = track.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if item_count_for_keys == 0 {
                    return;
                }
                let new_index = match key {
                    "left" => Some(if selected == 0 {
                        item_count_for_keys - 1
                    } else {
                        selected - 1
                    }),
                    "right" => Some((selected + 1) % item_count_for_keys),
                    "home" => Some(0),
                    "end" => Some(item_count_for_keys - 1),
                    _ => None,
                };
                if let Some(idx) = new_index {
                    handler(idx, window, cx);
                }
            });
        }

        // Apply glass shadows (with focus ring when focused)
        let base_shadows = glass.shadows(GlassSize::Small);
        track = apply_focus_ring(track, theme, self.focused, base_shadows);
        track = apply_high_contrast_border(track, theme);

        let item_count = self.items.len();
        let last_idx = item_count.saturating_sub(1);
        for (i, item) in self.items.into_iter().enumerate() {
            let is_selected = i == self.selected;
            let handler = on_change.clone();
            let text_color = if is_selected {
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
                .min_h(px(theme.target_size() - 4.0)) // Account for track padding
                .px(theme.spacing_sm)
                .text_color(text_color)
                .text_style(TextStyle::Subheadline, theme)
                .cursor_pointer();

            if is_selected {
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
                        segment = segment
                            .rounded_tl(inner_radius)
                            .rounded_bl(inner_radius);
                    }
                    (false, true) => {
                        segment = segment
                            .rounded_tr(inner_radius)
                            .rounded_br(inner_radius);
                    }
                    (false, false) => {
                        // Interior chip: stays square so it reads as a
                        // flush rectangle between its neighbours.
                    }
                }
                // Glass is always present; no opaque shadow fallback needed.
            } else {
                segment = segment.hover(|style| style.bg(theme.hover));
            }

            if let Some(handler) = handler {
                segment = segment.on_click(move |_event, window, cx| {
                    handler(i, window, cx);
                });
            }

            if let Some(icon) = item.icon {
                segment = segment.child(icon);
            }
            segment = segment.child(item.label);

            track = track.child(segment);

            // Separator between non-selected adjacent segments
            if i + 1 < item_count && !is_selected && i + 1 != self.selected {
                track = track.child(
                    div()
                        .w(theme.separator_thickness)
                        .h(px(theme.row_height() * 0.55))
                        .bg(theme.border)
                        .flex_shrink_0(),
                );
            }
        }

        track
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
    fn focused_defaults_to_false() {
        let sc = SegmentedControl::new("test");
        assert!(!sc.focused);
    }

    #[test]
    fn focused_builder() {
        let sc = SegmentedControl::new("test").focused(true);
        assert!(sc.focused);
    }

    #[test]
    fn segment_item_label() {
        let item = SegmentItem::new("Test");
        assert_eq!(item.label.as_ref(), "Test");
        assert!(item.icon.is_none());
    }
}
