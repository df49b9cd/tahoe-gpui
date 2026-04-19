//! HIG Gauge — visual meter / level indicator.
//!
//! A stateless `RenderOnce` component that renders a value in a bounded
//! range as one of the HIG-defined gauge styles:
//!
//! * `Linear` — horizontal fill bar (macOS `NSLevelIndicator.Style.continuous`).
//! * `Compact` — taller horizontal bar for dashboard-style readouts.
//! * `Circular` — arc dial (SwiftUI `Gauge(value:).gaugeStyle(.accessoryCircular)`).
//! * `Discrete { segments }` — segmented bar (`NSLevelIndicator.Style.discreteCapacity`).
//! * `Tiered` — multi-color fill bands (`NSLevelIndicator.Style.ratingLevel`).
//! * `Relevance` — low-contrast shaded bar for search-result relevance.
//!
//! # HIG reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/gauges>

use std::f32::consts::PI;

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme, TahoeTheme};
use gpui::prelude::*;
use gpui::{
    App, Bounds, Hsla, SharedString, Window, canvas, div, fill, hsla, point, px, relative, size,
};

/// Visual style for the gauge.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum GaugeStyle {
    /// Horizontal bar with filled portion — default macOS
    /// `NSLevelIndicator.continuous`.
    #[default]
    Linear,
    /// Compact horizontal bar with increased height (dashboard-style).
    Compact,
    /// Circular arc dial — HIG Gauge “circular” style / SwiftUI
    /// `.accessoryCircular`.
    Circular,
    /// Segmented bar of equal-width cells — macOS
    /// `NSLevelIndicator.discreteCapacity`.
    Discrete {
        /// Total number of segments; must be >= 1.
        segments: usize,
    },
    /// Tiered capacity bar — like `Discrete` but fills each segment in the
    /// configured level color rather than a single fill, mirroring
    /// `NSLevelIndicator.ratingLevel`.
    Tiered,
    /// Relevance bar — low-contrast horizontal band used for search-result
    /// relevance (`NSLevelIndicator.relevancy`).
    Relevance,
}

/// Semantic direction of the gauge: does *high* or *low* represent a good
/// state? Drives the traffic-light color-threshold logic so that e.g.
/// battery % (high = good) and memory pressure (low = good) use the same
/// gauge without color inversion hacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GaugeDirection {
    /// Low values are good — e.g. battery drain, CPU load. This is the
    /// historical default (thresholds: 0.0..0.33 green, 0.33..0.66 yellow,
    /// 0.66..1.0 red).
    #[default]
    LowIsGood,
    /// High values are good — e.g. battery charge, signal strength.
    /// Thresholds are inverted (0.0..0.33 red, 0.33..0.66 yellow,
    /// 0.66..1.0 green).
    HighIsGood,
}

/// A gauge (level indicator) component.
#[derive(IntoElement)]
pub struct Gauge {
    value: f32,
    label: Option<SharedString>,
    min_label: Option<SharedString>,
    max_label: Option<SharedString>,
    value_label: Option<SharedString>,
    color: Option<Hsla>,
    style: GaugeStyle,
    direction: GaugeDirection,
}

impl Gauge {
    pub fn new(value: f32) -> Self {
        Self {
            value: if value.is_finite() {
                value.clamp(0.0, 1.0)
            } else {
                0.0
            },
            label: None,
            min_label: None,
            max_label: None,
            value_label: None,
            color: None,
            style: GaugeStyle::Linear,
            direction: GaugeDirection::LowIsGood,
        }
    }

    pub fn label(mut self, text: impl Into<SharedString>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Leading-end range label (e.g. `"0"` or `"Empty"`).
    pub fn min_label(mut self, text: impl Into<SharedString>) -> Self {
        self.min_label = Some(text.into());
        self
    }

    /// Trailing-end range label (e.g. `"100"` or `"Full"`).
    pub fn max_label(mut self, text: impl Into<SharedString>) -> Self {
        self.max_label = Some(text.into());
        self
    }

    /// Current-value readout (e.g. `"65%"`) rendered alongside the gauge.
    pub fn value_label(mut self, text: impl Into<SharedString>) -> Self {
        self.value_label = Some(text.into());
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn style(mut self, style: GaugeStyle) -> Self {
        self.style = style;
        self
    }

    pub fn direction(mut self, direction: GaugeDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Returns the level color based on the current value and direction.
    fn level_color(&self, theme: &TahoeTheme) -> Hsla {
        if let Some(color) = self.color {
            return color;
        }
        match self.direction {
            GaugeDirection::LowIsGood => {
                if self.value < 0.33 {
                    theme.success
                } else if self.value < 0.66 {
                    theme.warning
                } else {
                    theme.error
                }
            }
            GaugeDirection::HighIsGood => {
                if self.value < 0.33 {
                    theme.error
                } else if self.value < 0.66 {
                    theme.warning
                } else {
                    theme.success
                }
            }
        }
    }
}

impl RenderOnce for Gauge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let fill_color = self.level_color(theme);
        let track_bg = theme.semantic.system_fill;

        let percent = (self.value * 100.0).round() as i32;
        let a11y_label: SharedString = self
            .label
            .clone()
            .unwrap_or_else(|| SharedString::from("Gauge"));
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::ProgressIndicator)
            .value(SharedString::from(format!("{percent} percent")));

        let gauge_body: gpui::AnyElement = match self.style {
            GaugeStyle::Linear => {
                render_linear(self.value, fill_color, track_bg, px(8.0), px(4.0)).into_any_element()
            }
            GaugeStyle::Compact => {
                render_linear(self.value, fill_color, track_bg, px(16.0), px(8.0))
                    .into_any_element()
            }
            GaugeStyle::Circular => {
                render_circular(self.value, fill_color, track_bg).into_any_element()
            }
            GaugeStyle::Discrete { segments } => {
                render_discrete(self.value, segments, fill_color, track_bg, theme)
                    .into_any_element()
            }
            GaugeStyle::Tiered => {
                // HIG `ratingLevel`: same segment grid as Discrete but fills
                // each segment in the current level color (no per-cell fade).
                render_discrete(self.value, 5, fill_color, track_bg, theme).into_any_element()
            }
            GaugeStyle::Relevance => {
                // HIG relevancy bars render as a short, low-contrast line
                // segment instead of a filled bar; we dim the fill to match.
                let dimmed = hsla(fill_color.h, fill_color.s, fill_color.l, 0.6);
                render_linear(self.value, dimmed, track_bg, px(3.0), px(1.5)).into_any_element()
            }
        };

        let mut container = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_xs)
            .with_accessibility(&a11y_props);

        // Range labels row: leading = min, trailing = max. Only rendered
        // when either is provided.
        if self.min_label.is_some() || self.max_label.is_some() || self.value_label.is_some() {
            let caption = crate::foundations::theme::TextStyle::Caption1.attrs();
            let mut row = div()
                .flex()
                .w_full()
                .items_center()
                .justify_between()
                .text_size(caption.size)
                .font_weight(theme.effective_weight(caption.weight))
                .text_color(theme.text_muted);
            if let Some(min) = self.min_label.clone() {
                row = row.child(div().child(min));
            } else {
                row = row.child(div());
            }
            if let Some(val) = self.value_label.clone() {
                row = row.child(div().child(val));
            }
            if let Some(maxv) = self.max_label.clone() {
                row = row.child(div().child(maxv));
            } else {
                row = row.child(div());
            }
            container = container.child(row);
        }

        container = container.child(gauge_body);

        if let Some(label_text) = self.label {
            let caption = crate::foundations::theme::TextStyle::Caption1.attrs();
            container = container.child(
                div()
                    .text_size(caption.size)
                    .font_weight(theme.effective_weight(caption.weight))
                    .text_color(theme.text_muted)
                    .child(label_text),
            );
        }

        container
    }
}

fn render_linear(
    value: f32,
    fill_color: Hsla,
    track_bg: Hsla,
    track_height: gpui::Pixels,
    corner_radius: gpui::Pixels,
) -> impl IntoElement {
    div()
        .w_full()
        .h(track_height)
        .overflow_hidden()
        .rounded(corner_radius)
        .bg(track_bg)
        .child(
            div()
                .h_full()
                .bg(fill_color)
                .w(relative(value.clamp(0.0, 1.0))),
        )
}

fn render_discrete(
    value: f32,
    segments: usize,
    fill_color: Hsla,
    track_bg: Hsla,
    theme: &TahoeTheme,
) -> impl IntoElement {
    let segments = segments.max(1);
    let filled = (value.clamp(0.0, 1.0) * segments as f32).round() as usize;
    let mut row = div()
        .flex()
        .flex_row()
        .gap(theme.spacing_xs)
        .w_full()
        .h(px(8.0));
    for i in 0..segments {
        let bg = if i < filled { fill_color } else { track_bg };
        row = row.child(div().h_full().flex_1().rounded(px(2.0)).bg(bg));
    }
    row
}

fn render_circular(value: f32, fill_color: Hsla, track_bg: Hsla) -> impl IntoElement {
    // SwiftUI `.gaugeStyle(.accessoryCircular)` renders a ~280° arc dial —
    // we draw the track as a full circle border and paint an arc of dots
    // through the canvas API (matching `ActivityRing`'s technique).
    let display_size = px(48.0);
    let stroke = 4.0f32;
    let proportion = value.clamp(0.0, 1.0);

    canvas(
        move |_bounds, _window, _cx| {},
        move |bounds, _, window, _cx| {
            let s = f32::from(display_size);
            let cx_f = f32::from(bounds.origin.x) + s / 2.0;
            let cy_f = f32::from(bounds.origin.y) + s / 2.0;
            let radius = (s - stroke) / 2.0;

            let track_bounds = Bounds {
                origin: point(
                    px(cx_f - radius - stroke / 2.0),
                    px(cy_f - radius - stroke / 2.0),
                ),
                size: size(px(radius * 2.0 + stroke), px(radius * 2.0 + stroke)),
            };
            window.paint_quad(
                fill(
                    track_bounds,
                    Hsla {
                        a: 0.0,
                        ..fill_color
                    },
                )
                .corner_radii(px(s / 2.0))
                .border_widths(px(stroke))
                .border_color(track_bg),
            );

            if proportion > 0.0 {
                let total_angle = proportion * 2.0 * PI;
                let start_angle = -PI / 2.0; // top of circle
                let dot_size = stroke;
                let segments = ((64.0 * proportion).ceil() as usize).max(1);
                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let angle = start_angle + t * total_angle;
                    let qx = cx_f + radius * angle.cos();
                    let qy = cy_f + radius * angle.sin();
                    let dot_bounds = Bounds {
                        origin: point(px(qx - dot_size / 2.0), px(qy - dot_size / 2.0)),
                        size: size(px(dot_size), px(dot_size)),
                    };
                    window
                        .paint_quad(fill(dot_bounds, fill_color).corner_radii(px(dot_size / 2.0)));
                }
            }
        },
    )
    .size(display_size)
}

#[cfg(test)]
mod tests {
    use crate::components::status::gauge::{Gauge, GaugeDirection, GaugeStyle};
    use core::prelude::v1::test;
    use gpui::hsla;

    #[test]
    fn gauge_defaults() {
        let g = Gauge::new(0.5);
        assert!((g.value - 0.5).abs() < f32::EPSILON);
        assert!(g.label.is_none());
        assert!(g.min_label.is_none());
        assert!(g.max_label.is_none());
        assert!(g.value_label.is_none());
        assert!(g.color.is_none());
        assert_eq!(g.style, GaugeStyle::Linear);
        assert_eq!(g.direction, GaugeDirection::LowIsGood);
    }

    #[test]
    fn gauge_value_clamped() {
        let g = Gauge::new(1.5);
        assert!((g.value - 1.0).abs() < f32::EPSILON);

        let g = Gauge::new(-0.5);
        assert!((g.value - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gauge_label_builder() {
        let g = Gauge::new(0.3).label("CPU");
        assert_eq!(g.label.as_ref().map(|s| s.as_ref()), Some("CPU"));
    }

    #[test]
    fn gauge_range_label_builders() {
        let g = Gauge::new(0.5)
            .min_label("0")
            .max_label("100")
            .value_label("50%");
        assert_eq!(g.min_label.as_ref().map(|s| s.as_ref()), Some("0"));
        assert_eq!(g.max_label.as_ref().map(|s| s.as_ref()), Some("100"));
        assert_eq!(g.value_label.as_ref().map(|s| s.as_ref()), Some("50%"));
    }

    #[test]
    fn gauge_color_builder() {
        let custom = hsla(0.5, 0.8, 0.6, 1.0);
        let g = Gauge::new(0.5).color(custom);
        assert!(g.color.is_some());
    }

    #[test]
    fn gauge_style_variants() {
        let g = Gauge::new(0.5).style(GaugeStyle::Compact);
        assert_eq!(g.style, GaugeStyle::Compact);
        let g = Gauge::new(0.5).style(GaugeStyle::Circular);
        assert_eq!(g.style, GaugeStyle::Circular);
        let g = Gauge::new(0.5).style(GaugeStyle::Discrete { segments: 5 });
        assert!(matches!(g.style, GaugeStyle::Discrete { segments: 5 }));
        let g = Gauge::new(0.5).style(GaugeStyle::Tiered);
        assert_eq!(g.style, GaugeStyle::Tiered);
        let g = Gauge::new(0.5).style(GaugeStyle::Relevance);
        assert_eq!(g.style, GaugeStyle::Relevance);
    }

    #[test]
    fn gauge_direction_defaults_to_low_is_good() {
        assert_eq!(GaugeDirection::default(), GaugeDirection::LowIsGood);
    }

    #[test]
    fn gauge_direction_builder_switches() {
        let g = Gauge::new(0.5).direction(GaugeDirection::HighIsGood);
        assert_eq!(g.direction, GaugeDirection::HighIsGood);
    }

    #[test]
    fn gauge_nan_defaults_to_zero() {
        let g = Gauge::new(f32::NAN);
        assert!((g.value - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn gauge_infinity_defaults_to_zero() {
        let g = Gauge::new(f32::INFINITY);
        assert!((g.value - 0.0).abs() < f32::EPSILON);

        let g = Gauge::new(f32::NEG_INFINITY);
        assert!((g.value - 0.0).abs() < f32::EPSILON);
    }
}
