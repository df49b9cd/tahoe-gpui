//! HIG color system.
//!
//! Provides the Human Interface Guidelines system color palette with
//! four appearance variants: light, dark, light high-contrast, and dark
//! high-contrast. Components should prefer semantic tokens from
//! [`crate::TahoeTheme`], but may import utility functions directly.
//!
//! # Architecture
//!
//! ```text
//! palette (raw Hsla tables)
//!     → resolved (linear-sRGB, SwiftUI parity)
//!     → theme::SemanticColors (semantic tokens)
//!     → components
//! ```
//!
//! ## Submodules
//!
//! - [`palette`] — `SystemColor`, `SystemGray`, `SystemPalette` and the 72
//!   HIG palette constants.
//! - [`settings`] — `AccentColor`, `HighlightColor`, `IconAndWidgetStyle`,
//!   `SidebarIconSize` (macOS System Settings enums).
//! - [`resolved`] — `ResolvedColor` (linear-sRGB paint primitive) and
//!   `RgbColorSpace`.
//! - [`ops`] — alpha/lightness helpers (`opacity`, `fade_out`, `darken`,
//!   `lighten`, `with_alpha`, `text_on_background`) and the `HslaAlphaExt`
//!   extension trait.
//! - [`srgb`] — sRGB ↔ linear conversion and `compose_black_tint_linear`.
//! - [`contrast`] — WCAG relative-luminance and contrast-ratio utilities.

pub mod contrast;
pub mod environment;
pub mod gpui_bridge;
pub mod gradient;
pub mod oklab;
pub mod ops;
pub mod paint;
pub mod palette;
pub mod parse;
pub mod resolved;
pub mod settings;
pub mod srgb;
pub mod token;

// ── Public re-exports — preserve the pre-split `foundations::color::X` surface. ──

pub use contrast::{contrast_ratio, meets_contrast};
pub use environment::ColorEnvironment;
pub use gradient::{
    AngularGradient, AnyGradient, Gradient, GradientStop, LinearGradient, RadialGradient, UnitPoint,
};
pub use ops::{HslaAlphaExt, darken, fade_out, lighten, opacity, text_on_background, with_alpha};
pub use paint::Paint;
pub use palette::{SystemColor, SystemGray, SystemPalette};
pub use parse::{
    ParseColorError, hex_to_hsla, hex_to_rgba_bytes, hsb_to_hsla, hsla_to_hex, hsla_to_hsb,
    hsla_to_rgb_bytes,
};
pub use resolved::{ResolvedColor, RgbColorSpace};
pub use settings::{AccentColor, HighlightColor, IconAndWidgetStyle, SidebarIconSize};
pub use srgb::compose_black_tint_linear;
pub use token::{Color, MixColorSpace, SemanticToken};

// ── Crate-internal re-exports — these helpers stay `pub(crate)` and were
//    reached via `crate::foundations::color::*` before the split; preserve that
//    reach so call sites outside this module don't have to know about the
//    submodule layout. `#[allow(unused_imports)]` because the in-tree callers
//    are all `#[cfg(test)]`-gated; without the attribute the lib build emits
//    a false-positive warning.
#[allow(unused_imports)]
pub(crate) use contrast::relative_luminance;
#[allow(unused_imports)]
pub(crate) use srgb::{linear_to_srgb, srgb_to_linear};

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Appearance
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// System appearance combining color scheme and contrast level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Appearance {
    /// Standard light mode.
    Light,
    /// Standard dark mode.
    #[default]
    Dark,
    /// Light mode with increased contrast (accessibility).
    LightHighContrast,
    /// Dark mode with increased contrast (accessibility).
    DarkHighContrast,
}

impl Appearance {
    /// Returns `true` for dark and dark-high-contrast appearances.
    pub fn is_dark(self) -> bool {
        matches!(self, Self::Dark | Self::DarkHighContrast)
    }

    /// Returns `true` for high-contrast appearances.
    pub fn is_high_contrast(self) -> bool {
        matches!(self, Self::LightHighContrast | Self::DarkHighContrast)
    }

    /// Returns a 0-3 index for lookup table access:
    /// Light = 0, Dark = 1, LightHighContrast = 2, DarkHighContrast = 3.
    pub fn index(self) -> usize {
        match self {
            Self::Light => 0,
            Self::Dark => 1,
            Self::LightHighContrast => 2,
            Self::DarkHighContrast => 3,
        }
    }

    /// Resolve a value from four appearance-specific variants.
    ///
    /// Replaces the common `match (is_dark, is_hc) { ... }` pattern with a
    /// single call. Arguments are ordered: light, dark, light high-contrast,
    /// dark high-contrast.
    pub fn resolve<T>(self, light: T, dark: T, light_hc: T, dark_hc: T) -> T {
        match self.index() {
            0 => light,
            1 => dark,
            2 => light_hc,
            _ => dark_hc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Appearance;
    use core::prelude::v1::test;

    #[test]
    fn appearance_is_dark() {
        assert!(!Appearance::Light.is_dark());
        assert!(Appearance::Dark.is_dark());
        assert!(!Appearance::LightHighContrast.is_dark());
        assert!(Appearance::DarkHighContrast.is_dark());
    }

    #[test]
    fn appearance_is_high_contrast() {
        assert!(!Appearance::Light.is_high_contrast());
        assert!(!Appearance::Dark.is_high_contrast());
        assert!(Appearance::LightHighContrast.is_high_contrast());
        assert!(Appearance::DarkHighContrast.is_high_contrast());
    }

    #[test]
    fn default_appearance_is_dark() {
        assert_eq!(Appearance::default(), Appearance::Dark);
    }
}
