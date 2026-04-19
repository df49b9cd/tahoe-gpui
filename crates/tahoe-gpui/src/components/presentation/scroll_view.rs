//! Scroll container (HIG Scroll views).
//!
//! Wraps children in an axis-scrollable container. macOS 26 Tahoe integrates
//! scroll-edge Liquid Glass effects that consuming apps render through this
//! wrapper.
//!
//! # Scrollbar behaviour
//!
//! macOS scrollbar visibility is system-managed via **General > Show
//! scroll bars** (Always / When scrolling / Automatically). GPUI defers
//! scrollbar chrome to the host platform, so there is no per-view thickness
//! or fade-on-idle token. HIG: "don't try to draw or manage overlay
//! scrollbars yourself — the system renders them over scroll views."
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/scroll-views>

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, Pixels, Window, div, px};

/// Scroll axis for a [`ScrollView`] — mirrors `UIScrollView.axis` /
/// `NSScrollView.horizontalScroller` + `verticalScroller` exposure.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ScrollAxis {
    /// Horizontal scrolling only. Default (matches the historical
    /// `ScrollView` that this component replaced).
    #[default]
    Horizontal,
    /// Vertical scrolling only.
    Vertical,
    /// Both axes scrollable independently.
    Both,
}

/// A scrollable container wrapping children in a flex line on the scroll axis.
///
/// Requires an `ElementId` because GPUI scroll state is stateful.
#[derive(IntoElement)]
pub struct ScrollView {
    id: ElementId,
    children: Vec<AnyElement>,
    gap: Option<Pixels>,
    axis: ScrollAxis,
}

impl ScrollView {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            gap: None,
            axis: ScrollAxis::default(),
        }
    }

    /// Configure the scroll axis. Defaults to [`ScrollAxis::Horizontal`].
    pub fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Convenience: vertical scrolling.
    pub fn vertical(mut self) -> Self {
        self.axis = ScrollAxis::Vertical;
        self
    }

    /// Convenience: bidirectional scrolling.
    pub fn bidirectional(mut self) -> Self {
        self.axis = ScrollAxis::Both;
        self
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
}

impl RenderOnce for ScrollView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let gap = self.gap.unwrap_or(px(8.0));
        let axis = self.axis;
        let children = self.children;

        let outer = div().id(self.id).size_full();
        let outer = match axis {
            ScrollAxis::Horizontal => outer.overflow_x_scroll(),
            ScrollAxis::Vertical => outer.overflow_y_scroll(),
            ScrollAxis::Both => outer.overflow_scroll(),
        };

        let inner = match axis {
            ScrollAxis::Horizontal => div()
                .flex()
                .flex_row()
                .flex_nowrap()
                .gap(gap)
                .children(children),
            ScrollAxis::Vertical => div()
                .flex()
                .flex_col()
                .flex_nowrap()
                .gap(gap)
                .children(children),
            ScrollAxis::Both => div()
                .flex()
                .flex_col()
                .flex_nowrap()
                .gap(gap)
                .children(children),
        };

        outer.child(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::{ScrollAxis, ScrollView};
    use core::prelude::v1::test;

    #[test]
    fn scroll_axis_default_is_horizontal() {
        assert_eq!(ScrollAxis::default(), ScrollAxis::Horizontal);
    }

    #[test]
    fn scroll_axis_builders() {
        let h = ScrollView::new("h");
        assert_eq!(h.axis, ScrollAxis::Horizontal);
        let v = ScrollView::new("v").vertical();
        assert_eq!(v.axis, ScrollAxis::Vertical);
        let b = ScrollView::new("b").bidirectional();
        assert_eq!(b.axis, ScrollAxis::Both);
    }

    #[test]
    fn scroll_axis_explicit_override() {
        let v = ScrollView::new("v").axis(ScrollAxis::Vertical);
        assert_eq!(v.axis, ScrollAxis::Vertical);
    }
}
