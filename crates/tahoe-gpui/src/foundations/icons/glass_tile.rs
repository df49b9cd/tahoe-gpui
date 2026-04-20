//! Frosted glass icon tile component.
//!
//! Renders an icon inside a translucent glass-effect container inspired by
//! Apple's Liquid Glass design language. Supports optional category tinting.

use crate::foundations::layout::SPACING_4;
use gpui::prelude::*;
use gpui::{App, Hsla, Pixels, SharedString, Window, div, px};

use super::{Icon, IconName};
use crate::foundations::surface_scope::GlassSurfaceScope;
use crate::foundations::theme::{ActiveTheme, TextStyle};

/// Proportional corner-radius factor for Liquid Glass icon tiles.
///
/// HIG macOS 26 Tahoe's Liquid Glass icon template defines the
/// Foreground layer with a corner radius of ~22.5% of the tile side. A
/// single constant keeps the proportion consistent across arbitrary tile
/// sizes instead of the previous hard-coded 20 pt which was correct only
/// at ~89 pt tiles and wrong everywhere else (see issue #139 finding #13).
///
/// Mirrors [`crate::foundations::app_icon::TILE_CORNER_RADIUS_RATIO`] so
/// consuming apps get the same corner radius whether they use the
/// runtime [`GlassIconTile`] or build app-icon assets via
/// [`crate::foundations::app_icon`].
const CORNER_RADIUS_FACTOR: f32 = crate::foundations::app_icon::TILE_CORNER_RADIUS_RATIO;

/// Compute the HIG-correct corner radius for a glass tile of the given size.
pub(crate) fn glass_tile_corner_radius(icon_size: Pixels) -> Pixels {
    // The tile's outer size is the icon size plus vertical padding (14 + 10)
    // and a bit of horizontal padding (4 + 4) — conceptually the tile side
    // is `icon_size + ~24`. Since the 22.5% ratio is defined against the
    // visible tile side, we apply it to that composed size.
    let side = f32::from(icon_size) + 24.0;
    px(side * CORNER_RADIUS_FACTOR)
}

/// Category tint for glass tiles, matching the Liquid Glass design reference.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlassTileTint {
    /// Green tint (Git icons, success states).
    Green,
    /// Blue tint (Dev Tools, info states).
    Blue,
    /// Purple tint (AI/Agents, LLM Providers).
    Purple,
    /// Amber tint (Languages, warning states).
    Amber,
}

/// A frosted glass container for Liquid Glass style icons.
///
/// Renders the icon at the specified size with a translucent background,
/// subtle border, and optional category tinting.
///
/// # Example
/// ```ignore
/// GlassIconTile::new(IconName::Check)
///     .tint(GlassTileTint::Green)
///     .label("Check")
/// ```
#[derive(IntoElement)]
pub struct GlassIconTile {
    name: IconName,
    icon_size: Pixels,
    tint: Option<GlassTileTint>,
    label: Option<SharedString>,
}

impl GlassIconTile {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            icon_size: px(24.0),
            tint: None,
            label: None,
        }
    }

    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = size;
        self
    }

    pub fn tint(mut self, tint: GlassTileTint) -> Self {
        self.tint = Some(tint);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for GlassIconTile {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let glass = &theme.glass;
        let (bg, border) = match self.tint {
            Some(GlassTileTint::Green) => (
                tint_bg(glass.icon_success, 0.06),
                tint_bg(glass.icon_success, 0.10),
            ),
            Some(GlassTileTint::Blue) => (
                tint_bg(glass.icon_info, 0.06),
                tint_bg(glass.icon_info, 0.10),
            ),
            Some(GlassTileTint::Purple) => {
                (tint_bg(glass.icon_ai, 0.06), tint_bg(glass.icon_ai, 0.10))
            }
            Some(GlassTileTint::Amber) => (
                tint_bg(glass.icon_warning, 0.05),
                tint_bg(glass.icon_warning, 0.08),
            ),
            None => (glass.tile_bg, glass.tile_border),
        };

        let mut tile = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.0))
            .pt(px(14.0))
            .pb(px(10.0))
            .pl(px(SPACING_4))
            .pr(px(SPACING_4))
            .rounded(glass_tile_corner_radius(self.icon_size))
            .bg(bg)
            .border_1()
            .border_color(border)
            .child(Icon::new(self.name).size(self.icon_size));

        if let Some(label) = self.label {
            // Use the HIG Caption1 size (10 pt) — the macOS minimum legible
            // size. The previous 8.5 pt value was below Apple's 10 pt floor.
            //
            // Under Reduce Transparency the glass tokens are tuned for a
            // translucent surface; the surface falls back to an opaque
            // fill, so derive the label color from `theme.text_muted`
            // instead to keep contrast predictable.
            let label_base = if theme.accessibility_mode.reduce_transparency() {
                theme.text_muted
            } else {
                theme.glass.icon_text
            };
            tile = tile.child(
                div()
                    .text_size(TextStyle::Caption1.attrs().size)
                    .text_color(Hsla {
                        a: 0.42,
                        ..label_base
                    })
                    .child(label),
            );
        }

        // Wrap in a glass surface scope so the child Icon (and any nested
        // badges a caller might add in future extensions) resolve their
        // default IconStyle to the glass vibrancy approximation.
        GlassSurfaceScope::new(tile)
    }
}

fn tint_bg(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

#[cfg(test)]
mod tests {
    use super::{CORNER_RADIUS_FACTOR, glass_tile_corner_radius};
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn corner_radius_scales_proportionally() {
        // At 24pt icon, tile side ≈ 48pt → radius ≈ 10.8pt
        let r_small = glass_tile_corner_radius(px(24.0));
        let expected_small = (24.0 + 24.0) * CORNER_RADIUS_FACTOR;
        assert!((f32::from(r_small) - expected_small).abs() < 0.01);

        // At 96pt icon, tile side ≈ 120pt → radius ≈ 27pt
        let r_large = glass_tile_corner_radius(px(96.0));
        let expected_large = (96.0 + 24.0) * CORNER_RADIUS_FACTOR;
        assert!((f32::from(r_large) - expected_large).abs() < 0.01);

        // Strict monotonic growth — a larger icon must produce a larger
        // radius, or the proportional contract is violated.
        assert!(f32::from(r_large) > f32::from(r_small));
    }

    #[test]
    fn corner_radius_ratio_matches_hig_spec() {
        assert!((CORNER_RADIUS_FACTOR - 0.225).abs() < f32::EPSILON);
    }
}
