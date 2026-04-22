//! Node-attached toolbar component.
//!
//! A positioned toolbar that attaches to workflow nodes, matching the
//! AI SDK Elements Toolbar (React Flow NodeToolbar wrapper). Supports
//! configurable position, alignment, visibility, and offset.
//!
//! The bar itself renders on a Liquid Glass material so it matches the
//! rest of the macOS 26 chrome; the `WhenHovered` visibility variant
//! matches Freeform and Keynote's contextual-format toolbar, which
//! reveals on hover without requiring a selection click first.

use crate::foundations::materials::{LensEffect, glass_lens_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{AnyElement, App, Window, div, px};

/// Where the toolbar is positioned relative to its node.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ToolbarPosition {
    Top,
    Right,
    #[default]
    Bottom,
    Left,
}

/// Alignment of the toolbar along the node edge.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ToolbarAlign {
    Start,
    #[default]
    Center,
    End,
}

/// Visibility mode for the toolbar.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ToolbarVisibility {
    /// Visible only when the attached node is selected (default).
    #[default]
    WhenSelected,
    /// Visible while the node is hovered — matches the Freeform / Keynote
    /// contextual-format toolbar that appears on hover without requiring
    /// a prior click-selection.
    WhenHovered,
    /// Always visible.
    Always,
    /// Always hidden.
    Hidden,
}

/// A toolbar that attaches to a workflow node.
///
/// Constructed via builder pattern and rendered by the canvas. The canvas
/// injects screen-space node context via `with_node_context` before rendering.
#[derive(IntoElement)]
pub struct NodeToolbar {
    position: ToolbarPosition,
    align: ToolbarAlign,
    visibility: ToolbarVisibility,
    offset: f32,
    children: Vec<AnyElement>,
    // Screen-space context injected by canvas:
    node_screen_x: f32,
    node_screen_y: f32,
    node_width: f32,
    node_height: f32,
    node_selected: bool,
    node_hovered: bool,
    multi_selected: bool,
}

impl Default for NodeToolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeToolbar {
    pub fn new() -> Self {
        Self {
            position: ToolbarPosition::default(),
            align: ToolbarAlign::default(),
            visibility: ToolbarVisibility::default(),
            offset: 10.0,
            children: Vec::new(),
            node_screen_x: 0.0,
            node_screen_y: 0.0,
            node_width: 0.0,
            node_height: 0.0,
            node_selected: false,
            node_hovered: false,
            multi_selected: false,
        }
    }

    /// Set the toolbar position relative to the node.
    pub fn position(mut self, position: ToolbarPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the alignment along the node edge.
    pub fn align(mut self, align: ToolbarAlign) -> Self {
        self.align = align;
        self
    }

    /// Set the visibility mode.
    pub fn visibility(mut self, visibility: ToolbarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Set the pixel offset between node and toolbar (default: 10.0).
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    /// Add a child element to the toolbar.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Add multiple children.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        for child in children {
            self.children.push(child.into_any_element());
        }
        self
    }

    /// Inject screen-space node context. Called by the canvas before rendering.
    pub(crate) fn with_node_context(
        mut self,
        screen_x: f32,
        screen_y: f32,
        width: f32,
        height: f32,
        selected: bool,
        hovered: bool,
        multi_selected: bool,
    ) -> Self {
        self.node_screen_x = screen_x;
        self.node_screen_y = screen_y;
        self.node_width = width;
        self.node_height = height;
        self.node_selected = selected;
        self.node_hovered = hovered;
        self.multi_selected = multi_selected;
        self
    }
}

/// Whether the toolbar should be visible given its configuration and node state.
pub(crate) fn should_show(
    visibility: ToolbarVisibility,
    node_selected: bool,
    node_hovered: bool,
    multi_selected: bool,
) -> bool {
    if multi_selected {
        return false;
    }
    match visibility {
        ToolbarVisibility::Hidden => false,
        ToolbarVisibility::Always => true,
        ToolbarVisibility::WhenSelected => node_selected,
        // Hover-reveal. We also keep the bar visible once the node becomes
        // the active selection, because the hover typically precedes the
        // click and losing the toolbar mid-interaction feels buggy.
        ToolbarVisibility::WhenHovered => node_hovered || node_selected,
    }
}

/// Compute the anchor point for the toolbar in screen space.
///
/// Returns `(x, y)` where the toolbar should be absolutely positioned.
/// For Top/Bottom positions, `x` is along the horizontal edge; for Left/Right,
/// `y` is along the vertical edge.
pub(crate) fn compute_toolbar_position(
    position: ToolbarPosition,
    align: ToolbarAlign,
    offset: f32,
    node_x: f32,
    node_y: f32,
    node_w: f32,
    node_h: f32,
) -> (f32, f32) {
    match position {
        ToolbarPosition::Bottom => {
            let y = node_y + node_h + offset;
            let x = align_along(align, node_x, node_w);
            (x, y)
        }
        ToolbarPosition::Top => {
            let y = node_y - offset;
            let x = align_along(align, node_x, node_w);
            (x, y)
        }
        ToolbarPosition::Right => {
            let x = node_x + node_w + offset;
            let y = align_along(align, node_y, node_h);
            (x, y)
        }
        ToolbarPosition::Left => {
            let x = node_x - offset;
            let y = align_along(align, node_y, node_h);
            (x, y)
        }
    }
}

/// Compute the coordinate along a node edge for the given alignment.
fn align_along(align: ToolbarAlign, origin: f32, size: f32) -> f32 {
    match align {
        ToolbarAlign::Start => origin,
        ToolbarAlign::Center => origin + size / 2.0,
        ToolbarAlign::End => origin + size,
    }
}

impl RenderOnce for NodeToolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !should_show(
            self.visibility,
            self.node_selected,
            self.node_hovered,
            self.multi_selected,
        ) {
            return div();
        }

        let theme = cx.theme();

        let (anchor_x, anchor_y) = compute_toolbar_position(
            self.position,
            self.align,
            self.offset,
            self.node_screen_x,
            self.node_screen_y,
            self.node_width,
            self.node_height,
        );

        let is_vertical = matches!(
            self.position,
            ToolbarPosition::Left | ToolbarPosition::Right
        );

        // Match the NodeToolbar chrome to the rest of the canvas
        // (WorkflowControls already uses Liquid Glass). Size Small matches
        // Apple's sizing for node-scoped contextual toolbars. Real lens
        // composite so the refracted canvas reads through.
        let mut effect = LensEffect::liquid_glass(GlassSize::Small, theme);
        effect.blur.corner_radius = f32::from(theme.radius_sm);
        let mut bar = glass_lens_surface(theme, &effect, GlassSize::Small)
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .p(theme.spacing_sm)
            .rounded(theme.radius_sm);

        if is_vertical {
            bar = bar.flex_col();
        }

        for child in self.children {
            bar = bar.child(child);
        }

        // Absolute-positioned anchor at the computed point.
        // The anchor_x/anchor_y from compute_toolbar_position already
        // accounts for offset. The toolbar content grows from this point.
        div()
            .absolute()
            .left(px(anchor_x))
            .top(px(anchor_y))
            .child(bar)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NodeToolbar, ToolbarAlign, ToolbarPosition, ToolbarVisibility, compute_toolbar_position,
        should_show,
    };
    use core::prelude::v1::test;

    // ── Enum defaults ───────────────────────────────────────────────

    #[test]
    fn position_default_is_bottom() {
        assert_eq!(ToolbarPosition::default(), ToolbarPosition::Bottom);
    }

    #[test]
    fn align_default_is_center() {
        assert_eq!(ToolbarAlign::default(), ToolbarAlign::Center);
    }

    #[test]
    fn visibility_default_is_when_selected() {
        assert_eq!(
            ToolbarVisibility::default(),
            ToolbarVisibility::WhenSelected
        );
    }

    // ── Enum equality ───────────────────────────────────────────────

    #[test]
    fn position_equality() {
        assert_eq!(ToolbarPosition::Top, ToolbarPosition::Top);
        assert_ne!(ToolbarPosition::Top, ToolbarPosition::Bottom);
        assert_ne!(ToolbarPosition::Left, ToolbarPosition::Right);
    }

    #[test]
    fn align_equality() {
        assert_eq!(ToolbarAlign::Start, ToolbarAlign::Start);
        assert_ne!(ToolbarAlign::Start, ToolbarAlign::Center);
        assert_ne!(ToolbarAlign::Center, ToolbarAlign::End);
    }

    #[test]
    fn visibility_equality() {
        assert_eq!(ToolbarVisibility::Always, ToolbarVisibility::Always);
        assert_ne!(ToolbarVisibility::Always, ToolbarVisibility::Hidden);
        assert_ne!(ToolbarVisibility::WhenSelected, ToolbarVisibility::Always);
    }

    // ── Enum debug ──────────────────────────────────────────────────

    #[test]
    fn position_debug() {
        assert!(format!("{:?}", ToolbarPosition::Top).contains("Top"));
        assert!(format!("{:?}", ToolbarPosition::Bottom).contains("Bottom"));
    }

    #[test]
    fn align_debug() {
        assert!(format!("{:?}", ToolbarAlign::Center).contains("Center"));
    }

    #[test]
    fn visibility_debug() {
        assert!(format!("{:?}", ToolbarVisibility::Hidden).contains("Hidden"));
    }

    // ── Builder defaults ────────────────────────────────────────────

    #[test]
    fn builder_defaults() {
        let t = NodeToolbar::new();
        assert_eq!(t.position, ToolbarPosition::Bottom);
        assert_eq!(t.align, ToolbarAlign::Center);
        assert_eq!(t.visibility, ToolbarVisibility::WhenSelected);
        assert!((t.offset - 10.0).abs() < f32::EPSILON);
        assert!(t.children.is_empty());
        assert!(!t.node_selected);
        assert!(!t.multi_selected);
    }

    #[test]
    fn builder_chaining() {
        let t = NodeToolbar::new()
            .position(ToolbarPosition::Top)
            .align(ToolbarAlign::End)
            .visibility(ToolbarVisibility::Always)
            .offset(20.0);
        assert_eq!(t.position, ToolbarPosition::Top);
        assert_eq!(t.align, ToolbarAlign::End);
        assert_eq!(t.visibility, ToolbarVisibility::Always);
        assert!((t.offset - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn with_node_context_sets_fields() {
        let t = NodeToolbar::new().with_node_context(100.0, 200.0, 160.0, 80.0, true, false, false);
        assert!((t.node_screen_x - 100.0).abs() < f32::EPSILON);
        assert!((t.node_screen_y - 200.0).abs() < f32::EPSILON);
        assert!((t.node_width - 160.0).abs() < f32::EPSILON);
        assert!((t.node_height - 80.0).abs() < f32::EPSILON);
        assert!(t.node_selected);
        assert!(!t.multi_selected);
    }

    // ── Visibility logic ────────────────────────────────────────────

    #[test]
    fn should_show_hidden_always_false() {
        assert!(!should_show(ToolbarVisibility::Hidden, true, true, false));
        assert!(!should_show(ToolbarVisibility::Hidden, false, false, false));
    }

    #[test]
    fn should_show_always_when_not_multi() {
        assert!(should_show(ToolbarVisibility::Always, false, false, false));
        assert!(should_show(ToolbarVisibility::Always, true, false, false));
    }

    #[test]
    fn should_show_when_selected() {
        assert!(should_show(
            ToolbarVisibility::WhenSelected,
            true,
            false,
            false
        ));
        assert!(!should_show(
            ToolbarVisibility::WhenSelected,
            false,
            false,
            false
        ));
    }

    #[test]
    fn should_show_multi_select_hides_all() {
        assert!(!should_show(ToolbarVisibility::Always, true, true, true));
        assert!(!should_show(
            ToolbarVisibility::WhenSelected,
            true,
            true,
            true
        ));
        assert!(!should_show(ToolbarVisibility::Hidden, true, true, true));
        assert!(!should_show(
            ToolbarVisibility::WhenHovered,
            true,
            true,
            true
        ));
    }

    #[test]
    fn should_show_when_hovered_reveals_on_hover_only() {
        // Hover alone triggers visibility; neither a selection nor a
        // prior click is required.
        assert!(should_show(
            ToolbarVisibility::WhenHovered,
            false,
            true,
            false
        ));
        assert!(!should_show(
            ToolbarVisibility::WhenHovered,
            false,
            false,
            false
        ));
    }

    #[test]
    fn should_show_when_hovered_retains_on_select() {
        // Once the user clicks the node, losing the toolbar while the
        // pointer slips off would feel buggy. Selection pins visibility.
        assert!(should_show(
            ToolbarVisibility::WhenHovered,
            true,
            false,
            false
        ));
    }

    // ── Position math ───────────────────────────────────────────────

    #[test]
    fn position_bottom_center() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Bottom,
            ToolbarAlign::Center,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 180.0).abs() < f32::EPSILON); // 100 + 160/2
        assert!((y - 290.0).abs() < f32::EPSILON); // 200 + 80 + 10
    }

    #[test]
    fn position_bottom_start() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Bottom,
            ToolbarAlign::Start,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 100.0).abs() < f32::EPSILON);
        assert!((y - 290.0).abs() < f32::EPSILON);
    }

    #[test]
    fn position_bottom_end() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Bottom,
            ToolbarAlign::End,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 260.0).abs() < f32::EPSILON); // 100 + 160
        assert!((y - 290.0).abs() < f32::EPSILON);
    }

    #[test]
    fn position_top_center() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Top,
            ToolbarAlign::Center,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 180.0).abs() < f32::EPSILON);
        assert!((y - 190.0).abs() < f32::EPSILON); // 200 - 10
    }

    #[test]
    fn position_right_center() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Right,
            ToolbarAlign::Center,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 270.0).abs() < f32::EPSILON); // 100 + 160 + 10
        assert!((y - 240.0).abs() < f32::EPSILON); // 200 + 80/2
    }

    #[test]
    fn position_left_center() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Left,
            ToolbarAlign::Center,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 90.0).abs() < f32::EPSILON); // 100 - 10
        assert!((y - 240.0).abs() < f32::EPSILON);
    }

    #[test]
    fn position_right_start() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Right,
            ToolbarAlign::Start,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 270.0).abs() < f32::EPSILON);
        assert!((y - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn position_left_end() {
        let (x, y) = compute_toolbar_position(
            ToolbarPosition::Left,
            ToolbarAlign::End,
            10.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((x - 90.0).abs() < f32::EPSILON);
        assert!((y - 280.0).abs() < f32::EPSILON); // 200 + 80
    }

    #[test]
    fn offset_zero() {
        let (_, y) = compute_toolbar_position(
            ToolbarPosition::Bottom,
            ToolbarAlign::Center,
            0.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((y - 280.0).abs() < f32::EPSILON); // 200 + 80 + 0
    }

    #[test]
    fn offset_large() {
        let (_, y) = compute_toolbar_position(
            ToolbarPosition::Bottom,
            ToolbarAlign::Center,
            50.0,
            100.0,
            200.0,
            160.0,
            80.0,
        );
        assert!((y - 330.0).abs() < f32::EPSILON); // 200 + 80 + 50
    }

    // ── Enum copy/clone ─────────────────────────────────────────────

    #[test]
    fn position_copy() {
        let p = ToolbarPosition::Top;
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn align_clone() {
        let a = ToolbarAlign::End;
        let a2 = a;
        assert_eq!(a, a2);
    }

    #[test]
    fn visibility_copy() {
        let v = ToolbarVisibility::Always;
        let v2 = v;
        assert_eq!(v, v2);
    }
}
