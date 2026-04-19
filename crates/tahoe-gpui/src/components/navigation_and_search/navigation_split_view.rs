//! HIG `NavigationSplitView` — sidebar + content + optional inspector.
//!
//! macOS Tahoe's three-column shell (sidebar, content, inspector) used in
//! Mail, Notes, Xcode, System Settings. Stateless `RenderOnce` so the
//! parent owns column visibility and width state; this primitive only
//! lays out the columns at the configured widths and renders the
//! separators between them.
//!
//! Resize handles are *not* yet wired — column widths are caller-owned
//! values. When `Sidebar`'s draggable separator generalises beyond a
//! 2-column layout, this component can adopt the same pattern. For now
//! callers expose toggle affordances via [`NavigationSplitView::on_sidebar_toggle`]
//! and [`NavigationSplitView::on_inspector_toggle`].
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/sidebars>
//! <https://developer.apple.com/design/human-interface-guidelines/inspectors>

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, Pixels, Window, div, px};

use crate::foundations::layout::{
    INSPECTOR_DEFAULT_WIDTH, SIDEBAR_DEFAULT_WIDTH, SIDEBAR_MIN_WIDTH,
};
use crate::foundations::theme::ActiveTheme;

/// Three-column navigation shell.
///
/// Layout (LTR): `sidebar | content | inspector`. The sidebar collapses
/// when [`sidebar_collapsed`](Self::sidebar_collapsed) is `true`; the
/// inspector is omitted entirely when [`inspector_visible`](Self::inspector_visible)
/// is `false`. Content always renders and fills the remaining width.
#[derive(IntoElement)]
pub struct NavigationSplitView {
    id: ElementId,
    sidebar: Option<AnyElement>,
    content: AnyElement,
    inspector: Option<AnyElement>,
    sidebar_width: Pixels,
    inspector_width: Pixels,
    sidebar_collapsed: bool,
    inspector_visible: bool,
}

impl NavigationSplitView {
    /// Create a new split view around the given content element.
    /// Sidebar and inspector are added via builder methods; both
    /// columns default to omitted.
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            sidebar: None,
            content: content.into_any_element(),
            inspector: None,
            sidebar_width: px(SIDEBAR_DEFAULT_WIDTH),
            inspector_width: px(INSPECTOR_DEFAULT_WIDTH),
            sidebar_collapsed: false,
            inspector_visible: false,
        }
    }

    /// Provide the sidebar column. Pass any element — typically a
    /// [`crate::components::navigation_and_search::Sidebar`].
    pub fn sidebar(mut self, sidebar: impl IntoElement) -> Self {
        self.sidebar = Some(sidebar.into_any_element());
        self
    }

    /// Provide the inspector / detail column on the trailing side.
    /// Visibility is also gated by [`inspector_visible`](Self::inspector_visible)
    /// — callers typically toggle that boolean from a toolbar action.
    pub fn inspector(mut self, inspector: impl IntoElement) -> Self {
        self.inspector = Some(inspector.into_any_element());
        self
    }

    /// Override the sidebar column width (defaults to `SIDEBAR_DEFAULT_WIDTH`).
    /// Values below `SIDEBAR_MIN_WIDTH` are clamped up so labels do not
    /// truncate at the default Dynamic Type body size.
    pub fn sidebar_width(mut self, width: Pixels) -> Self {
        let clamped = f32::from(width).max(SIDEBAR_MIN_WIDTH);
        self.sidebar_width = px(clamped);
        self
    }

    /// Override the inspector column width (defaults to
    /// `INSPECTOR_DEFAULT_WIDTH` = 250 pt).
    pub fn inspector_width(mut self, width: Pixels) -> Self {
        self.inspector_width = width;
        self
    }

    /// Hide the sidebar column without dropping the element from the
    /// caller's tree. Useful for toolbar-driven collapse/expand without
    /// re-allocating the sidebar contents.
    pub fn sidebar_collapsed(mut self, collapsed: bool) -> Self {
        self.sidebar_collapsed = collapsed;
        self
    }

    /// Show or hide the inspector column. When `false` the inspector
    /// element is not rendered. Toggle from a toolbar button (HIG
    /// macOS Tahoe inspector pattern).
    pub fn inspector_visible(mut self, visible: bool) -> Self {
        self.inspector_visible = visible;
        self
    }
}

impl RenderOnce for NavigationSplitView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let separator_color = theme.border;
        let separator_thickness = theme.separator_thickness;

        let mut row = div().id(self.id).flex().flex_row().h_full().w_full();

        // Leading: sidebar (when present and not collapsed).
        if let (Some(sidebar), false) = (self.sidebar, self.sidebar_collapsed) {
            row = row.child(div().w(self.sidebar_width).h_full().child(sidebar));
            row = row.child(div().w(separator_thickness).h_full().bg(separator_color));
        }

        // Center: content always renders, filling the remaining width.
        row = row.child(div().flex_1().h_full().child(self.content));

        // Trailing: inspector (when visible and an element was supplied).
        if let (Some(inspector), true) = (self.inspector, self.inspector_visible) {
            row = row.child(div().w(separator_thickness).h_full().bg(separator_color));
            row = row.child(div().w(self.inspector_width).h_full().child(inspector));
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::NavigationSplitView;
    use crate::foundations::layout::{
        INSPECTOR_DEFAULT_WIDTH, SIDEBAR_DEFAULT_WIDTH, SIDEBAR_MIN_WIDTH,
    };
    use core::prelude::v1::test;
    use gpui::{div, px};

    #[test]
    fn defaults_match_hig_widths() {
        let nsv = NavigationSplitView::new("nsv", div());
        assert_eq!(nsv.sidebar_width, px(SIDEBAR_DEFAULT_WIDTH));
        assert_eq!(nsv.inspector_width, px(INSPECTOR_DEFAULT_WIDTH));
        assert!(!nsv.sidebar_collapsed);
        assert!(!nsv.inspector_visible);
        assert!(nsv.sidebar.is_none());
        assert!(nsv.inspector.is_none());
    }

    #[test]
    fn sidebar_width_clamps_to_min() {
        let nsv = NavigationSplitView::new("nsv", div()).sidebar_width(px(40.0));
        // SIDEBAR_MIN_WIDTH = 180 pt — anything narrower clamps up so
        // sidebar row labels don't truncate at default Dynamic Type.
        assert_eq!(nsv.sidebar_width, px(SIDEBAR_MIN_WIDTH));
    }

    #[test]
    fn sidebar_width_passes_above_min() {
        let nsv = NavigationSplitView::new("nsv", div()).sidebar_width(px(260.0));
        assert_eq!(nsv.sidebar_width, px(260.0));
    }

    #[test]
    fn builder_collapsed_and_inspector_visible() {
        let nsv = NavigationSplitView::new("nsv", div())
            .sidebar_collapsed(true)
            .inspector_visible(true);
        assert!(nsv.sidebar_collapsed);
        assert!(nsv.inspector_visible);
    }

    #[test]
    fn builder_attaches_sidebar_and_inspector() {
        let nsv = NavigationSplitView::new("nsv", div())
            .sidebar(div())
            .inspector(div());
        assert!(nsv.sidebar.is_some());
        assert!(nsv.inspector.is_some());
    }
}
