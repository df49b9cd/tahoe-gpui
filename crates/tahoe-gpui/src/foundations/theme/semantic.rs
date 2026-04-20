use crate::foundations::color::{Appearance, SystemColor};
use gpui::{Hsla, hsla};

/// Semantic label and background colors per HIG.
/// Colors adapt to appearance mode (light/dark/high-contrast) and are named
/// by purpose, not appearance.
#[derive(Debug, Clone)]
pub struct SemanticColors {
    /// Primary text label.
    pub label: Hsla,
    /// Secondary text label.
    pub secondary_label: Hsla,
    /// Tertiary text label.
    pub tertiary_label: Hsla,
    /// Quaternary/disabled text label.
    pub quaternary_label: Hsla,
    /// Quinary text label (macOS Tahoe / iOS 26).
    pub quinary_label: Hsla,
    /// Primary background.
    pub system_background: Hsla,
    /// Secondary/elevated background.
    pub secondary_system_background: Hsla,
    /// Tertiary/grouped background.
    pub tertiary_system_background: Hsla,

    // --- HIG Extended Semantic Colors ---
    /// Separator (thin line, semi-transparent).
    pub separator: Hsla,
    /// Opaque separator (for sections that need a solid line).
    pub opaque_separator: Hsla,
    /// Placeholder text in text fields.
    pub placeholder_text: Hsla,
    /// Link/URL text color (distinct from accent for non-interactive contexts).
    pub link: Hsla,

    /// System fill (for thin/small elements like slider tracks).
    pub system_fill: Hsla,
    /// Secondary fill.
    pub secondary_system_fill: Hsla,
    /// Tertiary fill.
    pub tertiary_system_fill: Hsla,
    /// Quaternary fill (very subtle).
    pub quaternary_system_fill: Hsla,
    /// Quinary fill (barely visible, macOS Tahoe / iOS 26).
    pub quinary_system_fill: Hsla,

    /// Grouped background (primary), for grouped table views.
    pub system_grouped_background: Hsla,
    /// Secondary grouped background.
    pub secondary_system_grouped_background: Hsla,
    /// Tertiary grouped background.
    pub tertiary_system_grouped_background: Hsla,

    /// Elevated system background (dark mode: lighter than base; light mode: same as base).
    pub elevated_system_background: Hsla,
    /// Elevated secondary system background.
    pub elevated_secondary_system_background: Hsla,

    /// Backdrop for Apple Fitness-style Activity rings. HIG requires rings
    /// to sit on a black substrate, but `dark_mode.rs:20` forbids `#000000`
    /// — we pin to the system dark gray (L=0.07) in every appearance so the
    /// watch-face aesthetic survives light mode as well.
    pub activity_ring_backdrop: Hsla,

    /// Informational color (citations, chain-of-thought, neutral status).
    /// Resolved from `SystemColor::Cyan` for the current appearance — the HC
    /// variants raise lightness so the color stays ≥3:1 on system fills.
    pub info: Hsla,
    /// AI/agent color for assistant-specific affordances. Resolved from
    /// `SystemColor::Purple`; HC variants are taken from the palette.
    pub ai: Hsla,
}

impl SemanticColors {
    /// Create semantic colors for the given appearance mode.
    ///
    /// Models all four modes (dark, light, dark-HC, light-HC) natively.
    pub fn new(appearance: Appearance) -> Self {
        appearance.resolve(
            Self::light(),
            Self::dark(),
            Self::light_high_contrast(),
            Self::dark_high_contrast(),
        )
    }

    fn dark() -> Self {
        SemanticColors {
            label: hsla(0.0, 0.0, 1.0, 1.0),
            secondary_label: hsla(0.0, 0.0, 1.0, 0.60),
            tertiary_label: hsla(0.0, 0.0, 1.0, 0.30),
            quaternary_label: hsla(0.0, 0.0, 1.0, 0.18),
            quinary_label: hsla(0.0, 0.0, 1.0, 0.10),
            system_background: hsla(0.0, 0.0, 0.07, 1.0),
            secondary_system_background: hsla(0.0, 0.0, 0.11, 1.0),
            tertiary_system_background: hsla(0.0, 0.0, 0.15, 1.0),
            separator: hsla(0.0, 0.0, 0.33, 0.60),
            opaque_separator: hsla(0.0, 0.0, 0.23, 1.0),
            placeholder_text: hsla(0.0, 0.0, 1.0, 0.30),
            // Route through `SystemColor::Blue.resolve` so the HighContrast
            // variants of the palette automatically supply a ≥4.5:1 link
            // colour against `system_background`. Hardcoded HSL literals
            // (now removed) failed WCAG AA in DarkHighContrast.
            link: SystemColor::Blue.resolve(Appearance::Dark),
            system_fill: hsla(0.0, 0.0, 0.47, 0.36),
            secondary_system_fill: hsla(0.0, 0.0, 0.47, 0.32),
            tertiary_system_fill: hsla(0.0, 0.0, 0.46, 0.24),
            quaternary_system_fill: hsla(0.0, 0.0, 0.46, 0.18),
            quinary_system_fill: hsla(0.0, 0.0, 0.44, 0.12),
            // macOS substrate rule (see dark_mode.rs:20-27): never pure black.
            // Matches `system_background` (L=0.07) so the grouped primary
            // doesn't clip into wallpaper when a window floats over the
            // desktop.
            system_grouped_background: hsla(0.0, 0.0, 0.07, 1.0),
            secondary_system_grouped_background: hsla(0.0, 0.0, 0.11, 1.0),
            tertiary_system_grouped_background: hsla(0.0, 0.0, 0.17, 1.0),
            elevated_system_background: hsla(0.0, 0.0, 0.11, 1.0),
            elevated_secondary_system_background: hsla(0.0, 0.0, 0.17, 1.0),
            activity_ring_backdrop: hsla(0.0, 0.0, 0.07, 1.0),
            info: SystemColor::Cyan.resolve(Appearance::Dark),
            ai: SystemColor::Purple.resolve(Appearance::Dark),
        }
    }

    fn dark_high_contrast() -> Self {
        let mut s = Self::dark();
        // HC: fully opaque labels. All five levels comfortably exceed the
        // 3:1 UI/large-text floor from `foundations.md:65` against
        // system_background (L=0.07).
        s.label = hsla(0.0, 0.0, 1.0, 1.0); // ~21:1 contrast
        s.secondary_label = hsla(0.0, 0.0, 0.80, 1.0); // ~12:1
        s.tertiary_label = hsla(0.0, 0.0, 0.70, 1.0); // ~9:1
        s.quaternary_label = hsla(0.0, 0.0, 0.63, 1.0); // ~7:1
        s.quinary_label = hsla(0.0, 0.0, 0.56, 1.0); // ~5.7:1
        s.info = SystemColor::Cyan.resolve(Appearance::DarkHighContrast);
        s.ai = SystemColor::Purple.resolve(Appearance::DarkHighContrast);
        s
    }

    fn light() -> Self {
        SemanticColors {
            label: hsla(0.0, 0.0, 0.0, 1.0),
            secondary_label: hsla(0.0, 0.0, 0.0, 0.60),
            tertiary_label: hsla(0.0, 0.0, 0.0, 0.40),
            quaternary_label: hsla(0.0, 0.0, 0.0, 0.18),
            quinary_label: hsla(0.0, 0.0, 0.0, 0.10),
            system_background: hsla(0.0, 0.0, 1.0, 1.0),
            secondary_system_background: hsla(0.0, 0.0, 0.97, 1.0),
            tertiary_system_background: hsla(0.0, 0.0, 0.94, 1.0),
            separator: hsla(0.0, 0.0, 0.24, 0.29),
            opaque_separator: hsla(0.0, 0.0, 0.78, 1.0),
            placeholder_text: hsla(0.0, 0.0, 0.24, 0.30),
            // See `dark()` for rationale: route through SystemColor so
            // HighContrast variants stay WCAG-AA-compliant.
            link: SystemColor::Blue.resolve(Appearance::Light),
            system_fill: hsla(0.0, 0.0, 0.47, 0.20),
            secondary_system_fill: hsla(0.0, 0.0, 0.47, 0.16),
            tertiary_system_fill: hsla(0.0, 0.0, 0.46, 0.12),
            quaternary_system_fill: hsla(0.0, 0.0, 0.45, 0.08),
            quinary_system_fill: hsla(0.0, 0.0, 0.44, 0.05),
            system_grouped_background: hsla(0.0, 0.0, 0.95, 1.0),
            secondary_system_grouped_background: hsla(0.0, 0.0, 1.0, 1.0),
            tertiary_system_grouped_background: hsla(0.0, 0.0, 0.95, 1.0),
            elevated_system_background: hsla(0.0, 0.0, 1.0, 1.0),
            elevated_secondary_system_background: hsla(0.0, 0.0, 0.97, 1.0),
            activity_ring_backdrop: hsla(0.0, 0.0, 0.07, 1.0),
            info: SystemColor::Cyan.resolve(Appearance::Light),
            ai: SystemColor::Purple.resolve(Appearance::Light),
        }
    }

    fn light_high_contrast() -> Self {
        let mut s = Self::light();
        // HC: fully opaque labels. All five levels exceed the 3:1
        // UI/large-text floor from `foundations.md:65` against
        // system_background (L=1.0). Lifted quaternary/quinary off the prior
        // alpha-based defaults that fell to ~3.7:1 / ~2.6:1.
        s.label = hsla(0.0, 0.0, 0.0, 1.0); // ~21:1 contrast
        s.secondary_label = hsla(0.0, 0.0, 0.20, 1.0); // ~12:1
        s.tertiary_label = hsla(0.0, 0.0, 0.30, 1.0); // ~9:1
        s.quaternary_label = hsla(0.0, 0.0, 0.37, 1.0); // ~7:1
        s.quinary_label = hsla(0.0, 0.0, 0.44, 1.0); // ~5.1:1
        s.info = SystemColor::Cyan.resolve(Appearance::LightHighContrast);
        s.ai = SystemColor::Purple.resolve(Appearance::LightHighContrast);
        s
    }
}

#[cfg(test)]
mod tests {
    use crate::foundations::color::Appearance;
    use crate::foundations::theme::semantic::SemanticColors;
    use core::prelude::v1::test;

    // Regression for https://github.com/df49b9cd/tahoe-gpui/issues/55.
    // Activity ring backdrop must never be pure black (HIG dark-mode rule)
    // and must keep the watch-face aesthetic in light mode too.
    #[test]
    fn activity_ring_backdrop_is_not_pure_black_in_any_mode() {
        for appearance in [
            Appearance::Dark,
            Appearance::Light,
            Appearance::DarkHighContrast,
            Appearance::LightHighContrast,
        ] {
            let s = SemanticColors::new(appearance);
            assert!(
                s.activity_ring_backdrop.l > 0.02,
                "activity_ring_backdrop is pure black in {appearance:?}"
            );
            assert!(
                (s.activity_ring_backdrop.l - 0.07).abs() < 1e-4,
                "activity_ring_backdrop lightness should be 0.07 in {appearance:?}, got {}",
                s.activity_ring_backdrop.l
            );
            assert!(
                (s.activity_ring_backdrop.a - 1.0).abs() < f32::EPSILON,
                "activity_ring_backdrop must be opaque"
            );
        }
    }
}
