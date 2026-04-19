//! HIG Progress Indicator — determinate or indeterminate progress.
//!
//! A stateless `RenderOnce` component that renders either a horizontal
//! progress bar at a given value (`Determinate`) or an animated 12-tick
//! radial spinner (`Indeterminate`), matching the two `NSProgressIndicator`
//! styles (`.bar` and `.spinning`).
//!
//! # HIG reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/progress-indicators>

use std::time::Duration;

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{AnimatedIcon, IconAnimation, IconName};
use crate::foundations::materials::{GlassSize, glass_surface};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{App, ElementId, Hsla, Pixels, SharedString, Window, div, px, relative};

/// HIG-aligned size variants. `Small` matches an 8 pt spinner / 2 pt bar
/// (`NSProgressIndicator.controlSize = .small`); `Regular` matches a
/// 20 pt spinner / 4 pt bar (`NSProgressIndicator` regular default).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProgressIndicatorSize {
    /// Small: 8 pt spinner / 2 pt bar.
    Small,
    /// Regular: 20 pt spinner / 4 pt bar. Matches `NSProgressIndicator` default.
    #[default]
    Regular,
}

impl ProgressIndicatorSize {
    fn bar_height(self) -> Pixels {
        match self {
            ProgressIndicatorSize::Small => px(2.0),
            ProgressIndicatorSize::Regular => px(4.0),
        }
    }

    fn spinner_diameter(self) -> Pixels {
        match self {
            ProgressIndicatorSize::Small => px(8.0),
            ProgressIndicatorSize::Regular => px(20.0),
        }
    }
}

/// Value mode for the progress indicator.
///
/// `Determinate(f32)` renders a horizontal bar filled to the given
/// fraction (0.0..=1.0). `Indeterminate` renders an animated 12-tick
/// radial spinner for tasks of unknown duration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressIndicatorValue {
    /// Bounded progress in `0.0..=1.0`.
    Determinate(f32),
    /// Unbounded / unknown duration — renders a spinning spinner.
    Indeterminate,
}

/// A progress indicator (bar or spinner).
#[derive(IntoElement)]
pub struct ProgressIndicator {
    id: Option<ElementId>,
    value: ProgressIndicatorValue,
    hig_size: ProgressIndicatorSize,
    height: Option<Pixels>,
    color: Option<Hsla>,
    track_color: Option<Hsla>,
    label: Option<SharedString>,
    glass: bool,
}

impl ProgressIndicator {
    /// Create a *determinate* progress bar with the given value (clamped to
    /// `0.0..=1.0`). Non-finite values (`NaN`, `±∞`) become `0.0`.
    pub fn new(value: f32) -> Self {
        let safe = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            id: None,
            value: ProgressIndicatorValue::Determinate(safe),
            hig_size: ProgressIndicatorSize::Regular,
            height: None,
            color: None,
            track_color: None,
            label: None,
            glass: false,
        }
    }

    /// Create an *indeterminate* progress spinner. The `id` is required so
    /// GPUI can drive the continuous rotation animation.
    pub fn indeterminate(id: impl Into<ElementId>) -> Self {
        Self {
            id: Some(id.into()),
            value: ProgressIndicatorValue::Indeterminate,
            hig_size: ProgressIndicatorSize::Regular,
            height: None,
            color: None,
            track_color: None,
            label: None,
            glass: false,
        }
    }

    /// Explicit HIG size variant (default: `Regular`).
    pub fn size(mut self, size: ProgressIndicatorSize) -> Self {
        self.hig_size = size;
        self
    }

    /// Override bar height in points. Ignored for indeterminate spinners.
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn track_color(mut self, color: Hsla) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Optional caption text shown below the bar / spinner.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// When `true` and the current theme is a Liquid Glass theme, wrap
    /// the indicator in a glass surface so it adopts the glass chrome
    /// used by macOS 26 Tahoe system indicators.
    pub fn glass(mut self, glass: bool) -> Self {
        self.glass = glass;
        self
    }
}

impl RenderOnce for ProgressIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let color = self.color.unwrap_or(theme.accent);
        let glass_enabled = self.glass;

        let a11y_label: SharedString = match self.value {
            ProgressIndicatorValue::Determinate(v) => SharedString::from(format!(
                "Progress, {}%",
                (v * 100.0).round() as i32
            )),
            ProgressIndicatorValue::Indeterminate => SharedString::from("Loading"),
        };
        let mut a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::ProgressIndicator);
        if let ProgressIndicatorValue::Determinate(v) = self.value {
            a11y_props = a11y_props.value(SharedString::from(format!(
                "{} percent",
                (v * 100.0).round() as i32
            )));
        }

        let inner: gpui::AnyElement = match self.value {
            ProgressIndicatorValue::Determinate(v) => {
                render_bar(
                    v,
                    self.hig_size,
                    self.height,
                    color,
                    self.track_color.unwrap_or(theme.semantic.system_fill),
                    theme.is_rtl(),
                )
                .into_any_element()
            }
            ProgressIndicatorValue::Indeterminate => {
                let id = self.id.clone().unwrap_or_else(|| "progress-spinner".into());
                render_spinner(id, self.hig_size, color).into_any_element()
            }
        };

        let mut container = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(theme.spacing_xs)
            .child(inner)
            .with_accessibility(&a11y_props);

        if let Some(label) = self.label {
            container = container.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(label),
            );
        }

        if glass_enabled {
            glass_surface(
                div().p(theme.spacing_sm).child(container),
                theme,
                GlassSize::Small,
            )
            .into_any_element()
        } else {
            container.into_any_element()
        }
    }
}

fn render_bar(
    value: f32,
    hig_size: ProgressIndicatorSize,
    height_override: Option<Pixels>,
    color: Hsla,
    track: Hsla,
    is_rtl: bool,
) -> impl IntoElement {
    let h = height_override.unwrap_or_else(|| hig_size.bar_height());
    // HIG: `NSProgressIndicator` uses a 2 pt corner radius on the
    // determinate bar track — never a full pill.
    let natural_radius = px(2.0f32.min(f32::from(h) / 2.0));

    let mut track_el = div()
        .w_full()
        .h(h)
        .overflow_hidden()
        .rounded(natural_radius)
        .bg(track)
        .flex();
    if is_rtl {
        track_el = track_el.flex_row_reverse();
    } else {
        track_el = track_el.flex_row();
    }

    track_el.child(
        div()
            .h_full()
            .rounded(natural_radius)
            .bg(color)
            .w(relative(value)),
    )
}

fn render_spinner(
    id: ElementId,
    hig_size: ProgressIndicatorSize,
    color: Hsla,
) -> impl IntoElement {
    // HIG: the macOS indeterminate spinner is a 12-tick radial
    // `NSProgressIndicator.style = .spinning`. `AnimatedIcon::Spin` handles
    // the rotation and the REDUCE_MOTION fallback to a static render.
    AnimatedIcon::new(
        id,
        IconName::ProgressSpinner,
        IconAnimation::Spin {
            duration: Duration::from_millis(1200),
        },
    )
    .size(hig_size.spinner_diameter())
    .color(color)
}

#[cfg(test)]
mod tests {
    use crate::components::status::progress_indicator::{
        ProgressIndicator, ProgressIndicatorSize, ProgressIndicatorValue,
    };
    use core::prelude::v1::test;
    use gpui::{hsla, px};

    #[test]
    fn value_clamped_to_0_1() {
        let bar = ProgressIndicator::new(-0.5);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON));

        let bar = ProgressIndicator::new(1.5);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 1.0).abs() < f32::EPSILON));

        let bar = ProgressIndicator::new(0.5);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.5).abs() < f32::EPSILON));
    }

    #[test]
    fn new_defaults_to_determinate_regular() {
        let bar = ProgressIndicator::new(0.5);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(_)));
        assert_eq!(bar.hig_size, ProgressIndicatorSize::Regular);
        assert!(bar.height.is_none());
        assert!(bar.label.is_none());
        assert!(!bar.glass);
    }

    #[test]
    fn indeterminate_constructor_stores_id() {
        let bar = ProgressIndicator::indeterminate("spinner");
        assert!(matches!(bar.value, ProgressIndicatorValue::Indeterminate));
        assert!(bar.id.is_some());
    }

    #[test]
    fn color_builder() {
        let color = hsla(0.5, 1.0, 0.5, 1.0);
        let bar = ProgressIndicator::new(0.5).color(color);
        assert_eq!(bar.color, Some(color));
    }

    #[test]
    fn track_color_builder() {
        let color = hsla(0.0, 0.0, 0.8, 1.0);
        let bar = ProgressIndicator::new(0.5).track_color(color);
        assert_eq!(bar.track_color, Some(color));
    }

    #[test]
    fn nan_value_treated_as_zero() {
        let bar = ProgressIndicator::new(f32::NAN);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON));
    }

    #[test]
    fn infinity_value_treated_as_zero() {
        let bar = ProgressIndicator::new(f32::INFINITY);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON));

        let bar = ProgressIndicator::new(f32::NEG_INFINITY);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON));
    }

    #[test]
    fn label_builder() {
        let bar = ProgressIndicator::new(0.25).label("Downloading…");
        assert_eq!(bar.label.as_ref().map(|s| s.as_ref()), Some("Downloading…"));
    }

    #[test]
    fn size_builder_selects_small() {
        let bar = ProgressIndicator::new(0.5).size(ProgressIndicatorSize::Small);
        assert_eq!(bar.hig_size, ProgressIndicatorSize::Small);
    }

    #[test]
    fn bar_height_defaults_match_hig() {
        // HIG: NSProgressIndicator regular determinate bar = 4pt, small = 2pt.
        assert_eq!(ProgressIndicatorSize::Regular.bar_height(), px(4.0));
        assert_eq!(ProgressIndicatorSize::Small.bar_height(), px(2.0));
    }

    #[test]
    fn spinner_diameter_defaults_match_hig() {
        // HIG: NSProgressIndicator regular spinner = 20pt, small = 8pt.
        assert_eq!(ProgressIndicatorSize::Regular.spinner_diameter(), px(20.0));
        assert_eq!(ProgressIndicatorSize::Small.spinner_diameter(), px(8.0));
    }

    #[test]
    fn explicit_height_override_wins() {
        let bar = ProgressIndicator::new(0.5).height(px(9.0));
        assert_eq!(bar.height, Some(px(9.0)));
    }

    #[test]
    fn glass_builder_opt_in() {
        let bar = ProgressIndicator::new(0.5);
        assert!(!bar.glass);
        let bar = bar.glass(true);
        assert!(bar.glass);
    }
}
