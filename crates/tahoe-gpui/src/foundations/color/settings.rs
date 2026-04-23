//! macOS System Settings enums — accent colour, highlight colour, icon
//! style, sidebar icon size. These model the user's *intent* from
//! `System Settings > Appearance`; the resolved `Hsla` values live on
//! [`crate::TahoeTheme`].

use gpui::Hsla;

use super::palette::SystemPalette;

/// Accent color per macOS System Settings > Theme > Color.
///
/// Maps to the 9 macOS accent color options. Each resolves to a
/// system color from the palette. `Multicolor` uses context-dependent
/// colors (blue for buttons, orange for Find highlights, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AccentColor {
    /// System picks per-context color (default macOS behavior).
    #[default]
    Multicolor,
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    Green,
    /// Neutral gray accent (no color).
    Graphite,
}

impl AccentColor {
    /// Resolves this accent color to an Hsla value from the system palette.
    /// `Multicolor` defaults to blue (the macOS default).
    pub fn resolve(self, palette: &SystemPalette) -> Hsla {
        match self {
            Self::Multicolor | Self::Blue => palette.blue,
            Self::Purple => palette.purple,
            Self::Pink => palette.pink,
            Self::Red => palette.red,
            Self::Orange => palette.orange,
            Self::Yellow => palette.yellow,
            Self::Green => palette.green,
            Self::Graphite => palette.gray,
        }
    }
}

/// Highlight color for text selection per macOS System Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HighlightColor {
    /// Derives from the accent color.
    #[default]
    Automatic,
    Blue,
    Purple,
    Pink,
    Red,
    Orange,
    Yellow,
    Green,
    Graphite,
}

/// Icon and widget style per macOS System Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IconAndWidgetStyle {
    #[default]
    Automatic,
    Dark,
    Clear,
    Tinted,
}

/// Sidebar icon size per macOS System Settings > Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SidebarIconSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl SidebarIconSize {
    /// Returns the icon size in points per HIG.
    pub fn points(self) -> f32 {
        match self {
            Self::Small => 16.0,
            Self::Medium => 20.0,
            Self::Large => 24.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AccentColor, SidebarIconSize};
    use crate::foundations::color::{Appearance, SystemPalette};
    use core::prelude::v1::test;

    #[test]
    fn accent_color_default_is_multicolor() {
        assert_eq!(AccentColor::default(), AccentColor::Multicolor);
    }

    #[test]
    fn accent_color_resolve_multicolor_is_blue() {
        let palette = SystemPalette::new(Appearance::Light);
        let resolved = AccentColor::Multicolor.resolve(&palette);
        assert_eq!(resolved, palette.blue);
    }

    #[test]
    fn accent_color_resolve_all_variants() {
        let palette = SystemPalette::new(Appearance::Dark);
        assert_eq!(AccentColor::Blue.resolve(&palette), palette.blue);
        assert_eq!(AccentColor::Purple.resolve(&palette), palette.purple);
        assert_eq!(AccentColor::Pink.resolve(&palette), palette.pink);
        assert_eq!(AccentColor::Red.resolve(&palette), palette.red);
        assert_eq!(AccentColor::Orange.resolve(&palette), palette.orange);
        assert_eq!(AccentColor::Yellow.resolve(&palette), palette.yellow);
        assert_eq!(AccentColor::Green.resolve(&palette), palette.green);
        assert_eq!(AccentColor::Graphite.resolve(&palette), palette.gray);
    }

    #[test]
    fn accent_color_all_variants_distinct_from_default() {
        let variants = [
            AccentColor::Multicolor,
            AccentColor::Blue,
            AccentColor::Purple,
            AccentColor::Pink,
            AccentColor::Red,
            AccentColor::Orange,
            AccentColor::Yellow,
            AccentColor::Green,
            AccentColor::Graphite,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn sidebar_icon_size_points() {
        assert_eq!(SidebarIconSize::Small.points(), 16.0);
        assert_eq!(SidebarIconSize::Medium.points(), 20.0);
        assert_eq!(SidebarIconSize::Large.points(), 24.0);
        assert_eq!(SidebarIconSize::default(), SidebarIconSize::Medium);
    }
}
