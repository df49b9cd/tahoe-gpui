//! Connection line component for workflow graphs.
//!
//! Renders an animated bezier curve preview while the user drags from a port
//! handle to create a new connection. Shows a circle indicator at the target
//! (mouse) position.

use super::util::{HandlePosition, compute_bezier_control_points, cubic_bezier};
use crate::foundations::theme::{ActiveTheme};
use gpui::prelude::*;
use gpui::{App, Bounds, Hsla, Window, canvas, fill, point, px, size};

/// A visual connection preview line drawn as a bezier curve with a target indicator.
///
/// Used during drag-to-connect interactions to show the pending connection
/// from a source port to the current mouse position.
#[derive(IntoElement)]
pub struct ConnectionLine {
    from: (f32, f32),
    to: (f32, f32),
    color: Option<Hsla>,
    stroke_width: f32,
    indicator_radius: f32,
    source_position: HandlePosition,
    target_position: HandlePosition,
}

impl ConnectionLine {
    /// Create a new connection line from a source position to a target position.
    pub fn new(from: (f32, f32), to: (f32, f32)) -> Self {
        Self {
            from,
            to,
            color: None,
            stroke_width: 1.0,
            indicator_radius: 3.0,
            source_position: HandlePosition::Right,
            target_position: HandlePosition::Left,
        }
    }

    /// Override the line color (defaults to `theme.ring`).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the stroke width (default: 1.0).
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Set the target indicator circle radius (default: 3.0).
    pub fn indicator_radius(mut self, radius: f32) -> Self {
        self.indicator_radius = radius;
        self
    }

    /// Set the handle exit direction at the source node.
    pub fn source_position(mut self, pos: HandlePosition) -> Self {
        self.source_position = pos;
        self
    }

    /// Set the handle exit direction at the target node.
    pub fn target_position(mut self, pos: HandlePosition) -> Self {
        self.target_position = pos;
        self
    }
}

impl RenderOnce for ConnectionLine {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let line_color = self.color.unwrap_or(theme.ring);
        let indicator_fill = theme.text_on_accent;
        let stroke = self.stroke_width;
        let indicator_r = self.indicator_radius;
        let from = self.from;
        let to = self.to;
        let source_pos = self.source_position;
        let target_pos = self.target_position;

        // Compute control points before the canvas to include them in the bounding box.
        let (bezier_ctrl1, bezier_ctrl2) =
            compute_bezier_control_points(from, to, source_pos, target_pos);

        // Bounding box with padding, including control points
        let padding = indicator_r + 4.0;
        let min_x = from.0.min(to.0).min(bezier_ctrl1.0).min(bezier_ctrl2.0) - padding;
        let min_y = from.1.min(to.1).min(bezier_ctrl1.1).min(bezier_ctrl2.1) - padding;
        let max_x = from.0.max(to.0).max(bezier_ctrl1.0).max(bezier_ctrl2.0) + padding;
        let max_y = from.1.max(to.1).max(bezier_ctrl1.1).max(bezier_ctrl2.1) + padding;
        let width = max_x - min_x;
        let height = max_y - min_y;

        canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let segments = 48u32;
                let ctrl1 = bezier_ctrl1;
                let ctrl2 = bezier_ctrl2;

                let offset_x = f32::from(bounds.origin.x) - min_x;
                let offset_y = f32::from(bounds.origin.y) - min_y;

                // Draw bezier curve segments
                for i in 0..segments {
                    let t0 = i as f32 / segments as f32;
                    let t1 = (i + 1) as f32 / segments as f32;

                    let p0 = cubic_bezier(from, ctrl1, ctrl2, to, t0);
                    let p1 = cubic_bezier(from, ctrl1, ctrl2, to, t1);

                    let cx_pos = (p0.0 + p1.0) / 2.0 + offset_x;
                    let cy_pos = (p0.1 + p1.1) / 2.0 + offset_y;
                    let dx = p1.0 - p0.0;
                    let dy = p1.1 - p0.1;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);

                    let seg_bounds = Bounds {
                        origin: point(px(cx_pos - len / 2.0), px(cy_pos - stroke / 2.0)),
                        size: size(px(len), px(stroke)),
                    };

                    window.paint_quad(fill(seg_bounds, line_color));
                }

                // Circle indicator at target position (white fill + colored border)
                let border_w = 1.0;
                let outer = Bounds {
                    origin: point(
                        px(to.0 + offset_x - indicator_r - border_w),
                        px(to.1 + offset_y - indicator_r - border_w),
                    ),
                    size: size(
                        px((indicator_r + border_w) * 2.0),
                        px((indicator_r + border_w) * 2.0),
                    ),
                };
                window.paint_quad(fill(outer, line_color).corner_radii(px(indicator_r + border_w)));
                let inner = Bounds {
                    origin: point(
                        px(to.0 + offset_x - indicator_r),
                        px(to.1 + offset_y - indicator_r),
                    ),
                    size: size(px(indicator_r * 2.0), px(indicator_r * 2.0)),
                };
                window.paint_quad(fill(inner, indicator_fill).corner_radii(px(indicator_r)));
            },
        )
        .absolute()
        .left(px(min_x))
        .top(px(min_y))
        .w(px(width))
        .h(px(height))
    }
}

#[cfg(test)]
mod tests {
    use super::super::util::{HandlePosition, connection_control_points, cubic_bezier};
    use super::ConnectionLine;
    use core::prelude::v1::test;

    #[test]
    fn connection_line_default_values() {
        let line = ConnectionLine::new((0.0, 0.0), (100.0, 100.0));
        assert_eq!(line.from, (0.0, 0.0));
        assert_eq!(line.to, (100.0, 100.0));
        assert!(line.color.is_none());
        assert!((line.stroke_width - 1.0).abs() < f32::EPSILON);
        assert!((line.indicator_radius - 3.0).abs() < f32::EPSILON);
        assert_eq!(line.source_position, HandlePosition::Right);
        assert_eq!(line.target_position, HandlePosition::Left);
    }

    #[test]
    fn connection_line_builder_handle_positions() {
        let line = ConnectionLine::new((0.0, 0.0), (100.0, 100.0))
            .source_position(HandlePosition::Top)
            .target_position(HandlePosition::Bottom);
        assert_eq!(line.source_position, HandlePosition::Top);
        assert_eq!(line.target_position, HandlePosition::Bottom);
    }

    #[test]
    fn connection_line_builder_chain() {
        let color = crate::foundations::color::SystemColor::Teal
            .resolve(crate::foundations::color::Appearance::Dark);
        let line = ConnectionLine::new((10.0, 20.0), (200.0, 300.0))
            .color(color)
            .stroke_width(3.0)
            .indicator_radius(6.0);
        assert!(line.color.is_some());
        assert!((line.stroke_width - 3.0).abs() < f32::EPSILON);
        assert!((line.indicator_radius - 6.0).abs() < f32::EPSILON);
    }

    /// Snapshot test: verify control points against hardcoded expected values
    /// derived from the upstream AI SDK Elements Connection formula:
    /// `C (fromX + (toX-fromX)*0.5, fromY) (fromX + (toX-fromX)*0.5, toY) (toX, toY)`
    #[test]
    fn control_points_snapshot() {
        let (ctrl1, ctrl2) = connection_control_points((10.0, 20.0), (200.0, 300.0));
        // ctrl_x = (10 + 200) / 2 = 105
        assert!((ctrl1.0 - 105.0).abs() < f32::EPSILON);
        assert!((ctrl1.1 - 20.0).abs() < f32::EPSILON);
        assert!((ctrl2.0 - 105.0).abs() < f32::EPSILON);
        assert!((ctrl2.1 - 300.0).abs() < f32::EPSILON);
    }

    /// Verify the S-curve midpoint (t=0.5) for known inputs,
    /// exercising the actual control-point + bezier pipeline.
    #[test]
    fn curve_midpoint_for_known_inputs() {
        let from = (0.0_f32, 0.0_f32);
        let to = (100.0_f32, 200.0_f32);
        let (ctrl1, ctrl2) = connection_control_points(from, to);

        let mid = cubic_bezier(from, ctrl1, ctrl2, to, 0.5);
        // For a symmetric S-curve, the midpoint x = (from.x + to.x) / 2
        assert!((mid.0 - 50.0).abs() < 0.01);
        // Midpoint y = (from.y + to.y) / 2 for a symmetric cubic
        assert!((mid.1 - 100.0).abs() < 0.01);
    }
}
