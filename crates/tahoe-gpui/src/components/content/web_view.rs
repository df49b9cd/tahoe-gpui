//! HIG Web view component.
//!
//! Stateless `RenderOnce` wrapper that reserves a region of the window for an
//! embedded browser surface. GPUI (crate version 0.2.2) does not expose a
//! managed native view embedding API yet, so the visible output is a
//! placeholder frame that honours the HIG Web views contract:
//!
//! - a chrome-free rectangle with the theme's content surface fill,
//! - an activity indicator or determinate progress bar while loading,
//! - a caption and accessibility label announcing the embedded URL, and
//! - a clear visual state for load failures.
//!
//! When GPUI gains a native web-view element (see
//! <https://github.com/zed-industries/zed/pull/54433>, which uses
//! the `wry` crate to embed WKWebView / WebKitGTK), the same builder can
//! mount a real surface without an API break — callers already supply the
//! URL and loading state.
//!
//! # Platform support
//!
//! Web views are supported on macOS and iOS/iPadOS per the HIG. They are
//! **not supported** on tvOS or watchOS. This component renders a
//! placeholder on all platforms; the actual native surface is the host's
//! responsibility.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/web-views>

use gpui::prelude::*;
use gpui::{App, Pixels, SharedString, Window, div};

use crate::components::status::activity_indicator::ActivityIndicator;
use crate::components::status::progress_indicator::ProgressIndicator;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::patterns::loading::LoadingState;

/// Embedded web view placeholder.
///
/// Renders a chrome-free rectangle suitable for hosting web content. The
/// builder collects the URL, loading state, and navigation policy so a
/// future native surface can be wired without an API break.
#[derive(IntoElement)]
pub struct WebView {
    url: SharedString,
    width: Option<Pixels>,
    height: Option<Pixels>,
    loading_state: LoadingState,
    allow_navigation: bool,
    accessibility_label: Option<SharedString>,
}

impl WebView {
    /// Create a new web view pointed at `url`.
    pub fn new(url: impl Into<SharedString>) -> Self {
        Self {
            url: url.into(),
            width: None,
            height: None,
            loading_state: LoadingState::Idle,
            allow_navigation: true,
            accessibility_label: None,
        }
    }

    /// Set the placeholder dimensions in pixels.
    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set the placeholder width in pixels.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the placeholder height in pixels.
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Set the full loading state machine (idle, loading, loading with
    /// progress, loaded, failed).
    pub fn loading_state(mut self, state: LoadingState) -> Self {
        self.loading_state = state;
        self
    }

    /// Convenience: set loading to `true` (indeterminate spinner) or
    /// `false` (resets to [`LoadingState::Idle`], **not** Loaded).
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading_state = if loading {
            LoadingState::Loading
        } else {
            LoadingState::Idle
        };
        self
    }

    /// Toggle navigation allowance. When `false`, the placeholder
    /// announces that navigation is disabled and a future native surface
    /// should block link navigation (WKWebView's
    /// `decidePolicyForNavigationAction`).
    pub fn allow_navigation(mut self, allow: bool) -> Self {
        self.allow_navigation = allow;
        self
    }

    /// Override the auto-generated accessibility label.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

impl RenderOnce for WebView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let a11y_label: SharedString =
            self.accessibility_label
                .clone()
                .unwrap_or_else(|| match self.loading_state {
                    LoadingState::Failed => {
                        SharedString::from(format!("Web view, failed to load {}", self.url))
                    }
                    LoadingState::Loading | LoadingState::LoadingWithProgress(_) => {
                        SharedString::from(format!("Web view, loading {}", self.url))
                    }
                    _ => match self.allow_navigation {
                        true => SharedString::from(format!("Web view, {}", self.url)),
                        false => SharedString::from(format!(
                            "Web view, {} (navigation disabled)",
                            self.url
                        )),
                    },
                });
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Group);

        let caption = match self.loading_state {
            LoadingState::Failed => SharedString::from(format!("Failed to load {}", self.url)),
            LoadingState::Idle | LoadingState::Loaded => {
                SharedString::from(format!("Web content: {}", self.url))
            }
            LoadingState::Loading | LoadingState::LoadingWithProgress(_) => {
                SharedString::from(format!("Loading {}…", self.url))
            }
        };

        let is_failed = matches!(self.loading_state, LoadingState::Failed);

        let mut frame = div()
            .when_some(self.width, |el, w| el.w(w))
            .when_some(self.height, |el, h| el.h(h))
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(if is_failed { theme.error } else { theme.border })
            .overflow_hidden()
            .relative()
            .with_accessibility(&a11y_props)
            .child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .px(theme.spacing_md)
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(if is_failed {
                        theme.error
                    } else {
                        theme.text_muted
                    })
                    .child(caption),
            );

        match self.loading_state {
            LoadingState::Loading => {
                frame = frame.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            ActivityIndicator::new("web-view-loading")
                                .label(format!("Loading {}", self.url)),
                        ),
                );
            }
            LoadingState::LoadingWithProgress(_) => {
                if let Some(fraction) = self.loading_state.progress() {
                    frame = frame.child(
                        div()
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                ProgressIndicator::new(fraction)
                                    .label(format!("Loading {}", self.url)),
                            ),
                    );
                }
            }
            _ => {}
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::WebView;
    use crate::patterns::loading::LoadingState;
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn web_view_defaults() {
        let wv = WebView::new("https://apple.com");
        assert_eq!(wv.url.as_ref(), "https://apple.com");
        assert!(wv.width.is_none());
        assert!(wv.height.is_none());
        assert_eq!(wv.loading_state, LoadingState::Idle);
        assert!(wv.allow_navigation);
        assert!(wv.accessibility_label.is_none());
    }

    #[test]
    fn web_view_size_builder() {
        let wv = WebView::new("x").size(px(640.0), px(480.0));
        assert_eq!(wv.width, Some(px(640.0)));
        assert_eq!(wv.height, Some(px(480.0)));
    }

    #[test]
    fn web_view_loading_bool_builder() {
        let wv = WebView::new("x").loading(true);
        assert_eq!(wv.loading_state, LoadingState::Loading);

        let wv = WebView::new("x").loading(false);
        assert_eq!(wv.loading_state, LoadingState::Idle);
    }

    #[test]
    fn web_view_loading_state_builder() {
        let wv = WebView::new("x").loading_state(LoadingState::LoadingWithProgress(0.5));
        assert_eq!(wv.loading_state, LoadingState::LoadingWithProgress(0.5));
    }

    #[test]
    fn web_view_failed_state() {
        let wv = WebView::new("x").loading_state(LoadingState::Failed);
        assert!(wv.loading_state.is_failed());
    }

    #[test]
    fn web_view_loaded_state() {
        let wv = WebView::new("x").loading_state(LoadingState::Loaded);
        assert_eq!(wv.loading_state, LoadingState::Loaded);
    }

    #[test]
    fn web_view_allow_navigation_builder() {
        let wv = WebView::new("x").allow_navigation(false);
        assert!(!wv.allow_navigation);
    }

    #[test]
    fn web_view_accessibility_label_builder() {
        let wv = WebView::new("x").accessibility_label("Help content");
        assert_eq!(
            wv.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Help content")
        );
    }
}
