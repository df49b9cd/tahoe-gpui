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
//! callers drive collapse / visibility via the
//! [`NavigationSplitView::sidebar_collapsed`] and
//! [`NavigationSplitView::inspector_visible`] builders, typically wired
//! to a toolbar action.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/sidebars>
//! <https://developer.apple.com/design/human-interface-guidelines/inspectors>
//!
//! # Keyboard
//!
//! Pass an [`Open`-mode][FocusGroupMode::Open] [`FocusGroup`] per pane via
//! [`sidebar_focus_group`](Self::sidebar_focus_group) /
//! [`content_focus_group`](Self::content_focus_group) /
//! [`inspector_focus_group`](Self::inspector_focus_group) to turn each pane
//! into a distinct focus region. When the user presses
//! **Cmd-Opt-Left / Cmd-Opt-Right** with focus inside a pane, focus jumps
//! to the first registered member of the previous / next **visible,
//! non-empty** pane (order: sidebar → content → inspector). Hidden panes
//! (`sidebar_collapsed(true)`, `inspector_visible(false)`) and panes with
//! no registered group members are skipped.
//!
//! The caller owns each `FocusGroup` across renders and is responsible for
//! calling [`FocusGroup::set_members`] each render with the current
//! focusable children of that pane. See `examples/gallery/focus_groups.rs`
//! for the reference wiring.

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, KeyDownEvent, Pixels, SharedString, Window, div, px};

use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup, FocusGroupMode,
};
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
    sidebar_focus_group: Option<FocusGroup>,
    content_focus_group: Option<FocusGroup>,
    inspector_focus_group: Option<FocusGroup>,
    sidebar_accessibility_label: Option<SharedString>,
    content_accessibility_label: Option<SharedString>,
    inspector_accessibility_label: Option<SharedString>,
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
            sidebar_focus_group: None,
            content_focus_group: None,
            inspector_focus_group: None,
            sidebar_accessibility_label: None,
            content_accessibility_label: None,
            inspector_accessibility_label: None,
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

    /// Attach a caller-owned [`FocusGroup`] covering the sidebar pane's
    /// focusable children. Enables `Cmd-Opt-Left / Cmd-Opt-Right` pane
    /// jumps (see the module-level `# Keyboard` section). Must be in
    /// [`FocusGroupMode::Open`] — trips a `debug_assert` otherwise.
    pub fn sidebar_focus_group(mut self, group: FocusGroup) -> Self {
        debug_assert_eq!(
            group.mode(),
            FocusGroupMode::Open,
            "sidebar_focus_group requires an Open-mode FocusGroup",
        );
        self.sidebar_focus_group = Some(group);
        self
    }

    /// Attach a caller-owned [`FocusGroup`] covering the content pane's
    /// focusable children. Enables `Cmd-Opt-Left / Cmd-Opt-Right` pane
    /// jumps (see the module-level `# Keyboard` section). Must be in
    /// [`FocusGroupMode::Open`] — trips a `debug_assert` otherwise.
    pub fn content_focus_group(mut self, group: FocusGroup) -> Self {
        debug_assert_eq!(
            group.mode(),
            FocusGroupMode::Open,
            "content_focus_group requires an Open-mode FocusGroup",
        );
        self.content_focus_group = Some(group);
        self
    }

    /// Attach a caller-owned [`FocusGroup`] covering the inspector pane's
    /// focusable children. Enables `Cmd-Opt-Left / Cmd-Opt-Right` pane
    /// jumps (see the module-level `# Keyboard` section). Must be in
    /// [`FocusGroupMode::Open`] — trips a `debug_assert` otherwise.
    pub fn inspector_focus_group(mut self, group: FocusGroup) -> Self {
        debug_assert_eq!(
            group.mode(),
            FocusGroupMode::Open,
            "inspector_focus_group requires an Open-mode FocusGroup",
        );
        self.inspector_focus_group = Some(group);
        self
    }

    /// Override the sidebar pane's accessibility label (default: none).
    /// Supply a caller-chosen string (e.g. "Mailboxes", "Project
    /// navigator") so assistive tech announces the right semantic name.
    pub fn sidebar_accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.sidebar_accessibility_label = Some(label.into());
        self
    }

    /// Override the content pane's accessibility label (default: none).
    pub fn content_accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.content_accessibility_label = Some(label.into());
        self
    }

    /// Override the inspector pane's accessibility label (default: none).
    pub fn inspector_accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.inspector_accessibility_label = Some(label.into());
        self
    }
}

impl RenderOnce for NavigationSplitView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let separator_color = theme.border;
        let separator_thickness = theme.separator_thickness;

        let mut row = div().id(self.id).flex().flex_row().h_full().w_full();

        let sidebar_visible = self.sidebar.is_some() && !self.sidebar_collapsed;
        let inspector_visible = self.inspector.is_some() && self.inspector_visible;

        // Leading: sidebar (when present and not collapsed).
        if let (Some(sidebar), true) = (self.sidebar, sidebar_visible) {
            let mut pane = div().w(self.sidebar_width).h_full().child(sidebar);
            if let Some(label) = self.sidebar_accessibility_label {
                let props = AccessibilityProps::new()
                    .role(AccessibilityRole::Group)
                    .label(label);
                pane = pane.with_accessibility(&props);
            }
            row = row.child(pane);
            row = row.child(div().w(separator_thickness).h_full().bg(separator_color));
        }

        // Center: content always renders, filling the remaining width.
        let mut content_pane = div().flex_1().h_full().child(self.content);
        if let Some(label) = self.content_accessibility_label {
            let props = AccessibilityProps::new()
                .role(AccessibilityRole::Group)
                .label(label);
            content_pane = content_pane.with_accessibility(&props);
        }
        row = row.child(content_pane);

        // Trailing: inspector (when visible and an element was supplied).
        if let (Some(inspector), true) = (self.inspector, inspector_visible) {
            row = row.child(div().w(separator_thickness).h_full().bg(separator_color));
            let mut pane = div().w(self.inspector_width).h_full().child(inspector);
            if let Some(label) = self.inspector_accessibility_label {
                let props = AccessibilityProps::new()
                    .role(AccessibilityRole::Group)
                    .label(label);
                pane = pane.with_accessibility(&props);
            }
            row = row.child(pane);
        }

        // Pane navigation: Cmd-Opt-Left / Cmd-Opt-Right jumps between the
        // visible, non-empty pane groups (order: sidebar → content →
        // inspector). Hidden or empty panes are skipped. No-op when no
        // pane group contains focus or when no neighbor is reachable.
        let sidebar_group = self.sidebar_focus_group.filter(|_| sidebar_visible);
        let content_group = self.content_focus_group;
        let inspector_group = self.inspector_focus_group.filter(|_| inspector_visible);

        let any_group_configured =
            sidebar_group.is_some() || content_group.is_some() || inspector_group.is_some();

        if any_group_configured {
            row = row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let m = &event.keystroke.modifiers;
                if !(m.platform && m.alt) || m.shift || m.control {
                    return;
                }
                let direction: i32 = match event.keystroke.key.as_str() {
                    "left" => -1,
                    "right" => 1,
                    _ => return,
                };

                // Ordered list of `(group, visible, non_empty)` tuples
                // matches LTR pane order. We filter to the "jumpable" set
                // (visible + non-empty) and then walk forwards or
                // backwards from the currently focused pane.
                let panes: [Option<&FocusGroup>; 3] = [
                    sidebar_group.as_ref(),
                    content_group.as_ref(),
                    inspector_group.as_ref(),
                ];

                let jumpable: Vec<(usize, &FocusGroup)> = panes
                    .iter()
                    .enumerate()
                    .filter_map(|(i, g)| g.and_then(|g| (!g.is_empty()).then_some((i, g))))
                    .collect();

                let Some(current_idx) = jumpable
                    .iter()
                    .position(|(_, g)| g.contains_focused(window))
                else {
                    return;
                };

                let target_idx = current_idx as i32 + direction;
                if target_idx < 0 || target_idx as usize >= jumpable.len() {
                    return;
                }
                jumpable[target_idx as usize].1.focus_first(window, cx);
                cx.stop_propagation();
            });
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
        assert!(nsv.sidebar_focus_group.is_none());
        assert!(nsv.content_focus_group.is_none());
        assert!(nsv.inspector_focus_group.is_none());
        assert!(nsv.sidebar_accessibility_label.is_none());
        assert!(nsv.content_accessibility_label.is_none());
        assert!(nsv.inspector_accessibility_label.is_none());
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

    #[test]
    fn builder_stores_pane_focus_groups() {
        use crate::foundations::accessibility::{FocusGroup, FocusGroupMode};
        let nsv = NavigationSplitView::new("nsv", div())
            .sidebar_focus_group(FocusGroup::open())
            .content_focus_group(FocusGroup::open())
            .inspector_focus_group(FocusGroup::open());
        let sidebar = nsv.sidebar_focus_group.expect("sidebar group stored");
        let content = nsv.content_focus_group.expect("content group stored");
        let inspector = nsv.inspector_focus_group.expect("inspector group stored");
        assert_eq!(sidebar.mode(), FocusGroupMode::Open);
        assert_eq!(content.mode(), FocusGroupMode::Open);
        assert_eq!(inspector.mode(), FocusGroupMode::Open);
    }

    #[test]
    fn builder_stores_pane_accessibility_labels() {
        let nsv = NavigationSplitView::new("nsv", div())
            .sidebar_accessibility_label("Mailboxes")
            .content_accessibility_label("Messages")
            .inspector_accessibility_label("Details");
        assert_eq!(
            nsv.sidebar_accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Mailboxes")
        );
        assert_eq!(
            nsv.content_accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Messages")
        );
        assert_eq!(
            nsv.inspector_accessibility_label
                .as_ref()
                .map(|s| s.as_ref()),
            Some("Details")
        );
    }

    #[test]
    #[should_panic(expected = "Open-mode FocusGroup")]
    fn sidebar_focus_group_rejects_non_open_mode_in_debug() {
        use crate::foundations::accessibility::FocusGroup;
        let _ = NavigationSplitView::new("nsv", div()).sidebar_focus_group(FocusGroup::cycle());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    use super::NavigationSplitView;
    use crate::foundations::accessibility::{FocusGroup, FocusGroupExt};
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    /// Harness that owns three pane `FocusGroup`s with two handles each,
    /// rendered inside a `NavigationSplitView`. The `sidebar_collapsed` and
    /// `inspector_visible` flags let tests verify hidden-pane skipping.
    struct NsvHarness {
        sidebar_handles: [FocusHandle; 2],
        content_handles: [FocusHandle; 2],
        inspector_handles: [FocusHandle; 2],
        sidebar_group: FocusGroup,
        content_group: FocusGroup,
        inspector_group: FocusGroup,
        sidebar_collapsed: bool,
        inspector_visible: bool,
    }

    impl NsvHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                sidebar_handles: [cx.focus_handle(), cx.focus_handle()],
                content_handles: [cx.focus_handle(), cx.focus_handle()],
                inspector_handles: [cx.focus_handle(), cx.focus_handle()],
                sidebar_group: FocusGroup::open(),
                content_group: FocusGroup::open(),
                inspector_group: FocusGroup::open(),
                sidebar_collapsed: false,
                inspector_visible: true,
            }
        }
    }

    impl Render for NsvHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            NavigationSplitView::new(
                "nsv",
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .id("content-0")
                            .focus_group(&self.content_group, &self.content_handles[0])
                            .child("C0"),
                    )
                    .child(
                        div()
                            .id("content-1")
                            .focus_group(&self.content_group, &self.content_handles[1])
                            .child("C1"),
                    ),
            )
            .sidebar(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .id("sidebar-0")
                            .focus_group(&self.sidebar_group, &self.sidebar_handles[0])
                            .child("S0"),
                    )
                    .child(
                        div()
                            .id("sidebar-1")
                            .focus_group(&self.sidebar_group, &self.sidebar_handles[1])
                            .child("S1"),
                    ),
            )
            .inspector(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .id("inspector-0")
                            .focus_group(&self.inspector_group, &self.inspector_handles[0])
                            .child("I0"),
                    )
                    .child(
                        div()
                            .id("inspector-1")
                            .focus_group(&self.inspector_group, &self.inspector_handles[1])
                            .child("I1"),
                    ),
            )
            .sidebar_width(px(200.0))
            .inspector_width(px(200.0))
            .sidebar_collapsed(self.sidebar_collapsed)
            .inspector_visible(self.inspector_visible)
            .sidebar_focus_group(self.sidebar_group.clone())
            .content_focus_group(self.content_group.clone())
            .inspector_focus_group(self.inspector_group.clone())
        }
    }

    #[gpui::test]
    async fn cmd_alt_right_from_sidebar_jumps_to_content(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| NsvHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.sidebar_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.content_handles[0].is_focused(window),
                "cmd-alt-right from sidebar → content[0]"
            );
        });
    }

    #[gpui::test]
    async fn cmd_alt_right_from_content_jumps_to_inspector(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| NsvHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.content_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.inspector_handles[0].is_focused(window),
                "cmd-alt-right from content → inspector[0]"
            );
        });
    }

    #[gpui::test]
    async fn cmd_alt_left_from_inspector_jumps_to_content(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| NsvHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.inspector_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.content_handles[0].is_focused(window),
                "cmd-alt-left from inspector → content[0]"
            );
        });
    }

    #[gpui::test]
    async fn cmd_alt_left_from_content_jumps_to_sidebar(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| NsvHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.content_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.sidebar_handles[0].is_focused(window),
                "cmd-alt-left from content → sidebar[0]"
            );
        });
    }

    #[gpui::test]
    async fn cmd_alt_right_at_inspector_edge_is_noop(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| NsvHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.inspector_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.inspector_handles[0].is_focused(window),
                "cmd-alt-right on rightmost pane is a no-op"
            );
        });
    }

    #[gpui::test]
    async fn cmd_alt_left_at_sidebar_edge_is_noop(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| NsvHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.sidebar_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.sidebar_handles[0].is_focused(window),
                "cmd-alt-left on leftmost pane is a no-op"
            );
        });
    }

    #[gpui::test]
    async fn collapsed_sidebar_is_skipped(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            let mut h = NsvHarness::new(cx);
            h.sidebar_collapsed = true;
            h
        });
        // Content is the leftmost visible pane; cmd-alt-left should be a
        // no-op because the collapsed sidebar is skipped.
        host.update_in(cx, |host, window, cx| {
            host.content_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.content_handles[0].is_focused(window),
                "cmd-alt-left with collapsed sidebar is a no-op"
            );
        });
    }

    #[gpui::test]
    async fn hidden_inspector_is_skipped(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            let mut h = NsvHarness::new(cx);
            h.inspector_visible = false;
            h
        });
        host.update_in(cx, |host, window, cx| {
            host.content_handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.content_handles[0].is_focused(window),
                "cmd-alt-right with hidden inspector is a no-op"
            );
        });
    }

    #[gpui::test]
    async fn plain_arrow_not_consumed(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| NsvHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.content_handles[0].focus(window, cx);
        });
        // Plain left (no modifiers) should not trigger pane navigation.
        cx.press("left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.content_handles[0].is_focused(window),
                "plain left without cmd-alt does not jump panes"
            );
        });
    }
}
