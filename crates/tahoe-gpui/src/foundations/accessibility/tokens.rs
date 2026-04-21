//! Glass-surface accessibility tokens and motion/contrast helpers.

use gpui::Hsla;

use crate::foundations::theme::TahoeTheme;

/// Accessibility tokens for Liquid Glass per HIG.
#[derive(Debug, Clone)]
pub struct AccessibilityTokens {
    /// Reduced transparency mode: higher opacity glass fill (e.g. 0.85).
    pub reduced_transparency_bg: Hsla,
    /// Increased contrast mode: visible border color.
    pub high_contrast_border: Hsla,
    /// Reduced motion: duration multiplier (0.0 = no motion, 1.0 = full).
    pub reduced_motion_scale: f32,
}

/// In IncreaseContrast mode, adds a visible border per HIG.
/// No-op for other accessibility modes.
///
/// Generic over any GPUI element implementing `Styled` (works with both
/// `Div` and `Stateful<Div>`).
pub fn apply_high_contrast_border<E: gpui::Styled>(mut el: E, theme: &TahoeTheme) -> E {
    if theme.accessibility_mode.increase_contrast() {
        el = el
            .border_1()
            .border_color(theme.glass.accessibility.high_contrast_border);
    }
    el
}

/// Returns the effective animation duration respecting the Reduced Motion accessibility setting.
///
/// When `AccessibilityMode::REDUCE_MOTION` is active, returns 0 to suppress animations.
/// Applies to all themes (glass and non-glass).
///
/// **Note:** Returning 0 produces a zero-duration snap rather than the
/// cross-fade HIG actually prescribes ("replace large, dramatic transitions
/// with subtle cross-fades"). For transition sites that can honour a short
/// cross-fade instead, prefer [`reduce_motion_substitute_ms`] or route the
/// call through `super::super::motion::accessible_transition_animation`, which
/// returns the 150 ms `REDUCE_MOTION_CROSSFADE` duration when Reduce Motion
/// is on.
pub fn effective_duration(theme: &TahoeTheme, base_ms: u64) -> u64 {
    if theme.accessibility_mode.reduce_motion() {
        0
    } else {
        base_ms
    }
}

/// Returns an animation duration in milliseconds that substitutes a short
/// cross-fade for the caller's original animation when Reduce Motion is on.
///
/// Finding 22 in the Zed cross-reference audit:
/// HIG Motion says "replace large, dramatic transitions with subtle
/// cross-fades." Zeroing the duration — which is what
/// [`effective_duration`] does — produces an instant snap instead of the
/// cross-fade. Routing through this helper preserves a short (150 ms by
/// default — matches `super::super::motion::REDUCE_MOTION_CROSSFADE`) visual
/// continuity while still honouring the user's preference for minimal
/// movement.
///
/// Use this at transition sites (sheet / modal / popover presentation,
/// segmented-control glass morph) where an abrupt position snap would feel
/// worse than a short opacity-only fade. For movement sites where the
/// motion itself is the distraction (e.g. a spinning loader), keep using
/// [`effective_duration`] so the animation actually stops.
pub fn reduce_motion_substitute_ms(theme: &TahoeTheme, base_ms: u64) -> u64 {
    if theme.accessibility_mode.reduce_motion() {
        // Matches the `REDUCE_MOTION_CROSSFADE` constant in
        // `super::super::motion`. Kept inline here to avoid a dependency
        // cycle between `accessibility` and `motion`.
        150
    } else {
        base_ms
    }
}

pub use super::super::materials::apply_focus_ring;

#[cfg(test)]
mod tests {
    use super::reduce_motion_substitute_ms;
    use crate::foundations::accessibility::AccessibilityMode;
    use crate::foundations::theme::TahoeTheme;
    use core::prelude::v1::test;

    #[test]
    fn reduce_motion_substitute_ms_keeps_base_when_flag_off() {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DEFAULT;
        assert_eq!(reduce_motion_substitute_ms(&theme, 350), 350);
    }

    #[test]
    fn reduce_motion_substitute_ms_returns_crossfade_duration() {
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
        // Matches REDUCE_MOTION_CROSSFADE in super::super::motion (150 ms).
        assert_eq!(reduce_motion_substitute_ms(&theme, 350), 150);
    }
}
