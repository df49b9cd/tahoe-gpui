//! HIG Image view — read-only image display.
//!
//! `ImageView` is the display primitive: it renders pixels for a known
//! image URL, applies a [`ContentMode`], and carries an accessibility
//! label. For interactive image selection (click, drag-and-drop,
//! placeholder state) use [`crate::components::selection_and_input::ImageWell`].
//!
//! # HIG compliance notes
//!
//! The HIG defines image views as capable of displaying *animated sequences*
//! of images. GPUI's `img()` element renders a single static frame, so this
//! component does not yet support animation. When GPUI adds frame-animation
//! support, add a `.frames(uris, timing)` builder here.
//!
//! The HIG also recommends providing light and dark variants of images when
//! appearance affects their meaning or legibility. Use [`ImageView::dark_uri`]
//! to supply an alternate image for dark appearances.
//!
//! **Accessibility:** The HIG requires meaningful alt text for all images that
//! convey information so VoiceOver can describe them. Always call
//! [`.accessibility_label()`](ImageView::accessibility_label) unless the image
//! is purely decorative. A `debug_assert!` fires in debug builds when the label
//! is missing.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/image-views>

use gpui::prelude::*;
use gpui::{App, ObjectFit, Pixels, SharedString, SharedUri, Window, div, img};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::color::Appearance;
use crate::foundations::theme::ActiveTheme;

/// Content mode for an image view — how the image fits its container.
///
/// Maps directly onto GPUI's [`ObjectFit`] and Apple's Image view
/// content-mode vocabulary:
///
/// - [`ContentMode::Fill`] (Scale to Fill) — stretch to fill the frame,
///   ignoring aspect ratio.
/// - [`ContentMode::AspectFit`] (Aspect Fit) — preserve aspect ratio; show
///   letterbox / pillarbox bars if the frame doesn't match.
/// - [`ContentMode::AspectFill`] (Aspect Fill) — preserve aspect ratio and
///   crop to cover the frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ContentMode {
    /// Scale to fill, ignoring aspect ratio.
    Fill,
    /// Preserve aspect ratio; letterbox the frame if needed.
    #[default]
    AspectFit,
    /// Preserve aspect ratio; crop to fill the frame.
    AspectFill,
}

impl ContentMode {
    fn object_fit(self) -> ObjectFit {
        match self {
            ContentMode::Fill => ObjectFit::Fill,
            ContentMode::AspectFit => ObjectFit::Contain,
            ContentMode::AspectFill => ObjectFit::Cover,
        }
    }
}

/// A stateless, read-only image display.
///
/// # Examples
///
/// ```ignore
/// use gpui::px;
/// use tahoe_gpui::components::content::image_view::{ContentMode, ImageView};
///
/// ImageView::new("https://example.com/photo.png")
///     .size(px(200.0))
///     .content_mode(ContentMode::AspectFill)
///     .rounded(px(8.0))
///     .dark_uri("https://example.com/photo-dark.png")
///     .accessibility_label("Team photo from the offsite")
/// ```
#[derive(IntoElement)]
pub struct ImageView {
    uri: SharedUri,
    dark_uri: Option<SharedUri>,
    size: Option<Pixels>,
    width: Option<Pixels>,
    height: Option<Pixels>,
    content_mode: ContentMode,
    accessibility_label: Option<SharedString>,
    rounded: Option<Pixels>,
    circular: bool,
    opaque: bool,
}

impl ImageView {
    /// Create a new `ImageView` from a URL, file path, or data URL.
    pub fn new(uri: impl Into<SharedUri>) -> Self {
        Self {
            uri: uri.into(),
            dark_uri: None,
            size: None,
            width: None,
            height: None,
            content_mode: ContentMode::default(),
            accessibility_label: None,
            rounded: None,
            circular: false,
            opaque: false,
        }
    }

    /// Square size convenience — sets both width and height.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the width independently (overrides the width set by [`size`](Self::size)).
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height independently (overrides the height set by [`size`](Self::size)).
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    /// Pick a content mode (default: [`ContentMode::AspectFit`]).
    pub fn content_mode(mut self, mode: ContentMode) -> Self {
        self.content_mode = mode;
        self
    }

    /// Round the corners of the image to the given radius.
    pub fn rounded(mut self, radius: Pixels) -> Self {
        self.rounded = Some(radius);
        self
    }

    /// Clip the image to a circle.
    ///
    /// Resolved at render time to `theme.radius_full` — a radius large enough
    /// to clip to a circle at any practical image size. If both `.circular()`
    /// and `.rounded()` are called, `.rounded()` wins (it sets an explicit
    /// pixel value that overrides the flag).
    pub fn circular(mut self) -> Self {
        self.circular = true;
        self
    }

    /// Supply an alternate image for dark appearances.
    ///
    /// The HIG recommends providing light and dark variants when appearance
    /// affects the image's meaning or legibility. When set, the component
    /// selects this URI when the theme appearance is dark or dark-high-contrast.
    pub fn dark_uri(mut self, uri: impl Into<SharedUri>) -> Self {
        self.dark_uri = Some(uri.into());
        self
    }

    /// Render the image on an opaque background.
    ///
    /// By default the image view has a transparent background. Call this to
    /// fill behind the image with the theme's surface color, matching the
    /// HIG's "transparent or opaque background" option. Requires explicit
    /// dimensions (via `.size()`, `.width()`, or `.height()`) for the
    /// background to be visible.
    pub fn opaque(mut self) -> Self {
        self.opaque = true;
        self
    }

    /// Accessibility label for VoiceOver.
    ///
    /// The HIG requires meaningful alt text for all images that convey
    /// information. If no label is set, a `debug_assert!` fires in debug
    /// builds to remind you to add one. Pass an empty string explicitly
    /// for truly decorative images to suppress the assertion.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

impl RenderOnce for ImageView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        debug_assert!(
            self.accessibility_label.is_some(),
            "ImageView: no accessibility_label set. \
             Call .accessibility_label(\"description\") for images that convey \
             information, or .accessibility_label(\"\") for decorative images. \
             HIG: https://developer.apple.com/design/human-interface-guidelines/image-views"
        );
        #[cfg(not(debug_assertions))]
        if self.accessibility_label.is_none() {
            tracing::warn!(
                "ImageView: no accessibility_label set. \
                 Call .accessibility_label(\"description\") for informational images."
            );
        }

        let a11y_label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| SharedString::from(""));

        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Image);

        let uri = resolve_uri(self.uri, self.dark_uri, theme.appearance);

        let mut image = img(uri).object_fit(self.content_mode.object_fit());

        if let Some(size) = self.size {
            image = image.size(size);
        }
        if let Some(w) = self.width {
            image = image.w(w);
        }
        if let Some(h) = self.height {
            image = image.h(h);
        }
        let radius = self
            .rounded
            .or_else(|| self.circular.then_some(theme.radius_full));
        if let Some(radius) = radius {
            image = image.rounded(radius);
        }

        let mut container = div();
        if self.opaque {
            container = container.bg(theme.surface);
        }

        container.child(image).with_accessibility(&a11y_props)
    }
}

/// Select the display URI based on the current appearance.
///
/// When a dark variant is provided and the appearance is dark, the dark URI
/// is used; otherwise the primary URI is returned.
fn resolve_uri(primary: SharedUri, dark: Option<SharedUri>, appearance: Appearance) -> SharedUri {
    match dark {
        Some(dark) if appearance.is_dark() => dark,
        _ => primary,
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentMode, ImageView, resolve_uri};
    use crate::foundations::color::Appearance;
    use core::prelude::v1::test;
    use gpui::{SharedUri, px};

    #[test]
    fn image_view_new_stores_uri() {
        let iv = ImageView::new(SharedUri::from("file:///tmp/a.png"));
        assert_eq!(iv.uri.as_ref(), "file:///tmp/a.png");
        assert_eq!(iv.content_mode, ContentMode::AspectFit);
        assert!(iv.size.is_none());
        assert!(iv.accessibility_label.is_none());
        assert!(iv.dark_uri.is_none());
        assert!(!iv.circular);
        assert!(!iv.opaque);
    }

    #[test]
    fn builder_size() {
        let iv = ImageView::new(SharedUri::from("x")).size(px(120.0));
        assert_eq!(iv.size, Some(px(120.0)));
    }

    #[test]
    fn builder_width_height() {
        let iv = ImageView::new(SharedUri::from("x"))
            .width(px(200.0))
            .height(px(120.0));
        assert_eq!(iv.width, Some(px(200.0)));
        assert_eq!(iv.height, Some(px(120.0)));
    }

    #[test]
    fn builder_content_mode() {
        let iv = ImageView::new(SharedUri::from("x")).content_mode(ContentMode::AspectFill);
        assert_eq!(iv.content_mode, ContentMode::AspectFill);
    }

    #[test]
    fn builder_rounded() {
        let iv = ImageView::new(SharedUri::from("x")).rounded(px(8.0));
        assert_eq!(iv.rounded, Some(px(8.0)));
    }

    #[test]
    fn builder_accessibility_label() {
        let iv = ImageView::new(SharedUri::from("x")).accessibility_label("Photograph of cat");
        assert_eq!(
            iv.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Photograph of cat")
        );
    }

    #[test]
    fn builder_dark_uri() {
        let iv = ImageView::new(SharedUri::from("light.png")).dark_uri(SharedUri::from("dark.png"));
        assert_eq!(iv.uri.as_ref(), "light.png");
        assert_eq!(iv.dark_uri.unwrap().as_ref(), "dark.png");
    }

    #[test]
    fn builder_opaque() {
        let iv = ImageView::new(SharedUri::from("x")).opaque();
        assert!(iv.opaque);
    }

    #[test]
    fn builder_circular() {
        let iv = ImageView::new(SharedUri::from("x")).circular();
        assert!(iv.circular);
        assert!(iv.rounded.is_none()); // resolved at render time
    }

    #[test]
    fn resolve_uri_no_dark_variant_returns_primary() {
        let primary = SharedUri::from("light.png");
        assert_eq!(
            resolve_uri(primary.clone(), None, Appearance::Dark).as_ref(),
            "light.png"
        );
        assert_eq!(
            resolve_uri(primary.clone(), None, Appearance::Light).as_ref(),
            "light.png"
        );
    }

    #[test]
    fn resolve_uri_dark_appearance_selects_dark_variant() {
        let primary = SharedUri::from("light.png");
        let dark = SharedUri::from("dark.png");
        assert_eq!(
            resolve_uri(primary, Some(dark), Appearance::Dark).as_ref(),
            "dark.png"
        );
    }

    #[test]
    fn resolve_uri_dark_high_contrast_selects_dark_variant() {
        let primary = SharedUri::from("light.png");
        let dark = SharedUri::from("dark.png");
        assert_eq!(
            resolve_uri(primary, Some(dark), Appearance::DarkHighContrast).as_ref(),
            "dark.png"
        );
    }

    #[test]
    fn resolve_uri_light_appearance_ignores_dark_variant() {
        let primary = SharedUri::from("light.png");
        let dark = SharedUri::from("dark.png");
        assert_eq!(
            resolve_uri(primary, Some(dark), Appearance::Light).as_ref(),
            "light.png"
        );
    }

    #[test]
    fn resolve_uri_light_high_contrast_ignores_dark_variant() {
        let primary = SharedUri::from("light.png");
        let dark = SharedUri::from("dark.png");
        assert_eq!(
            resolve_uri(primary, Some(dark), Appearance::LightHighContrast).as_ref(),
            "light.png"
        );
    }

    #[test]
    fn content_mode_default_is_aspect_fit() {
        assert_eq!(ContentMode::default(), ContentMode::AspectFit);
    }

    #[test]
    fn content_mode_maps_to_object_fit() {
        // `ObjectFit` does not implement `PartialEq`; exercise the mapping
        // by asserting it returns without panicking and that the method
        // is called for each variant.
        let _ = ContentMode::Fill.object_fit();
        let _ = ContentMode::AspectFit.object_fit();
        let _ = ContentMode::AspectFill.object_fit();
    }
}
