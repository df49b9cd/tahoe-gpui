//! Workflow panel overlay — GPUI equivalent of the AI SDK Elements `Panel`.
//!
//! A positioned container for custom UI elements on the workflow canvas.
//! Supports six overlay positions (pill-shaped with semi-transparent background
//! and subtle shadow) plus a docked right-side variant.
//!
//! Overlay positions use `theme.panel_surface` — a semi-transparent surface
//! color. When paired with `WindowBackgroundAppearance::Blurred` (Liquid Glass
//! theme), the macOS window blur shows through for a true glass effect.
//!
//! The parent element must have `relative()` for overlay positioning.

use crate::foundations::theme::{ActiveTheme};
use gpui::prelude::*;
use gpui::{AnyElement, App, Window, div, px};

/// Position of a panel overlay on the canvas.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum WorkflowPanelPosition {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    /// Docked right-side panel (Rust-specific extension, not present in AI SDK Elements).
    Right,
}

/// A positioned container for custom UI elements on the workflow canvas.
///
/// Overlay positions render as a pill-shaped card with a semi-transparent
/// background (`theme.panel_surface`) and subtle shadow, approximating
/// the backdrop-blur styling of the AI SDK Elements `Panel` component.
/// The `Right` position renders as a docked side panel with opaque background.
#[derive(IntoElement)]
pub struct WorkflowPanel {
    position: WorkflowPanelPosition,
    children: Vec<AnyElement>,
}

impl WorkflowPanel {
    pub fn new(position: WorkflowPanelPosition) -> Self {
        Self {
            position,
            children: Vec::new(),
        }
    }

    /// Append a child element.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Append multiple child elements.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for WorkflowPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let spacing = theme.spacing_sm;

        if self.position == WorkflowPanelPosition::Right {
            // Docked side panel: participates in parent flex layout.
            return div()
                .h_full()
                .w(px(280.0))
                .flex()
                .flex_col()
                .overflow_hidden()
                .bg(theme.surface)
                .border_l_1()
                .border_color(theme.border)
                .children(self.children)
                .into_any_element();
        }

        // Overlay positions: pill-shaped card with shadow.
        let pill = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .p(theme.spacing_sm)
            .bg(theme.panel_surface)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_full)
            .shadow_sm()
            .children(self.children);

        // For center positions we need a full-width transparent wrapper
        // to achieve horizontal centering without CSS transforms.
        let needs_center_wrapper = matches!(
            self.position,
            WorkflowPanelPosition::TopCenter | WorkflowPanelPosition::BottomCenter
        );

        if needs_center_wrapper {
            let mut wrapper = div().absolute().left_0().right_0().flex().justify_center();

            wrapper = match self.position {
                WorkflowPanelPosition::TopCenter => wrapper.top(spacing),
                WorkflowPanelPosition::BottomCenter => wrapper.bottom(spacing),
                _ => unreachable!(),
            };

            wrapper.child(pill).into_any_element()
        } else {
            let mut positioned = div().absolute();

            positioned = match self.position {
                WorkflowPanelPosition::TopLeft => positioned.top(spacing).left(spacing),
                WorkflowPanelPosition::TopRight => positioned.top(spacing).right(spacing),
                WorkflowPanelPosition::BottomLeft => positioned.bottom(spacing).left(spacing),
                WorkflowPanelPosition::BottomRight => positioned.bottom(spacing).right(spacing),
                _ => unreachable!(),
            };

            positioned.child(pill).into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkflowPanel, WorkflowPanelPosition};
    use core::prelude::v1::test;
    use gpui::div;

    #[test]
    fn default_position_is_top_left() {
        assert_eq!(
            WorkflowPanelPosition::default(),
            WorkflowPanelPosition::TopLeft
        );
    }

    #[test]
    fn all_positions_are_distinct() {
        let positions = [
            WorkflowPanelPosition::TopLeft,
            WorkflowPanelPosition::TopCenter,
            WorkflowPanelPosition::TopRight,
            WorkflowPanelPosition::BottomLeft,
            WorkflowPanelPosition::BottomCenter,
            WorkflowPanelPosition::BottomRight,
            WorkflowPanelPosition::Right,
        ];
        for (i, a) in positions.iter().enumerate() {
            for (j, b) in positions.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn builder_default_has_no_children() {
        let panel = WorkflowPanel::new(WorkflowPanelPosition::TopLeft);
        assert!(panel.children.is_empty());
    }

    #[test]
    fn builder_single_child() {
        let panel = WorkflowPanel::new(WorkflowPanelPosition::TopLeft).child(div());
        assert_eq!(panel.children.len(), 1);
    }

    #[test]
    fn builder_multiple_children() {
        let panel = WorkflowPanel::new(WorkflowPanelPosition::BottomRight).children(vec![
            div(),
            div(),
            div(),
        ]);
        assert_eq!(panel.children.len(), 3);
    }

    #[test]
    fn builder_child_and_children_combine() {
        let panel = WorkflowPanel::new(WorkflowPanelPosition::TopCenter)
            .child(div())
            .children(vec![div(), div()]);
        assert_eq!(panel.children.len(), 3);
    }

    #[test]
    fn position_copy_and_clone() {
        let a = WorkflowPanelPosition::TopRight;
        let b = a;
        assert_eq!(a, b);
        assert_eq!(a.clone(), b);
    }

    #[test]
    fn position_debug_format() {
        assert_eq!(format!("{:?}", WorkflowPanelPosition::TopLeft), "TopLeft");
        assert_eq!(format!("{:?}", WorkflowPanelPosition::Right), "Right");
    }
}
