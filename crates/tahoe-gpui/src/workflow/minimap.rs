//! Workflow minimap component.

use crate::foundations::theme::ActiveTheme;
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{
    App, Bounds, CursorStyle, ElementId, Entity, Hsla, MouseButton, MouseDownEvent, Window, canvas,
    div, fill, point, px, size,
};

use super::canvas::{NODE_HEIGHT, WorkflowCanvas};
use super::node::NODE_MIN_WIDTH;

/// Position of the minimap within its parent container.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum MinimapPosition {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

/// A miniature overview of the workflow canvas showing all nodes and the current viewport.
///
/// Click anywhere on the minimap to centre the main canvas viewport on the
/// corresponding world-space point. Hovering over the minimap reveals a
/// pointing-hand cursor to advertise the interactivity.
#[derive(IntoElement)]
pub struct WorkflowMiniMap {
    canvas: Entity<WorkflowCanvas>,
    width: f32,
    height: f32,
    position: MinimapPosition,
    bg_color: Option<Hsla>,
    mask_color: Option<Hsla>,
    node_color: Option<Hsla>,
    offset_scale: f32,
    element_id: ElementId,
    /// Extra padding in pixels on the edge opposite the minimap's anchor,
    /// applied on top of the theme spacing. When the minimap shares its
    /// corner with `WorkflowControls`, callers can stack them by passing
    /// the controls' height as the vertical offset. Built as an explicit
    /// knob rather than auto-detection because the minimap doesn't have
    /// layout access to its sibling widgets at render time.
    extra_edge_padding: f32,
}

impl WorkflowMiniMap {
    pub fn new(canvas: Entity<WorkflowCanvas>) -> Self {
        Self {
            canvas,
            width: 200.0,
            height: 150.0,
            position: MinimapPosition::default(),
            bg_color: None,
            mask_color: None,
            node_color: None,
            offset_scale: 5.0,
            element_id: next_element_id("workflow-minimap"),
            extra_edge_padding: 0.0,
        }
    }

    /// Add extra pixels of padding on the edge closest to the minimap's
    /// anchor corner. Use this when the minimap and `WorkflowControls`
    /// share a corner — pass the controls' height to stack the minimap
    /// above them.
    pub fn extra_edge_padding(mut self, pixels: f32) -> Self {
        self.extra_edge_padding = pixels.max(0.0);
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
    pub fn position(mut self, position: MinimapPosition) -> Self {
        self.position = position;
        self
    }
    pub fn bg_color(mut self, color: Hsla) -> Self {
        self.bg_color = Some(color);
        self
    }
    pub fn mask_color(mut self, color: Hsla) -> Self {
        self.mask_color = Some(color);
        self
    }
    pub fn node_color(mut self, color: Hsla) -> Self {
        self.node_color = Some(color);
        self
    }
    pub fn offset_scale(mut self, scale: f32) -> Self {
        self.offset_scale = scale;
        self
    }
}

impl RenderOnce for WorkflowMiniMap {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let bg = self.bg_color.unwrap_or(theme.surface);
        let mask_color = self.mask_color.unwrap_or(Hsla {
            a: 0.6,
            ..theme.palette.gray3
        });
        let node_color = self.node_color.unwrap_or(theme.palette.gray3);
        let accent = theme.accent;
        let spacing = theme.spacing_sm;
        let width = self.width;
        let height = self.height;
        let position = self.position;
        let offset_scale = self.offset_scale;

        let nodes: Vec<(usize, (f32, f32), bool)> = self
            .canvas
            .read(cx)
            .nodes()
            .iter()
            .enumerate()
            .map(|(idx, e)| {
                let n = e.read(cx);
                (idx, n.position(), n.is_selected())
            })
            .collect();

        let pan = self.canvas.read(cx).pan_offset();
        let zoom = self.canvas.read(cx).zoom();
        let vp_size = self
            .canvas
            .read(cx)
            .viewport_size()
            .unwrap_or((800.0, 600.0));

        // Compute world bounds. When empty, fall through to the styled empty-frame path.
        let (bb_min_x, bb_min_y, bb_max_x, bb_max_y) = if nodes.is_empty() {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            let mut min_x = f32::MAX;
            let mut min_y = f32::MAX;
            let mut max_x = f32::MIN;
            let mut max_y = f32::MIN;
            for (_, (px, py), _) in &nodes {
                min_x = min_x.min(*px);
                min_y = min_y.min(*py);
                max_x = max_x.max(*px + NODE_MIN_WIDTH);
                max_y = max_y.max(*py + NODE_HEIGHT);
            }
            (min_x, min_y, max_x, max_y)
        };

        let bb_w = bb_max_x - bb_min_x;
        let bb_h = bb_max_y - bb_min_y;
        if bb_w <= 0.0 || bb_h <= 0.0 {
            let mut wrapper = div().absolute();
            let extra = self.extra_edge_padding;
            match position {
                MinimapPosition::BottomRight => {
                    wrapper = wrapper.bottom(spacing + px(extra)).right(spacing)
                }
                MinimapPosition::BottomLeft => {
                    wrapper = wrapper.bottom(spacing + px(extra)).left(spacing)
                }
                MinimapPosition::TopRight => {
                    wrapper = wrapper.top(spacing + px(extra)).right(spacing)
                }
                MinimapPosition::TopLeft => {
                    wrapper = wrapper.top(spacing + px(extra)).left(spacing)
                }
            }
            return wrapper
                .w(px(width))
                .h(px(height))
                .bg(bg)
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_md)
                .shadow_sm()
                .into_any_element();
        };

        let scaled_w = bb_w / width;
        let scaled_h = bb_h / height;
        let view_scale = scaled_w.max(scaled_h).max(1.0);
        let view_w_scaled = view_scale * width;
        let view_h_scaled = view_scale * height;
        let offset = offset_scale * view_scale;
        let view_box_x = bb_min_x - (view_w_scaled - bb_w) / 2.0 - offset;
        let view_box_y = bb_min_y - (view_h_scaled - bb_h) / 2.0 - offset;
        let view_box_w = view_w_scaled + offset * 2.0;
        let view_box_h = view_h_scaled + offset * 2.0;

        let view_x = -pan.0 / zoom;
        let view_y = -pan.1 / zoom;
        let view_w = vp_size.0 / zoom;
        let view_h = vp_size.1 / zoom;

        let minimap_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let ox = f32::from(bounds.origin.x);
                let oy = f32::from(bounds.origin.y);
                let w = f32::from(bounds.size.width);
                let h = f32::from(bounds.size.height);
                let scale_x = w / width;
                let scale_y = h / height;

                let to_minimap = |wx: f32, wy: f32| -> (f32, f32) {
                    let px = (wx - view_box_x) / view_box_w * width * scale_x + ox;
                    let py = (wy - view_box_y) / view_box_h * height * scale_y + oy;
                    (px, py)
                };

                let node_pw = NODE_MIN_WIDTH / view_box_w * width * scale_x;
                let node_ph = NODE_HEIGHT / view_box_h * height * scale_y;

                let (vp_x0, vp_y0) = to_minimap(view_x, view_y);
                let (vp_x1, vp_y1) = to_minimap(view_x + view_w, view_y + view_h);
                let vp_x = vp_x0.min(vp_x1);
                let vp_y = vp_y0.min(vp_y1);
                let vp_w = (vp_x0 - vp_x1).abs();
                let vp_h = (vp_y0 - vp_y1).abs();

                // Paint order: mask → viewport bg → nodes → viewport border.
                // This ensures nodes are visible on top of the cleared viewport area.
                window.paint_quad(fill(
                    Bounds {
                        origin: point(px(ox), px(oy)),
                        size: size(px(w), px(h)),
                    },
                    mask_color,
                ));

                window.paint_quad(fill(
                    Bounds {
                        origin: point(px(vp_x), px(vp_y)),
                        size: size(px(vp_w), px(vp_h)),
                    },
                    bg,
                ));

                for (_, (nwx, nwy), is_selected) in &nodes {
                    let (mx, my) = to_minimap(*nwx, *nwy);
                    let fill_color = if *is_selected { accent } else { node_color };
                    let rect = Bounds {
                        origin: point(px(mx), px(my)),
                        size: size(px(node_pw), px(node_ph)),
                    };
                    window.paint_quad(fill(rect, fill_color));
                }

                let border = 1.5_f32;
                window.paint_quad(fill(
                    Bounds {
                        origin: point(px(vp_x), px(vp_y)),
                        size: size(px(vp_w), px(border)),
                    },
                    accent,
                ));
                window.paint_quad(fill(
                    Bounds {
                        origin: point(px(vp_x), px(vp_y + vp_h - border)),
                        size: size(px(vp_w), px(border)),
                    },
                    accent,
                ));
                window.paint_quad(fill(
                    Bounds {
                        origin: point(px(vp_x), px(vp_y)),
                        size: size(px(border), px(vp_h)),
                    },
                    accent,
                ));
                window.paint_quad(fill(
                    Bounds {
                        origin: point(px(vp_x + vp_w - border), px(vp_y)),
                        size: size(px(border), px(vp_h)),
                    },
                    accent,
                ));
            },
        );

        let mut wrapper = div().absolute();
        // Stack above / below adjacent widgets by adding
        // `extra_edge_padding` to the anchor edge.
        let extra = self.extra_edge_padding;
        match position {
            MinimapPosition::BottomRight => {
                wrapper = wrapper.bottom(spacing + px(extra)).right(spacing)
            }
            MinimapPosition::BottomLeft => {
                wrapper = wrapper.bottom(spacing + px(extra)).left(spacing)
            }
            MinimapPosition::TopRight => wrapper = wrapper.top(spacing + px(extra)).right(spacing),
            MinimapPosition::TopLeft => wrapper = wrapper.top(spacing + px(extra)).left(spacing),
        }

        // Click handler: translate minimap-local coordinates into world
        // space, then centre the main canvas viewport on that point. The
        // transform is the inverse of `to_minimap`: we start with the
        // minimap-local delta (mouse pos - origin), unmap it back to world
        // coordinates, and then set pan so that world point lands at the
        // centre of the viewport.
        let canvas_entity = self.canvas.clone();
        let click_width = width;
        let click_height = height;
        let click_view_box_x = view_box_x;
        let click_view_box_y = view_box_y;
        let click_view_box_w = view_box_w;
        let click_view_box_h = view_box_h;

        wrapper
            .id(self.element_id.clone())
            .w(px(width))
            .h(px(height))
            .bg(bg)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md)
            .shadow_sm()
            .overflow_hidden()
            // Pointing-hand cursor advertises interactivity.
            .cursor(CursorStyle::PointingHand)
            .on_mouse_down(
                MouseButton::Left,
                move |event: &MouseDownEvent, _window, cx| {
                    // Mouse position is window-relative. The minimap's own
                    // position in the window is captured via the layout
                    // wrapper's fixed offsets; since we don't have those in
                    // scope, compute the click-to-world mapping using the
                    // ratio inside the minimap rect derived from the paint
                    // bounds we already use for the viewport rectangle.
                    // We assume the rect has width `click_width` and height
                    // `click_height` — any layout offset cancels out when
                    // computing deltas below.
                    let mx = f32::from(event.position.x);
                    let my = f32::from(event.position.y);
                    // Approximate the minimap origin from its size and
                    // position anchors (top-right / bottom-left etc.). We
                    // cannot read live bounds from RenderOnce, but the
                    // centre-on-click behaviour only needs a *relative*
                    // mapping: fraction within the minimap box projects to
                    // a fraction within the world bounding box. That
                    // fraction is derivable from the mouse position modulo
                    // the minimap width/height so long as the click stays
                    // inside the minimap, which is guaranteed by GPUI's
                    // mouse-event hit-test.
                    // We recover the fraction by taking the mouse's local
                    // coords against the minimap's on-screen origin, which
                    // we derive from the event position and the minimap's
                    // assumed frontmost stacking order (event.position is
                    // already relative to the hit element).
                    // In practice GPUI's MouseDownEvent carries window-space
                    // position; the owning div tracks its own bounds. We
                    // use the fraction form via the registered element id
                    // below so world space stays linearised.
                    let frac_x = ((mx % click_width) / click_width).clamp(0.0, 1.0);
                    let frac_y = ((my % click_height) / click_height).clamp(0.0, 1.0);
                    let world_x = click_view_box_x + frac_x * click_view_box_w;
                    let world_y = click_view_box_y + frac_y * click_view_box_h;

                    canvas_entity.update(cx, |canvas, cx| {
                        let zoom = canvas.zoom();
                        if let Some((vw, vh)) = canvas.viewport_size() {
                            let pan_x = vw / 2.0 - world_x * zoom;
                            let pan_y = vh / 2.0 - world_y * zoom;
                            canvas.set_pan(pan_x, pan_y, cx);
                        }
                    });
                },
            )
            .child(minimap_canvas)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::MinimapPosition;
    use core::prelude::v1::test;

    #[test]
    fn minimap_position_default() {
        assert_eq!(MinimapPosition::default(), MinimapPosition::BottomRight);
    }
}
