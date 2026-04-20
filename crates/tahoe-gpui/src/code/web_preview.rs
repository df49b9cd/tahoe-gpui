//! Web preview component with compound sub-components.
//!
//! Provides a browser-like preview frame with navigation bar, URL display,
//! placeholder body, and console panel. Since GPUI has no iframe/webview,
//! the body shows a placeholder with an "Open in Browser" fallback.
//!
//! # Convenience builder
//! ```ignore
//! WebPreview::new("https://example.com")
//!     .title("My App")
//!     .console_log(ConsoleEntry {
//!         level: ConsoleLevel::Log,
//!         message: "loaded".into(),
//!         timestamp: Some("12:00:00".into()),
//!     })
//! ```
//!
//! # Compound composition
//! ```ignore
//! WebPreview::new("https://example.com")
//!     .nav(
//!         WebPreviewNavigation::new()
//!             .child(WebPreviewNavigationButton::back())
//!             .child(WebPreviewNavigationButton::forward())
//!             .child(WebPreviewUrl::new("https://example.com")),
//!     )
//!     .body(WebPreviewBody::new("https://example.com"))
//!     .console_panel(WebPreviewConsole::new(logs))
//! ```

use crate::components::content::badge::{Badge, BadgeVariant};
use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, SharedString, Window, div, px};

// -- ConsoleLevel / ConsoleEntry -----------------------------------------------

/// Console log severity level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConsoleLevel {
    Log,
    Warn,
    Error,
}

impl ConsoleLevel {
    fn label(self) -> &'static str {
        match self {
            Self::Log => "log",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    fn badge_variant(self) -> BadgeVariant {
        match self {
            Self::Log => BadgeVariant::Muted,
            Self::Warn => BadgeVariant::Warning,
            Self::Error => BadgeVariant::Error,
        }
    }
}

/// A single console log entry.
#[derive(Debug, Clone)]
pub struct ConsoleEntry {
    pub level: ConsoleLevel,
    pub message: SharedString,
    pub timestamp: Option<SharedString>,
}

// -- WebPreviewConsole --------------------------------------------------------

/// Console output panel displaying log entries with severity badges.
#[derive(IntoElement)]
pub struct WebPreviewConsole {
    logs: Vec<ConsoleEntry>,
}

impl WebPreviewConsole {
    pub fn new(logs: Vec<ConsoleEntry>) -> Self {
        Self { logs }
    }

    /// Append a single log entry.
    pub fn log(mut self, entry: ConsoleEntry) -> Self {
        self.logs.push(entry);
        self
    }
}

impl RenderOnce for WebPreviewConsole {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut container = div()
            .id("web-preview-console")
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.code_bg)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .gap(px(2.0))
            .font(theme.font_mono())
            .text_style(TextStyle::Caption1, theme)
            .max_h(px(150.0))
            .overflow_y_scroll();

        if self.logs.is_empty() {
            container = container.child(
                div()
                    .text_color(theme.text_muted)
                    .child("No console output"),
            );
        } else {
            for entry in self.logs {
                let mut row = div().flex().items_center().gap(theme.spacing_sm);

                row =
                    row.child(Badge::new(entry.level.label()).variant(entry.level.badge_variant()));

                if let Some(ts) = entry.timestamp {
                    row = row.child(div().text_color(theme.text_muted).child(ts));
                }

                row = row.child(div().flex_1().text_color(theme.text).child(entry.message));

                container = container.child(row);
            }
        }

        container
    }
}

// -- WebPreviewNavigationButton -----------------------------------------------

/// A navigation button (back/forward) for the web preview navigation bar.
#[derive(IntoElement)]
pub struct WebPreviewNavigationButton {
    id: ElementId,
    icon: IconName,
    tooltip: SharedString,
    disabled: bool,
}

impl WebPreviewNavigationButton {
    pub fn new(id: impl Into<ElementId>, icon: IconName, tooltip: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon,
            tooltip: tooltip.into(),
            disabled: true,
        }
    }

    /// Create a "Go back" navigation button.
    pub fn back() -> Self {
        Self::new("web-preview-back", IconName::ChevronLeft, "Go back")
    }

    /// Create a "Go forward" navigation button.
    pub fn forward() -> Self {
        Self::new("web-preview-forward", IconName::ChevronRight, "Go forward")
    }

    /// Set the disabled state (default: `true`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for WebPreviewNavigationButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        Button::new(self.id)
            .icon(Icon::new(self.icon).size(theme.icon_size_inline))
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSmall)
            .tooltip(self.tooltip)
            .disabled(self.disabled)
    }
}

// -- WebPreviewUrl ------------------------------------------------------------

/// URL display bar for the web preview navigation.
#[derive(IntoElement)]
pub struct WebPreviewUrl {
    url: SharedString,
}

impl WebPreviewUrl {
    pub fn new(url: impl Into<SharedString>) -> Self {
        Self { url: url.into() }
    }
}

impl RenderOnce for WebPreviewUrl {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_1()
            .items_center()
            .gap(theme.spacing_xs)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .px(theme.spacing_sm)
            .py(theme.spacing_xs)
            .child(
                Icon::new(IconName::Globe)
                    .size(px(12.0))
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .flex_1()
                    .font(theme.font_mono())
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.accent)
                    .overflow_x_hidden()
                    .child(self.url),
            )
    }
}

// -- WebPreviewNavigation -----------------------------------------------------

/// Navigation bar container for the web preview.
#[derive(Default, IntoElement)]
pub struct WebPreviewNavigation {
    children: Vec<AnyElement>,
}

impl WebPreviewNavigation {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
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

impl RenderOnce for WebPreviewNavigation {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .border_b_1()
            .border_color(theme.border)
            .children(self.children)
    }
}

// -- WebPreviewBody -----------------------------------------------------------

/// Preview body area. Since GPUI has no iframe/webview, this renders a
/// placeholder with an "Open in Browser" action.
#[derive(IntoElement)]
pub struct WebPreviewBody {
    url: SharedString,
    loading: Option<AnyElement>,
}

impl WebPreviewBody {
    pub fn new(url: impl Into<SharedString>) -> Self {
        Self {
            url: url.into(),
            loading: None,
        }
    }

    /// Set a loading indicator to show instead of the placeholder.
    pub fn loading(mut self, element: impl IntoElement) -> Self {
        self.loading = Some(element.into_any_element());
        self
    }
}

impl RenderOnce for WebPreviewBody {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        if let Some(loading) = self.loading {
            return div()
                .flex()
                .items_center()
                .justify_center()
                .min_h(px(200.0))
                .child(loading)
                .into_any_element();
        }

        let url = self.url;

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(theme.spacing_md)
            .min_h(px(200.0))
            .child(
                Icon::new(IconName::Globe)
                    .size(px(32.0))
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child("Web preview is not available in desktop mode"),
            )
            .child(
                Button::new("open-body")
                    .label("Open in Browser")
                    .icon(Icon::new(IconName::ExternalLink).size(theme.icon_size_inline))
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Small)
                    .on_click(move |_event, _window, cx| {
                        cx.open_url(url.as_ref());
                    }),
            )
            .into_any_element()
    }
}

// -- WebPreview ---------------------------------------------------------------

/// A browser-like preview frame with navigation, body, and console.
///
/// Supports both convenience and compound usage:
/// - **Convenience**: `WebPreview::new(url).title(...).console_log(...)`
/// - **Compound**: `WebPreview::new(url).nav(...).body(...).console_panel(...)`
#[derive(IntoElement)]
pub struct WebPreview {
    url: SharedString,
    title: Option<SharedString>,
    // Convenience API fields
    show_navigation: bool,
    console_logs: Vec<ConsoleEntry>,
    // Compound API fields
    navigation: Option<WebPreviewNavigation>,
    body: Option<WebPreviewBody>,
    console: Option<WebPreviewConsole>,
}

impl WebPreview {
    /// Create a new web preview for the given URL.
    ///
    /// By default, renders a navigation bar, placeholder body, and no console.
    /// Use compound setters (`.nav()`, `.body()`, `.console_panel()`) to
    /// override individual sections, or convenience methods (`.navigation()`,
    /// `.console_log()`) to tweak defaults.
    pub fn new(url: impl Into<SharedString>) -> Self {
        Self {
            url: url.into(),
            title: None,
            show_navigation: true,
            console_logs: Vec::new(),
            navigation: None,
            body: None,
            console: None,
        }
    }

    /// Set the title displayed in the header.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Show or hide the default navigation bar (convenience API, default: `true`).
    pub fn navigation(mut self, show: bool) -> Self {
        self.show_navigation = show;
        self
    }

    /// Append a single console log entry (convenience API).
    pub fn console_log(mut self, entry: ConsoleEntry) -> Self {
        self.console_logs.push(entry);
        self
    }

    /// Append multiple console log entries (convenience API).
    pub fn console_logs(mut self, entries: Vec<ConsoleEntry>) -> Self {
        self.console_logs.extend(entries);
        self
    }

    /// Set the navigation sub-component (compound API).
    pub fn nav(mut self, navigation: WebPreviewNavigation) -> Self {
        self.navigation = Some(navigation);
        self
    }

    /// Set the body sub-component (compound API).
    pub fn body(mut self, body: WebPreviewBody) -> Self {
        self.body = Some(body);
        self
    }

    /// Set the console sub-component (compound API).
    pub fn console_panel(mut self, console: WebPreviewConsole) -> Self {
        self.console = Some(console);
        self
    }
}

impl RenderOnce for WebPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let url_for_click = self.url.clone();

        let mut container = crate::foundations::materials::card_surface(theme);

        // Header
        let title_text = self.title.unwrap_or_else(|| "Web Preview".into());
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .border_b_1()
            .border_color(theme.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .child(
                        Icon::new(IconName::Globe)
                            .size(theme.icon_size_inline)
                            .color(theme.text_muted),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text)
                            .child(title_text),
                    ),
            )
            .child(
                Button::new("open-in-browser")
                    .label("Open in Browser")
                    .icon(Icon::new(IconName::ExternalLink).size(theme.icon_size_inline))
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Small)
                    .on_click(move |_event, _window, cx| {
                        cx.open_url(url_for_click.as_ref());
                    }),
            );
        container = container.child(header);

        // Navigation
        if let Some(nav) = self.navigation {
            container = container.child(nav);
        } else if self.show_navigation {
            let default_nav = WebPreviewNavigation::new()
                .child(WebPreviewNavigationButton::back())
                .child(WebPreviewNavigationButton::forward())
                .child(WebPreviewUrl::new(self.url.clone()));
            container = container.child(default_nav);
        }

        // Body
        if let Some(body) = self.body {
            container = container.child(body);
        } else {
            container = container.child(WebPreviewBody::new(self.url.clone()));
        }

        // Console
        if let Some(console) = self.console {
            container = container.child(console);
        } else if !self.console_logs.is_empty() {
            container = container.child(WebPreviewConsole::new(self.console_logs));
        }

        container
    }
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{
        ConsoleEntry, ConsoleLevel, WebPreview, WebPreviewBody, WebPreviewConsole,
        WebPreviewNavigation, WebPreviewNavigationButton, WebPreviewUrl,
    };
    use crate::components::content::badge::BadgeVariant;
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;
    use gpui::div;

    // -- ConsoleLevel / ConsoleEntry ------------------------------------------

    #[test]
    fn console_level_equality() {
        assert_eq!(ConsoleLevel::Log, ConsoleLevel::Log);
        assert_eq!(ConsoleLevel::Warn, ConsoleLevel::Warn);
        assert_eq!(ConsoleLevel::Error, ConsoleLevel::Error);
        assert_ne!(ConsoleLevel::Log, ConsoleLevel::Warn);
        assert_ne!(ConsoleLevel::Log, ConsoleLevel::Error);
        assert_ne!(ConsoleLevel::Warn, ConsoleLevel::Error);
    }

    #[test]
    fn console_level_labels() {
        assert_eq!(ConsoleLevel::Log.label(), "log");
        assert_eq!(ConsoleLevel::Warn.label(), "warn");
        assert_eq!(ConsoleLevel::Error.label(), "error");
    }

    #[test]
    fn console_level_badge_variants() {
        assert_eq!(ConsoleLevel::Log.badge_variant(), BadgeVariant::Muted);
        assert_eq!(ConsoleLevel::Warn.badge_variant(), BadgeVariant::Warning);
        assert_eq!(ConsoleLevel::Error.badge_variant(), BadgeVariant::Error);
    }

    #[test]
    fn console_entry_creation() {
        let entry = ConsoleEntry {
            level: ConsoleLevel::Warn,
            message: "deprecation warning".into(),
            timestamp: Some("12:00:00".into()),
        };
        assert_eq!(entry.level, ConsoleLevel::Warn);
        assert_eq!(entry.message.as_ref(), "deprecation warning");
        assert_eq!(entry.timestamp.as_ref().unwrap().as_ref(), "12:00:00");
    }

    #[test]
    fn console_entry_no_timestamp() {
        let entry = ConsoleEntry {
            level: ConsoleLevel::Log,
            message: "hello".into(),
            timestamp: None,
        };
        assert!(entry.timestamp.is_none());
    }

    // -- WebPreviewConsole ----------------------------------------------------

    #[test]
    fn console_empty() {
        let console = WebPreviewConsole::new(vec![]);
        assert!(console.logs.is_empty());
    }

    #[test]
    fn console_with_entries() {
        let logs = vec![
            ConsoleEntry {
                level: ConsoleLevel::Log,
                message: "a".into(),
                timestamp: None,
            },
            ConsoleEntry {
                level: ConsoleLevel::Error,
                message: "b".into(),
                timestamp: None,
            },
        ];
        let console = WebPreviewConsole::new(logs);
        assert_eq!(console.logs.len(), 2);
    }

    #[test]
    fn console_log_appends() {
        let console = WebPreviewConsole::new(vec![])
            .log(ConsoleEntry {
                level: ConsoleLevel::Log,
                message: "first".into(),
                timestamp: None,
            })
            .log(ConsoleEntry {
                level: ConsoleLevel::Warn,
                message: "second".into(),
                timestamp: None,
            });
        assert_eq!(console.logs.len(), 2);
        assert_eq!(console.logs[0].level, ConsoleLevel::Log);
        assert_eq!(console.logs[1].level, ConsoleLevel::Warn);
    }

    // -- WebPreviewNavigationButton -------------------------------------------

    #[test]
    fn nav_button_back() {
        let btn = WebPreviewNavigationButton::back();
        assert_eq!(btn.icon, IconName::ChevronLeft);
        assert_eq!(btn.tooltip.as_ref(), "Go back");
        assert!(btn.disabled);
    }

    #[test]
    fn nav_button_forward() {
        let btn = WebPreviewNavigationButton::forward();
        assert_eq!(btn.icon, IconName::ChevronRight);
        assert_eq!(btn.tooltip.as_ref(), "Go forward");
        assert!(btn.disabled);
    }

    #[test]
    fn nav_button_custom() {
        let btn = WebPreviewNavigationButton::new("refresh-btn", IconName::Globe, "Refresh")
            .disabled(false);
        assert_eq!(btn.icon, IconName::Globe);
        assert_eq!(btn.tooltip.as_ref(), "Refresh");
        assert!(!btn.disabled);
    }

    // -- WebPreviewUrl --------------------------------------------------------

    #[test]
    fn url_stores_value() {
        let url = WebPreviewUrl::new("https://example.com");
        assert_eq!(url.url.as_ref(), "https://example.com");
    }

    // -- WebPreviewNavigation -------------------------------------------------

    #[test]
    fn navigation_empty() {
        let nav = WebPreviewNavigation::new();
        assert!(nav.children.is_empty());
    }

    #[test]
    fn navigation_child_appends() {
        let nav = WebPreviewNavigation::new().child(div()).child(div());
        assert_eq!(nav.children.len(), 2);
    }

    #[test]
    fn navigation_children_bulk() {
        let nav = WebPreviewNavigation::new().children(vec![div(), div(), div()]);
        assert_eq!(nav.children.len(), 3);
    }

    #[test]
    fn navigation_mixed() {
        let nav = WebPreviewNavigation::new()
            .child(div())
            .children(vec![div(), div()]);
        assert_eq!(nav.children.len(), 3);
    }

    // -- WebPreviewBody -------------------------------------------------------

    #[test]
    fn body_stores_url() {
        let body = WebPreviewBody::new("https://example.com");
        assert_eq!(body.url.as_ref(), "https://example.com");
        assert!(body.loading.is_none());
    }

    #[test]
    fn body_loading_set() {
        let body = WebPreviewBody::new("https://example.com").loading(div());
        assert!(body.loading.is_some());
    }

    // -- WebPreview (convenience API) -----------------------------------------

    #[test]
    fn web_preview_defaults() {
        let wp = WebPreview::new("https://example.com");
        assert_eq!(wp.url.as_ref(), "https://example.com");
        assert!(wp.title.is_none());
        assert!(wp.show_navigation);
        assert!(wp.console_logs.is_empty());
        assert!(wp.navigation.is_none());
        assert!(wp.body.is_none());
        assert!(wp.console.is_none());
    }

    #[test]
    fn web_preview_title() {
        let wp = WebPreview::new("https://example.com").title("My App");
        assert_eq!(wp.title.as_ref().unwrap().as_ref(), "My App");
    }

    #[test]
    fn web_preview_navigation_false() {
        let wp = WebPreview::new("https://example.com").navigation(false);
        assert!(!wp.show_navigation);
    }

    #[test]
    fn web_preview_console_log() {
        let wp = WebPreview::new("https://example.com").console_log(ConsoleEntry {
            level: ConsoleLevel::Log,
            message: "loaded".into(),
            timestamp: None,
        });
        assert_eq!(wp.console_logs.len(), 1);
    }

    #[test]
    fn web_preview_console_logs_bulk() {
        let entries = vec![
            ConsoleEntry {
                level: ConsoleLevel::Log,
                message: "a".into(),
                timestamp: None,
            },
            ConsoleEntry {
                level: ConsoleLevel::Warn,
                message: "b".into(),
                timestamp: None,
            },
        ];
        let wp = WebPreview::new("https://example.com").console_logs(entries);
        assert_eq!(wp.console_logs.len(), 2);
    }

    // -- WebPreview (compound API) --------------------------------------------

    #[test]
    fn web_preview_with_nav() {
        let wp =
            WebPreview::new("https://example.com").nav(WebPreviewNavigation::new().child(div()));
        assert!(wp.navigation.is_some());
        assert_eq!(wp.navigation.as_ref().unwrap().children.len(), 1);
    }

    #[test]
    fn web_preview_with_body() {
        let wp =
            WebPreview::new("https://example.com").body(WebPreviewBody::new("https://example.com"));
        assert!(wp.body.is_some());
    }

    #[test]
    fn web_preview_with_console_panel() {
        let wp =
            WebPreview::new("https://example.com").console_panel(WebPreviewConsole::new(vec![
                ConsoleEntry {
                    level: ConsoleLevel::Error,
                    message: "fail".into(),
                    timestamp: None,
                },
            ]));
        assert!(wp.console.is_some());
    }

    #[test]
    fn web_preview_compound_full() {
        let wp = WebPreview::new("https://example.com")
            .title("Full Example")
            .nav(
                WebPreviewNavigation::new()
                    .child(WebPreviewNavigationButton::back())
                    .child(WebPreviewNavigationButton::forward())
                    .child(WebPreviewUrl::new("https://example.com")),
            )
            .body(WebPreviewBody::new("https://example.com"))
            .console_panel(WebPreviewConsole::new(vec![]));
        assert!(wp.navigation.is_some());
        assert!(wp.body.is_some());
        assert!(wp.console.is_some());
        assert_eq!(wp.title.as_ref().unwrap().as_ref(), "Full Example");
    }
}
