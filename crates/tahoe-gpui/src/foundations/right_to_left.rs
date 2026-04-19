//! Right-to-left layout support aligned with HIG.
//!
//! Apple platforms automatically mirror UI layouts for RTL languages like
//! Arabic and Hebrew. This module provides utilities for components that
//! need explicit RTL handling.
//!
//! # What mirrors automatically
//!
//! Components using [`flex_row_directed`](super::materials::flex_row_directed) already
//! respect the layout direction from `TahoeTheme::layout_direction`.
//!
//! # What needs manual mirroring
//!
//! - Directional icons (arrows, chevrons) — flip horizontally
//! - Progress indicators that fill left-to-right — reverse direction via
//!   [`progress_fill_direction`]
//! - Asymmetric padding/margins — swap leading/trailing via
//!   [`leading_inset`] / [`trailing_inset`]

use gpui::Pixels;

pub use super::layout::LayoutDirection;

/// Returns the flex direction to use when laying out a row whose first child
/// should appear at the *leading* edge. In LTR the leading edge is the left,
/// so a plain `Row` places the first child on the left; in RTL the leading
/// edge is the right, so `RowReverse` places the first child on the right.
pub fn flex_direction_for_layout(direction: LayoutDirection) -> gpui::FlexDirection {
    match direction {
        LayoutDirection::LeftToRight => gpui::FlexDirection::Row,
        LayoutDirection::RightToLeft => gpui::FlexDirection::RowReverse,
    }
}

/// Returns whether a directional icon should be flipped for the current layout.
///
/// Per HIG, icons like arrows and chevrons that indicate direction
/// should be mirrored in RTL layouts. Icons that represent physical objects
/// (e.g., a clock with hands) should NOT be mirrored.
///
/// Prefer [`icon_direction`] for per-icon decisions — this helper only
/// reports the reading direction and leaves the flip-vs-localized-variant
/// choice to the caller. Directional behavior per [`IconName`] is
/// authoritative on the `IconName` enum (`layout_behavior()` method).
pub fn should_flip_icon(direction: LayoutDirection) -> bool {
    direction.is_rtl()
}

/// Per-icon RTL treatment.
///
/// Finding 31 in the Zed cross-reference audit:
/// SF Symbols ship different behaviours for different glyphs. Arrows and
/// chevrons must be mirrored geometrically; the signature glyph and a
/// handful of rich-text symbols ship with a localised Arabic / Hebrew
/// variant that callers should swap in rather than flipping. Symbols
/// without direction (clocks, search magnifier, cameras) must stay
/// upright regardless of reading direction.
///
/// `IconName::layout_behavior()` (in `foundations/icons/names.rs`)
/// reports the authoritative per-symbol decision; [`icon_direction`]
/// folds the current layout direction into that decision so callers only
/// need a single call at render time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconDirection {
    /// Leave the glyph as rendered by the asset — either LTR-only or
    /// directionally neutral.
    #[default]
    Neutral,
    /// Mirror the glyph horizontally in RTL. Used for arrows, chevrons,
    /// progress indicators, and other purely-directional glyphs.
    Flip,
    /// Swap in a localised asset variant in RTL — HIG says the signature
    /// glyph and certain rich-text symbols "offer different versions for
    /// use with Arabic and Hebrew." Callers that receive this result
    /// should select the matching `IconName::…_arabic` / `_hebrew`
    /// sibling when it is present; we still return the neutral glyph as
    /// the default until the localised assets land.
    LocalizedVariant,
}

/// Resolve the [`IconDirection`] for a symbol in the given layout
/// direction.
///
/// The heavy lifting sits on `IconName::layout_behavior()` so the
/// per-symbol classification stays close to the enum definition. This
/// helper folds the current layout direction into the result: in LTR all
/// symbols report [`IconDirection::Neutral`]; in RTL we return what the
/// symbol itself declared (`Flip`, `LocalizedVariant`, or `Neutral` for
/// direction-free glyphs like clocks).
///
/// Components that render icons should call this in their render path
/// and apply a horizontal flip transform when the result is
/// [`IconDirection::Flip`]. Callers with access to localised assets
/// should pick the matching variant when the result is
/// [`IconDirection::LocalizedVariant`]; a generic fallback to the
/// neutral glyph is acceptable until those assets ship.
pub fn icon_direction(
    behavior: super::icons::IconLayoutBehavior,
    direction: LayoutDirection,
) -> IconDirection {
    match direction {
        LayoutDirection::LeftToRight => IconDirection::Neutral,
        LayoutDirection::RightToLeft => match behavior {
            super::icons::IconLayoutBehavior::Neutral => IconDirection::Neutral,
            super::icons::IconLayoutBehavior::Directional => IconDirection::Flip,
            super::icons::IconLayoutBehavior::Localized => IconDirection::LocalizedVariant,
        },
    }
}

/// Returns the flex direction to use when laying out a progress indicator or
/// slider fill.
///
/// Per HIG Right-to-Left (Controls): "Flip controls that show progress from
/// one value to another." In LTR the fill grows left→right (`Row`); in RTL
/// it grows right→left (`RowReverse`). Components that render the fill with
/// a child `div` should apply the returned direction on the track container
/// so the fill anchors to the correct edge automatically.
///
/// Components that paint the fill manually (e.g. `Slider` uses a custom
/// GPUI `Element`) must mirror the fill position themselves — this helper
/// only covers the flex-based case.
pub fn progress_fill_direction(direction: LayoutDirection) -> gpui::FlexDirection {
    // Progress fills reverse together with the reading direction, so the
    // active edge always sits on the leading side.
    flex_direction_for_layout(direction)
}

/// Resolve a leading/trailing inset pair into the correct physical side for
/// the current layout direction.
///
/// Returns `(left, right)` padding. In LTR the `leading` value is on the
/// left and the `trailing` value on the right; in RTL the sides swap so
/// the same semantic inset follows the reading direction.
///
/// Use when a component has asymmetric horizontal padding — e.g. a control
/// with a leading icon that needs a larger inset on its trailing edge to
/// balance the optical weight of the icon.
pub fn leading_trailing_insets(
    direction: LayoutDirection,
    leading: Pixels,
    trailing: Pixels,
) -> (Pixels, Pixels) {
    match direction {
        LayoutDirection::LeftToRight => (leading, trailing),
        LayoutDirection::RightToLeft => (trailing, leading),
    }
}

/// Resolve the left-side inset for the given layout direction.
///
/// In LTR returns `ltr_leading`; in RTL returns `rtl_leading` (which is
/// the value that was the _trailing_ inset in LTR reading order).
pub fn leading_inset(direction: LayoutDirection, ltr: Pixels, rtl: Pixels) -> Pixels {
    match direction {
        LayoutDirection::LeftToRight => ltr,
        LayoutDirection::RightToLeft => rtl,
    }
}

/// Resolve the right-side inset for the given layout direction.
///
/// Mirror of [`leading_inset`] for the physical trailing edge.
pub fn trailing_inset(direction: LayoutDirection, ltr: Pixels, rtl: Pixels) -> Pixels {
    match direction {
        LayoutDirection::LeftToRight => ltr,
        LayoutDirection::RightToLeft => rtl,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LayoutDirection, flex_direction_for_layout, leading_inset, leading_trailing_insets,
        progress_fill_direction, trailing_inset,
    };
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn ltr_keeps_sides() {
        let (left, right) =
            leading_trailing_insets(LayoutDirection::LeftToRight, px(12.0), px(20.0));
        assert_eq!(left, px(12.0));
        assert_eq!(right, px(20.0));
    }

    #[test]
    fn rtl_swaps_sides() {
        let (left, right) =
            leading_trailing_insets(LayoutDirection::RightToLeft, px(12.0), px(20.0));
        assert_eq!(left, px(20.0));
        assert_eq!(right, px(12.0));
    }

    #[test]
    fn inset_helpers_pick_per_direction() {
        assert_eq!(
            leading_inset(LayoutDirection::LeftToRight, px(12.0), px(20.0)),
            px(12.0)
        );
        assert_eq!(
            leading_inset(LayoutDirection::RightToLeft, px(12.0), px(20.0)),
            px(20.0)
        );
        assert_eq!(
            trailing_inset(LayoutDirection::LeftToRight, px(20.0), px(12.0)),
            px(20.0)
        );
        assert_eq!(
            trailing_inset(LayoutDirection::RightToLeft, px(20.0), px(12.0)),
            px(12.0)
        );
    }

    #[test]
    fn flex_direction_helpers_match_direction() {
        assert_eq!(
            flex_direction_for_layout(LayoutDirection::LeftToRight),
            gpui::FlexDirection::Row
        );
        assert_eq!(
            flex_direction_for_layout(LayoutDirection::RightToLeft),
            gpui::FlexDirection::RowReverse
        );
    }

    #[test]
    fn progress_fill_matches_layout_direction() {
        // Progress fills track the reading direction — leading edge grows
        // first, trailing edge last.
        assert_eq!(
            progress_fill_direction(LayoutDirection::LeftToRight),
            gpui::FlexDirection::Row
        );
        assert_eq!(
            progress_fill_direction(LayoutDirection::RightToLeft),
            gpui::FlexDirection::RowReverse
        );
    }

    #[test]
    fn icon_direction_ltr_is_always_neutral() {
        use super::{IconDirection, icon_direction};
        use crate::foundations::icons::IconLayoutBehavior;
        // In LTR, every symbol reports Neutral regardless of its RTL
        // classification.
        assert_eq!(
            icon_direction(
                IconLayoutBehavior::Directional,
                LayoutDirection::LeftToRight
            ),
            IconDirection::Neutral
        );
        assert_eq!(
            icon_direction(IconLayoutBehavior::Localized, LayoutDirection::LeftToRight),
            IconDirection::Neutral
        );
    }

    #[test]
    fn icon_direction_rtl_preserves_behavior() {
        use super::{IconDirection, icon_direction};
        use crate::foundations::icons::IconLayoutBehavior;
        assert_eq!(
            icon_direction(IconLayoutBehavior::Neutral, LayoutDirection::RightToLeft),
            IconDirection::Neutral
        );
        assert_eq!(
            icon_direction(
                IconLayoutBehavior::Directional,
                LayoutDirection::RightToLeft
            ),
            IconDirection::Flip
        );
        assert_eq!(
            icon_direction(IconLayoutBehavior::Localized, LayoutDirection::RightToLeft),
            IconDirection::LocalizedVariant
        );
    }

    #[test]
    fn chevron_right_classified_directional() {
        use crate::foundations::icons::{IconLayoutBehavior, IconName};
        // Chevrons should be geometrically mirrored in RTL — smoke test
        // so adding a new icon without classifying it doesn't silently
        // regress the behavior map.
        assert_eq!(
            IconName::ChevronRight.layout_behavior(),
            IconLayoutBehavior::Directional
        );
        assert_eq!(
            IconName::ChevronLeft.layout_behavior(),
            IconLayoutBehavior::Directional
        );
        assert_eq!(
            IconName::ArrowRight.layout_behavior(),
            IconLayoutBehavior::Directional
        );
        assert_eq!(
            IconName::Send.layout_behavior(),
            IconLayoutBehavior::Directional
        );
    }

    #[test]
    fn clock_classified_neutral() {
        use crate::foundations::icons::{IconLayoutBehavior, IconName};
        // Clock is a physical-object glyph — never flip.
        assert_eq!(
            IconName::Clock.layout_behavior(),
            IconLayoutBehavior::Neutral
        );
    }
}
