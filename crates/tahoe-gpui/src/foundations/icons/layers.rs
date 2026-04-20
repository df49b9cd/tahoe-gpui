//! Multi-color icon layer rendering.
//!
//! Renders multi-color icons by stacking multiple single-color SVG layers,
//! each tinted with a semantic theme color.

use gpui::prelude::*;
use gpui::{App, Hsla, Pixels, Transformation, Window, div, size as gpui_size, svg};

use super::assets::IconColorRole;
use super::icon::IconRenderMode;
use crate::foundations::theme::{ActiveTheme, TahoeTheme};

/// Layer transformation applied to individual SVG layers. Returns a
/// horizontal mirror around `bounds.center()` (see `gpui::elements::svg`)
/// when `flip_horizontal` is true — stacking multiple flipped layers
/// produces the same visual as flipping the stack as a unit. When false,
/// returns the identity transform so the builder pattern stays uniform
/// across both branches.
fn layer_transform(flip_horizontal: bool) -> Transformation {
    if flip_horizontal {
        Transformation::scale(gpui_size(-1.0, 1.0))
    } else {
        Transformation::default()
    }
}

/// Render a single-layer (monochrome path) icon. Supports the single-layer
/// render modes: Monochrome (plain), VariableColor (opacity-driven by
/// progress), and Gradient (linear gradient applied via two overlaid SVGs).
///
/// Hierarchical / Palette / MultiColor need a multi-layer source and fall
/// back to plain Monochrome when applied to a single-layer icon — that
/// matches SF Symbols' behavior where asking for a palette render on a
/// monochrome symbol renders it monochrome with the first palette color.
///
/// `flip_horizontal` mirrors the glyph across the vertical axis — used to
/// honour [`Icon::follow_layout_direction`] under RTL themes for
/// directionally-classified symbols (chevrons, arrows, `Send`).
pub(super) fn render_monochrome(
    path: &'static str,
    size: Pixels,
    color: Hsla,
    mode: IconRenderMode,
    flip_horizontal: bool,
) -> impl IntoElement {
    let resolved_color = match mode {
        IconRenderMode::VariableColor { progress } => {
            let p = progress.clamp(0.0, 1.0);
            // Variable color ramps opacity from 0.35 (dim) to 1.0 (full).
            // The 0.35 floor matches Apple's documented low-end opacity
            // for variable-color symbols; completely invisible layers
            // would make the symbol unreadable.
            Hsla {
                a: color.a * (0.35 + 0.65 * p),
                ..color
            }
        }
        IconRenderMode::Gradient { source } => source.unwrap_or(color),
        IconRenderMode::Palette { palette } if !palette.is_empty() => palette[0],
        _ => color,
    };
    div().size(size).child(
        svg()
            .path(path)
            .size(size)
            .text_color(resolved_color)
            .with_transformation(layer_transform(flip_horizontal)),
    )
}

/// Resolve an [`IconColorRole`] to a concrete color from the theme.
///
/// If `caller_color` is provided, it overrides the `Muted` role
/// (matching how `Icon::new().color()` works for the default layer).
pub(super) fn resolve_role_color(
    role: IconColorRole,
    caller_color: Option<Hsla>,
    theme: &TahoeTheme,
) -> Hsla {
    match role {
        IconColorRole::Muted => caller_color.unwrap_or(theme.text_muted),
        IconColorRole::Success => theme.success,
        IconColorRole::Info => theme.info,
        IconColorRole::Warning => theme.warning,
        IconColorRole::Error => theme.error,
        IconColorRole::Ai => theme.ai,
    }
}

/// Resolve a color role to Liquid Glass bright pastel colors.
pub(super) fn resolve_role_color_glass(
    role: IconColorRole,
    caller_color: Option<Hsla>,
    theme: &TahoeTheme,
) -> Hsla {
    match role {
        IconColorRole::Muted => caller_color.unwrap_or(theme.glass.icon_text),
        IconColorRole::Success => theme.glass.icon_success,
        IconColorRole::Info => theme.glass.icon_info,
        IconColorRole::Warning => theme.glass.icon_warning,
        IconColorRole::Error => theme.glass.icon_error,
        IconColorRole::Ai => theme.glass.icon_ai,
    }
}

/// Render a multi-color icon as stacked SVG layers with Liquid Glass colors.
pub(super) fn render_multi_color_layers_glass(
    layers: &'static [(&'static str, IconColorRole)],
    size: Pixels,
    caller_color: Option<Hsla>,
    flip_horizontal: bool,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let theme = cx.theme();

    let mut container = div().relative().size(size);

    for &(path, role) in layers {
        let color = resolve_role_color_glass(role, caller_color, theme);
        container = container.child(
            svg()
                .path(path)
                .size(size)
                .text_color(color)
                .absolute()
                .top_0()
                .left_0()
                .with_transformation(layer_transform(flip_horizontal)),
        );
    }

    container
}

/// Render a multi-color icon as stacked SVG layers.
///
/// Each layer is an absolutely-positioned `svg()` element within a
/// relatively-positioned container, tinted with its semantic color.
pub(super) fn render_multi_color_layers(
    layers: &'static [(&'static str, IconColorRole)],
    size: Pixels,
    caller_color: Option<Hsla>,
    flip_horizontal: bool,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let theme = cx.theme();

    let mut container = div().relative().size(size);

    for &(path, role) in layers {
        let color = resolve_role_color(role, caller_color, theme);
        container = container.child(
            svg()
                .path(path)
                .size(size)
                .text_color(color)
                .absolute()
                .top_0()
                .left_0()
                .with_transformation(layer_transform(flip_horizontal)),
        );
    }

    container
}

/// Render a multi-color icon with caller-supplied palette colors, one per
/// layer. If `palette` is shorter than the layer count, later layers
/// repeat the last palette entry.
///
/// Maps to `IconRenderMode::Palette`. Bypasses the semantic `IconColorRole`
/// mapping entirely — the palette is the authoritative color source.
pub(super) fn render_multi_color_layers_palette(
    layers: &'static [(&'static str, IconColorRole)],
    size: Pixels,
    palette: &'static [Hsla],
    flip_horizontal: bool,
    _window: &mut Window,
    _cx: &mut App,
) -> impl IntoElement {
    let mut container = div().relative().size(size);

    for (i, &(path, _role)) in layers.iter().enumerate() {
        let color = if palette.is_empty() {
            // Degenerate empty palette — fall back to transparent so the
            // caller sees nothing rather than rendering with a random
            // theme color, matching SF Symbols' "empty palette = no
            // render" behaviour.
            Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.0,
                a: 0.0,
            }
        } else {
            palette[i.min(palette.len() - 1)]
        };
        container = container.child(
            svg()
                .path(path)
                .size(size)
                .text_color(color)
                .absolute()
                .top_0()
                .left_0()
                .with_transformation(layer_transform(flip_horizontal)),
        );
    }

    container
}

/// Render a multi-color icon with Variable Color semantics: layers ramp
/// from dim to full as `progress` (0.0 ..= 1.0) increases. Layers fill in
/// order (layer 0 fills at progress=1/N, layer 1 at 2/N, etc.), matching
/// SF Symbols' documented variable-color behavior.
pub(super) fn render_multi_color_layers_variable(
    layers: &'static [(&'static str, IconColorRole)],
    size: Pixels,
    caller_color: Option<Hsla>,
    progress: f32,
    is_glass: bool,
    flip_horizontal: bool,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let theme = cx.theme();
    let p = progress.clamp(0.0, 1.0);
    let n = layers.len().max(1);

    let mut container = div().relative().size(size);

    for (i, &(path, role)) in layers.iter().enumerate() {
        let base = if is_glass {
            resolve_role_color_glass(role, caller_color, theme)
        } else {
            resolve_role_color(role, caller_color, theme)
        };
        // Each layer has a threshold at `(i+1)/N`. Below it, the layer is
        // dimmed to the 0.35 floor. At or above, it's fully lit.
        let threshold = (i + 1) as f32 / n as f32;
        let alpha_mul = if p + 1.0 / (2.0 * n as f32) >= threshold {
            1.0
        } else {
            0.35
        };
        let color = Hsla {
            a: base.a * alpha_mul,
            ..base
        };
        container = container.child(
            svg()
                .path(path)
                .size(size)
                .text_color(color)
                .absolute()
                .top_0()
                .left_0()
                .with_transformation(layer_transform(flip_horizontal)),
        );
    }

    container
}

/// Render a multi-color icon as a two-stop linear gradient overlay. GPUI
/// exposes no gradient fill for `svg()`, so we approximate by rendering
/// the same layer stack twice: once with the source color and once with
/// the darker stop at 50 % opacity on top. The result is not a true
/// fragment-shader gradient but produces the perceptual top-to-bottom
/// color shift SF Symbols 7 ships.
pub(super) fn render_multi_color_layers_gradient(
    layers: &'static [(&'static str, IconColorRole)],
    size: Pixels,
    source: Option<Hsla>,
    fallback: Hsla,
    is_glass: bool,
    flip_horizontal: bool,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let theme = cx.theme();
    let src = source.unwrap_or(fallback);
    // Derive a darker stop by halving lightness.
    let stop = Hsla {
        l: (src.l * 0.55).max(0.0),
        a: src.a,
        ..src
    };

    let mut container = div().relative().size(size);

    // Primary pass: source color on the semantic layer stack (the primary
    // layer gets `src`; remaining layers stay on their semantic role so
    // multi-color icons keep their identity).
    for (i, &(path, role)) in layers.iter().enumerate() {
        let color = if i == 0 {
            src
        } else if is_glass {
            resolve_role_color_glass(role, Some(src), theme)
        } else {
            resolve_role_color(role, Some(src), theme)
        };
        container = container.child(
            svg()
                .path(path)
                .size(size)
                .text_color(color)
                .absolute()
                .top_0()
                .left_0()
                .with_transformation(layer_transform(flip_horizontal)),
        );
    }

    // Gradient stop overlay: same shape, darker stop at reduced opacity.
    if let Some(&(primary_path, _)) = layers.first() {
        container = container.child(
            svg()
                .path(primary_path)
                .size(size)
                .text_color(Hsla {
                    a: stop.a * 0.5,
                    ..stop
                })
                .absolute()
                .top_0()
                .left_0()
                .with_transformation(layer_transform(flip_horizontal)),
        );
    }

    container
}

/// Render a multi-color icon with hierarchical opacity.
///
/// Primary layer (index 0) = full opacity, secondary (1) = 0.50, tertiary (2+) = 0.25.
/// Uses glass or standard colors depending on `is_glass`.
pub(super) fn render_multi_color_layers_hierarchical(
    layers: &'static [(&'static str, IconColorRole)],
    size: Pixels,
    caller_color: Option<Hsla>,
    is_glass: bool,
    flip_horizontal: bool,
    _window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let theme = cx.theme();

    let mut container = div().relative().size(size);

    for (i, &(path, role)) in layers.iter().enumerate() {
        let mut color = if is_glass {
            resolve_role_color_glass(role, caller_color, theme)
        } else {
            resolve_role_color(role, caller_color, theme)
        };
        // Apply hierarchical opacity
        color.a *= super::hierarchical_opacity(i);
        container = container.child(
            svg()
                .path(path)
                .size(size)
                .text_color(color)
                .absolute()
                .top_0()
                .left_0()
                .with_transformation(layer_transform(flip_horizontal)),
        );
    }

    container
}

#[cfg(test)]
mod tests {
    use super::super::assets::IconColorRole;
    use super::{resolve_role_color, resolve_role_color_glass};
    use crate::foundations::theme::TahoeTheme;
    use core::prelude::v1::test;
    use gpui::hsla;

    #[test]
    fn muted_uses_caller_color_when_provided() {
        let theme = TahoeTheme::dark();
        let custom = hsla(0.5, 0.5, 0.5, 1.0);
        let result = resolve_role_color(IconColorRole::Muted, Some(custom), &theme);
        assert_eq!(result, custom);
    }

    #[test]
    fn muted_falls_back_to_theme_text_muted() {
        let theme = TahoeTheme::dark();
        let result = resolve_role_color(IconColorRole::Muted, None, &theme);
        assert_eq!(result, theme.text_muted);
    }

    #[test]
    fn role_colors_map_to_theme_fields() {
        let theme = TahoeTheme::dark();
        assert_eq!(
            resolve_role_color(IconColorRole::Success, None, &theme),
            theme.success
        );
        assert_eq!(
            resolve_role_color(IconColorRole::Info, None, &theme),
            theme.info
        );
        assert_eq!(
            resolve_role_color(IconColorRole::Warning, None, &theme),
            theme.warning
        );
        assert_eq!(
            resolve_role_color(IconColorRole::Error, None, &theme),
            theme.error
        );
        assert_eq!(
            resolve_role_color(IconColorRole::Ai, None, &theme),
            theme.ai
        );
    }

    #[test]
    fn glass_muted_uses_caller_color_when_provided() {
        let theme = TahoeTheme::dark();
        let custom = hsla(0.5, 0.5, 0.5, 1.0);
        let result = resolve_role_color_glass(IconColorRole::Muted, Some(custom), &theme);
        assert_eq!(result, custom);
    }

    #[test]
    fn glass_muted_falls_back_to_theme_glass_text() {
        let theme = TahoeTheme::dark();
        let result = resolve_role_color_glass(IconColorRole::Muted, None, &theme);
        assert_eq!(result, theme.glass.icon_text);
    }

    #[test]
    fn glass_role_colors_map_to_glass_theme_fields() {
        let theme = TahoeTheme::dark();
        assert_eq!(
            resolve_role_color_glass(IconColorRole::Success, None, &theme),
            theme.glass.icon_success
        );
        assert_eq!(
            resolve_role_color_glass(IconColorRole::Info, None, &theme),
            theme.glass.icon_info
        );
        assert_eq!(
            resolve_role_color_glass(IconColorRole::Warning, None, &theme),
            theme.glass.icon_warning
        );
        assert_eq!(
            resolve_role_color_glass(IconColorRole::Error, None, &theme),
            theme.glass.icon_error
        );
        assert_eq!(
            resolve_role_color_glass(IconColorRole::Ai, None, &theme),
            theme.glass.icon_ai
        );
    }
}
