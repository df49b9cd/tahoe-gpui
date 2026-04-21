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
use crate::foundations::layout::Platform;
use crate::foundations::materials::{apply_focus_ring, resolve_focused};
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
    /// Optional host-supplied focus handle. Precedence rules live on
    /// [`resolve_focused`](crate::foundations::materials::resolve_focused):
    /// when set, the focus-ring derives from `handle.is_focused(window)`
    /// and the root element threads `track_focus(&handle)`.
    focus_handle: Option<FocusHandle>,
    on_change: OnUsizeChange,
}

impl PageControls {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            total: 0,
            current: 0,
            focused: false,
            focus_handle: None,
            on_change: None,
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

    /// Show a focus ring around the dot row when keyboard-focused.
    /// Ignored when a [`focus_handle`](Self::focus_handle) is also attached
    /// — the handle's live `is_focused(window)` state wins.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the page controls participate in the
    /// host's focus graph. Takes precedence over [`focused`](Self::focused)
    /// per [`resolve_focused`].
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PageControls {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let focused = resolve_focused(self.focus_handle.as_ref(), window, self.focused);

        // Clamp current index to valid range.
        let current = if self.total > 0 {
            self.current.min(self.total - 1)
        } else {
            0
        };

        let on_change = rc_wrap(self.on_change);

        let dot_size = px(8.0);
        let touch_size = px(theme.target_size());

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

        if let Some(handle) = self.focus_handle.as_ref() {
            dot_row = dot_row.track_focus(handle);
        }

        dot_row = apply_focus_ring(dot_row, theme, focused, &[]);

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

            dot_row = dot_row.child(touch_target);
        }

        if let Some(handler) = on_change {
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
    use core::prelude::v1::test;

    #[test]
    fn page_controls_defaults() {
        let pc = PageControls::new("test");
        assert_eq!(pc.total, 0);
        assert_eq!(pc.current, 0);
        assert!(pc.on_change.is_none());
        assert!(!pc.focused);
        assert!(pc.focus_handle.is_none());
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
        use crate::foundations::layout::Platform;
        assert!(!super::page_controls_supported_on(Platform::MacOS));
        assert!(super::page_controls_supported_on(Platform::IOS));
        assert!(super::page_controls_supported_on(Platform::TvOS));
        assert!(super::page_controls_supported_on(Platform::VisionOS));
        assert!(super::page_controls_supported_on(Platform::WatchOS));
    }
}
