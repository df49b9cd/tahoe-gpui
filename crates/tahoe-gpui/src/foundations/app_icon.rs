//! App icon design tokens and Icon Composer layer templates (HIG).
//!
//! macOS 26 Tahoe ships Liquid Glass layered app icons assembled in Icon
//! Composer from a **Background** layer (glass material, rounded-rect
//! shape) and a **Foreground** layer (the app's mark). Consuming apps
//! are responsible for shipping actual `.icns` / `AppIcon.appiconset`
//! bundles — a library crate like `tahoe-gpui` cannot substitute for
//! Xcode's asset compiler. What this module does provide:
//!
//! - [`AppIconLayer`] — the two named layers + their HIG constraints.
//! - [`AppIconPlatform`] — per-platform required master sizes.
//! - [`TILE_CORNER_RADIUS_RATIO`] — the proportional corner radius used
//!   by Icon Composer's Foreground layer (~22.5 %). Also drives
//!   [`super::icons::glass_tile::GlassIconTile`].
//! - [`foreground_corner_radius`] — utility for computing the correct
//!   corner radius at arbitrary tile sizes.
//!
//! # References
//!
//! - HIG §App icons: `docs/hig/foundations.md`
//! - Xcode "Icon Composer" templates.

use gpui::{Pixels, px};

/// Layer name in the Icon Composer assembly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppIconLayer {
    /// Background layer — carries the glass material (rounded rectangle
    /// with Liquid Glass tint). 1024×1024 master; no content other than
    /// the material and tint layer.
    Background,
    /// Foreground layer — carries the app's distinctive mark. Respects
    /// the proportional corner radius defined by
    /// [`TILE_CORNER_RADIUS_RATIO`] and should provide ~12.5 % keyline
    /// margin so the mark doesn't crop when clipped on rounded masks.
    Foreground,
}

/// HIG corner-radius ratio for Liquid Glass app icon tiles.
///
/// Defined as a fraction of the icon's tile side (the square
/// bounding box). All platforms that render Liquid Glass icons apply
/// this ratio; keeping it here as a single source of truth lets
/// [`super::icons::glass_tile::GlassIconTile`] and downstream app-icon
/// generators stay consistent.
pub const TILE_CORNER_RADIUS_RATIO: f32 = 0.225;

/// Convenience: corner radius in points for a given tile side.
pub fn foreground_corner_radius(side_pt: f32) -> Pixels {
    px(side_pt * TILE_CORNER_RADIUS_RATIO)
}

/// Target platform for app icon asset generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppIconPlatform {
    /// macOS — `.icns` with @1x/@2x variants at 1024/512/256/128/64/32/16 pt.
    MacOS,
    /// iOS / iPadOS — `AppIcon.appiconset` with Light/Dark/Tinted
    /// appearances. 11 sizes from 1024 pt down to 20 pt.
    IOS,
    /// tvOS — 1280×768 and 400×240 layered PDFs.
    TvOS,
    /// visionOS — 1024 pt circular.
    VisionOS,
    /// watchOS — 1024/196/172/100 pt circular.
    WatchOS,
}

impl AppIconPlatform {
    /// Required point sizes for this platform's icon set. The master
    /// (highest) size is always 1024 pt except for tvOS (which uses
    /// non-square 1280×768) and watchOS (1024 pt circular).
    pub fn required_sizes(self) -> &'static [u32] {
        match self {
            // macOS .icns sizes (each needs @1x and @2x where the @2x is
            // double the listed value).
            Self::MacOS => &[16, 32, 64, 128, 256, 512, 1024],
            // iOS / iPadOS AppIcon.appiconset sizes (1024 master plus
            // all ten compositor sizes, in points).
            Self::IOS => &[20, 29, 40, 58, 60, 76, 80, 87, 120, 152, 167, 180, 1024],
            // tvOS main/front/back sizes (in points; width×height pairs
            // are 1280×768 for App Store and 400×240 for the in-device
            // layered icon). Exposed as square maxima here; consumers
            // that need the non-square pair use the platform API.
            Self::TvOS => &[400, 1280],
            Self::VisionOS => &[1024],
            Self::WatchOS => &[100, 172, 196, 1024],
        }
    }

    /// Whether this platform's app icon requires a Light/Dark/Tinted
    /// appearance triplet. True for iOS/iPadOS (iOS 18+) and macOS (26+
    /// adopts the same model via Icon Composer).
    pub fn requires_appearance_triplet(self) -> bool {
        matches!(self, Self::IOS | Self::MacOS)
    }
}

/// The two layers that Icon Composer assembles for a macOS 26 Tahoe
/// Liquid Glass app icon.
pub const LIQUID_GLASS_LAYERS: &[AppIconLayer] =
    &[AppIconLayer::Background, AppIconLayer::Foreground];

/// Canonical master asset side length in points. Every platform's asset
/// pipeline consumes a 1024 pt master for scaling; this constant lets
/// design scripts pick up the size from one place.
pub const MASTER_ICON_SIZE_PT: u32 = 1024;

/// Approximate keyline margin in points a Foreground layer should leave
/// unused at a 1024 pt master so the mark doesn't crop under a rounded
/// mask. Apple's Icon Composer template reserves ~12.5 % per edge.
pub const FOREGROUND_KEYLINE_MARGIN_PT: u32 = 128;

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn corner_radius_ratio_is_hig_spec() {
        assert!((TILE_CORNER_RADIUS_RATIO - 0.225).abs() < f32::EPSILON);
    }

    #[test]
    fn foreground_corner_radius_scales_linearly() {
        let r48 = foreground_corner_radius(48.0);
        let r1024 = foreground_corner_radius(1024.0);
        let ratio = f32::from(r1024) / f32::from(r48);
        // 1024 / 48 ≈ 21.333 — linear scaling preserves the ratio.
        assert!((ratio - 1024.0 / 48.0).abs() < 0.01);
    }

    #[test]
    fn macos_and_ios_require_appearance_triplet() {
        assert!(AppIconPlatform::MacOS.requires_appearance_triplet());
        assert!(AppIconPlatform::IOS.requires_appearance_triplet());
        assert!(!AppIconPlatform::TvOS.requires_appearance_triplet());
        assert!(!AppIconPlatform::WatchOS.requires_appearance_triplet());
    }

    #[test]
    fn every_platform_includes_a_master_size() {
        for p in [
            AppIconPlatform::MacOS,
            AppIconPlatform::IOS,
            AppIconPlatform::VisionOS,
            AppIconPlatform::WatchOS,
        ] {
            assert!(
                p.required_sizes().contains(&MASTER_ICON_SIZE_PT),
                "{p:?} missing 1024 pt master size",
            );
        }
    }

    #[test]
    fn liquid_glass_has_both_named_layers() {
        assert_eq!(LIQUID_GLASS_LAYERS.len(), 2);
        assert!(LIQUID_GLASS_LAYERS.contains(&AppIconLayer::Background));
        assert!(LIQUID_GLASS_LAYERS.contains(&AppIconLayer::Foreground));
    }
}
