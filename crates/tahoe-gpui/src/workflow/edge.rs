//! Edge element for workflow graphs.
//!
//! Renders a bezier curve between two points using the GPUI `canvas()` element.
//! Supports three styles: Solid, Dashed (temporary/preview), and Animated
//! (moving circle indicator). Uses `paint_quad` to draw small filled rectangles
//! along the curve path, since GPUI's `Path` API produces filled shapes rather
//! than stroked lines.

use super::util::{
    HandlePosition, compute_bezier_control_points, compute_simple_control_points, cubic_bezier,
};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    App, Bounds, Hsla, IntoElement, SharedString, Window, canvas, div, fill, point, px, size,
};

/// Visual style for an edge.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum EdgeStyle {
    /// Solid line (default).
    #[default]
    Solid,
    /// Dashed line for temporary or preview connections.
    Dashed,
    /// Solid line with an animated circle moving along the path.
    Animated,
}

/// A visual edge drawn as a bezier curve between two points.
#[derive(IntoElement)]
pub struct EdgeElement {
    from: (f32, f32),
    to: (f32, f32),
    color: Option<Hsla>,
    selected: bool,
    /// F20 (#149): edge stroke thickens when hovered so a thin default line
    /// is easier to target before a click. Set by the canvas when the
    /// pointer hit-test resolves to this edge.
    hovered: bool,
    stroke_width: f32,
    style: EdgeStyle,
    /// Animation parameter (0.0..1.0) for the animated circle position.
    anim_t: f32,
    /// Whether to draw a small circle at the target endpoint.
    show_target_indicator: bool,
    /// Handle exit direction at the source node.
    source_position: HandlePosition,
    /// Handle exit direction at the target node.
    target_position: HandlePosition,
    /// Optional label text (F-OQ4 from #149). Rendered as a small pill
    /// centred on the curve. `None` when the connection has no label or
    /// when the caller explicitly hides labels at low zoom.
    label: Option<String>,
}

impl EdgeElement {
    /// Create a new edge from point `from` to point `to`.
    pub fn new(from: (f32, f32), to: (f32, f32)) -> Self {
        Self {
            from,
            to,
            color: None,
            selected: false,
            hovered: false,
            stroke_width: 1.0,
            style: EdgeStyle::default(),
            anim_t: 0.0,
            show_target_indicator: false,
            source_position: HandlePosition::Right,
            target_position: HandlePosition::Left,
            label: None,
        }
    }

    /// Flag the edge as hovered so the renderer draws a slightly thicker
    /// stroke — matches the HIG guidance that points-of-interest
    /// enlarge on hover to help the pointer land (F20).
    pub fn hovered(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }

    /// Attach a label that will render as a pill centred along the curve.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Override the edge color.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Mark the edge as selected (thicker, accent-colored).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set the stroke width.
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    /// Set the edge style (Solid, Dashed, or Animated).
    pub fn style(mut self, style: EdgeStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the animation time (0.0..1.0) for animated edges.
    pub fn animation_t(mut self, t: f32) -> Self {
        self.anim_t = t;
        self
    }

    /// Show a small circle indicator at the target endpoint.
    pub fn target_indicator(mut self, show: bool) -> Self {
        self.show_target_indicator = show;
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

impl RenderOnce for EdgeElement {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();

        let default_color = if self.style == EdgeStyle::Dashed {
            theme.border
        } else {
            theme.text_muted
        };
        let edge_color = if self.selected {
            theme.accent
        } else {
            self.color.unwrap_or(default_color)
        };
        let accent = theme.accent;
        let indicator_fill = theme.text_on_accent;

        let stroke = if self.selected {
            self.stroke_width + 1.0
        } else if self.hovered {
            // Hover adds half a point — perceptible but still lighter than
            // the selected stroke so hover never reads as selection.
            self.stroke_width + 0.5
        } else {
            self.stroke_width
        };

        let from = self.from;
        let to = self.to;
        let style = self.style;
        let anim_t = self.anim_t;
        let show_target = self.show_target_indicator;
        let source_pos = self.source_position;
        let target_pos = self.target_position;
        let is_dashed = matches!(style, EdgeStyle::Dashed);

        // Compute control points before the canvas to include them in the bounding box.
        let (bezier_ctrl1, bezier_ctrl2) = if is_dashed {
            compute_simple_control_points(from, to, source_pos, target_pos)
        } else {
            compute_bezier_control_points(from, to, source_pos, target_pos)
        };

        // Calculate bounding box including control points
        let min_x = from.0.min(to.0).min(bezier_ctrl1.0).min(bezier_ctrl2.0) - 20.0;
        let min_y = from.1.min(to.1).min(bezier_ctrl1.1).min(bezier_ctrl2.1) - 20.0;
        let max_x = from.0.max(to.0).max(bezier_ctrl1.0).max(bezier_ctrl2.0) + 20.0;
        let max_y = from.1.max(to.1).max(bezier_ctrl1.1).max(bezier_ctrl2.1) + 20.0;
        let width = max_x - min_x;
        let height = max_y - min_y;

        let label_text = self.label.clone();
        let label_anchor = {
            // Midpoint of the cubic bezier at t=0.5 — a stable visual
            // centre irrespective of the control-point layout.
            let p = cubic_bezier(from, bezier_ctrl1, bezier_ctrl2, to, 0.5);
            (p.0, p.1)
        };

        let curve = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let segments = 48u32;
                let ctrl1 = bezier_ctrl1;
                let ctrl2 = bezier_ctrl2;

                let eval = |t: f32| -> (f32, f32) { cubic_bezier(from, ctrl1, ctrl2, to, t) };

                let offset_x = f32::from(bounds.origin.x) - min_x;
                let offset_y = f32::from(bounds.origin.y) - min_y;

                // Pre-compute segment points and cumulative arc lengths
                let mut points: Vec<(f32, f32)> = Vec::with_capacity(segments as usize + 1);
                let mut arc_lengths = Vec::with_capacity(segments as usize + 1);
                let mut cumulative = 0.0_f32;
                arc_lengths.push(0.0);

                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let p = eval(t);
                    if i > 0 {
                        let prev: (f32, f32) = points[i as usize - 1];
                        let dx = p.0 - prev.0;
                        let dy = p.1 - prev.1;
                        cumulative += (dx * dx + dy * dy).sqrt();
                        arc_lengths.push(cumulative);
                    }
                    points.push(p);
                }

                // Dash pattern: 5px on, 5px off (matching upstream strokeDasharray "5, 5")
                let dash_len = 5.0;
                let pattern_len = dash_len * 2.0;

                for i in 0..segments as usize {
                    // For dashed style, skip segments in the "gap" phase
                    if is_dashed {
                        let mid_arc = (arc_lengths[i] + arc_lengths[i + 1]) / 2.0;
                        if mid_arc % pattern_len >= dash_len {
                            continue;
                        }
                    }

                    let p0 = points[i];
                    let p1 = points[i + 1];

                    let cx_pos = (p0.0 + p1.0) / 2.0 + offset_x;
                    let cy_pos = (p0.1 + p1.1) / 2.0 + offset_y;
                    let dx = p1.0 - p0.0;
                    let dy = p1.1 - p0.1;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);

                    let seg_bounds = Bounds {
                        origin: point(px(cx_pos - len / 2.0), px(cy_pos - stroke / 2.0)),
                        size: size(px(len), px(stroke)),
                    };

                    window.paint_quad(fill(seg_bounds, edge_color));
                }

                // Animated circle moving along the path
                if matches!(style, EdgeStyle::Animated) {
                    let circle_pos = eval(anim_t);
                    let r = 4.0;
                    let circle_bounds = Bounds {
                        origin: point(
                            px(circle_pos.0 + offset_x - r),
                            px(circle_pos.1 + offset_y - r),
                        ),
                        size: size(px(r * 2.0), px(r * 2.0)),
                    };
                    window.paint_quad(fill(circle_bounds, accent).corner_radii(px(r)));
                }

                // Target endpoint indicator (white fill with colored border ring)
                if show_target {
                    let r = 3.0;
                    let border_w = 1.0;
                    // Outer ring (border)
                    let outer = Bounds {
                        origin: point(
                            px(to.0 + offset_x - r - border_w),
                            px(to.1 + offset_y - r - border_w),
                        ),
                        size: size(px((r + border_w) * 2.0), px((r + border_w) * 2.0)),
                    };
                    window.paint_quad(fill(outer, edge_color).corner_radii(px(r + border_w)));
                    // Inner fill (white)
                    let inner = Bounds {
                        origin: point(px(to.0 + offset_x - r), px(to.1 + offset_y - r)),
                        size: size(px(r * 2.0), px(r * 2.0)),
                    };
                    window.paint_quad(fill(inner, indicator_fill).corner_radii(px(r)));
                }
            },
        )
        .absolute()
        .left(px(min_x))
        .top(px(min_y))
        .w(px(width))
        .h(px(height));

        // If there's a label, stack the canvas and a centred text pill
        // inside a common container. The pill floats over the curve at
        // t=0.5 with a subtle surface background so it remains readable
        // against the canvas grid. OQ4 from #149.
        if let Some(text) = label_text {
            let pill = div()
                .absolute()
                // Centre the pill on the midpoint. We can't measure the
                // pill's width here, so use a small negative offset and
                // let the pill size itself around the text.
                .left(px(label_anchor.0 - 16.0))
                .top(px(label_anchor.1 - 8.0))
                .px(theme.spacing_xs)
                .py(px(1.0))
                .bg(theme.surface)
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_sm)
                .text_style(TextStyle::Caption2, &theme)
                .text_color(theme.text_muted)
                .child(SharedString::from(text));
            div()
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full()
                .child(curve.into_any_element())
                .child(pill)
                .into_any_element()
        } else {
            curve.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EdgeElement, HandlePosition};
    use core::prelude::v1::test;

    #[test]
    fn handle_position_default_is_right() {
        assert_eq!(HandlePosition::default(), HandlePosition::Right);
    }

    #[test]
    fn edge_element_default_handle_positions() {
        let edge = EdgeElement::new((0.0, 0.0), (100.0, 100.0));
        assert_eq!(edge.source_position, HandlePosition::Right);
        assert_eq!(edge.target_position, HandlePosition::Left);
    }

    #[test]
    fn edge_element_builder_sets_positions() {
        let edge = EdgeElement::new((0.0, 0.0), (100.0, 100.0))
            .source_position(HandlePosition::Top)
            .target_position(HandlePosition::Bottom);
        assert_eq!(edge.source_position, HandlePosition::Top);
        assert_eq!(edge.target_position, HandlePosition::Bottom);
    }
}
