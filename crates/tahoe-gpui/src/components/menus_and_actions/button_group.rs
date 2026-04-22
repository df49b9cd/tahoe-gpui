//! ButtonGroup: groups buttons into a visually cohesive segmented row.
//!
//! Renders a glass background behind the children with square inner corners
//! on the middle items and rounded corners on the ends — the HIG macOS 26
//! segmented-control pattern. Between adjacent children a 1pt hairline
//! divider is inserted so the group reads as distinct slices rather than
//! an undivided tile.

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, Hsla, Pixels, Window, div, px};

use crate::foundations::color::compose_black_tint_linear;
use crate::foundations::materials::GLASS_LAYER_TINT_ALPHA;
use crate::foundations::theme::{ActiveTheme, GlassSize};

/// A grouped row of buttons rendered as a cohesive segmented unit.
///
/// The group provides the shared background, border, and rounded corners.
/// Children keep their own focus rings because the background is painted
/// behind them rather than clipping them (no `overflow_hidden`).
///
/// ```ignore
/// ButtonGroup::new("controls")
///     .child(Button::new("play").icon(play_icon).variant(ButtonVariant::Ghost))
///     .child(Button::new("stop").icon(stop_icon).variant(ButtonVariant::Ghost))
/// ```
#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    children: Vec<AnyElement>,
    gap: Option<Pixels>,
    /// When `false` (default), a 1pt hairline is inserted between adjacent
    /// children. Set to `false` explicitly if the children already carry
    /// their own dividers.
    separators: bool,
}

impl ButtonGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            gap: None,
            separators: true,
        }
    }

    pub fn gap(mut self, gap: Pixels) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }

    /// Toggle the 1pt hairline between adjacent children (default: on).
    pub fn separators(mut self, separators: bool) -> Self {
        self.separators = separators;
        self
    }
}

impl RenderOnce for ButtonGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let gap = self.gap.unwrap_or(px(0.0));
        let glass = &theme.glass;
        let base_bg = glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
        // Layer-2 black-tint composite in linear light so the fill matches
        // `glass_surface` bit-for-bit. Applied inline because the container
        // is `Stateful<Div>` (has `.id()`) and glass helpers take `Div`.
        let bg = compose_black_tint_linear(base_bg, GLASS_LAYER_TINT_ALPHA);
        let radius = glass.radius(GlassSize::Small);
        let shadows = glass.shadows(GlassSize::Small).to_vec();

        // Outer element: paints the glass background/shadow behind the row
        // but does NOT clip, so child focus rings remain visible.
        let mut outer = div()
            .id(self.id)
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .bg(bg)
            .rounded(radius)
            .shadow(shadows);
        outer = crate::foundations::materials::apply_high_contrast_border(outer, theme);

        let count = self.children.len();
        let separator_color: Hsla = theme.border;
        let insert_separators = self.separators && gap == px(0.0);

        let mut out = outer;
        for (i, child) in self.children.into_iter().enumerate() {
            if i > 0 && insert_separators {
                out = out.child(
                    div()
                        .w(px(1.0))
                        .h_full()
                        .bg(separator_color)
                        .flex_shrink_0(),
                );
            }
            out = out.child(child);
            let _ = count; // silence warning when count unused below
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonGroup;
    use crate::foundations::layout::SPACING_4;
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn test_new_creates_empty_group() {
        let group = ButtonGroup::new("test-group");
        assert!(group.children.is_empty());
        assert!(group.gap.is_none());
        assert!(group.separators, "separators default to on");
    }

    #[test]
    fn test_child_accumulates() {
        let group = ButtonGroup::new("test")
            .child(gpui::div())
            .child(gpui::div());
        assert_eq!(group.children.len(), 2);
    }

    #[test]
    fn test_children_accumulates() {
        let group = ButtonGroup::new("test").children(vec![gpui::div(), gpui::div(), gpui::div()]);
        assert_eq!(group.children.len(), 3);
    }

    #[test]
    fn test_gap_stores_value() {
        let group = ButtonGroup::new("test").gap(px(SPACING_4));
        assert_eq!(group.gap, Some(px(4.0)));
    }

    #[test]
    fn test_separators_builder_toggles_flag() {
        let group = ButtonGroup::new("test").separators(false);
        assert!(!group.separators);
    }
}
