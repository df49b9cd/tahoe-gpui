//! Indeterminate activity indicator component (HIG).
//!
//! Renders the macOS 12-tick radial spinner (`IconName::ProgressSpinner`,
//! matching `NSProgressIndicator.style = .spinning`). Under
//! `AccessibilityMode::REDUCE_MOTION` the spinner is rendered fully
//! static — no pulse, no opacity oscillation — matching
//! `UIActivityIndicatorView`'s behaviour when `isAnimating` is false.
//!
//! # HIG reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/progress-indicators>

use gpui::prelude::*;
use gpui::{App, Hsla, Pixels, SharedString, Window, div, px};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{AnimatedIcon, Icon, IconAnimation, IconName};
use crate::foundations::materials::{Elevation, Glass, Shape, glass_effect_lens};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// Default spin animation duration.
const SPIN_DURATION: std::time::Duration = std::time::Duration::from_millis(1200);

/// HIG-aligned size variant for [`ActivityIndicator`]. Small / Regular /
/// Large map to `NSProgressIndicator.controlSize` of `.small`,
/// `.regular`, and `.large` — 16pt, 24pt, and 32pt respectively.
///
/// Marked `#[non_exhaustive]` so we can add additional sizes (e.g. an
/// `ExtraLarge` for watch faces) without breaking downstream callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ActivityIndicatorSize {
    /// 16pt — dense list rows, inline toolbar glyphs.
    Small,
    /// 24pt — default control size.
    #[default]
    Regular,
    /// 32pt — prominent empty-state / full-width progress panels.
    Large,
}

impl ActivityIndicatorSize {
    /// Resolve the variant to its logical-pixel diameter.
    pub fn diameter(self) -> Pixels {
        match self {
            ActivityIndicatorSize::Small => px(16.0),
            ActivityIndicatorSize::Regular => px(24.0),
            ActivityIndicatorSize::Large => px(32.0),
        }
    }
}

/// Activity indicator style per HIG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActivityIndicatorStyle {
    /// Standard 12-tick radial spinning indicator.
    #[default]
    Spinning,
    /// Pulsing dot indicator (for subtle loading states).
    Pulsing,
}

/// An indeterminate activity indicator per HIG.
///
/// Renders [`IconName::ProgressSpinner`] — the 12-tick radial spinner
/// matching `NSProgressIndicator.style = .spinning` — with a continuous
/// rotation animation. Under `REDUCE_MOTION` the animation is suppressed
/// entirely and the spinner is rendered as a static image; under
/// `.stopped(true)` the indicator renders nothing at all (equivalent to
/// `UIActivityIndicatorView.hidesWhenStopped`).
///
/// # Example
/// ```ignore
/// ActivityIndicator::new("loading")
///     .label("Loading\u{2026}")
///     .size(px(24.0))
/// ```
#[derive(IntoElement)]
pub struct ActivityIndicator {
    id: gpui::ElementId,
    label: Option<SharedString>,
    style: ActivityIndicatorStyle,
    size: Option<Pixels>,
    /// HIG size token. Controls the fallback diameter when an explicit
    /// `size(Pixels)` override is not provided. Defaults to `Regular`.
    hig_size: ActivityIndicatorSize,
    color: Option<Hsla>,
    stopped: bool,
    glass: bool,
}

impl ActivityIndicator {
    pub fn new(id: impl Into<gpui::ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            style: ActivityIndicatorStyle::default(),
            size: None,
            hig_size: ActivityIndicatorSize::default(),
            color: None,
            stopped: false,
            glass: false,
        }
    }

    /// Optional label displayed *below* the spinner (HIG macOS: spinner
    /// first, label beneath).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the indicator style (default: Spinning).
    pub fn style(mut self, style: ActivityIndicatorStyle) -> Self {
        self.style = style;
        self
    }

    /// Custom indicator size in explicit pixels. When both this override
    /// and [`ActivityIndicator::hig_size`] are set, the pixel override
    /// wins. Preserved for back-compat with callers that want off-HIG
    /// diameters (e.g. voice-player inline controls).
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    /// HIG-aligned size token. Default = [`ActivityIndicatorSize::Regular`]
    /// (24pt, matching `NSProgressIndicator.controlSize = .regular`).
    ///
    /// Prefer this over [`ActivityIndicator::size`] whenever a HIG size
    /// applies — it keeps layout consistent across the app and scales
    /// with any future Dynamic Type or `#[non_exhaustive]` additions.
    pub fn hig_size(mut self, size: ActivityIndicatorSize) -> Self {
        self.hig_size = size;
        self
    }

    /// Custom indicator color (default: `theme.text_muted`).
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// When `true`, render nothing — matches
    /// `UIActivityIndicatorView.hidesWhenStopped = true` (the default on
    /// Apple platforms).
    pub fn stopped(mut self, stopped: bool) -> Self {
        self.stopped = stopped;
        self
    }

    /// When `true` and the current theme is a Liquid Glass theme, wrap the
    /// spinner in a glass surface so it matches `NSProgressIndicator` inside
    /// glass chrome on macOS 26 Tahoe.
    pub fn glass(mut self, glass: bool) -> Self {
        self.glass = glass;
        self
    }
}

impl RenderOnce for ActivityIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        // HIG: `UIActivityIndicatorView.hidesWhenStopped = true` by default —
        // stopped indicators are removed from the view hierarchy.
        if self.stopped {
            return div().into_any_element();
        }

        // Prefer the explicit pixel override if provided; otherwise fall
        // back to the HIG size token. Literal defaults moved into
        // `ActivityIndicatorSize::diameter()` so the internal
        // `DEFAULT_SIZE` constant is no longer needed.
        let size = self.size.unwrap_or_else(|| self.hig_size.diameter());
        let color = self.color.unwrap_or(theme.text_muted);
        let glass_enabled = self.glass;

        let a11y_label: SharedString = self
            .label
            .clone()
            .unwrap_or_else(|| SharedString::from("Loading"));
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::ProgressIndicator);

        // HIG Motion: when the user enables Reduce Motion, replace dramatic
        // transitions with subtle cross-fades or omit them entirely. Apple's
        // `UIActivityIndicatorView` fully stops animation under Reduce
        // Motion — we mirror that by rendering a static icon (no pulse, no
        // opacity oscillation).
        let spinner: gpui::AnyElement = if theme.accessibility_mode.reduce_motion() {
            Icon::new(IconName::ProgressSpinner)
                .size(size)
                .color(color)
                .into_any_element()
        } else {
            AnimatedIcon::new(
                self.id,
                IconName::ProgressSpinner,
                IconAnimation::Spin {
                    duration: SPIN_DURATION,
                },
            )
            .size(size)
            .color(color)
            .into_any_element()
        };

        let mut container = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(theme.spacing_xs)
            .with_accessibility(&a11y_props)
            .child(spinner);

        // HIG macOS: spinner first, label below. Reordered from the prior
        // implementation which placed the label above the spinner.
        if let Some(label) = self.label {
            container = container.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(color)
                    .child(label),
            );
        }

        if glass_enabled {
            glass_effect_lens(
                theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Resting,
                None,
            )
            .p(theme.spacing_sm)
            .child(container)
            .into_any_element()
        } else {
            container.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::components::status::activity_indicator::{
        ActivityIndicator, ActivityIndicatorSize, ActivityIndicatorStyle,
    };
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn default_construction() {
        let indicator = ActivityIndicator::new("test");
        assert!(indicator.label.is_none());
        assert!(indicator.size.is_none());
        assert!(indicator.color.is_none());
        assert!(!indicator.stopped);
        assert!(!indicator.glass);
        assert_eq!(indicator.style, ActivityIndicatorStyle::Spinning);
        assert_eq!(indicator.hig_size, ActivityIndicatorSize::Regular);
    }

    #[test]
    fn with_label() {
        let indicator = ActivityIndicator::new("test").label("Loading...");
        assert!(indicator.label.is_some());
        assert_eq!(indicator.label.unwrap().as_ref(), "Loading...");
    }

    #[test]
    fn with_custom_size() {
        let indicator = ActivityIndicator::new("test").size(px(32.0));
        assert_eq!(indicator.size, Some(px(32.0)));
    }

    #[test]
    fn with_custom_color() {
        let color = gpui::hsla(0.5, 0.5, 0.5, 1.0);
        let indicator = ActivityIndicator::new("test").color(color);
        assert_eq!(indicator.color, Some(color));
    }

    #[test]
    fn builder_chaining() {
        let color = gpui::hsla(0.0, 1.0, 0.5, 1.0);
        let indicator = ActivityIndicator::new("test")
            .label("Please wait")
            .size(px(48.0))
            .color(color);
        assert_eq!(indicator.label.unwrap().as_ref(), "Please wait");
        assert_eq!(indicator.size, Some(px(48.0)));
        assert_eq!(indicator.color, Some(color));
    }

    #[test]
    fn stopped_flag_toggles() {
        let indicator = ActivityIndicator::new("test");
        assert!(!indicator.stopped);
        let indicator = indicator.stopped(true);
        assert!(indicator.stopped);
    }

    #[test]
    fn glass_flag_toggles() {
        let indicator = ActivityIndicator::new("test").glass(true);
        assert!(indicator.glass);
    }

    /// HIG diameters: Small = 16pt, Regular = 24pt, Large = 32pt.
    #[test]
    fn hig_size_diameter_mapping_matches_hig() {
        assert_eq!(ActivityIndicatorSize::Small.diameter(), px(16.0));
        assert_eq!(ActivityIndicatorSize::Regular.diameter(), px(24.0));
        assert_eq!(ActivityIndicatorSize::Large.diameter(), px(32.0));
    }

    /// `.hig_size(..)` flows into the struct and overrides the default.
    #[test]
    fn hig_size_builder_stores_variant() {
        let indicator = ActivityIndicator::new("test").hig_size(ActivityIndicatorSize::Large);
        assert_eq!(indicator.hig_size, ActivityIndicatorSize::Large);
    }
}
