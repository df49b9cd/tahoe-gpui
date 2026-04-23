//! HIG system palette — 12 named colours and 6 grays × 4 appearance variants.
//!
//! Source: Human Interface Guidelines — Color Specifications
//! <https://developer.apple.com/design/human-interface-guidelines/color>

use gpui::Hsla;

use super::Appearance;

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
    h: 0.1388,
    s: 1.0,
    l: 0.5196,
    a: 1.0,
}; // #FFD60A = rgb(255, 214, 10)
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
//
// Per Apple HIG (UIColor.systemGray): `systemGray` is the one gray-family
// token that resolves to the *same* rgb value (rgb(142, 142, 147)) in both
// light and dark default appearances. `systemGray2..6` diverge per mode.
// Keeping the identical literals here is intentional — do not "dedupe" or
// adjust without re-checking Apple's color table.
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
}; // rgb(142, 142, 147) — intentionally equal to GRAY_LIGHT, see note above.
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

#[cfg(test)]
mod tests {
    use super::{SystemColor, SystemGray, SystemPalette};
    use crate::foundations::color::Appearance;
    use core::prelude::v1::test;

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
        let blue = SystemColor::Blue.resolve(Appearance::Light);
        assert!(
            (0.55..0.62).contains(&blue.h),
            "Blue hue {} not in blue range",
            blue.h
        );
        assert!(
            blue.s > 0.95,
            "Blue saturation {} should be near 1.0",
            blue.s
        );
        assert!(
            (0.4..0.6).contains(&blue.l),
            "Blue lightness {} not in expected range",
            blue.l
        );
    }

    #[test]
    fn green_light_matches_apple_spec() {
        let green = SystemColor::Green.resolve(Appearance::Light);
        assert!(
            (0.35..0.42).contains(&green.h),
            "Green hue {} not in green range",
            green.h
        );
        assert!(
            green.s > 0.5,
            "Green saturation {} should be above 0.5",
            green.s
        );
    }
}
