//! HIG Page Controls — pagination dots.
//!
//! A stateless `RenderOnce` component that renders a row of indicator dots
//! for paging through content. The active page is highlighted with the accent
//! color; inactive dots use the tertiary label color for WCAG-compliant
//! contrast against both light and dark appearances.
//!
//! # Platform notes
//!
//! HIG `#page-controls` explicitly marks page controls as
//! **not supported on macOS**:
//!
//! > "Not supported in macOS."
//!
//! This component is intended for iOS, iPadOS, tvOS, visionOS, and
//! watchOS callers. When rendering on macOS, prefer a
//! [`crate::components::navigation_and_search::tab_bar::TabBar`] or
//! a segmented control. The component does not assert its platform at
//! runtime — callers that target macOS should gate the component
//! themselves (e.g. `if theme.platform != Platform::MacOS { … }`).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/page-controls>

use crate::callback_types::{OnUsizeChange, rc_wrap};
use crate::foundations::accessibility::{FocusGroup, FocusGroupExt};
use crate::foundations::layout::Platform;
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::{App, ElementId, FocusHandle, KeyDownEvent, SharedString, Window, div, px};

/// Returns `true` when page controls are supported on the given
/// platform per HIG `#page-controls`. Returns `false` for macOS (HIG:
/// "Not supported in macOS.").
pub fn page_controls_supported_on(platform: Platform) -> bool {
    !matches!(platform, Platform::MacOS)
}

/// Pagination dot indicator following HIG.
///
/// Renders a centered flex row of clickable circular dots. Each dot has a
/// 44x44pt touch target wrapping an 8px visible circle.
#[derive(IntoElement)]
pub struct PageControls {
    id: ElementId,
    total: usize,
    current: usize,
    focused: bool,
    on_change: OnUsizeChange,
    /// Host-owned focus group used when
    /// `AccessibilityMode::FULL_KEYBOARD_ACCESS` is active. When paired
    /// with [`PageControls::dot_focus_handles`], each dot becomes its
    /// own Tab stop.
    dot_focus_group: Option<FocusGroup>,
    /// Host-owned per-dot focus handles. Expected to hold exactly one
    /// handle per dot (i.e. `total` handles). Ignored unless FKA is on
    /// and the group is also supplied.
    dot_focus_handles: Vec<FocusHandle>,
}

impl PageControls {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            total: 0,
            current: 0,
            focused: false,
            on_change: None,
            dot_focus_group: None,
            dot_focus_handles: Vec::new(),
        }
    }

    pub fn total(mut self, total: usize) -> Self {
        self.total = total;
        self
    }

    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Attach a host-owned [`FocusGroup`] for per-dot arrow-nav and
    /// Tab-reachability under macOS Full Keyboard Access. When paired
    /// with [`PageControls::dot_focus_handles`] and the active theme
    /// reports FKA, each dot becomes its own Tab stop. Use
    /// [`FocusGroup::open`] so Tab still exits the control naturally.
    pub fn dot_focus_group(mut self, group: FocusGroup) -> Self {
        self.dot_focus_group = Some(group);
        self
    }

    /// Per-dot [`FocusHandle`]s. Expected to hold exactly one handle per
    /// dot (i.e. `total` handles), in dot order. Host-owned because
    /// `PageControls` is a stateless `RenderOnce` component and cannot
    /// persist handles across renders.
    pub fn dot_focus_handles(mut self, handles: Vec<FocusHandle>) -> Self {
        self.dot_focus_handles = handles;
        self
    }
}

impl RenderOnce for PageControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // Clamp current index to valid range.
        let current = if self.total > 0 {
            self.current.min(self.total - 1)
        } else {
            0
        };

        let dot_size = px(8.0);
        let touch_size = px(theme.target_size());

        // FKA: only attach per-dot focus when the flag is set AND the
        // host supplied both a FocusGroup and exactly one handle per dot.
        let fka_dots = FocusGroup::bind_if_fka(
            theme.full_keyboard_access(),
            self.dot_focus_group,
            self.dot_focus_handles,
            self.total,
        );

        let on_change = rc_wrap(self.on_change);

        // Inactive color: tertiary label — HIG's third-level label
        // hierarchy is tuned to remain legible as a non-primary
        // indicator on both light and dark backgrounds, clearing WCAG
        // 3:1 for non-text UI. Earlier the inactive dot was
        // `theme.text_muted * 0.3` which composited too lightly on
        // light backgrounds and fell below the non-text contrast floor.
        let inactive_color = theme.text_tertiary();

        // Inner dot row: tightly wraps dots so the focus ring hugs them.
        let mut dot_row = div()
            .id(self.id)
            .focusable()
            .flex()
            .flex_shrink_0()
            .items_center()
            .gap(theme.spacing_xs)
            .rounded(theme.radius_md)
            .px(theme.spacing_xs)
            .py(theme.spacing_xs);

        dot_row = apply_focus_ring(dot_row, theme, self.focused, &[]);

        for i in 0..self.total {
            let is_active = i == current;
            let dot_color = if is_active {
                theme.accent
            } else {
                inactive_color
            };

            let dot = div()
                .w(dot_size)
                .h(dot_size)
                .rounded(dot_size)
                .bg(dot_color);

            // Wrap dot in a 44x44 touch target.
            let handler = on_change.clone();
            let mut touch_target = div()
                .id(ElementId::from(SharedString::from(format!("page-dot-{i}"))))
                .w(touch_size)
                .h(touch_size)
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .child(dot);

            if let Some(handler) = handler {
                touch_target = touch_target.on_click(move |_event, window, cx| {
                    handler(i, window, cx);
                });
            }

            // FKA: attach per-dot focus + arrow-nav + focus ring. When
            // the per-dot focus is active, Left/Right call the group's
            // focus_previous/focus_next to walk the dot chain. Home/End
            // jump to the first/last dot. Enter/Space activates via
            // the click handler above.
            if let Some((group, handles)) = fka_dots.as_ref() {
                let handle = &handles[i];
                let is_focused = handle.is_focused(window);
                touch_target = touch_target.focus_group(group, handle);
                touch_target = apply_focus_ring(touch_target, theme, is_focused, &[]);
                let nav_group = group.clone();
                let nav_change = on_change.clone();
                let dot_idx = i;
                touch_target = touch_target.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                    match ev.keystroke.key.as_str() {
                        "left" => {
                            nav_group.focus_previous(window, cx);
                            cx.stop_propagation();
                        }
                        "right" => {
                            nav_group.focus_next(window, cx);
                            cx.stop_propagation();
                        }
                        "home" => {
                            nav_group.focus_first(window, cx);
                            cx.stop_propagation();
                        }
                        "end" => {
                            nav_group.focus_last(window, cx);
                            cx.stop_propagation();
                        }
                        _ => {
                            if crate::foundations::keyboard::is_activation_key(ev)
                                && let Some(ref h) = nav_change
                            {
                                h(dot_idx, window, cx);
                                cx.stop_propagation();
                            }
                        }
                    }
                });
            }

            dot_row = dot_row.child(touch_target);
        }

        // Row-level arrow-nav: kept as a fallback when FKA is off (no
        // per-dot handles). Calls `on_change` directly to move the
        // selected dot.
        if fka_dots.is_none()
            && let Some(handler) = on_change
        {
            let total = self.total;
            dot_row = dot_row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "left" => {
                        let new_index = current.saturating_sub(1);
                        if new_index != current {
                            handler(new_index, window, cx);
                        }
                    }
                    "right" => {
                        let new_index = if total > 0 {
                            (current + 1).min(total - 1)
                        } else {
                            0
                        };
                        if new_index != current {
                            handler(new_index, window, cx);
                        }
                    }
                    _ => {}
                }
            });
        }

        // Outer container centers the dot row without expanding the focus ring.
        div().flex().items_center().justify_center().child(dot_row)
    }
}

#[cfg(test)]
mod tests {
    use super::PageControls;
    use crate::foundations::accessibility::{AccessibilityMode, FocusGroup};
    use crate::foundations::layout::Platform;
    use crate::foundations::theme::TahoeTheme;
    use crate::test_helpers::helpers::setup_test_window;
    use core::prelude::v1::test;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, Window};

    const DOT_COUNT: usize = 5;

    #[test]
    fn page_controls_defaults() {
        let pc = PageControls::new("test");
        assert_eq!(pc.total, 0);
        assert_eq!(pc.current, 0);
        assert!(pc.on_change.is_none());
    }

    #[test]
    fn total_builder() {
        let pc = PageControls::new("test").total(5);
        assert_eq!(pc.total, 5);
    }

    #[test]
    fn current_builder() {
        let pc = PageControls::new("test").current(3);
        assert_eq!(pc.current, 3);
    }

    #[test]
    fn on_change_is_some() {
        let pc = PageControls::new("test").on_change(|_, _, _| {});
        assert!(pc.on_change.is_some());
    }

    #[test]
    fn current_clamped_to_total() {
        // Verify the clamping logic used during render.
        let total: usize = 3;
        let current: usize = 10;
        let clamped = if total > 0 { current.min(total - 1) } else { 0 };
        assert_eq!(clamped, 2);
    }

    #[test]
    fn current_clamped_zero_total() {
        let total: usize = 0;
        let current: usize = 5;
        let clamped = if total > 0 { current.min(total - 1) } else { 0 };
        assert_eq!(clamped, 0);
    }

    #[test]
    fn page_controls_unsupported_on_macos() {
        assert!(!super::page_controls_supported_on(Platform::MacOS));
        assert!(super::page_controls_supported_on(Platform::IOS));
        assert!(super::page_controls_supported_on(Platform::TvOS));
        assert!(super::page_controls_supported_on(Platform::VisionOS));
        assert!(super::page_controls_supported_on(Platform::WatchOS));
    }

    #[test]
    fn dot_focus_fields_default_empty() {
        let pc = PageControls::new("test");
        assert!(pc.dot_focus_group.is_none());
        assert!(pc.dot_focus_handles.is_empty());
    }

    // ─── HIG: Full Keyboard Access ───────────────────────────────────

    struct PageFkaHarness {
        handles: Vec<FocusHandle>,
        group: FocusGroup,
    }

    impl PageFkaHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                handles: (0..DOT_COUNT).map(|_| cx.focus_handle()).collect(),
                group: FocusGroup::open(),
            }
        }
    }

    impl Render for PageFkaHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            PageControls::new("page-fka-test")
                .total(DOT_COUNT)
                .current(2)
                .dot_focus_group(self.group.clone())
                .dot_focus_handles(self.handles.clone())
        }
    }

    #[gpui::test]
    async fn fka_off_does_not_register_dot_handles(cx: &mut TestAppContext) {
        let (host, _cx) = setup_test_window(cx, |_window, cx| PageFkaHarness::new(cx));
        host.update(cx, |host, _cx| {
            assert!(host.group.is_empty());
        });
    }

    #[gpui::test]
    async fn fka_on_registers_one_focus_per_dot(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PageFkaHarness::new(cx)
        });
        host.update(vcx, |host, _cx| {
            assert_eq!(host.group.len(), DOT_COUNT);
        });
    }

    #[gpui::test]
    async fn fka_on_preserves_registration_order(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PageFkaHarness::new(cx)
        });
        host.update(vcx, |host, _cx| {
            for (i, handle) in host.handles.iter().enumerate() {
                assert_eq!(host.group.register(handle), i);
            }
        });
    }

    #[gpui::test]
    async fn fka_on_mismatched_handle_count_skips_registration(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            PageFkaHarness {
                group: FocusGroup::open(),
                handles: vec![cx.focus_handle()], // 1 for 5 dots
            }
        });
        host.update(vcx, |host, _cx| {
            assert!(host.group.is_empty());
        });
    }

    // Harness with total=0: the `bind_if_fka` gate must short-circuit
    // even with FKA on and a group present — an empty control has no
    // per-dot Tab stops to register.
    struct ZeroDotHarness {
        group: FocusGroup,
    }

    impl Render for ZeroDotHarness {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            PageControls::new("page-fka-zero")
                .total(0)
                .dot_focus_group(self.group.clone())
                .dot_focus_handles(Vec::new())
        }
    }

    #[gpui::test]
    async fn fka_on_with_zero_total_skips_registration(cx: &mut TestAppContext) {
        let (host, vcx) = cx.add_window_view(|_window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            ZeroDotHarness {
                group: FocusGroup::open(),
            }
        });
        host.update(vcx, |host, _cx| {
            assert!(
                host.group.is_empty(),
                "total=0 must short-circuit the FKA gate"
            );
        });
    }
}
