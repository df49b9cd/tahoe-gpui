//! HIG Rating Indicator — star rating display and optional input.
//!
//! A stateless `RenderOnce` component that renders a row of star icons
//! representing a rating value. When interactive, each star acts as a
//! clickable touch target.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/rating-indicators>

use crate::callback_types::{OnF32Change, rc_wrap};
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::{
    App, ElementId, FocusHandle, Hsla, KeyDownEvent, Pixels, SharedString, Window, div, px,
};

/// Star fill state for a single position in the rating row.
#[derive(Debug, Clone, Copy, PartialEq)]
enum StarFill {
    Full,
    Half,
    Empty,
}

impl StarFill {
    fn icon_name(self) -> IconName {
        match self {
            StarFill::Full => IconName::StarFill,
            StarFill::Half => IconName::StarLeadingHalfFilled,
            StarFill::Empty => IconName::Star,
        }
    }
}

/// Default star glyph size in points.
const DEFAULT_STAR_SIZE: Pixels = px(16.0);

/// A star rating indicator component.
///
/// Stateless `RenderOnce` — the parent owns the value and provides an
/// `on_change` callback for interactive mode.
#[derive(IntoElement)]
pub struct RatingIndicator {
    id: ElementId,
    value: f32,
    max: usize,
    interactive: bool,
    focused: bool,
    /// Optional host-supplied focus handle. Finding 18 in
    /// the Zed cross-reference audit — when set, the focus-ring derives from
    /// `handle.is_focused(window)` and the root element threads
    /// `track_focus(&handle)`; falls back to [`focused`](Self::focused).
    focus_handle: Option<FocusHandle>,
    color: Option<Hsla>,
    star_size: Option<Pixels>,
    accessibility_label: Option<SharedString>,
    on_change: OnF32Change,
}

impl RatingIndicator {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: 0.0,
            max: 5,
            interactive: false,
            focused: false,
            focus_handle: None,
            color: None,
            star_size: None,
            accessibility_label: None,
            on_change: None,
        }
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = value;
        self
    }

    pub fn max(mut self, max: usize) -> Self {
        self.max = max.max(1);
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the rating indicator participates in
    /// the host's focus graph. When set, the focus-ring is derived from
    /// `handle.is_focused(window)` and the root element threads
    /// `track_focus(&handle)` so Tab-cycling and keyboard shortcuts
    /// scoped to the handle fire correctly. Finding 18 in
    /// the Zed cross-reference audit.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Override the star glyph size (default: 16 pt).
    pub fn star_size(mut self, size: Pixels) -> Self {
        self.star_size = Some(size);
        self
    }

    /// Override the VoiceOver label. Defaults to `"<value> of <max> stars"`.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

/// Determine the fill state for a star at the given 1-based position.
fn star_fill_for(value: f32, position: usize) -> StarFill {
    let pos = position as f32;
    if pos <= value.floor() {
        StarFill::Full
    } else if pos == value.ceil() && value.fract() > 0.0 {
        StarFill::Half
    } else {
        StarFill::Empty
    }
}

/// Format the default VoiceOver label for a `value` / `max` pair.
///
/// Whole-number ratings read as "3 of 5 stars"; fractional ratings read as
/// "3.5 of 5 stars". VoiceOver announces the string after the component's
/// role (slider) so the user hears "3.5 of 5 stars, slider, adjustable".
fn default_accessibility_label(value: f32, max: usize) -> String {
    if (value.fract()).abs() < f32::EPSILON {
        format!("{} of {} stars", value as i64, max)
    } else {
        format!("{value:.1} of {max} stars")
    }
}

impl RenderOnce for RatingIndicator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let accent_color = self.color.unwrap_or(theme.accent);
        let muted_color = theme.text_muted;

        let max = self.max;
        let value = self.value.clamp(0.0, self.max as f32);
        let interactive = self.interactive;
        let on_change = rc_wrap(self.on_change);
        // Finding 18 in the Zed cross-reference audit.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        let star_size = self.star_size.unwrap_or(DEFAULT_STAR_SIZE);
        let target_size = px(theme.target_size());
        let half_target = px(theme.target_size() / 2.0);

        let a11y_label: SharedString = self
            .accessibility_label
            .unwrap_or_else(|| SharedString::from(default_accessibility_label(value, max)));
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(if interactive {
                AccessibilityRole::Slider
            } else {
                AccessibilityRole::StaticText
            })
            .value(SharedString::from(format!("{value:.1}")));

        let mut row = div()
            .id(self.id)
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .with_accessibility(&a11y_props);

        if interactive {
            row = row.focusable();
            if let Some(handle) = self.focus_handle.as_ref() {
                row = row.track_focus(handle);
            }
            row = apply_focus_ring(row, theme, focused, &[]);
        }

        for position in 1..=max {
            let fill = star_fill_for(value, position);
            let icon_color = match fill {
                StarFill::Full | StarFill::Half => accent_color,
                StarFill::Empty => muted_color,
            };

            let icon = Icon::new(fill.icon_name())
                .size(star_size)
                .color(icon_color);

            if interactive {
                let on_change_left = on_change.clone();
                let on_change_right = on_change.clone();
                let half_value = position as f32 - 0.5;
                let full_value = position as f32;

                let mut star_container = div()
                    .id(ElementId::NamedInteger(
                        "rating-star".into(),
                        position as u64,
                    ))
                    .min_w(target_size)
                    .min_h(target_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .relative()
                    .child(div().child(icon));

                let mut left_half = div()
                    .id(ElementId::NamedInteger(
                        "rating-star-left".into(),
                        position as u64,
                    ))
                    .absolute()
                    .left_0()
                    .top_0()
                    .w(half_target)
                    .h_full()
                    .cursor_pointer();

                if let Some(handler) = on_change_left {
                    left_half = left_half.on_click(move |_event, window, cx| {
                        handler(half_value, window, cx);
                    });
                }

                let mut right_half = div()
                    .id(ElementId::NamedInteger(
                        "rating-star-right".into(),
                        position as u64,
                    ))
                    .absolute()
                    .right_0()
                    .top_0()
                    .w(half_target)
                    .h_full()
                    .cursor_pointer();

                if let Some(handler) = on_change_right {
                    right_half = right_half.on_click(move |_event, window, cx| {
                        handler(full_value, window, cx);
                    });
                }

                star_container = star_container.child(left_half).child(right_half);

                row = row.child(star_container);
            } else {
                row = row.child(div().child(icon));
            }
        }

        if interactive && let Some(handler) = on_change {
            let max_f32 = max as f32;
            row = row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "left" => {
                        let new_value = (value - 0.5).max(0.0);
                        handler(new_value, window, cx);
                    }
                    "right" => {
                        let new_value = (value + 0.5).min(max_f32);
                        handler(new_value, window, cx);
                    }
                    "home" => {
                        handler(0.0, window, cx);
                    }
                    "end" => {
                        handler(max_f32, window, cx);
                    }
                    _ => {}
                }
            });
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_STAR_SIZE, RatingIndicator, StarFill, default_accessibility_label, star_fill_for,
    };
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn rating_defaults() {
        let r = RatingIndicator::new("test");
        assert!((r.value - 0.0).abs() < f32::EPSILON);
        assert_eq!(r.max, 5);
        assert!(!r.interactive);
        assert!(r.color.is_none());
        assert!(r.star_size.is_none());
        assert!(r.accessibility_label.is_none());
        assert!(r.on_change.is_none());
    }

    #[test]
    fn rating_value_builder() {
        let r = RatingIndicator::new("test").value(3.5);
        assert!((r.value - 3.5).abs() < f32::EPSILON);
    }

    #[test]
    fn rating_value_stores_raw() {
        let r = RatingIndicator::new("test").value(10.0);
        assert!((r.value - 10.0).abs() < f32::EPSILON);

        let r = RatingIndicator::new("test").value(-2.0);
        assert!((r.value - -2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rating_max_builder() {
        let r = RatingIndicator::new("test").max(10);
        assert_eq!(r.max, 10);
    }

    #[test]
    fn rating_star_size_builder() {
        let r = RatingIndicator::new("test").star_size(px(24.0));
        assert_eq!(r.star_size, Some(px(24.0)));
    }

    #[test]
    fn rating_star_size_default_is_16pt() {
        assert!((f32::from(DEFAULT_STAR_SIZE) - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rating_accessibility_label_builder() {
        let r = RatingIndicator::new("test").accessibility_label("Stars rating");
        assert_eq!(
            r.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Stars rating")
        );
    }

    #[test]
    fn rating_star_fill_full() {
        assert_eq!(star_fill_for(3.0, 1), StarFill::Full);
        assert_eq!(star_fill_for(3.0, 2), StarFill::Full);
        assert_eq!(star_fill_for(3.0, 3), StarFill::Full);
        assert_eq!(star_fill_for(3.0, 4), StarFill::Empty);
        assert_eq!(star_fill_for(3.0, 5), StarFill::Empty);
    }

    #[test]
    fn rating_star_fill_half() {
        assert_eq!(star_fill_for(3.5, 1), StarFill::Full);
        assert_eq!(star_fill_for(3.5, 2), StarFill::Full);
        assert_eq!(star_fill_for(3.5, 3), StarFill::Full);
        assert_eq!(star_fill_for(3.5, 4), StarFill::Half);
        assert_eq!(star_fill_for(3.5, 5), StarFill::Empty);
    }

    #[test]
    fn star_fill_maps_to_sf_symbol_icon_names() {
        assert_eq!(StarFill::Full.icon_name(), IconName::StarFill);
        assert_eq!(StarFill::Half.icon_name(), IconName::StarLeadingHalfFilled);
        assert_eq!(StarFill::Empty.icon_name(), IconName::Star);
    }

    #[test]
    fn rating_interactive_and_on_change() {
        let r = RatingIndicator::new("test")
            .interactive(true)
            .on_change(|_, _, _| {});
        assert!(r.interactive);
        assert!(r.on_change.is_some());
    }

    #[test]
    fn default_accessibility_label_whole_value() {
        assert_eq!(default_accessibility_label(3.0, 5), "3 of 5 stars");
        assert_eq!(default_accessibility_label(0.0, 5), "0 of 5 stars");
        assert_eq!(default_accessibility_label(5.0, 5), "5 of 5 stars");
    }

    #[test]
    fn default_accessibility_label_fractional_value() {
        assert_eq!(default_accessibility_label(3.5, 5), "3.5 of 5 stars");
        assert_eq!(default_accessibility_label(4.25, 10), "4.2 of 10 stars");
    }
}
