//! SF Symbols integration aligned with HIG.
//!
//! SF Symbols is Apple's iconography library with 6,000+ symbols designed
//! to integrate with San Francisco system font. This module re-exports
//! the icon system and documents rendering mode semantics.
//!
//! # Rendering modes
//!
//! SF Symbols supports four rendering modes, each mapped to [`IconRenderMode`]:
//!
//! - **Monochrome** — Single color, inherits text color. Best for toolbars and navigation.
//! - **Hierarchical** — Single color with varying opacity layers for depth.
//! - **Palette** — Two or three custom colors for distinct layers.
//! - **Multicolor** — Fixed colors defined by the symbol design (e.g., folder.fill is always blue).
//!
//! # Symbol weights and scales
//!
//! Symbols automatically match the weight of adjacent text when using the system font.
//! The [`SymbolWeight`] and [`SymbolScale`] enums allow explicit control.

pub use super::icons::{AnimatedIcon, EmbeddedIconAssets, IconAnimation};
pub use super::icons::{GlassIconTile, GlassTileTint};
pub use super::icons::{Icon, IconName, IconRenderMode, IconScale, IconStyle};

/// Canonical alias for [`IconScale`]. SF Symbols documentation uses
/// "SymbolScale" while the icon module uses `IconScale`; both refer to the
/// same three-scale ladder (Small / Medium / Large) relative to the cap
/// height of adjacent text.
///
/// A single type backs both names so `Icon::scale()` and any SF-Symbols
/// call site cannot drift out of sync.
pub type SymbolScale = IconScale;

/// Symbol weight matching SF Pro font weights.
///
/// When a symbol appears alongside text, it should match the text weight.
/// Use `SymbolWeight::from_font_weight()` to convert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SymbolWeight {
    UltraLight,
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    Semibold,
    Bold,
    Heavy,
    Black,
}

impl SymbolWeight {
    /// Convert from GPUI FontWeight to SymbolWeight.
    pub fn from_font_weight(w: gpui::FontWeight) -> Self {
        if w == gpui::FontWeight::THIN {
            Self::UltraLight
        } else if w == gpui::FontWeight::EXTRA_LIGHT {
            Self::Thin
        } else if w == gpui::FontWeight::LIGHT {
            Self::Light
        } else if w == gpui::FontWeight::NORMAL {
            Self::Regular
        } else if w == gpui::FontWeight::MEDIUM {
            Self::Medium
        } else if w == gpui::FontWeight::SEMIBOLD {
            Self::Semibold
        } else if w == gpui::FontWeight::BOLD {
            Self::Bold
        } else if w == gpui::FontWeight::EXTRA_BOLD {
            Self::Heavy
        } else {
            Self::Black
        }
    }
}
