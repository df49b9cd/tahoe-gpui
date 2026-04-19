//! HIG Image Well — placeholder / preview for image selection.
//!
//! A stateless `RenderOnce` component that renders a rounded square
//! placeholder or preview area for image selection. When no image is set,
//! it shows a dashed border with an Image icon and placeholder text. When
//! an image URL is set, it renders the pixels via `img()`. Click fires an
//! `on_click` callback so the host app can open a file picker; `on_drop`
//! fires when paths are dragged onto the well.
//!
//! `ImageWell` is the input / selection primitive. For read-only display
//! of image pixels use [`super::super::content::ImageView`].
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/image-wells>

use std::path::PathBuf;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    App, ElementId, ExternalPaths, KeyDownEvent, ObjectFit, Pixels, SharedString, SharedUri,
    Window, div, img, px,
};

use crate::callback_types::OnMutCallback;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::materials::apply_high_contrast_border;
use crate::foundations::materials::glass_surface;
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// Default size for the image well (80x80pt).
const DEFAULT_SIZE: Pixels = px(80.0);

/// Default placeholder text shown when no image is selected.
const DEFAULT_PLACEHOLDER: &str = "Select image";

/// Callback signature fired when files are dropped onto the well.
type OnDropPaths = Option<Rc<dyn Fn(&[PathBuf], &mut Window, &mut App) + 'static>>;

/// HIG Image Well — placeholder / preview for image selection.
///
/// Empty state shows a rounded square with a dashed border, centered Image
/// icon, and placeholder text below. When `image_url` is set, renders the
/// actual image pixels through GPUI's `img()`. Click fires `on_click` for
/// the host app to open a file picker; dragging paths onto the well fires
/// `on_drop`.
#[derive(IntoElement)]
pub struct ImageWell {
    id: ElementId,
    image_url: Option<SharedString>,
    placeholder: Option<SharedString>,
    size: Option<Pixels>,
    focused: bool,
    accessibility_label: Option<SharedString>,
    on_click: OnMutCallback,
    on_drop: OnDropPaths,
}

impl ImageWell {
    /// Create a new image well with the given element id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            image_url: None,
            placeholder: None,
            size: None,
            focused: false,
            accessibility_label: None,
            on_click: None,
            on_drop: None,
        }
    }

    /// Set the image URL to display as a preview. Local file paths and
    /// data URLs are rendered by GPUI's `img()`; remote URLs are
    /// supported when the host has networking enabled.
    pub fn image_url(mut self, url: impl Into<SharedString>) -> Self {
        self.image_url = Some(url.into());
        self
    }

    /// Set custom placeholder text (default: "Select image").
    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set the size of the image well (default: 80px).
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    /// Mark the well as keyboard-focused. Callers own focus state — the
    /// well draws the focus ring when this is `true`.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Override the VoiceOver label. Defaults to "Image well" /
    /// "Selected image" depending on state.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Set the handler called when the user clicks the image well.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set the handler called when external paths are dropped on the well.
    /// Receives the list of `PathBuf`s that were dragged in.
    pub fn on_drop(
        mut self,
        handler: impl Fn(&[PathBuf], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_drop = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ImageWell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let well_size = self.size.unwrap_or(DEFAULT_SIZE);
        // Ensure at least 44pt touch target when interactive.
        let touch_size = if f32::from(well_size) < theme.target_size() {
            px(theme.target_size())
        } else {
            well_size
        };

        let placeholder_text: SharedString = self
            .placeholder
            .unwrap_or_else(|| SharedString::from(DEFAULT_PLACEHOLDER));

        let has_image = self.image_url.is_some();
        // Per HIG: Liquid Glass is only for controls, not content.
        // When on_click is set the well acts as a control (glass is appropriate).
        // When on_click is None it's display-only content (standard surface).
        let is_interactive = self.on_click.is_some() || self.on_drop.is_some();

        let a11y_label: SharedString = self.accessibility_label.clone().unwrap_or_else(|| {
            if has_image {
                SharedString::from("Selected image")
            } else {
                SharedString::from("Image well, drag or click to select")
            }
        });
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(if is_interactive {
                AccessibilityRole::Button
            } else {
                AccessibilityRole::Image
            });

        let mut inner = div()
            .size(touch_size)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(theme.spacing_xs)
            .flex_shrink_0();

        if let Some(url) = self.image_url.clone() {
            // Real pixel preview via GPUI's img().
            inner = inner.bg(theme.surface).rounded(theme.radius_lg).child(
                img(SharedUri::from(url.to_string()))
                    .size(touch_size)
                    .rounded(theme.radius_lg)
                    .object_fit(ObjectFit::Cover),
            );

            if is_interactive {
                inner = inner.shadow(theme.glass.shadows(GlassSize::Small).to_vec());
                inner = apply_high_contrast_border(inner, theme);
            } else {
                inner = inner.border_1().border_color(theme.border);
            }
        } else {
            // Placeholder state: dashed-style surface with the Image icon.
            if is_interactive {
                inner = glass_surface(inner, theme, GlassSize::Small);
            } else {
                inner = inner
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md);
            }

            inner = inner.child(
                Icon::new(IconName::Image)
                    .size(px(24.0))
                    .color(theme.text_muted),
            );

            inner = inner.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child(placeholder_text),
            );
        }

        // Wrap in a stateful container for id + event handlers.
        let mut well = div()
            .id(self.id)
            .child(inner)
            .with_accessibility(&a11y_props);

        if is_interactive {
            well = well.focusable();
            well = apply_focus_ring(well, theme, self.focused, &[]);
            well = well.cursor_pointer();
        }

        // Wrap the click handler so both pointer clicks and Space / Enter
        // key presses can fire it.
        let on_click_rc = self.on_click.map(Rc::new);

        if let Some(handler) = on_click_rc.clone() {
            well = well.on_click(move |_event, window, cx| {
                handler(window, cx);
            });
        }

        if let Some(handler) = on_click_rc {
            well = well.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                if key == "space" || key == "enter" {
                    handler(window, cx);
                }
            });
        }

        if let Some(handler) = self.on_drop {
            well = well.on_drop(move |paths: &ExternalPaths, window, cx| {
                handler(&paths.0, window, cx);
            });
        }

        well
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::px;

    use crate::components::selection_and_input::image_well::{
        DEFAULT_PLACEHOLDER, DEFAULT_SIZE, ImageWell,
    };

    #[test]
    fn image_well_defaults() {
        let iw = ImageWell::new("test");
        assert!(iw.image_url.is_none());
        assert!(iw.placeholder.is_none());
        assert!(iw.size.is_none());
        assert!(!iw.focused);
        assert!(iw.accessibility_label.is_none());
        assert!(iw.on_click.is_none());
        assert!(iw.on_drop.is_none());
    }

    #[test]
    fn image_well_image_url_builder() {
        let iw = ImageWell::new("test").image_url("https://example.com/photo.jpg");
        assert!(iw.image_url.is_some());
        assert_eq!(
            iw.image_url.unwrap().as_ref(),
            "https://example.com/photo.jpg"
        );
    }

    #[test]
    fn image_well_placeholder_builder() {
        let iw = ImageWell::new("test").placeholder("Drop image here");
        assert!(iw.placeholder.is_some());
        assert_eq!(iw.placeholder.unwrap().as_ref(), "Drop image here");
    }

    #[test]
    fn image_well_size_builder() {
        let iw = ImageWell::new("test").size(px(120.0));
        assert_eq!(iw.size, Some(px(120.0)));
    }

    #[test]
    fn image_well_focused_builder() {
        let iw = ImageWell::new("test").focused(true);
        assert!(iw.focused);
    }

    #[test]
    fn image_well_accessibility_label_builder() {
        let iw = ImageWell::new("test").accessibility_label("Avatar well");
        assert_eq!(
            iw.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Avatar well")
        );
    }

    #[test]
    fn image_well_on_click_is_some() {
        let iw = ImageWell::new("test").on_click(|_, _| {});
        assert!(iw.on_click.is_some());
    }

    #[test]
    fn image_well_on_drop_is_some() {
        let iw = ImageWell::new("test").on_drop(|_, _, _| {});
        assert!(iw.on_drop.is_some());
    }

    #[test]
    fn default_placeholder_text() {
        assert_eq!(DEFAULT_PLACEHOLDER, "Select image");
    }

    #[test]
    fn default_size_is_80pt() {
        assert!((f32::from(DEFAULT_SIZE) - 80.0).abs() < f32::EPSILON);
    }
}
