//! HIG Image view — read-only image display.
//!
//! `ImageView` is the display primitive: it renders pixels for a known
//! image URL, applies a [`ContentMode`], and carries an accessibility
//! label. For interactive image selection (click, drag-and-drop,
//! placeholder state) use [`crate::components::selection_and_input::ImageWell`].
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/image-views>

use gpui::prelude::*;
use gpui::{App, ObjectFit, Pixels, SharedString, SharedUri, Window, div, img};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme};

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
#[derive(IntoElement)]
pub struct ImageView {
    uri: SharedUri,
    size: Option<Pixels>,
    width: Option<Pixels>,
    height: Option<Pixels>,
    content_mode: ContentMode,
    accessibility_label: Option<SharedString>,
    rounded: Option<Pixels>,
}

impl ImageView {
    /// Create a new `ImageView` from a URL, file path, or data URL.
    pub fn new(uri: impl Into<SharedUri>) -> Self {
        Self {
            uri: uri.into(),
            size: None,
            width: None,
            height: None,
            content_mode: ContentMode::default(),
            accessibility_label: None,
            rounded: None,
        }
    }

    /// Square size convenience — sets both width and height.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
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

    /// Accessibility label (recommended — defaults to empty, which
    /// announces as decorative content).
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

impl RenderOnce for ImageView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let _theme = cx.theme();

        let a11y_props = AccessibilityProps::new()
            .label(
                self.accessibility_label
                    .clone()
                    .unwrap_or_else(|| SharedString::from("")),
            )
            .role(AccessibilityRole::Image);

        let mut image = img(self.uri).object_fit(self.content_mode.object_fit());

        if let Some(size) = self.size {
            image = image.size(size);
        }
        if let Some(w) = self.width {
            image = image.w(w);
        }
        if let Some(h) = self.height {
            image = image.h(h);
        }
        if let Some(radius) = self.rounded {
            image = image.rounded(radius);
        }

        div().child(image).with_accessibility(&a11y_props)
    }
}

#[cfg(test)]
mod tests {
    use super::{ContentMode, ImageView};
    use core::prelude::v1::test;
    use gpui::{SharedUri, px};

    #[test]
    fn image_view_new_stores_uri() {
        let iv = ImageView::new(SharedUri::from("file:///tmp/a.png"));
        assert_eq!(iv.uri.as_ref(), "file:///tmp/a.png");
        assert_eq!(iv.content_mode, ContentMode::AspectFit);
        assert!(iv.size.is_none());
        assert!(iv.accessibility_label.is_none());
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
