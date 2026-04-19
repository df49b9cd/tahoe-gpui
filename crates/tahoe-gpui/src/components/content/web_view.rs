//! HIG Web view component.
//!
//! Stateless `RenderOnce` wrapper that reserves a region of the window for
//! a `WKWebView`-backed surface on macOS (or an equivalent embedded
//! browser on other platforms). GPUI does not yet expose a public foreign
//! view API in `0.231.1-pre`, so the visible output is a placeholder
//! frame that honours the HIG Web views contract:
//!
//! - a chrome-free rectangle with the theme's content surface fill,
//! - an `ActivityIndicator` overlay while `loading` is `true`,
//! - an accessibility label announcing the embedded URL, and
//! - a visible caption telling the user the web surface is not yet
//!   rendered (so no app ships a silent empty area that looks broken).
//!
//! When GPUI lands `ForeignView` the same builder can mount a real
//! `WKWebView` without an API break — callers already supply the URL and
//! loading state.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/web-views>

use gpui::prelude::*;
use gpui::{App, Pixels, SharedString, Window, div, px};

use crate::components::status::activity_indicator::ActivityIndicator;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// Embedded web view.
#[derive(IntoElement)]
pub struct WebView {
    url: SharedString,
    width: Option<Pixels>,
    height: Option<Pixels>,
    loading: bool,
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
            loading: false,
            allow_navigation: true,
            accessibility_label: None,
        }
    }

    pub fn size(mut self, width: Pixels, height: Pixels) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Show the activity indicator overlay. Callers set this while the
    /// underlying `WKWebView` is loading.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Toggle navigation allowance. HIG requires web views either allow
    /// navigation fully or disable interactive links entirely — the
    /// field is surfaced so the host wiring can forward the decision to
    /// `WKWebView`'s `decidePolicyForNavigationAction` once the surface
    /// is live.
    pub fn allow_navigation(mut self, allow: bool) -> Self {
        self.allow_navigation = allow;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

impl RenderOnce for WebView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let width = self.width.unwrap_or(px(480.0));
        let height = self.height.unwrap_or(px(320.0));

        let a11y_label: SharedString = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| SharedString::from(format!("Web view, {}", self.url)));
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Group);

        let caption = if self.loading {
            format!("Loading {}…", self.url)
        } else {
            // Per HIG "Don't create a fake native UI" — we surface the
            // URL verbatim rather than rendering a spoofed browser chrome.
            format!("Web content: {}", self.url)
        };

        let mut frame = div()
            .w(width)
            .h(height)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border)
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
                    .text_color(theme.text_muted)
                    .child(SharedString::from(caption)),
            );

        if self.loading {
            frame = frame.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(ActivityIndicator::new("web-view-loading")),
            );
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::WebView;
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn web_view_defaults() {
        let wv = WebView::new("https://apple.com");
        assert_eq!(wv.url.as_ref(), "https://apple.com");
        assert!(wv.width.is_none());
        assert!(wv.height.is_none());
        assert!(!wv.loading);
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
    fn web_view_loading_builder() {
        let wv = WebView::new("x").loading(true);
        assert!(wv.loading);
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
