//! Circular progress ring drawn via canvas.
//!
//! Matches the SVG progress ring from the TypeScript AI Elements `ContextTrigger`:
//! a 20×20 circular indicator with a background ring and a filled progress arc.

use std::f32::consts::PI;

use gpui::prelude::*;
use gpui::{App, Bounds, Hsla, Pixels, Window, canvas, div, fill, point, px, size};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::ActiveTheme;

/// Default display size matching the TS SVG (20×20).
const DEFAULT_SIZE: f32 = 20.0;

/// Stroke width matching the TS SVG `strokeWidth={2}`.
const STROKE_WIDTH: f32 = 2.0;

/// Number of segments used to approximate the progress arc.
const ARC_SEGMENTS: usize = 64;

/// A circular progress ring element drawn on a GPUI canvas.
///
/// Renders a background ring at low opacity and a progress arc
/// proportional to `percentage` (0.0–1.0), starting from the top (-90°).
///
/// Matches the appearance of the TypeScript `ContextIcon` SVG:
/// - Background circle: 0.25 opacity, full ring
/// - Progress arc: 0.7 opacity, partial ring
/// - Color: `currentColor` (defaults to `theme.text`)
#[derive(IntoElement)]
pub struct ContextRing {
    percentage: f32,
    display_size: Pixels,
    color: Option<Hsla>,
}

impl ContextRing {
    /// Creates a new ring with the given progress percentage (0.0–1.0).
    pub fn new(percentage: f32) -> Self {
        Self {
            percentage: percentage.clamp(0.0, 1.0),
            display_size: px(DEFAULT_SIZE),
            color: None,
        }
    }

    /// Sets the display size.
    pub fn size(mut self, size: Pixels) -> Self {
        self.display_size = size;
        self
    }

    /// Sets the color (defaults to `theme.text`).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl RenderOnce for ContextRing {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let color = self.color.unwrap_or(theme.text);
        let percentage = self.percentage;
        let display_size = self.display_size;

        // Canvas alone cannot carry accessibility metadata — VoiceOver
        // sees the percentage arc as an opaque rectangle. Wrap the
        // canvas in a `div` so AccessibleExt can attach the ProgressIndicator
        // role + a "N percent" value string. Today `AccessibleExt` is a
        // no-op (GPUI upstream gap, tracked in foundations/accessibility.rs);
        // the wrapper ensures the one-line wiring lands when GPUI ships
        // the AX API.
        let percent_int = (percentage * 100.0).round() as u32;
        let a11y = AccessibilityProps::new()
            .label(format!("Context usage {percent_int} percent"))
            .role(AccessibilityRole::ProgressIndicator)
            .value(format!("{percent_int} percent"));

        let ring_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let s = f32::from(display_size);
                let cx_f = f32::from(bounds.origin.x) + s / 2.0;
                let cy_f = f32::from(bounds.origin.y) + s / 2.0;
                let radius = (s - STROKE_WIDTH) / 2.0;

                // Background ring: full circle border at 0.25 opacity.
                let bg_color = Hsla { a: 0.25, ..color };
                let bg_bounds = Bounds {
                    origin: point(
                        px(cx_f - radius - STROKE_WIDTH / 2.0),
                        px(cy_f - radius - STROKE_WIDTH / 2.0),
                    ),
                    size: size(
                        px(radius * 2.0 + STROKE_WIDTH),
                        px(radius * 2.0 + STROKE_WIDTH),
                    ),
                };
                window.paint_quad(
                    fill(bg_bounds, Hsla { a: 0.0, ..color })
                        .corner_radii(px(s / 2.0))
                        .border_widths(px(STROKE_WIDTH))
                        .border_color(bg_color),
                );

                // Progress arc: small filled quads along the arc at 0.7 opacity.
                if percentage > 0.0 {
                    let arc_color = Hsla { a: 0.7, ..color };
                    let total_angle = percentage * 2.0 * PI;
                    let start_angle = -PI / 2.0; // top of circle
                    let quad_size = STROKE_WIDTH;

                    let segments = ((ARC_SEGMENTS as f32 * percentage).ceil() as usize).max(1);
                    for i in 0..segments {
                        let t = i as f32 / segments as f32;
                        let angle = start_angle + t * total_angle;
                        let qx = cx_f + radius * angle.cos();
                        let qy = cy_f + radius * angle.sin();

                        let q_bounds = Bounds {
                            origin: point(px(qx - quad_size / 2.0), px(qy - quad_size / 2.0)),
                            size: size(px(quad_size), px(quad_size)),
                        };
                        window.paint_quad(
                            fill(q_bounds, arc_color).corner_radii(px(quad_size / 2.0)),
                        );
                    }
                }
            },
        )
        .size(display_size);

        div()
            .size(display_size)
            .child(ring_canvas)
            .with_accessibility(&a11y)
    }
}

#[cfg(test)]
mod tests {
    use super::ContextRing;
    use core::prelude::v1::test;
    use gpui::{Hsla, px};

    #[test]
    fn percentage_clamped() {
        let ring = ContextRing::new(1.5);
        assert_eq!(ring.percentage, 1.0);

        let ring = ContextRing::new(-0.5);
        assert_eq!(ring.percentage, 0.0);
    }

    #[test]
    fn default_size() {
        let ring = ContextRing::new(0.5);
        assert_eq!(ring.display_size, px(20.0));
    }

    #[test]
    fn custom_size() {
        let ring = ContextRing::new(0.5).size(px(40.0));
        assert_eq!(ring.display_size, px(40.0));
    }

    #[test]
    fn custom_color() {
        let color = Hsla {
            h: 0.5,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        };
        let ring = ContextRing::new(0.5).color(color);
        assert_eq!(ring.color, Some(color));
    }
}
