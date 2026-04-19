//! Image handling aligned with HIG.
//!
//! Apple platforms support multiple image scales for different display densities.
//! GPUI handles image rendering via `img()` and `SharedUri`.
//!
//! # Best practices (from HIG)
//!
//! - Provide images at @2x scale for Retina displays
//! - Use vector formats (SVG, PDF) when possible for resolution independence
//! - Prefer SF Symbols over custom images for standard actions
//! - Support Dark Mode by providing appearance-adapted image variants
//! - Use appropriate compression (PNG for UI elements, JPEG for photos)

/// Image scale factor for multi-resolution asset support.
///
/// Apple displays use these scale factors:
/// - @1x: Standard resolution (pre-Retina / low-DPI external displays)
/// - @2x: Retina displays (most current Macs, all iPhones since 4)
/// - @3x: Super Retina displays (iPhone Plus/Pro Max)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ImageScale {
    /// Standard resolution (1 pixel = 1 point).
    X1,
    /// Retina resolution (2 pixels = 1 point).
    #[default]
    X2,
    /// Super Retina resolution (3 pixels = 1 point).
    X3,
}

impl ImageScale {
    /// Returns the scale multiplier.
    pub fn factor(self) -> f32 {
        match self {
            Self::X1 => 1.0,
            Self::X2 => 2.0,
            Self::X3 => 3.0,
        }
    }

    /// Returns the `@Nx` suffix string for asset naming conventions.
    pub fn suffix(self) -> &'static str {
        match self {
            Self::X1 => "",
            Self::X2 => "@2x",
            Self::X3 => "@3x",
        }
    }

    /// Pick the best scale for a given window backing scale factor.
    ///
    /// HIG asset guidance: raster images must ship at @2x for all Retina
    /// macOS / iPadOS displays and @3x for iPhone Plus/Pro Max. Matching
    /// rule:
    ///
    /// | Backing factor `f`        | Scale |
    /// |---------------------------|-------|
    /// | `f <= 1.25`               | `@1x` |
    /// | `1.25 < f <= 2.5`         | `@2x` |
    /// | `f > 2.5`                 | `@3x` |
    ///
    /// The 1.25 / 2.5 crossover points bias upward so that 1.5x and 2.5x
    /// backings (typical on Apple external displays) pick the sharper
    /// asset. Call once per image with `window.scale_factor()` or an
    /// equivalent DPR; the returned scale maps to a file suffix via
    /// [`ImageScale::suffix`].
    pub fn for_backing_scale(factor: f32) -> Self {
        if factor > 2.5 {
            Self::X3
        } else if factor > 1.25 {
            Self::X2
        } else {
            Self::X1
        }
    }

    /// Build a suffixed asset path, inserting `@2x` / `@3x` before the file
    /// extension. `base` is a path without scale suffix (`icons/logo.png`);
    /// the returned string is what should be passed to `img()`:
    ///
    /// ```
    /// use tahoe_gpui::foundations::images::ImageScale;
    /// let path = ImageScale::X2.asset_path("icons/logo.png");
    /// assert_eq!(path, "icons/logo@2x.png");
    /// let plain = ImageScale::X1.asset_path("icons/logo.png");
    /// assert_eq!(plain, "icons/logo.png");
    /// ```
    ///
    /// For paths without an extension the suffix is appended at the end:
    /// `asset_path("logo")` returns `"logo@2x"` at `X2`.
    pub fn asset_path(self, base: &str) -> String {
        let suffix = self.suffix();
        if suffix.is_empty() {
            return base.to_string();
        }
        match base.rfind('.') {
            Some(dot) if dot > base.rfind('/').unwrap_or(0) => {
                let (stem, ext) = base.split_at(dot);
                format!("{stem}{suffix}{ext}")
            }
            _ => format!("{base}{suffix}"),
        }
    }

    /// Convenience: the best-matching asset path for the given backing scale.
    pub fn asset_path_for_backing_scale(base: &str, backing_factor: f32) -> String {
        Self::for_backing_scale(backing_factor).asset_path(base)
    }
}

#[cfg(test)]
mod tests {
    use super::ImageScale;
    use core::prelude::v1::test;

    #[test]
    fn for_backing_scale_selects_1x_at_standard_resolution() {
        assert_eq!(ImageScale::for_backing_scale(1.0), ImageScale::X1);
        assert_eq!(ImageScale::for_backing_scale(1.25), ImageScale::X1);
    }

    #[test]
    fn for_backing_scale_selects_2x_on_retina() {
        assert_eq!(ImageScale::for_backing_scale(2.0), ImageScale::X2);
        assert_eq!(ImageScale::for_backing_scale(2.5), ImageScale::X2);
        assert_eq!(ImageScale::for_backing_scale(1.5), ImageScale::X2);
    }

    #[test]
    fn for_backing_scale_selects_3x_on_super_retina() {
        assert_eq!(ImageScale::for_backing_scale(3.0), ImageScale::X3);
        assert_eq!(ImageScale::for_backing_scale(4.0), ImageScale::X3);
    }

    #[test]
    fn asset_path_inserts_suffix_before_extension() {
        assert_eq!(
            ImageScale::X2.asset_path("icons/logo.png"),
            "icons/logo@2x.png"
        );
        assert_eq!(
            ImageScale::X3.asset_path("icons/logo.png"),
            "icons/logo@3x.png"
        );
        assert_eq!(
            ImageScale::X1.asset_path("icons/logo.png"),
            "icons/logo.png"
        );
    }

    #[test]
    fn asset_path_handles_extensionless_base() {
        assert_eq!(ImageScale::X2.asset_path("logo"), "logo@2x");
    }

    #[test]
    fn asset_path_handles_directory_only_dot() {
        // A dot in the directory, no extension on the filename, should still
        // append the suffix at the very end rather than before the dot.
        assert_eq!(
            ImageScale::X2.asset_path("./assets/logo"),
            "./assets/logo@2x"
        );
    }
}
