//! Avatar component for user/assistant identification.
//!
//! Renders an image (preferred) or an initials fallback in a circular
//! container, optionally overlaid with a presence/status dot.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/image-views>

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme, TextStyle};
use gpui::prelude::*;
use gpui::{App, Hsla, ObjectFit, Pixels, SharedString, SharedUri, Window, div, img, px};

/// Canonical HIG avatar size stops (in points).
///
/// HIG picks a small set of avatar sizes that cover the vast majority
/// of macOS / iOS contexts — toolbar icons, inline mentions, list rows,
/// profile chips, and large account-summary panes. Callers that need a
/// custom size can still call [`Avatar::size`] with an arbitrary [`Pixels`]
/// value; the enum captures the canonical defaults so typical UI code does
/// not invent ad-hoc sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvatarSize {
    /// 16 pt — inline mentions inside body text.
    Inline,
    /// 24 pt — toolbar / sidebar row.
    Toolbar,
    /// 32 pt — standard list row / table cell.
    Standard,
    /// 40 pt — contacts list / conversation row.
    Contacts,
    /// 48 pt — profile card / settings row.
    Profile,
    /// 64 pt — account summary / large row.
    Summary,
    /// 96 pt — prominent profile panel.
    Prominent,
}

impl AvatarSize {
    /// Returns the diameter in points for this canonical size.
    pub fn points(self) -> Pixels {
        match self {
            AvatarSize::Inline => px(16.0),
            AvatarSize::Toolbar => px(24.0),
            AvatarSize::Standard => px(32.0),
            AvatarSize::Contacts => px(40.0),
            AvatarSize::Profile => px(48.0),
            AvatarSize::Summary => px(64.0),
            AvatarSize::Prominent => px(96.0),
        }
    }

    /// Returns the HIG-specified initials text style for this size.
    ///
    /// Tiny avatars use Caption metrics; large avatars step up through the
    /// Title scale so the monogram stays balanced against the circle
    /// diameter.
    pub fn initials_text_style(self) -> TextStyle {
        match self {
            AvatarSize::Inline | AvatarSize::Toolbar => TextStyle::Caption1,
            AvatarSize::Standard => TextStyle::Subheadline,
            AvatarSize::Contacts | AvatarSize::Profile => TextStyle::Body,
            AvatarSize::Summary => TextStyle::Title3,
            AvatarSize::Prominent => TextStyle::Title1,
        }
    }
}

/// Presence / availability indicator overlaid on an avatar.
///
/// Rendered as an 8-point circle at the trailing-bottom corner with a
/// semantic color. The `Online`, `Away`, `Busy` states use green / amber /
/// red respectively; `Offline` uses a muted fill so the indicator is still
/// visible without suggesting live presence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvatarStatus {
    Online,
    Away,
    Busy,
    Offline,
}

/// An avatar displaying an image or initials fallback.
#[derive(IntoElement)]
pub struct Avatar {
    fallback: SharedString,
    image_url: Option<SharedUri>,
    size: Option<Pixels>,
    canonical_size: Option<AvatarSize>,
    bg_color: Option<Hsla>,
    status: Option<AvatarStatus>,
    accessibility_label: Option<SharedString>,
}

impl Avatar {
    pub fn new(fallback: impl Into<SharedString>) -> Self {
        Self {
            fallback: fallback.into(),
            image_url: None,
            size: None,
            canonical_size: None,
            bg_color: None,
            status: None,
            accessibility_label: None,
        }
    }

    /// Attach an image source. When set, renders the image and drops the
    /// monogram fallback.
    pub fn image(mut self, uri: impl Into<SharedUri>) -> Self {
        self.image_url = Some(uri.into());
        self
    }

    /// Attach an image source from a `String` or string-like URL.
    pub fn image_url(mut self, url: impl Into<String>) -> Self {
        self.image_url = Some(SharedUri::from(url.into()));
        self
    }

    /// Override the diameter with an arbitrary pixel value.
    ///
    /// Prefer [`Avatar::canonical_size`] for HIG-aligned sizes.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the canonical [`AvatarSize`] for this avatar.
    pub fn canonical_size(mut self, size: AvatarSize) -> Self {
        self.canonical_size = Some(size);
        self
    }

    pub fn bg(mut self, color: Hsla) -> Self {
        self.bg_color = Some(color);
        self
    }

    /// Overlay a presence dot at the trailing-bottom corner.
    pub fn status(mut self, status: AvatarStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Override the VoiceOver label. Defaults to the fallback text — callers
    /// should supply the user's full name ("Jane Doe's avatar") so screen
    /// readers do not announce a single initial.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

fn status_color(status: AvatarStatus, theme: &TahoeTheme) -> Hsla {
    match status {
        AvatarStatus::Online => theme.success,
        AvatarStatus::Away => theme.warning,
        AvatarStatus::Busy => theme.error,
        AvatarStatus::Offline => theme.text_muted,
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let size = self
            .size
            .or_else(|| self.canonical_size.map(AvatarSize::points))
            .unwrap_or(theme.avatar_size);
        let canonical = self
            .canonical_size
            .unwrap_or_else(|| AvatarSize::for_diameter(f32::from(size)));
        let font_size = canonical.initials_text_style().attrs().size;
        let glass = &theme.glass;
        let bg = self
            .bg_color
            .unwrap_or_else(|| glass.accessible_bg(GlassSize::Small, theme.accessibility_mode));

        let a11y_label: SharedString = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| self.fallback.clone());
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::Image);

        let mut circle = div()
            .size(size)
            .rounded(theme.radius_full)
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden()
            .text_size(font_size)
            .text_color(theme.text)
            .bg(bg)
            .shadow(glass.shadows(GlassSize::Small).to_vec());

        circle = crate::foundations::materials::apply_high_contrast_border(circle, theme);

        circle = if let Some(uri) = self.image_url {
            circle.child(
                img(uri)
                    .size(size)
                    .rounded(theme.radius_full)
                    .object_fit(ObjectFit::Cover),
            )
        } else {
            circle.child(self.fallback)
        };

        // Wrapper carries the accessibility props and (optionally) the
        // status dot. When no dot is set we return the plain circle; the
        // wrapper adds an `.relative()` container around both.
        if let Some(status) = self.status {
            let dot_size = px(status_dot_size(f32::from(size)));
            let dot_bg = status_color(status, theme);
            let ring_color = theme.surface;

            // DWC shape cue: overlay a tiny icon so state is not color-only.
            // Online=Check, Away=Minus (moon unavailable), Busy=Minus, Offline=XmarkCircleFill
            let dwc_icon = if theme.accessibility_mode.differentiate_without_color() {
                let icon_name = match status {
                    AvatarStatus::Online => IconName::Check,
                    AvatarStatus::Away | AvatarStatus::Busy => IconName::Minus,
                    AvatarStatus::Offline => IconName::XmarkCircleFill,
                };
                let icon_size = (f32::from(dot_size) * 0.6).max(6.0);
                let icon_color = crate::foundations::color::text_on_background(dot_bg);
                Some(Icon::new(icon_name).size(px(icon_size)).color(icon_color))
            } else {
                None
            };

            let mut status_dot = div()
                .absolute()
                .right_0()
                .bottom_0()
                .size(dot_size)
                .rounded(theme.radius_full)
                .bg(dot_bg)
                .border_2()
                .border_color(ring_color)
                .flex()
                .items_center()
                .justify_center();
            if let Some(icon) = dwc_icon {
                status_dot = status_dot.child(icon);
            }

            div()
                .relative()
                .size(size)
                .child(circle)
                .child(status_dot)
                .with_accessibility(&a11y_props)
        } else {
            div()
                .size(size)
                .child(circle)
                .with_accessibility(&a11y_props)
        }
    }
}

impl AvatarSize {
    /// Snaps an arbitrary diameter to the nearest canonical stop.
    ///
    /// Mid-points between two stops prefer the *smaller* stop so that sizes
    /// don't accidentally grow beyond the caller's intent.
    pub fn for_diameter(diameter: f32) -> Self {
        if diameter <= 20.0 {
            AvatarSize::Inline
        } else if diameter <= 28.0 {
            AvatarSize::Toolbar
        } else if diameter <= 36.0 {
            AvatarSize::Standard
        } else if diameter <= 44.0 {
            AvatarSize::Contacts
        } else if diameter <= 56.0 {
            AvatarSize::Profile
        } else if diameter <= 80.0 {
            AvatarSize::Summary
        } else {
            AvatarSize::Prominent
        }
    }
}

/// Status dot diameter for a given avatar diameter.
///
/// HIG presence indicators are ~25% of the avatar diameter with an 8 pt
/// minimum so the dot stays legible on toolbar / inline avatars.
fn status_dot_size(avatar_diameter: f32) -> f32 {
    (avatar_diameter * 0.25).max(8.0)
}

#[cfg(test)]
mod tests {
    use crate::components::content::avatar::{Avatar, AvatarSize, AvatarStatus, status_dot_size};
    use crate::foundations::theme::TextStyle;
    use core::prelude::v1::test;
    use gpui::{hsla, px};

    #[test]
    fn new_with_label() {
        let avatar = Avatar::new("U");
        assert_eq!(avatar.fallback.as_ref(), "U");
        assert!(avatar.image_url.is_none());
        assert!(avatar.status.is_none());
        assert!(avatar.accessibility_label.is_none());
    }

    #[test]
    fn size_builder() {
        let avatar = Avatar::new("A").size(px(48.0));
        assert!(avatar.size.is_some());
        assert_eq!(avatar.size.unwrap(), px(48.0));
    }

    #[test]
    fn canonical_size_builder() {
        let avatar = Avatar::new("A").canonical_size(AvatarSize::Contacts);
        assert_eq!(avatar.canonical_size, Some(AvatarSize::Contacts));
    }

    #[test]
    fn bg_builder() {
        let color = hsla(0.5, 1.0, 0.5, 1.0);
        let avatar = Avatar::new("X").bg(color);
        assert!(avatar.bg_color.is_some());
        assert_eq!(avatar.bg_color.unwrap(), color);
    }

    #[test]
    fn image_url_builder_sets_image() {
        let avatar = Avatar::new("X").image_url("https://example.com/avatar.png");
        assert!(avatar.image_url.is_some());
    }

    #[test]
    fn status_builder() {
        let avatar = Avatar::new("X").status(AvatarStatus::Online);
        assert_eq!(avatar.status, Some(AvatarStatus::Online));
    }

    #[test]
    fn accessibility_label_builder() {
        let avatar = Avatar::new("JD").accessibility_label("Jane Doe's avatar");
        assert_eq!(
            avatar.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Jane Doe's avatar")
        );
    }

    #[test]
    fn canonical_size_points_are_hig_stops() {
        assert_eq!(AvatarSize::Inline.points(), px(16.0));
        assert_eq!(AvatarSize::Toolbar.points(), px(24.0));
        assert_eq!(AvatarSize::Standard.points(), px(32.0));
        assert_eq!(AvatarSize::Contacts.points(), px(40.0));
        assert_eq!(AvatarSize::Profile.points(), px(48.0));
        assert_eq!(AvatarSize::Summary.points(), px(64.0));
        assert_eq!(AvatarSize::Prominent.points(), px(96.0));
    }

    #[test]
    fn initials_text_style_matches_hig_tiers() {
        assert_eq!(
            AvatarSize::Inline.initials_text_style(),
            TextStyle::Caption1
        );
        assert_eq!(
            AvatarSize::Toolbar.initials_text_style(),
            TextStyle::Caption1
        );
        assert_eq!(
            AvatarSize::Standard.initials_text_style(),
            TextStyle::Subheadline
        );
        assert_eq!(AvatarSize::Contacts.initials_text_style(), TextStyle::Body);
        assert_eq!(AvatarSize::Profile.initials_text_style(), TextStyle::Body);
        assert_eq!(AvatarSize::Summary.initials_text_style(), TextStyle::Title3);
        assert_eq!(
            AvatarSize::Prominent.initials_text_style(),
            TextStyle::Title1
        );
    }

    #[test]
    fn for_diameter_snaps_to_nearest_stop() {
        assert_eq!(AvatarSize::for_diameter(12.0), AvatarSize::Inline);
        assert_eq!(AvatarSize::for_diameter(20.0), AvatarSize::Inline);
        assert_eq!(AvatarSize::for_diameter(24.0), AvatarSize::Toolbar);
        assert_eq!(AvatarSize::for_diameter(28.0), AvatarSize::Toolbar);
        assert_eq!(AvatarSize::for_diameter(32.0), AvatarSize::Standard);
        assert_eq!(AvatarSize::for_diameter(40.0), AvatarSize::Contacts);
        assert_eq!(AvatarSize::for_diameter(48.0), AvatarSize::Profile);
        assert_eq!(AvatarSize::for_diameter(64.0), AvatarSize::Summary);
        assert_eq!(AvatarSize::for_diameter(120.0), AvatarSize::Prominent);
    }

    #[test]
    fn status_dot_has_8pt_minimum() {
        assert!((status_dot_size(16.0) - 8.0).abs() < f32::EPSILON);
        assert!((status_dot_size(24.0) - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn status_dot_scales_with_avatar_diameter() {
        // 40pt avatar -> 10pt dot (25%).
        assert!((status_dot_size(40.0) - 10.0).abs() < f32::EPSILON);
        // 96pt avatar -> 24pt dot.
        assert!((status_dot_size(96.0) - 24.0).abs() < f32::EPSILON);
    }
}
