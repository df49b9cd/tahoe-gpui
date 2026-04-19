//! iOS navigation bar component (`NavigationBarIOS`).
//!
//! Top navigation bar with leading/trailing actions and centered title —
//! a direct port of `UINavigationBar`. The 44 pt default height, centered
//! title, and leading/trailing button convention match iOS / iPadOS /
//! watchOS. **This is not the macOS idiom.** On macOS, the HIG equivalent
//! is a unified [`Toolbar`](super::Toolbar) sitting in the window's title
//! bar; use that instead of `NavigationBarIOS` in macOS targets.
//!
//! Default height is platform-aware (36 pt on macOS, 44 pt on iOS/iPadOS /
//! watchOS, 60 pt on visionOS, 88 pt on tvOS) — see
//! [`Platform::navigation_bar_height`](crate::foundations::layout::Platform::navigation_bar_height).

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, FontWeight, Pixels, SharedString, Window, div, px};

use crate::foundations::materials::{SurfaceContext, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// iOS-style navigation bar primitive.
///
/// **Platform note.** This component mirrors `UINavigationBar` and is
/// intended for iOS / iPadOS / watchOS / visionOS hosts. macOS apps
/// should use [`Toolbar`](super::Toolbar) instead — the HIG macOS idiom
/// is a unified toolbar in the window chrome, not a centered-title bar.
///
/// Provides a centered title, optional leading action area (e.g. back
/// button), and trailing action area (e.g. edit/share buttons).
#[derive(IntoElement)]
pub struct NavigationBarIOS {
    id: ElementId,
    title: Option<SharedString>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    height: Option<Pixels>,
}

impl NavigationBarIOS {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            title: None,
            leading: None,
            trailing: None,
            height: None,
        }
    }

    /// Set the centered title text.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the leading (left-side) element, e.g. a back button.
    pub fn leading(mut self, el: impl IntoElement) -> Self {
        self.leading = Some(el.into_any_element());
        self
    }

    /// Set the trailing (right-side) element, e.g. action buttons.
    pub fn trailing(mut self, el: impl IntoElement) -> Self {
        self.trailing = Some(el.into_any_element());
        self
    }

    /// Override the default bar height. When unset the height comes from
    /// `Platform::navigation_bar_height()` on the active theme — 36 pt on
    /// macOS, 44 pt on iOS/iPadOS/watchOS, 60 pt on visionOS, 88 pt on tvOS.
    pub fn height(mut self, h: Pixels) -> Self {
        self.height = Some(h);
        self
    }
}

impl RenderOnce for NavigationBarIOS {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let bar_height = self
            .height
            .unwrap_or_else(|| px(theme.platform.navigation_bar_height()));

        // ── Title layer (absolute-centered over entire bar width) ──────────
        let title_layer = div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .text_color(theme.label_color(SurfaceContext::GlassDim))
            .text_style(TextStyle::Title3, theme)
            .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
            .children(self.title.map(|t| div().child(t)));

        // ── Controls layer (leading / spacer / trailing) ──────────────────
        let leading_el = if let Some(leading) = self.leading {
            div()
                .flex()
                .items_center()
                .child(leading)
                .into_any_element()
        } else {
            div().into_any_element()
        };

        let trailing_el = if let Some(trailing) = self.trailing {
            div()
                .flex()
                .items_center()
                .child(trailing)
                .into_any_element()
        } else {
            div().into_any_element()
        };

        let controls_layer = div()
            .w_full()
            .h_full()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(leading_el)
            .child(trailing_el);

        // ── Bar surface ───────────────────────────────────────────────────
        glass_surface(
            div().relative().h(bar_height).px(theme.spacing_md),
            theme,
            GlassSize::Small,
        )
        .id(self.id)
        .child(title_layer)
        .child(controls_layer)
    }
}

#[cfg(test)]
mod tests {
    use super::NavigationBarIOS;
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn navigation_bar_new_defaults() {
        let bar = NavigationBarIOS::new("nav");
        assert!(bar.title.is_none());
        assert!(bar.leading.is_none());
        assert!(bar.trailing.is_none());
        assert!(bar.height.is_none());
    }

    #[test]
    fn navigation_bar_builder_sets_title() {
        let bar = NavigationBarIOS::new("nav").title("Settings");
        assert_eq!(bar.title.unwrap().as_ref(), "Settings");
    }

    #[test]
    fn navigation_bar_builder_sets_height() {
        let bar = NavigationBarIOS::new("nav").height(px(56.0));
        assert_eq!(bar.height, Some(px(56.0)));
    }

    #[test]
    fn navigation_bar_builder_all_fields() {
        let bar = NavigationBarIOS::new("nav").title("Home").height(px(50.0));
        assert_eq!(bar.title.unwrap().as_ref(), "Home");
        assert!(bar.leading.is_none());
        assert!(bar.trailing.is_none());
        assert_eq!(bar.height, Some(px(50.0)));
    }
}
