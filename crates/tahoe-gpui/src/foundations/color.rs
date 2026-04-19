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
//! color.rs (palette)  →  theme.rs (semantic tokens)  →  components
//! ```

use gpui::Hsla;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Accent & System Settings Enums
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

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

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// System Color
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HIG named system colors.
///
/// Each variant resolves to a different HSLA value depending on the
/// [`Appearance`]. Values are taken directly from the Apple Human Interface
/// Guidelines color specifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemColor {
    Red,
    Orange,
    Yellow,
    Green,
    Mint,
    Teal,
    Cyan,
    Blue,
    Indigo,
    Purple,
    Pink,
    Brown,
}

impl SystemColor {
    /// Resolve this color for the given appearance.
    ///
    /// Uses an indexed lookup table instead of a 48-arm match for O(1) access.
    pub fn resolve(self, appearance: Appearance) -> Hsla {
        // [Light, Dark, LightHC, DarkHC] per color variant
        const TABLE: [[Hsla; 4]; 12] = [
            // Red
            [RED_LIGHT, RED_DARK, RED_LIGHT_HC, RED_DARK_HC],
            // Orange
            [ORANGE_LIGHT, ORANGE_DARK, ORANGE_LIGHT_HC, ORANGE_DARK_HC],
            // Yellow
            [YELLOW_LIGHT, YELLOW_DARK, YELLOW_LIGHT_HC, YELLOW_DARK_HC],
            // Green
            [GREEN_LIGHT, GREEN_DARK, GREEN_LIGHT_HC, GREEN_DARK_HC],
            // Mint
            [MINT_LIGHT, MINT_DARK, MINT_LIGHT_HC, MINT_DARK_HC],
            // Teal
            [TEAL_LIGHT, TEAL_DARK, TEAL_LIGHT_HC, TEAL_DARK_HC],
            // Cyan
            [CYAN_LIGHT, CYAN_DARK, CYAN_LIGHT_HC, CYAN_DARK_HC],
            // Blue
            [BLUE_LIGHT, BLUE_DARK, BLUE_LIGHT_HC, BLUE_DARK_HC],
            // Indigo
            [INDIGO_LIGHT, INDIGO_DARK, INDIGO_LIGHT_HC, INDIGO_DARK_HC],
            // Purple
            [PURPLE_LIGHT, PURPLE_DARK, PURPLE_LIGHT_HC, PURPLE_DARK_HC],
            // Pink
            [PINK_LIGHT, PINK_DARK, PINK_LIGHT_HC, PINK_DARK_HC],
            // Brown
            [BROWN_LIGHT, BROWN_DARK, BROWN_LIGHT_HC, BROWN_DARK_HC],
        ];
        TABLE[self as usize][appearance.index()]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// System Gray
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// HIG gray scale levels.
///
/// Six levels of gray, each with four appearance variants. Gray is the
/// most saturated; Gray6 is the lightest in light mode / darkest in dark mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemGray {
    Gray,
    Gray2,
    Gray3,
    Gray4,
    Gray5,
    Gray6,
}

impl SystemGray {
    /// Resolve this gray level for the given appearance.
    ///
    /// Uses an indexed lookup table instead of a 24-arm match for O(1) access.
    pub fn resolve(self, appearance: Appearance) -> Hsla {
        // [Light, Dark, LightHC, DarkHC] per gray level
        const TABLE: [[Hsla; 4]; 6] = [
            // Gray
            [GRAY_LIGHT, GRAY_DARK, GRAY_LIGHT_HC, GRAY_DARK_HC],
            // Gray2
            [GRAY2_LIGHT, GRAY2_DARK, GRAY2_LIGHT_HC, GRAY2_DARK_HC],
            // Gray3
            [GRAY3_LIGHT, GRAY3_DARK, GRAY3_LIGHT_HC, GRAY3_DARK_HC],
            // Gray4
            [GRAY4_LIGHT, GRAY4_DARK, GRAY4_LIGHT_HC, GRAY4_DARK_HC],
            // Gray5
            [GRAY5_LIGHT, GRAY5_DARK, GRAY5_LIGHT_HC, GRAY5_DARK_HC],
            // Gray6
            [GRAY6_LIGHT, GRAY6_DARK, GRAY6_LIGHT_HC, GRAY6_DARK_HC],
        ];
        TABLE[self as usize][appearance.index()]
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// System Palette
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// All 18 system colors pre-resolved for a given appearance.
///
/// Stored on [`crate::TahoeTheme`] so components can access the full
/// HIG palette without knowing the current appearance.
#[derive(Debug, Clone, Copy)]
pub struct SystemPalette {
    pub red: Hsla,
    pub orange: Hsla,
    pub yellow: Hsla,
    pub green: Hsla,
    pub mint: Hsla,
    pub teal: Hsla,
    pub cyan: Hsla,
    pub blue: Hsla,
    pub indigo: Hsla,
    pub purple: Hsla,
    pub pink: Hsla,
    pub brown: Hsla,
    pub gray: Hsla,
    pub gray2: Hsla,
    pub gray3: Hsla,
    pub gray4: Hsla,
    pub gray5: Hsla,
    pub gray6: Hsla,
}

impl SystemPalette {
    /// Build a palette for the given appearance.
    pub fn new(appearance: Appearance) -> Self {
        Self {
            red: SystemColor::Red.resolve(appearance),
            orange: SystemColor::Orange.resolve(appearance),
            yellow: SystemColor::Yellow.resolve(appearance),
            green: SystemColor::Green.resolve(appearance),
            mint: SystemColor::Mint.resolve(appearance),
            teal: SystemColor::Teal.resolve(appearance),
            cyan: SystemColor::Cyan.resolve(appearance),
            blue: SystemColor::Blue.resolve(appearance),
            indigo: SystemColor::Indigo.resolve(appearance),
            purple: SystemColor::Purple.resolve(appearance),
            pink: SystemColor::Pink.resolve(appearance),
            brown: SystemColor::Brown.resolve(appearance),
            gray: SystemGray::Gray.resolve(appearance),
            gray2: SystemGray::Gray2.resolve(appearance),
            gray3: SystemGray::Gray3.resolve(appearance),
            gray4: SystemGray::Gray4.resolve(appearance),
            gray5: SystemGray::Gray5.resolve(appearance),
            gray6: SystemGray::Gray6.resolve(appearance),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Palette Constants — System Colors
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
// Converted from HIG RGB specifications. Each color has four variants:
// LIGHT, DARK, LIGHT_HC (increased contrast), DARK_HC (increased contrast).
//
// Source: Human Interface Guidelines — Color Specifications
// https://developer.apple.com/design/human-interface-guidelines/color

// Red — canonical systemRed from HIG `foundations.md:288`.
// All four variants use a small positive hue (≤0.02) so lerps and
// complementary-color arithmetic stay on the short side of the hue wheel.
const RED_LIGHT: Hsla = Hsla {
    h: 0.0089,
    s: 1.0,
    l: 0.5941,
    a: 1.0,
}; // #FF3B30 = rgb(255, 59, 48)
const RED_DARK: Hsla = Hsla {
    h: 0.0093,
    s: 1.0,
    l: 0.6137,
    a: 1.0,
}; // #FF453A = rgb(255, 69, 58)
const RED_LIGHT_HC: Hsla = Hsla {
    h: 0.0,
    s: 0.7553,
    l: 0.4804,
    a: 1.0,
}; // #D71E1E = rgb(215, 30, 30) — deep red, ≥5:1 on white
const RED_DARK_HC: Hsla = Hsla {
    h: 0.0,
    s: 1.0,
    l: 0.6961,
    a: 1.0,
}; // #FF6464 = rgb(255, 100, 100) — bright red, ≥5:1 on dark bg

// Orange — canonical systemOrange from HIG `foundations.md:289`.
const ORANGE_LIGHT: Hsla = Hsla {
    h: 0.0974,
    s: 1.0,
    l: 0.5,
    a: 1.0,
}; // #FF9500 = rgb(255, 149, 0)
const ORANGE_DARK: Hsla = Hsla {
    h: 0.1014,
    s: 1.0,
    l: 0.5196,
    a: 1.0,
}; // #FF9F0A = rgb(255, 159, 10)
const ORANGE_LIGHT_HC: Hsla = Hsla {
    h: 0.0702,
    s: 1.0,
    l: 0.3863,
    a: 1.0,
}; // rgb(197, 83, 0)
const ORANGE_DARK_HC: Hsla = Hsla {
    h: 0.073,
    s: 1.0,
    l: 0.6686,
    a: 1.0,
}; // rgb(255, 160, 86)

// Yellow
const YELLOW_LIGHT: Hsla = Hsla {
    h: 0.1333,
    s: 1.0,
    l: 0.5,
    a: 1.0,
}; // rgb(255, 204, 0)
const YELLOW_DARK: Hsla = Hsla {
    h: 0.1399,
    s: 1.0,
    l: 0.5,
    a: 1.0,
}; // rgb(255, 214, 0)
const YELLOW_LIGHT_HC: Hsla = Hsla {
    h: 0.1097,
    s: 1.0,
    l: 0.3157,
    a: 1.0,
}; // rgb(161, 106, 0)
const YELLOW_DARK_HC: Hsla = Hsla {
    h: 0.139,
    s: 0.9894,
    l: 0.6294,
    a: 1.0,
}; // rgb(254, 223, 67)

// Green
const GREEN_LIGHT: Hsla = Hsla {
    h: 0.3753,
    s: 0.5857,
    l: 0.4922,
    a: 1.0,
}; // rgb(52, 199, 89)
const GREEN_DARK: Hsla = Hsla {
    h: 0.3747,
    s: 0.6364,
    l: 0.5039,
    a: 1.0,
}; // rgb(48, 209, 88)
const GREEN_LIGHT_HC: Hsla = Hsla {
    h: 0.3942,
    s: 1.0,
    l: 0.2686,
    a: 1.0,
}; // rgb(0, 137, 50)
const GREEN_DARK_HC: Hsla = Hsla {
    h: 0.3683,
    s: 0.653,
    l: 0.5706,
    a: 1.0,
}; // rgb(74, 217, 104)

// Mint
// Mint — canonical systemMint from HIG `foundations.md:292`.
// Apple does not publish HC variants for systemMint, so the HC entries here
// are derived from the standard values: HC-light deepens lightness for
// white backgrounds and HC-dark brightens for dark backgrounds.
const MINT_LIGHT: Hsla = Hsla {
    h: 0.4925,
    s: 1.0,
    l: 0.3902,
    a: 1.0,
}; // #00C7BE = rgb(0, 199, 190)
const MINT_DARK: Hsla = Hsla {
    h: 0.4949,
    s: 0.7238,
    l: 0.6451,
    a: 1.0,
}; // #63E6E2 = rgb(99, 230, 226)
const MINT_LIGHT_HC: Hsla = Hsla {
    h: 0.4895,
    s: 1.0,
    l: 0.2549,
    a: 1.0,
}; // #00827A = rgb(0, 130, 122) — derived for ≥4.5:1 on white
const MINT_DARK_HC: Hsla = Hsla {
    h: 0.4905,
    s: 0.7778,
    l: 0.7353,
    a: 1.0,
}; // #87F0EA = rgb(135, 240, 234) — derived for ≥5:1 on dark bg

// Teal — macOS-aligned. macOS's `NSColor.systemTeal` (used by AppKit
// controls) uses a deeper, less saturated teal than iOS's systemTeal. This
// crate targets macOS first (per CLAUDE.md), so Teal stays at the macOS
// hue/lightness and Cyan below carries the iOS-style sky cyan.
//
// iOS reference (informational): iOS 15+ `systemTeal` = `#30B0C7`/`#40C8E0`
// — substantially lighter and more cyan than the values here. Pre-iOS-15
// `systemTeal` was `#5AC8FA`/`#64D2FF` (now `systemCyan` in iOS 15+).
// `foundations.md:302` documents this naming history in detail.
const TEAL_LIGHT: Hsla = Hsla {
    h: 0.5104,
    s: 1.0,
    l: 0.4078,
    a: 1.0,
}; // #00C3D0 = rgb(0, 195, 208)
const TEAL_DARK: Hsla = Hsla {
    h: 0.5104,
    s: 1.0,
    l: 0.4392,
    a: 1.0,
}; // #00D2E0 = rgb(0, 210, 224)
const TEAL_LIGHT_HC: Hsla = Hsla {
    h: 0.5252,
    s: 1.0,
    l: 0.298,
    a: 1.0,
}; // #008198 = rgb(0, 129, 152)
const TEAL_DARK_HC: Hsla = Hsla {
    h: 0.5141,
    s: 0.8233,
    l: 0.5784,
    a: 1.0,
}; // #3BDDEC = rgb(59, 221, 236)

// Cyan — iOS-aligned approximate. HIG `foundations.md:294` lists
// `systemCyan` = `#32ADE6` / `#64D2FF`; the values here are close but
// slightly more saturated/deeper for visual impact in `theme.info` and the
// cyan glass tint. Renderers that need pixel-exact iOS systemCyan should
// resolve `SystemColor::Cyan` and accept the small delta. The history of
// the iOS 15 split that introduced `systemCyan` is documented at
// `foundations.md:302`.
const CYAN_LIGHT: Hsla = Hsla {
    h: 0.5287,
    s: 1.0,
    l: 0.4549,
    a: 1.0,
}; // #00C0E8 = rgb(0, 192, 232)
const CYAN_DARK: Hsla = Hsla {
    h: 0.5369,
    s: 0.9898,
    l: 0.6157,
    a: 1.0,
}; // #3CD3FE = rgb(60, 211, 254)
const CYAN_LIGHT_HC: Hsla = Hsla {
    h: 0.546,
    s: 1.0,
    l: 0.3412,
    a: 1.0,
}; // #007EAE = rgb(0, 126, 174)
const CYAN_DARK_HC: Hsla = Hsla {
    h: 0.5434,
    s: 1.0,
    l: 0.7137,
    a: 1.0,
}; // #6DD9FF = rgb(109, 217, 255)

// Blue — canonical systemBlue from HIG `foundations.md:295`.
const BLUE_LIGHT: Hsla = Hsla {
    h: 0.5869,
    s: 1.0,
    l: 0.5,
    a: 1.0,
}; // #007AFF = rgb(0, 122, 255)
const BLUE_DARK: Hsla = Hsla {
    h: 0.5837,
    s: 1.0,
    l: 0.5196,
    a: 1.0,
}; // #0A84FF = rgb(10, 132, 255)
const BLUE_LIGHT_HC: Hsla = Hsla {
    h: 0.6044,
    s: 0.9068,
    l: 0.5373,
    a: 1.0,
}; // rgb(30, 110, 244)
const BLUE_DARK_HC: Hsla = Hsla {
    h: 0.5726,
    s: 1.0,
    l: 0.6804,
    a: 1.0,
}; // rgb(92, 184, 255)

// Indigo — canonical systemIndigo from HIG `foundations.md:296`.
const INDIGO_LIGHT: Hsla = Hsla {
    h: 0.6693,
    s: 0.6095,
    l: 0.5882,
    a: 1.0,
}; // #5856D6 = rgb(88, 86, 214)
const INDIGO_DARK: Hsla = Hsla {
    h: 0.6691,
    s: 0.7340,
    l: 0.6314,
    a: 1.0,
}; // #5E5CE6 = rgb(94, 92, 230)
const INDIGO_LIGHT_HC: Hsla = Hsla {
    h: 0.6802,
    s: 0.6916,
    l: 0.5804,
    a: 1.0,
}; // rgb(86, 74, 222)
const INDIGO_DARK_HC: Hsla = Hsla {
    h: 0.661,
    s: 1.0,
    l: 0.8275,
    a: 1.0,
}; // rgb(167, 170, 255)

// Purple — canonical systemPurple from HIG `foundations.md:297`.
const PURPLE_LIGHT: Hsla = Hsla {
    h: 0.7774,
    s: 0.6796,
    l: 0.5961,
    a: 1.0,
}; // #AF52DE = rgb(175, 82, 222)
const PURPLE_DARK: Hsla = Hsla {
    h: 0.7774,
    s: 0.8539,
    l: 0.6510,
    a: 1.0,
}; // #BF5AF2 = rgb(191, 90, 242)
const PURPLE_LIGHT_HC: Hsla = Hsla {
    h: 0.8129,
    s: 0.61,
    l: 0.4725,
    a: 1.0,
}; // rgb(176, 47, 194)
const PURPLE_DARK_HC: Hsla = Hsla {
    h: 0.8026,
    s: 1.0,
    l: 0.7765,
    a: 1.0,
}; // rgb(234, 141, 255)

// Pink
const PINK_LIGHT: Hsla = Hsla {
    h: 0.9683,
    s: 1.0,
    l: 0.5882,
    a: 1.0,
}; // rgb(255, 45, 85)
const PINK_DARK: Hsla = Hsla {
    h: 0.9667,
    s: 1.0,
    l: 0.6078,
    a: 1.0,
}; // rgb(255, 55, 95)
const PINK_LIGHT_HC: Hsla = Hsla {
    h: 0.9538,
    s: 0.8554,
    l: 0.4882,
    a: 1.0,
}; // rgb(231, 18, 77)
const PINK_DARK_HC: Hsla = Hsla {
    h: 0.9174,
    s: 1.0,
    l: 0.7706,
    a: 1.0,
}; // rgb(255, 138, 196)

// Brown — canonical systemBrown from HIG `foundations.md:299`.
const BROWN_LIGHT: Hsla = Hsla {
    h: 0.0931,
    s: 0.2677,
    l: 0.5020,
    a: 1.0,
}; // #A2845E = rgb(162, 132, 94)
const BROWN_DARK: Hsla = Hsla {
    h: 0.0931,
    s: 0.2906,
    l: 0.5412,
    a: 1.0,
}; // #AC8E68 = rgb(172, 142, 104)
const BROWN_LIGHT_HC: Hsla = Hsla {
    h: 0.0686,
    s: 0.2957,
    l: 0.451,
    a: 1.0,
}; // rgb(149, 109, 81)
const BROWN_DARK_HC: Hsla = Hsla {
    h: 0.0765,
    s: 0.5765,
    l: 0.6667,
    a: 1.0,
}; // rgb(219, 166, 121)

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Palette Constants — System Grays
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

// Gray
const GRAY_LIGHT: Hsla = Hsla {
    h: 0.6667,
    s: 0.0226,
    l: 0.5667,
    a: 1.0,
}; // rgb(142, 142, 147)
const GRAY_DARK: Hsla = Hsla {
    h: 0.6667,
    s: 0.0226,
    l: 0.5667,
    a: 1.0,
}; // rgb(142, 142, 147)
const GRAY_LIGHT_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0182,
    l: 0.4314,
    a: 1.0,
}; // rgb(108, 108, 112)
const GRAY_DARK_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0253,
    l: 0.6902,
    a: 1.0,
}; // rgb(174, 174, 178)

// Gray 2
const GRAY2_LIGHT: Hsla = Hsla {
    h: 0.6667,
    s: 0.0253,
    l: 0.6902,
    a: 1.0,
}; // rgb(174, 174, 178)
const GRAY2_DARK: Hsla = Hsla {
    h: 0.6667,
    s: 0.0149,
    l: 0.3941,
    a: 1.0,
}; // rgb(99, 99, 102)
const GRAY2_LIGHT_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0226,
    l: 0.5667,
    a: 1.0,
}; // rgb(142, 142, 147)
const GRAY2_DARK_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0159,
    l: 0.4941,
    a: 1.0,
}; // rgb(124, 124, 128)

// Gray 3
const GRAY3_LIGHT: Hsla = Hsla {
    h: 0.6667,
    s: 0.0467,
    l: 0.7902,
    a: 1.0,
}; // rgb(199, 199, 204)
const GRAY3_DARK: Hsla = Hsla {
    h: 0.6667,
    s: 0.0137,
    l: 0.2863,
    a: 1.0,
}; // rgb(72, 72, 74)
const GRAY3_LIGHT_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0253,
    l: 0.6902,
    a: 1.0,
}; // rgb(174, 174, 178)
const GRAY3_DARK_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0118,
    l: 0.3333,
    a: 1.0,
}; // rgb(84, 84, 86)

// Gray 4
const GRAY4_LIGHT: Hsla = Hsla {
    h: 0.6667,
    s: 0.0575,
    l: 0.8294,
    a: 1.0,
}; // rgb(209, 209, 214)
const GRAY4_DARK: Hsla = Hsla {
    h: 0.6667,
    s: 0.0169,
    l: 0.2314,
    a: 1.0,
}; // rgb(58, 58, 60)
const GRAY4_LIGHT_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0308,
    l: 0.7451,
    a: 1.0,
}; // rgb(188, 188, 192)
const GRAY4_DARK_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0145,
    l: 0.2706,
    a: 1.0,
}; // rgb(68, 68, 70)

// Gray 5
const GRAY5_LIGHT: Hsla = Hsla {
    h: 0.6667,
    s: 0.1064,
    l: 0.9078,
    a: 1.0,
}; // rgb(229, 229, 234)
const GRAY5_DARK: Hsla = Hsla {
    h: 0.6667,
    s: 0.0222,
    l: 0.1765,
    a: 1.0,
}; // rgb(44, 44, 46)
const GRAY5_LIGHT_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0541,
    l: 0.8549,
    a: 1.0,
}; // rgb(216, 216, 220)
const GRAY5_DARK_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.0182,
    l: 0.2157,
    a: 1.0,
}; // rgb(54, 54, 56)

// Gray 6
const GRAY6_LIGHT: Hsla = Hsla {
    h: 0.6667,
    s: 0.2381,
    l: 0.9588,
    a: 1.0,
}; // rgb(242, 242, 247)
const GRAY6_DARK: Hsla = Hsla {
    h: 0.6667,
    s: 0.0345,
    l: 0.1137,
    a: 1.0,
}; // rgb(28, 28, 30)
const GRAY6_LIGHT_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.1429,
    l: 0.9314,
    a: 1.0,
}; // rgb(235, 235, 240)
const GRAY6_DARK_HC: Hsla = Hsla {
    h: 0.6667,
    s: 0.027,
    l: 0.1451,
    a: 1.0,
}; // rgb(36, 36, 38)

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Utility Functions
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Return `color` with its alpha channel replaced.
///
/// Semantic mirror of Zed's `Hsla::alpha` — assigns the alpha directly
/// without consulting the existing value. Use [`opacity`] to multiply the
/// existing alpha, or [`fade_out`] to shift alpha towards transparency.
pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

/// Return `color` with its alpha multiplied by `factor`.
///
/// Mirrors Zed's `Hsla::opacity(f32)` — scales the existing alpha rather
/// than replacing it, so `#000 @ 60%` composed with `opacity(0.5)` yields
/// `#000 @ 30%`. `factor` is clamped to `[0.0, 1.0]`; non-finite inputs are
/// treated as `1.0` (no change) so a single bad caller cannot poison the
/// color pipeline.
pub fn opacity(color: Hsla, factor: f32) -> Hsla {
    let factor = if factor.is_finite() {
        factor.clamp(0.0, 1.0)
    } else {
        1.0
    };
    Hsla {
        a: (color.a * factor).clamp(0.0, 1.0),
        ..color
    }
}

/// Return `color` with its alpha multiplicatively faded towards zero.
///
/// Mirrors Zed's `Hsla::fade_out(f32)`: a `factor` of `1.0` produces fully
/// transparent, `0.0` leaves the color untouched. Equivalent to
/// `opacity(color, 1.0 - factor)` but phrased so callers can express
/// "fade this out by 30 %" directly.
pub fn fade_out(color: Hsla, factor: f32) -> Hsla {
    let factor = if factor.is_finite() {
        factor.clamp(0.0, 1.0)
    } else {
        0.0
    };
    opacity(color, 1.0 - factor)
}

/// Extension trait that adds Zed-style alpha helpers to [`Hsla`].
///
/// Exposing the helpers as methods lets call sites chain `some_hsla
/// .opacity(0.6)` the same way Zed does, instead of wrapping every alpha
/// tweak in `with_alpha(color, color.a * 0.6)`. Finding 10 in the
/// the Zed cross-reference audit Zed cross-reference audit tracks this gap.
pub trait HslaAlphaExt: Copy {
    /// Replace the alpha channel (see [`with_alpha`]).
    fn alpha(self, alpha: f32) -> Self;
    /// Multiply the alpha channel by `factor` (see [`opacity`]).
    fn opacity(self, factor: f32) -> Self;
    /// Fade towards transparency by `factor` (see [`fade_out`]).
    fn fade_out(self, factor: f32) -> Self;
}

impl HslaAlphaExt for Hsla {
    fn alpha(self, alpha: f32) -> Self {
        with_alpha(self, alpha)
    }

    fn opacity(self, factor: f32) -> Self {
        opacity(self, factor)
    }

    fn fade_out(self, factor: f32) -> Self {
        fade_out(self, factor)
    }
}

/// Darken a color by reducing lightness. Clamps to 0.0.
/// Returns the color unchanged if either `color.l` or `amount` is NaN/infinity.
pub fn darken(color: Hsla, amount: f32) -> Hsla {
    if !color.l.is_finite() || !amount.is_finite() {
        return color;
    }
    Hsla {
        l: (color.l - amount).clamp(0.0, 1.0),
        ..color
    }
}

/// Lighten a color by increasing lightness. Clamps to 1.0.
/// Returns the color unchanged if either `color.l` or `amount` is NaN/infinity.
pub fn lighten(color: Hsla, amount: f32) -> Hsla {
    if !color.l.is_finite() || !amount.is_finite() {
        return color;
    }
    Hsla {
        l: (color.l + amount).clamp(0.0, 1.0),
        ..color
    }
}

/// Choose white or black text for legibility over `bg`.
///
/// The threshold `0.55` is a lightness heuristic that agrees with WCAG AA
/// for the most common accent colors (system blue, green, purple, etc.).
/// It favours white on medium blues (`l = 0.50`) which is the correct
/// choice since white-on-blue yields ~4.7:1 contrast whereas black-on-blue
/// yields ~4.3:1.
///
/// If `bg.l` is NaN, the comparison `> 0.55` is false, so white is
/// returned — the conservative default for unknown backgrounds.
pub fn text_on_background(bg: Hsla) -> Hsla {
    if bg.l > 0.55 {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 1.0,
        } // black
    } else {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
        } // white
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// sRGB / Linear-RGB conversion
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Convert a single sRGB channel value [0..1] to linear-light RGB.
pub(crate) fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert a single linear-light RGB channel value [0..1] back to sRGB.
pub(crate) fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Composite an opaque black tint of `tint_alpha` over `bg` in linear-light
/// RGB space, preserving `bg.a`.
///
/// This is the correct math for Apple's Liquid Glass "Layer 2" tint
/// (`#000000 @ 20%` in Figma) composited over the base Layer 1 fill. A naive
/// blend in HSL lightness diverges from the Porter–Duff linear-light result
/// by up to ~17% on bright surfaces; this helper keeps the tinted surface
/// visually consistent across the full 0..1 lightness range.
///
/// Color channels: `out = (1 - tint_alpha) * srgb_to_linear(bg)` per channel,
/// re-encoded back to sRGB. Alpha is passed through unchanged so the
/// glass surface retains the same translucency it had before tinting.
pub fn compose_black_tint_linear(bg: Hsla, tint_alpha: f32) -> Hsla {
    let keep = 1.0 - tint_alpha.clamp(0.0, 1.0);
    let rgba = bg.to_rgb();
    let blend = |v: f32| linear_to_srgb(keep * srgb_to_linear(v));
    let tinted = gpui::Rgba {
        r: blend(rgba.r),
        g: blend(rgba.g),
        b: blend(rgba.b),
        a: rgba.a,
    };
    let mut out: Hsla = tinted.into();
    out.a = bg.a;
    out
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Contrast Utilities
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Computes the relative luminance of an HSLA color (simplified sRGB).
/// Uses the HSL-to-RGB conversion, then applies the sRGB luminance formula.
pub(crate) fn relative_luminance(c: Hsla) -> f32 {
    // Convert HSL to linear RGB
    let h = c.h * 360.0;
    let s = c.s;
    let l = c.l;
    let c_val = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c_val * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = l - c_val / 2.0;
    let (r1, g1, b1) = if h < 60.0 {
        (c_val, x, 0.0)
    } else if h < 120.0 {
        (x, c_val, 0.0)
    } else if h < 180.0 {
        (0.0, c_val, x)
    } else if h < 240.0 {
        (0.0, x, c_val)
    } else if h < 300.0 {
        (x, 0.0, c_val)
    } else {
        (c_val, 0.0, x)
    };
    0.2126 * srgb_to_linear(r1 + m)
        + 0.7152 * srgb_to_linear(g1 + m)
        + 0.0722 * srgb_to_linear(b1 + m)
}

/// Returns the WCAG 2.1 contrast ratio between two colors (1.0 to 21.0).
/// Does not account for alpha compositing — assumes both colors are opaque
/// or pre-composited against their background.
pub fn contrast_ratio(fg: Hsla, bg: Hsla) -> f32 {
    debug_assert!(
        fg.a > 0.99 && bg.a > 0.99,
        "contrast_ratio requires opaque colors; pre-composite against background first"
    );
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Returns `true` if the foreground/background pair meets the specified
/// minimum contrast ratio per WCAG 2.1.
///
/// Common thresholds:
/// - 4.5:1 for normal text (AA)
/// - 3.0:1 for large text and non-text elements (AA)
/// - 7.0:1 for enhanced contrast (AAA)
pub fn meets_contrast(fg: Hsla, bg: Hsla, threshold: f32) -> bool {
    contrast_ratio(fg, bg) >= threshold
}

#[cfg(test)]
mod tests {
    use super::{
        Appearance, HslaAlphaExt, SystemColor, SystemGray, SystemPalette, darken, fade_out,
        lighten, opacity, text_on_background, with_alpha,
    };
    use core::prelude::v1::test;
    use gpui::Hsla;

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

    #[test]
    fn all_system_colors_resolve() {
        let colors = [
            SystemColor::Red,
            SystemColor::Orange,
            SystemColor::Yellow,
            SystemColor::Green,
            SystemColor::Mint,
            SystemColor::Teal,
            SystemColor::Cyan,
            SystemColor::Blue,
            SystemColor::Indigo,
            SystemColor::Purple,
            SystemColor::Pink,
            SystemColor::Brown,
        ];
        let appearances = [
            Appearance::Light,
            Appearance::Dark,
            Appearance::LightHighContrast,
            Appearance::DarkHighContrast,
        ];
        for color in colors {
            for appearance in appearances {
                let hsla = color.resolve(appearance);
                assert_eq!(
                    hsla.a, 1.0,
                    "{color:?} in {appearance:?} should have full alpha"
                );
                assert!(
                    (0.0..=1.0).contains(&hsla.h),
                    "{color:?}/{appearance:?} h out of range"
                );
                assert!(
                    (0.0..=1.0).contains(&hsla.s),
                    "{color:?}/{appearance:?} s out of range"
                );
                assert!(
                    (0.0..=1.0).contains(&hsla.l),
                    "{color:?}/{appearance:?} l out of range"
                );
            }
        }
    }

    #[test]
    fn all_system_grays_resolve() {
        let grays = [
            SystemGray::Gray,
            SystemGray::Gray2,
            SystemGray::Gray3,
            SystemGray::Gray4,
            SystemGray::Gray5,
            SystemGray::Gray6,
        ];
        let appearances = [
            Appearance::Light,
            Appearance::Dark,
            Appearance::LightHighContrast,
            Appearance::DarkHighContrast,
        ];
        for gray in grays {
            for appearance in appearances {
                let hsla = gray.resolve(appearance);
                assert_eq!(
                    hsla.a, 1.0,
                    "{gray:?} in {appearance:?} should have full alpha"
                );
            }
        }
    }

    #[test]
    fn light_grays_get_lighter_at_higher_levels() {
        // In light mode, Gray6 should be lighter than Gray
        let g1 = SystemGray::Gray.resolve(Appearance::Light);
        let g6 = SystemGray::Gray6.resolve(Appearance::Light);
        assert!(
            g6.l > g1.l,
            "Gray6 (L={}) should be lighter than Gray (L={})",
            g6.l,
            g1.l
        );
    }

    #[test]
    fn dark_grays_get_darker_at_higher_levels() {
        // In dark mode, Gray6 should be darker than Gray
        let g1 = SystemGray::Gray.resolve(Appearance::Dark);
        let g6 = SystemGray::Gray6.resolve(Appearance::Dark);
        assert!(
            g6.l < g1.l,
            "Gray6 (L={}) should be darker than Gray (L={})",
            g6.l,
            g1.l
        );
    }

    #[test]
    fn high_contrast_colors_differ_from_standard() {
        // High-contrast red should differ from standard red
        let std_light = SystemColor::Red.resolve(Appearance::Light);
        let hc_light = SystemColor::Red.resolve(Appearance::LightHighContrast);
        assert_ne!(
            std_light.l, hc_light.l,
            "HC light red should differ from standard"
        );

        let std_dark = SystemColor::Red.resolve(Appearance::Dark);
        let hc_dark = SystemColor::Red.resolve(Appearance::DarkHighContrast);
        assert_ne!(
            std_dark.l, hc_dark.l,
            "HC dark red should differ from standard"
        );
    }

    #[test]
    fn system_palette_builds_for_all_appearances() {
        for appearance in [
            Appearance::Light,
            Appearance::Dark,
            Appearance::LightHighContrast,
            Appearance::DarkHighContrast,
        ] {
            let palette = SystemPalette::new(appearance);
            assert_eq!(palette.red.a, 1.0);
            assert_eq!(palette.blue.a, 1.0);
            assert_eq!(palette.gray6.a, 1.0);
        }
    }

    #[test]
    fn palette_matches_individual_resolve() {
        let appearance = Appearance::Dark;
        let palette = SystemPalette::new(appearance);
        assert_eq!(palette.red, SystemColor::Red.resolve(appearance));
        assert_eq!(palette.blue, SystemColor::Blue.resolve(appearance));
        assert_eq!(palette.gray, SystemGray::Gray.resolve(appearance));
        assert_eq!(palette.gray6, SystemGray::Gray6.resolve(appearance));
    }

    #[test]
    fn blue_light_matches_apple_spec() {
        // HIG Blue (light): RGB(0, 136, 255) → should be a vivid blue
        let blue = SystemColor::Blue.resolve(Appearance::Light);
        // Hue should be around 0.57-0.59 (blue range)
        assert!(
            (0.55..0.62).contains(&blue.h),
            "Blue hue {} not in blue range",
            blue.h
        );
        // Should be fully saturated
        assert!(
            blue.s > 0.95,
            "Blue saturation {} should be near 1.0",
            blue.s
        );
        // Lightness around 0.5
        assert!(
            (0.4..0.6).contains(&blue.l),
            "Blue lightness {} not in expected range",
            blue.l
        );
    }

    #[test]
    fn green_light_matches_apple_spec() {
        // HIG Green (light): RGB(52, 199, 89) → should be a medium green
        let green = SystemColor::Green.resolve(Appearance::Light);
        // Hue should be around 0.37-0.39 (green range)
        assert!(
            (0.35..0.42).contains(&green.h),
            "Green hue {} not in green range",
            green.h
        );
        // Medium saturation
        assert!(
            green.s > 0.5,
            "Green saturation {} should be above 0.5",
            green.s
        );
    }

    #[test]
    fn with_alpha_preserves_hsl() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.6,
            a: 1.0,
        };
        let r = with_alpha(c, 0.3);
        assert_eq!(r.h, 0.5);
        assert_eq!(r.s, 0.8);
        assert_eq!(r.l, 0.6);
        assert_eq!(r.a, 0.3);
    }

    #[test]
    fn opacity_multiplies_existing_alpha() {
        let c = Hsla {
            h: 0.25,
            s: 0.6,
            l: 0.5,
            a: 0.6,
        };
        let r = opacity(c, 0.5);
        assert!((r.a - 0.3).abs() < 1e-6, "opacity should scale alpha");
        // Non-alpha channels pass through untouched.
        assert_eq!((r.h, r.s, r.l), (c.h, c.s, c.l));
    }

    #[test]
    fn opacity_clamps_factor() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.5,
            a: 0.8,
        };
        // Negative factor clamps to 0 → alpha 0.
        assert!(opacity(c, -0.5).a.abs() < 1e-6);
        // Factor > 1 clamps to 1 → alpha unchanged.
        assert!((opacity(c, 2.0).a - 0.8).abs() < 1e-6);
        // NaN treated as 1.0 (leave color alone) so one bad caller can't
        // poison the pipeline.
        assert!((opacity(c, f32::NAN).a - 0.8).abs() < 1e-6);
    }

    #[test]
    fn fade_out_is_inverse_of_opacity() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.5,
            a: 1.0,
        };
        let faded = fade_out(c, 0.3);
        assert!((faded.a - 0.7).abs() < 1e-6);
        // fade_out(factor) == opacity(1 - factor)
        assert!((faded.a - opacity(c, 0.7).a).abs() < 1e-6);
    }

    #[test]
    fn hsla_alpha_ext_chain_matches_free_fns() {
        let c = Hsla {
            h: 0.1,
            s: 0.2,
            l: 0.3,
            a: 0.8,
        };
        assert_eq!(c.alpha(0.4), with_alpha(c, 0.4));
        assert_eq!(c.opacity(0.5), opacity(c, 0.5));
        assert_eq!(c.fade_out(0.25), fade_out(c, 0.25));
    }

    #[test]
    fn darken_reduces_lightness() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.6,
            a: 1.0,
        };
        let r = darken(c, 0.1);
        assert_eq!(r.h, 0.5);
        assert_eq!(r.s, 0.8);
        assert!((r.l - 0.5).abs() < 0.001);
        assert_eq!(r.a, 1.0);
    }

    #[test]
    fn darken_clamps_at_zero() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.05,
            a: 1.0,
        };
        let r = darken(c, 0.2);
        assert_eq!(r.l, 0.0);
    }

    #[test]
    fn lighten_increases_lightness() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.4,
            a: 1.0,
        };
        let r = lighten(c, 0.1);
        assert!((r.l - 0.5).abs() < 0.001);
        assert_eq!(r.a, 1.0);
    }

    #[test]
    fn lighten_clamps_at_one() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.95,
            a: 1.0,
        };
        let r = lighten(c, 0.2);
        assert_eq!(r.l, 1.0);
    }

    #[test]
    fn darken_with_negative_amount_clamps_at_one() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.8,
            a: 1.0,
        };
        let r = darken(c, -0.5);
        assert!(
            r.l <= 1.0,
            "darken with negative amount should clamp at 1.0, got {}",
            r.l
        );
    }

    #[test]
    fn lighten_with_negative_amount_clamps_at_zero() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.1,
            a: 1.0,
        };
        let r = lighten(c, -0.5);
        assert!(
            r.l >= 0.0,
            "lighten with negative amount should clamp at 0.0, got {}",
            r.l
        );
    }

    #[test]
    fn text_on_background_dark_bg_returns_white() {
        let dark = Hsla {
            h: 0.58,
            s: 1.0,
            l: 0.3,
            a: 1.0,
        };
        let result = text_on_background(dark);
        assert_eq!(result.l, 1.0, "dark bg should get white text");
    }

    #[test]
    fn text_on_background_light_bg_returns_black() {
        let light = Hsla {
            h: 0.58,
            s: 1.0,
            l: 0.7,
            a: 1.0,
        };
        let result = text_on_background(light);
        assert_eq!(result.l, 0.0, "light bg should get black text");
    }

    #[test]
    fn text_on_background_nan_defaults_to_white() {
        let nan_bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: f32::NAN,
            a: 1.0,
        };
        let result = text_on_background(nan_bg);
        assert_eq!(result.l, 1.0, "NaN bg.l should default to white");
    }

    // -- darken/lighten NaN guards --

    #[test]
    fn darken_nan_lightness_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: f32::NAN,
            a: 1.0,
        };
        let r = darken(c, 0.1);
        assert!(r.l.is_nan(), "NaN lightness should pass through unchanged");
        assert_eq!(r.h, 0.5);
    }

    #[test]
    fn lighten_infinity_lightness_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: f32::INFINITY,
            a: 1.0,
        };
        let r = lighten(c, 0.1);
        assert!(r.l.is_infinite());
    }

    #[test]
    fn darken_nan_amount_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.5,
            a: 1.0,
        };
        let r = darken(c, f32::NAN);
        assert_eq!(r.l, 0.5, "NaN amount should leave color unchanged");
    }

    #[test]
    fn lighten_nan_amount_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.5,
            a: 1.0,
        };
        let r = lighten(c, f32::NAN);
        assert_eq!(r.l, 0.5, "NaN amount should leave color unchanged");
    }

    // ── AccentColor tests ──────────────────────────────────────────────

    use super::AccentColor;

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
        // Each non-Multicolor variant should be a distinct enum value
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

    // ── sRGB / Linear-RGB tint tests ───────────────────────────────────

    use super::{compose_black_tint_linear, linear_to_srgb, srgb_to_linear};

    #[test]
    fn srgb_linear_roundtrip_is_identity() {
        for &v in &[0.0f32, 0.04, 0.17, 0.5, 0.8, 0.969, 1.0] {
            let round = linear_to_srgb(srgb_to_linear(v));
            assert!(
                (round - v).abs() < 1e-4,
                "sRGB->linear->sRGB roundtrip failed for {v}: got {round}"
            );
        }
    }

    #[test]
    fn compose_black_tint_preserves_alpha() {
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.17,
            a: 0.80,
        };
        let out = compose_black_tint_linear(bg, 0.20);
        assert!(
            (out.a - bg.a).abs() < f32::EPSILON,
            "alpha must be preserved, got {}",
            out.a
        );
    }

    #[test]
    fn compose_black_tint_matches_linear_rgb() {
        // At L=0.969 (light small bg), sRGB ≈ 0.969. Linear ≈ 0.930.
        // 80% of linear ≈ 0.744. Back to sRGB ≈ 0.874.
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.969,
            a: 1.0,
        };
        let out = compose_black_tint_linear(bg, 0.20);
        assert!(
            (out.l - 0.874).abs() < 0.01,
            "expected ~0.874 from linear-RGB blend, got {}",
            out.l
        );
    }

    #[test]
    fn compose_black_tint_scales_linear_dark() {
        // At L=0.17, sRGB=0.17, linear ≈ 0.0225. 80% ≈ 0.018, back to sRGB ≈ 0.144.
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.17,
            a: 0.80,
        };
        let out = compose_black_tint_linear(bg, 0.20);
        assert!(
            (out.l - 0.144).abs() < 0.01,
            "expected ~0.144 from linear-RGB blend, got {}",
            out.l
        );
    }

    #[test]
    fn compose_black_tint_zero_alpha_is_identity() {
        let bg = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.5,
            a: 0.67,
        };
        let out = compose_black_tint_linear(bg, 0.0);
        assert!((out.l - bg.l).abs() < 1e-4);
        assert!((out.a - bg.a).abs() < f32::EPSILON);
    }

    #[test]
    fn compose_black_tint_full_alpha_drives_to_black() {
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.8,
            a: 1.0,
        };
        let out = compose_black_tint_linear(bg, 1.0);
        assert!(
            out.l < 0.01,
            "full tint should collapse to black, got {}",
            out.l
        );
    }
}
