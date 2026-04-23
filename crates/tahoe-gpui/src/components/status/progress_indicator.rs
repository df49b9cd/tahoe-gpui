//! HIG Progress Indicator — determinate or indeterminate progress.
//!
//! A stateless `RenderOnce` component that renders either a horizontal
//! progress bar at a given value (`Determinate`) or an animated 12-tick
//! radial spinner (`Indeterminate`), matching the two `NSProgressIndicator`
//! styles (`.bar` and `.spinning`).
//!
//! # HIG reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/progress-indicators>

use std::time::Duration;

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{AnimatedIcon, Icon, IconAnimation, IconName};
use crate::foundations::materials::{Elevation, Glass, Shape, glass_effect_lens};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, App, ElementId, Hsla, Pixels, SharedString, Window, div, fill, point,
    px, relative,
};

/// HIG-aligned size variants. `Small` matches an 8 pt spinner / 4 pt bar
/// (`NSProgressIndicator.controlSize = .small`); `Regular` matches a
/// 20 pt spinner / 6 pt bar (`NSProgressIndicator` regular default);
/// `Large` matches a 32 pt spinner / 16 pt bar for prominent status
/// panels. Marked `#[non_exhaustive]` so future sizes can land without
/// breaking downstream callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum ProgressIndicatorSize {
    /// Small: 8 pt spinner / 4 pt bar.
    Small,
    /// Regular: 20 pt spinner / 6 pt bar. Matches `NSProgressIndicator` default.
    #[default]
    Regular,
    /// Large: 32 pt spinner / 16 pt bar. For prominent full-width panels.
    Large,
}

impl ProgressIndicatorSize {
    /// Bar-style track height for this size variant.
    pub fn bar_height(self) -> Pixels {
        match self {
            ProgressIndicatorSize::Small => px(4.0),
            ProgressIndicatorSize::Regular => px(6.0),
            ProgressIndicatorSize::Large => px(16.0),
        }
    }

    /// Spinner diameter for this size variant.
    pub fn spinner_diameter(self) -> Pixels {
        match self {
            ProgressIndicatorSize::Small => px(8.0),
            ProgressIndicatorSize::Regular => px(20.0),
            ProgressIndicatorSize::Large => px(32.0),
        }
    }
}

/// Value mode for the progress indicator.
///
/// * `Determinate(f32)` renders a horizontal bar filled to the given
///   fraction (0.0..=1.0).
/// * `Indeterminate` renders an animated 12-tick radial spinner for
///   tasks of unknown duration.
/// * `IndeterminateBar` renders a horizontal bar with a 60%-wide
///   accent stripe sliding across it — HIG indeterminate
///   `NSProgressIndicator.style = .bar` behaviour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProgressIndicatorValue {
    /// Bounded progress in `0.0..=1.0`.
    Determinate(f32),
    /// Unbounded / unknown duration — renders a spinning spinner.
    Indeterminate,
    /// Unbounded / unknown duration — renders an animated bar.
    IndeterminateBar,
}

/// Fraction of track width covered by the moving stripe in the
/// indeterminate bar variant. HIG uses a roughly two-thirds slider;
/// 60% reads as "in motion" without blanking the track.
const INDETERMINATE_BAR_STRIPE_FRAC: f32 = 0.6;

/// Full sweep duration for the indeterminate bar stripe.
const INDETERMINATE_BAR_DURATION: Duration = Duration::from_millis(1400);

/// A progress indicator (bar or spinner).
#[derive(IntoElement)]
pub struct ProgressIndicator {
    id: Option<ElementId>,
    value: ProgressIndicatorValue,
    hig_size: ProgressIndicatorSize,
    height: Option<Pixels>,
    color: Option<Hsla>,
    track_color: Option<Hsla>,
    label: Option<SharedString>,
    glass: bool,
}

impl ProgressIndicator {
    /// Create a *determinate* progress bar with the given value (clamped to
    /// `0.0..=1.0`). Non-finite values (`NaN`, `±∞`) become `0.0`.
    pub fn new(value: f32) -> Self {
        let safe = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            id: None,
            value: ProgressIndicatorValue::Determinate(safe),
            hig_size: ProgressIndicatorSize::Regular,
            height: None,
            color: None,
            track_color: None,
            label: None,
            glass: false,
        }
    }

    /// Create an *indeterminate* progress spinner. The `id` is required so
    /// GPUI can drive the continuous rotation animation.
    pub fn indeterminate(id: impl Into<ElementId>) -> Self {
        Self {
            id: Some(id.into()),
            value: ProgressIndicatorValue::Indeterminate,
            hig_size: ProgressIndicatorSize::Regular,
            height: None,
            color: None,
            track_color: None,
            label: None,
            glass: false,
        }
    }

    /// Create an *indeterminate* progress bar — a horizontal track with
    /// a 60%-wide accent stripe sliding left→right on repeat. The `id`
    /// anchors the animation frame so GPUI can drive the slide.
    ///
    /// Under `AccessibilityMode::REDUCE_MOTION` the stripe freezes at
    /// the 30% position rather than animating.
    pub fn indeterminate_bar(id: impl Into<ElementId>) -> Self {
        Self {
            id: Some(id.into()),
            value: ProgressIndicatorValue::IndeterminateBar,
            hig_size: ProgressIndicatorSize::Regular,
            height: None,
            color: None,
            track_color: None,
            label: None,
            glass: false,
        }
    }

    /// Explicit HIG size variant (default: `Regular`).
    pub fn size(mut self, size: ProgressIndicatorSize) -> Self {
        self.hig_size = size;
        self
    }

    /// Override bar height in points. Ignored for indeterminate spinners.
    pub fn height(mut self, height: Pixels) -> Self {
        self.height = Some(height);
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    pub fn track_color(mut self, color: Hsla) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Optional caption text shown below the bar / spinner.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// When `true` and the current theme is a Liquid Glass theme, wrap
    /// the indicator in a glass surface so it adopts the glass chrome
    /// used by macOS 26 Tahoe system indicators.
    pub fn glass(mut self, glass: bool) -> Self {
        self.glass = glass;
        self
    }
}

impl RenderOnce for ProgressIndicator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let color = self.color.unwrap_or(theme.accent);
        let glass_enabled = self.glass;

        let a11y_label: SharedString = match self.value {
            ProgressIndicatorValue::Determinate(v) => {
                SharedString::from(format!("Progress, {}%", (v * 100.0).round() as i32))
            }
            ProgressIndicatorValue::Indeterminate | ProgressIndicatorValue::IndeterminateBar => {
                SharedString::from("Loading")
            }
        };
        let mut a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::ProgressIndicator);
        if let ProgressIndicatorValue::Determinate(v) = self.value {
            a11y_props = a11y_props.value(SharedString::from(format!(
                "{} percent",
                (v * 100.0).round() as i32
            )));
        }

        let reduce_motion = theme.accessibility_mode.reduce_motion();
        let track_color = self.track_color.unwrap_or(theme.semantic.system_fill);

        let inner: gpui::AnyElement = match self.value {
            ProgressIndicatorValue::Determinate(v) => render_bar(
                v,
                self.hig_size,
                self.height,
                color,
                track_color,
                theme.is_rtl(),
            )
            .into_any_element(),
            ProgressIndicatorValue::Indeterminate => {
                let id = self.id.clone().unwrap_or_else(|| "progress-spinner".into());
                render_spinner(id, self.hig_size, color, reduce_motion).into_any_element()
            }
            ProgressIndicatorValue::IndeterminateBar => {
                let id = self
                    .id
                    .clone()
                    .unwrap_or_else(|| "progress-indeterminate-bar".into());
                render_indeterminate_bar(
                    id,
                    self.hig_size,
                    self.height,
                    color,
                    track_color,
                    theme.is_rtl(),
                    reduce_motion,
                )
                .into_any_element()
            }
        };

        let mut container = div()
            .flex()
            .flex_col()
            .items_start()
            .gap(theme.spacing_xs)
            .child(inner)
            .with_accessibility(&a11y_props);

        if let Some(label) = self.label {
            container = container.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
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

fn render_bar(
    value: f32,
    hig_size: ProgressIndicatorSize,
    height_override: Option<Pixels>,
    color: Hsla,
    track: Hsla,
    is_rtl: bool,
) -> impl IntoElement {
    let h = height_override.unwrap_or_else(|| hig_size.bar_height());
    // HIG: `NSProgressIndicator` uses a 2 pt corner radius on the
    // determinate bar track — never a full pill.
    let natural_radius = px(2.0f32.min(f32::from(h) / 2.0));

    let mut track_el = div()
        .w_full()
        .h(h)
        .overflow_hidden()
        .rounded(natural_radius)
        .bg(track)
        .flex();
    if is_rtl {
        track_el = track_el.flex_row_reverse();
    } else {
        track_el = track_el.flex_row();
    }

    track_el.child(
        div()
            .h_full()
            .rounded(natural_radius)
            .bg(color)
            .w(relative(value)),
    )
}

fn render_spinner(
    id: ElementId,
    hig_size: ProgressIndicatorSize,
    color: Hsla,
    reduce_motion: bool,
) -> gpui::AnyElement {
    // HIG: the macOS indeterminate spinner is a 12-tick radial
    // `NSProgressIndicator.style = .spinning`. Under Reduce Motion, HIG
    // requires halting the rotation — Apple's own `UIActivityIndicatorView`
    // freezes. Mirror that by rendering a static Icon. (`AnimatedIcon::Spin`
    // does not internally honour Reduce Motion, so the caller must gate.)
    if reduce_motion {
        Icon::new(IconName::ProgressSpinner)
            .size(hig_size.spinner_diameter())
            .color(color)
            .into_any_element()
    } else {
        AnimatedIcon::new(
            id,
            IconName::ProgressSpinner,
            IconAnimation::Spin {
                duration: Duration::from_millis(1200),
            },
        )
        .size(hig_size.spinner_diameter())
        .color(color)
        .into_any_element()
    }
}

/// Return the stripe position (as fraction of the track width to the
/// *right* of the stripe's right edge) for a given animation `delta`
/// (0.0..=1.0) and reduce-motion setting.
///
/// When reduce-motion is active the stripe freezes at the 30% position
/// so the user still sees a stable "work in progress" signal without
/// the perceived motion.
///
/// The stripe sweeps from fully off-screen-left to fully off-screen-right,
/// so the returned value spans `[0.0, 1.0 + stripe_frac]` mapped onto
/// the animation's normalized time. The stripe's *left* edge is
/// `right_edge - stripe_frac`.
fn indeterminate_bar_progress(delta: f32, stripe_frac: f32, reduce_motion: bool) -> f32 {
    if reduce_motion {
        // Freeze with the stripe sitting 30% into the track (its right
        // edge at 0.3 + stripe_frac).
        return 0.3 + stripe_frac;
    }
    // Travel range: right-edge goes from 0.0 (stripe fully left of track)
    // to 1.0 + stripe_frac (stripe fully right of track).
    delta * (1.0 + stripe_frac)
}

fn render_indeterminate_bar(
    id: ElementId,
    hig_size: ProgressIndicatorSize,
    height_override: Option<Pixels>,
    color: Hsla,
    track: Hsla,
    is_rtl: bool,
    reduce_motion: bool,
) -> impl IntoElement {
    let h = height_override.unwrap_or_else(|| hig_size.bar_height());
    // Match the determinate bar's 2pt corner radius (HIG
    // `NSProgressIndicator.bar`) — never a full pill.
    let natural_radius = px(2.0f32.min(f32::from(h) / 2.0));

    IndeterminateBarElement {
        id,
        height: h,
        radius: natural_radius,
        color,
        track,
        is_rtl,
        reduce_motion,
    }
}

/// Custom element that paints an indeterminate-bar stripe using
/// GPUI's `with_animation` delta — lets us respect `reduce_motion`
/// without pulling in a full stateful entity.
struct IndeterminateBarElement {
    id: ElementId,
    height: Pixels,
    radius: Pixels,
    color: Hsla,
    track: Hsla,
    is_rtl: bool,
    reduce_motion: bool,
}

impl IntoElement for IndeterminateBarElement {
    type Element = gpui::AnyElement;

    fn into_element(self) -> Self::Element {
        let IndeterminateBarElement {
            id,
            height,
            radius,
            color,
            track,
            is_rtl,
            reduce_motion,
        } = self;

        // Freeze the element at the 30% position when reduce-motion is on.
        if reduce_motion {
            return indeterminate_bar_frame(
                indeterminate_bar_progress(0.0, INDETERMINATE_BAR_STRIPE_FRAC, true),
                height,
                radius,
                color,
                track,
                is_rtl,
            )
            .into_any_element();
        }

        // Animate the stripe sliding from left to right on repeat.
        let base = indeterminate_bar_frame(0.0, height, radius, color, track, is_rtl);
        base.with_animation(
            id,
            Animation::new(INDETERMINATE_BAR_DURATION).repeat(),
            move |_el, delta| {
                let progress =
                    indeterminate_bar_progress(delta, INDETERMINATE_BAR_STRIPE_FRAC, false);
                indeterminate_bar_frame(progress, height, radius, color, track, is_rtl)
            },
        )
        .into_any_element()
    }
}

/// Render one frame of the indeterminate bar at `right_edge_frac` — the
/// fraction of the track width covered by the stripe's right edge.
fn indeterminate_bar_frame(
    right_edge_frac: f32,
    h: Pixels,
    radius: Pixels,
    color: Hsla,
    track: Hsla,
    is_rtl: bool,
) -> impl IntoElement {
    use gpui::{Bounds, Element, GlobalElementId, InspectorElementId, LayoutId, Style};

    struct Frame {
        right_edge_frac: f32,
        height: Pixels,
        radius: Pixels,
        color: Hsla,
        track: Hsla,
        is_rtl: bool,
    }

    impl IntoElement for Frame {
        type Element = Self;

        fn into_element(self) -> Self::Element {
            self
        }
    }

    impl Element for Frame {
        type RequestLayoutState = ();
        type PrepaintState = ();

        fn id(&self) -> Option<ElementId> {
            None
        }

        fn request_layout(
            &mut self,
            _id: Option<&GlobalElementId>,
            _inspector_id: Option<&InspectorElementId>,
            window: &mut Window,
            cx: &mut App,
        ) -> (LayoutId, Self::RequestLayoutState) {
            let mut style = Style::default();
            style.size.width = relative(1.0).into();
            style.size.height = self.height.into();
            (window.request_layout(style, [], cx), ())
        }

        fn prepaint(
            &mut self,
            _id: Option<&GlobalElementId>,
            _inspector_id: Option<&InspectorElementId>,
            _bounds: Bounds<Pixels>,
            _request_layout: &mut Self::RequestLayoutState,
            _window: &mut Window,
            _cx: &mut App,
        ) -> Self::PrepaintState {
        }

        fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
            None
        }

        fn paint(
            &mut self,
            _id: Option<&GlobalElementId>,
            _inspector_id: Option<&InspectorElementId>,
            bounds: Bounds<Pixels>,
            _request_layout: &mut Self::RequestLayoutState,
            _prepaint: &mut Self::PrepaintState,
            window: &mut Window,
            _cx: &mut App,
        ) {
            // Track background fills the full width.
            window.paint_quad(fill(bounds, self.track).corner_radii(self.radius));

            let stripe_frac = INDETERMINATE_BAR_STRIPE_FRAC;
            let width = bounds.size.width;
            // `right_edge_frac` is the normalised position of the stripe's
            // right edge. The stripe's left edge sits `stripe_frac` to its
            // left, clamped to track boundaries so we never overdraw.
            let raw_right = f32::from(width) * self.right_edge_frac;
            let raw_left = raw_right - f32::from(width) * stripe_frac;
            let left = raw_left.max(0.0);
            let right = raw_right.min(f32::from(width));
            if right <= left {
                return;
            }
            let stripe_w = px(right - left);
            let origin_x = if self.is_rtl {
                bounds.right() - px(right)
            } else {
                bounds.left() + px(left)
            };
            window.paint_quad(
                fill(
                    Bounds::new(
                        point(origin_x, bounds.top()),
                        gpui::size(stripe_w, bounds.size.height),
                    ),
                    self.color,
                )
                .corner_radii(self.radius),
            );
        }
    }

    Frame {
        right_edge_frac,
        height: h,
        radius,
        color,
        track,
        is_rtl,
    }
}

#[cfg(test)]
mod tests {
    use crate::components::status::progress_indicator::{
        ProgressIndicator, ProgressIndicatorSize, ProgressIndicatorValue,
    };
    use core::prelude::v1::test;
    use gpui::{hsla, px};

    #[test]
    fn value_clamped_to_0_1() {
        let bar = ProgressIndicator::new(-0.5);
        assert!(
            matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON)
        );

        let bar = ProgressIndicator::new(1.5);
        assert!(
            matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 1.0).abs() < f32::EPSILON)
        );

        let bar = ProgressIndicator::new(0.5);
        assert!(
            matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.5).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn new_defaults_to_determinate_regular() {
        let bar = ProgressIndicator::new(0.5);
        assert!(matches!(bar.value, ProgressIndicatorValue::Determinate(_)));
        assert_eq!(bar.hig_size, ProgressIndicatorSize::Regular);
        assert!(bar.height.is_none());
        assert!(bar.label.is_none());
        assert!(!bar.glass);
    }

    #[test]
    fn indeterminate_constructor_stores_id() {
        let bar = ProgressIndicator::indeterminate("spinner");
        assert!(matches!(bar.value, ProgressIndicatorValue::Indeterminate));
        assert!(bar.id.is_some());
    }

    #[test]
    fn color_builder() {
        let color = hsla(0.5, 1.0, 0.5, 1.0);
        let bar = ProgressIndicator::new(0.5).color(color);
        assert_eq!(bar.color, Some(color));
    }

    #[test]
    fn track_color_builder() {
        let color = hsla(0.0, 0.0, 0.8, 1.0);
        let bar = ProgressIndicator::new(0.5).track_color(color);
        assert_eq!(bar.track_color, Some(color));
    }

    #[test]
    fn nan_value_treated_as_zero() {
        let bar = ProgressIndicator::new(f32::NAN);
        assert!(
            matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn infinity_value_treated_as_zero() {
        let bar = ProgressIndicator::new(f32::INFINITY);
        assert!(
            matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON)
        );

        let bar = ProgressIndicator::new(f32::NEG_INFINITY);
        assert!(
            matches!(bar.value, ProgressIndicatorValue::Determinate(v) if (v - 0.0).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn label_builder() {
        let bar = ProgressIndicator::new(0.25).label("Downloading…");
        assert_eq!(bar.label.as_ref().map(|s| s.as_ref()), Some("Downloading…"));
    }

    #[test]
    fn size_builder_selects_small() {
        let bar = ProgressIndicator::new(0.5).size(ProgressIndicatorSize::Small);
        assert_eq!(bar.hig_size, ProgressIndicatorSize::Small);
    }

    #[test]
    fn bar_height_defaults_match_hig() {
        // HIG: NSProgressIndicator regular determinate bar = 6pt (with
        // the Large-variant bump landing 16pt on panels). Small = 4pt
        // is the revised dense-row height.
        assert_eq!(ProgressIndicatorSize::Regular.bar_height(), px(6.0));
        assert_eq!(ProgressIndicatorSize::Small.bar_height(), px(4.0));
        assert_eq!(ProgressIndicatorSize::Large.bar_height(), px(16.0));
    }

    #[test]
    fn spinner_diameter_defaults_match_hig() {
        // HIG: NSProgressIndicator regular spinner = 20pt, small = 8pt,
        // large = 32pt.
        assert_eq!(ProgressIndicatorSize::Regular.spinner_diameter(), px(20.0));
        assert_eq!(ProgressIndicatorSize::Small.spinner_diameter(), px(8.0));
        assert_eq!(ProgressIndicatorSize::Large.spinner_diameter(), px(32.0));
    }

    #[test]
    fn explicit_height_override_wins() {
        let bar = ProgressIndicator::new(0.5).height(px(9.0));
        assert_eq!(bar.height, Some(px(9.0)));
    }

    #[test]
    fn glass_builder_opt_in() {
        let bar = ProgressIndicator::new(0.5);
        assert!(!bar.glass);
        let bar = bar.glass(true);
        assert!(bar.glass);
    }

    /// `indeterminate_bar` produces the new bar variant and stashes the
    /// id for animation anchoring.
    #[test]
    fn indeterminate_bar_constructor_stores_id_and_variant() {
        let bar = ProgressIndicator::indeterminate_bar("ind-bar");
        assert!(matches!(
            bar.value,
            ProgressIndicatorValue::IndeterminateBar
        ));
        assert!(bar.id.is_some());
    }

    /// The indeterminate bar stripe freezes at 30% when reduce-motion is on.
    #[test]
    fn indeterminate_bar_freezes_under_reduce_motion() {
        use super::{INDETERMINATE_BAR_STRIPE_FRAC, indeterminate_bar_progress};
        let frozen = indeterminate_bar_progress(0.0, INDETERMINATE_BAR_STRIPE_FRAC, true);
        // At the frozen position the stripe's right edge sits at
        // 0.3 + stripe_frac; its left edge therefore sits at 0.3.
        assert!(
            (frozen - (0.3 + INDETERMINATE_BAR_STRIPE_FRAC)).abs() < f32::EPSILON,
            "frozen position wrong: {frozen}"
        );
    }

    /// Animated delta sweeps the stripe from fully off-screen left
    /// (right-edge at 0.0) to fully off-screen right
    /// (right-edge at 1.0 + stripe_frac).
    #[test]
    fn indeterminate_bar_sweeps_across_track() {
        use super::{INDETERMINATE_BAR_STRIPE_FRAC, indeterminate_bar_progress};
        let start = indeterminate_bar_progress(0.0, INDETERMINATE_BAR_STRIPE_FRAC, false);
        let mid = indeterminate_bar_progress(0.5, INDETERMINATE_BAR_STRIPE_FRAC, false);
        let end = indeterminate_bar_progress(1.0, INDETERMINATE_BAR_STRIPE_FRAC, false);
        assert!((start - 0.0).abs() < f32::EPSILON);
        assert!((end - (1.0 + INDETERMINATE_BAR_STRIPE_FRAC)).abs() < f32::EPSILON);
        assert!(
            mid > start && mid < end,
            "midpoint should fall between endpoints: {mid}"
        );
    }

    /// `Large` selects the 16pt bar and 32pt spinner tokens.
    #[test]
    fn size_builder_selects_large() {
        let bar = ProgressIndicator::new(0.5).size(ProgressIndicatorSize::Large);
        assert_eq!(bar.hig_size, ProgressIndicatorSize::Large);
    }
}
