//! Status badge component.
//!
//! Renders a small status pill, a numeric notification badge, or a solid
//! presence dot. The variant controls both the colour and the shape:
//!
//! - [`BadgeVariant::Default`] … [`BadgeVariant::Muted`] — semantic pills
//!   tinted with the theme's Liquid Glass palette.
//! - [`BadgeVariant::Notification`] — opaque red pill carrying an optional
//!   unread count (macOS Dock / iOS app-icon style).
//! - [`BadgeVariant::Dot`] — 8 pt opaque circle for silent presence /
//!   unread indicators (Zed's `UnreadIndicator` equivalent).

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::color::text_on_background;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::BADGE_DOT_SIZE;
use crate::foundations::theme::{ActiveTheme, GlassSize, GlassTintColor, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{App, SharedString, Window, div, px};

/// Badge style variant.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
    Muted,
    /// Numeric notification pill (opaque red with white text). Set
    /// `count = None` to render an unlabelled red pill; supply a count to
    /// render the number.
    Notification {
        count: Option<u32>,
    },
    /// Solid 8 pt presence dot (same opaque red as `Notification`).
    /// Silent indicator with no text; use for "unread" / "new" markers.
    Dot,
}

/// A small status badge/pill.
#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    variant: BadgeVariant,
    interactive: bool,
    accessibility_label: Option<SharedString>,
}

impl Badge {
    /// Create a badge displaying `label` in the default variant.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::default(),
            interactive: false,
            accessibility_label: None,
        }
    }

    /// Convenience constructor for a notification badge that carries a
    /// count. `count = 0` is treated as "no count" and renders an
    /// unlabelled red pill.
    pub fn notification(count: u32) -> Self {
        let variant = BadgeVariant::Notification {
            count: (count > 0).then_some(count),
        };
        let label = if count > 0 {
            if count > 99 {
                SharedString::from("99+")
            } else {
                SharedString::from(count.to_string())
            }
        } else {
            SharedString::from("")
        };
        Self {
            label,
            variant,
            interactive: false,
            accessibility_label: None,
        }
    }

    /// Convenience constructor for an unread dot.
    pub fn dot() -> Self {
        Self {
            label: SharedString::from(""),
            variant: BadgeVariant::Dot,
            interactive: false,
            accessibility_label: None,
        }
    }

    /// Set the badge variant, which controls color and shape. See [`BadgeVariant`].
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Mark the badge as interactive (e.g. a filter chip). Interactive
    /// badges use HIG-compliant 20 pt minimum height — non-interactive
    /// badges stay compact.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Override the VoiceOver label. Defaults to the badge's display text
    /// for text badges, "unread indicator" for Dot, and a count-derived
    /// string for Notification.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let glass = &theme.glass;

        // Derive display label from the variant's count when the caller used
        // `.variant(Notification { count })` directly (bypassing Badge::notification()).
        let effective_label: SharedString = if self.label.is_empty() {
            match self.variant {
                BadgeVariant::Notification { count: Some(c) } => {
                    if c > 99 {
                        SharedString::from("99+")
                    } else {
                        SharedString::from(c.to_string())
                    }
                }
                _ => self.label.clone(),
            }
        } else {
            self.label.clone()
        };

        // Dot: opaque circle, no text, no shadow.
        if matches!(self.variant, BadgeVariant::Dot) {
            let bg = theme.palette.red;
            let a11y_label = self
                .accessibility_label
                .clone()
                .unwrap_or_else(|| SharedString::from("unread indicator"));
            let a11y_props = AccessibilityProps::new()
                .label(a11y_label)
                .role(AccessibilityRole::Image);
            let mut el = div()
                .size(px(BADGE_DOT_SIZE))
                .rounded(theme.radius_full)
                .bg(bg)
                .with_accessibility(&a11y_props);
            el = crate::foundations::materials::apply_high_contrast_border(el, theme);
            return el.into_any_element();
        }

        let (bg, text_color, use_shadow, opaque) = match self.variant {
            BadgeVariant::Default => (
                glass.accessible_bg(GlassSize::Small, theme.accessibility_mode),
                theme.text,
                true,
                false,
            ),
            BadgeVariant::Success => {
                let bg = crate::foundations::materials::accessible_tint_bg(
                    glass.tints.get(GlassTintColor::Green),
                    theme.accessibility_mode,
                );
                (bg, text_on_background(bg), true, false)
            }
            BadgeVariant::Warning => {
                let bg = crate::foundations::materials::accessible_tint_bg(
                    glass.tints.get(GlassTintColor::Amber),
                    theme.accessibility_mode,
                );
                (bg, text_on_background(bg), true, false)
            }
            BadgeVariant::Error => {
                let bg = crate::foundations::materials::accessible_tint_bg(
                    glass.tints.get(GlassTintColor::Red),
                    theme.accessibility_mode,
                );
                (bg, text_on_background(bg), true, false)
            }
            BadgeVariant::Info => {
                let bg = crate::foundations::materials::accessible_tint_bg(
                    glass.tints.get(GlassTintColor::Blue),
                    theme.accessibility_mode,
                );
                (bg, text_on_background(bg), true, false)
            }
            BadgeVariant::Muted => (
                glass.accessible_bg(GlassSize::Small, theme.accessibility_mode),
                theme.text_muted,
                true,
                false,
            ),
            BadgeVariant::Notification { .. } => {
                // Opaque red pill per HIG notification guidance. Flat (no
                // shadow) to distinguish content badges from interactive
                // glass pills.
                let bg = theme.palette.red;
                (bg, theme.text_on_accent, false, true)
            }
            BadgeVariant::Dot => unreachable!("Dot variant handled by early return above"),
        };

        // Empty-label semantic pills produce visible glass shapes with no
        // informational content. Skip rendering for those variants.
        if effective_label.is_empty()
            && !matches!(
                self.variant,
                BadgeVariant::Notification { .. } | BadgeVariant::Dot
            )
        {
            return div().into_any_element();
        }

        let vertical_padding = px(f32::from(theme.spacing_xs) / 2.0);

        let mut el = div()
            .px(theme.spacing_sm)
            .py(vertical_padding)
            .rounded(theme.radius_full)
            .bg(bg)
            .text_color(text_color)
            .text_style(TextStyle::Caption1, theme);

        if self.interactive {
            el = el.min_h(px(theme.min_target_size()));
        }

        if use_shadow {
            el = el.shadow(glass.shadows(GlassSize::Small).to_vec());
        }
        if !opaque {
            el = crate::foundations::materials::apply_high_contrast_border(el, theme);
        }

        // DWC: prepend an icon so state is not conveyed by color alone.
        let dwc_icon: Option<IconName> = if theme.accessibility_mode.differentiate_without_color() {
            match self.variant {
                BadgeVariant::Success => Some(IconName::Check),
                BadgeVariant::Warning => Some(IconName::AlertTriangle),
                BadgeVariant::Error => Some(IconName::XmarkCircleFill),
                BadgeVariant::Info => Some(IconName::Info),
                BadgeVariant::Notification { .. } => Some(IconName::CircleFilled),
                _ => None,
            }
        } else {
            None
        };

        if let Some(icon_name) = dwc_icon {
            el = el.flex().items_center().gap(theme.spacing_xs).child(
                Icon::new(icon_name)
                    .size(theme.icon_size_xs)
                    .color(text_color),
            );
        }

        let a11y_label: SharedString = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| effective_label.clone());
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::StaticText);

        el.child(effective_label)
            .with_accessibility(&a11y_props)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{Badge, BadgeVariant};
    use core::prelude::v1::test;

    #[test]
    fn badge_variant_default() {
        assert_eq!(BadgeVariant::default(), BadgeVariant::Default);
    }

    #[test]
    fn badge_variant_equality() {
        assert_eq!(BadgeVariant::Success, BadgeVariant::Success);
        assert_ne!(BadgeVariant::Success, BadgeVariant::Error);
    }

    #[test]
    fn badge_variant_all_distinct() {
        let variants = [
            BadgeVariant::Default,
            BadgeVariant::Success,
            BadgeVariant::Warning,
            BadgeVariant::Error,
            BadgeVariant::Info,
            BadgeVariant::Muted,
            BadgeVariant::Notification { count: None },
            BadgeVariant::Notification { count: Some(3) },
            BadgeVariant::Dot,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i == j {
                    assert_eq!(variants[i], variants[j]);
                } else {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    #[test]
    fn badge_variant_copy() {
        let v = BadgeVariant::Warning;
        let v2 = v;
        assert_eq!(v, v2);
    }

    #[test]
    fn notification_constructor_sets_variant_and_label() {
        let b = Badge::notification(3);
        assert_eq!(b.variant, BadgeVariant::Notification { count: Some(3) });
        assert_eq!(b.label.as_ref(), "3");
    }

    #[test]
    fn notification_constructor_zero_count_is_unlabelled() {
        let b = Badge::notification(0);
        assert_eq!(b.variant, BadgeVariant::Notification { count: None });
        assert_eq!(b.label.as_ref(), "");
    }

    #[test]
    fn dot_constructor_sets_variant() {
        let b = Badge::dot();
        assert_eq!(b.variant, BadgeVariant::Dot);
        assert_eq!(b.label.as_ref(), "");
    }

    #[test]
    fn interactive_defaults_to_false() {
        let b = Badge::new("test");
        assert!(!b.interactive);
    }

    #[test]
    fn interactive_builder() {
        let b = Badge::new("Filter").interactive(true);
        assert!(b.interactive);
    }

    #[test]
    fn notification_count_at_99_shows_99() {
        let b = Badge::notification(99);
        assert_eq!(b.label.as_ref(), "99");
    }

    #[test]
    fn notification_count_capped_at_99() {
        let b = Badge::notification(100);
        assert_eq!(b.label.as_ref(), "99+");
    }

    #[test]
    fn notification_count_capped_large() {
        let b = Badge::notification(999_999_999);
        assert_eq!(b.label.as_ref(), "99+");
    }

    #[test]
    fn accessibility_label_defaults_to_none() {
        let b = Badge::new("test");
        assert!(b.accessibility_label.is_none());
    }

    #[test]
    fn accessibility_label_builder() {
        let b = Badge::new("Active").accessibility_label("Active status");
        assert_eq!(
            b.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Active status")
        );
    }

    #[test]
    fn dot_constructor_accessibility_label_none() {
        let b = Badge::dot();
        assert!(b.accessibility_label.is_none());
    }

    #[test]
    fn notification_constructor_accessibility_label_none() {
        let b = Badge::notification(5);
        assert!(b.accessibility_label.is_none());
    }

    #[test]
    fn notification_variant_with_empty_label_carries_count() {
        let b = Badge::new("").variant(BadgeVariant::Notification { count: Some(7) });
        assert!(b.label.is_empty());
        assert_eq!(b.variant, BadgeVariant::Notification { count: Some(7) });
    }

    #[test]
    fn notification_variant_with_zero_count_empty_label() {
        let b = Badge::new("").variant(BadgeVariant::Notification { count: None });
        assert!(b.label.is_empty());
        assert_eq!(b.variant, BadgeVariant::Notification { count: None });
    }
}
