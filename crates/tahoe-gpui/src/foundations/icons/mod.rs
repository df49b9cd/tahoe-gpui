//! SVG icon component.
//!
//! Provides [`Icon`] for rendering icons as GPU-accelerated SVGs via GPUI,
//! with Unicode symbol fallbacks when no asset source is registered.
//!
//! # Asset Setup
//!
//! To enable SVG rendering, register [`EmbeddedIconAssets`] with your app:
//! ```ignore
//! use tahoe_gpui::foundations::icons::EmbeddedIconAssets;
//! application().with_assets(EmbeddedIconAssets).run(|cx| { ... });
//! ```
//!
//! Without this, icons fall back to Unicode symbol placeholders (the
//! original behavior).
//!
//! # Surface scope
//!
//! Icons default to [`IconStyle::Auto`], which resolves to
//! [`IconStyle::LiquidGlass`] only when an ancestor wraps the subtree in
//! [`crate::foundations::GlassSurfaceScope`] (or the wrap is done for you
//! by a glass-aware component such as [`GlassIconTile`] or `Button` with a
//! glass variant). Under plain `TahoeTheme::liquid_glass()` alone, icons
//! render [`IconStyle::Standard`] — theme mode does not drive vibrancy,
//! **surface does**. See [`crate::foundations::surface_scope`] for the
//! scope mechanism and its known boundaries (deferred draws, sub-windows).

pub mod animated;
pub mod assets;
pub mod glass_tile;
pub mod icon;
pub mod layers;
pub mod names;
pub mod provider_anim;

pub use animated::{AnimatedIcon, IconAnimation};
pub use assets::EmbeddedIconAssets;
pub(crate) use assets::RenderStrategy;
pub use glass_tile::{GlassIconTile, GlassTileTint};
pub(crate) use icon::hierarchical_opacity;
pub use icon::{Icon, IconRenderMode, IconScale, IconStyle};
pub use names::{IconLayoutBehavior, IconName};
pub use provider_anim::AnimatedProviderIcon;
// Back-compat re-export: older call sites import `GlassSurfaceScope` from
// `foundations::icons`. New code should reach for
// `foundations::{GlassSurfaceScope, is_on_glass_surface, …}` — the
// canonical home — because Liquid Glass is a material concern that icons
// merely consume. See `foundations/surface_scope.rs`.
pub use crate::foundations::surface_scope::{GlassSurfaceScope, GlassSurfaceScopeElement};

use gpui::FontWeight;

/// Map a `FontWeight` to an icon stroke width, in points.
///
/// HIG: "Each of the nine symbol weights corresponds to a weight of the
/// San Francisco system font, helping you achieve precise weight matching
/// between symbols and adjacent text." This table maps GPUI's `FontWeight`
/// values onto Lucide-style stroke widths that track the SF Pro weight
/// axis. Default stroke widths without override: Standard = 1.2,
/// LiquidGlass = 1.5.
///
/// The returned value is applied at render time by mutating the SVG asset's
/// `stroke-width` attribute when `Icon::weight()` is explicitly set.
pub fn weight_to_stroke_width(weight: FontWeight) -> f32 {
    if weight == FontWeight::THIN {
        0.8
    } else if weight == FontWeight::EXTRA_LIGHT || weight == FontWeight::LIGHT {
        1.0
    } else if weight == FontWeight::NORMAL {
        1.2
    } else if weight == FontWeight::MEDIUM {
        1.4
    } else if weight == FontWeight::SEMIBOLD {
        1.5
    } else {
        1.8 // BOLD and above
    }
}

#[cfg(test)]
mod tests;
