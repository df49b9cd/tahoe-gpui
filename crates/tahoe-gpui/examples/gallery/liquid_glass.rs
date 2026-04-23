//! Liquid Glass demo for the primitive gallery (issue #156 F-08).
//!
//! macOS 26 Tahoe's Liquid Glass primitives — `tinted_glass_surface`,
//! `accent_tinted_glass_surface`, `GlassIconTile`, and the per-tint
//! palette — are exercised here as the headline material story for the
//! release. Surface sizes and material thickness levels still live on
//! the `Materials` page; this page focuses on the tinted variants and
//! the `GlassIconTile` shape that the standalone `liquid_glass_gallery`
//! binary previously owned.

use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, Window, div, px};

use tahoe_gpui::foundations::icons::{GlassIconTile, GlassTileTint, IconName};
use tahoe_gpui::foundations::materials::{Elevation, Glass, GlassTintColor, Shape, glass_effect};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

const TINTS: &[(GlassTintColor, &str)] = &[
    (GlassTintColor::Green, "Green"),
    (GlassTintColor::Blue, "Blue"),
    (GlassTintColor::Purple, "Purple"),
    (GlassTintColor::Amber, "Amber"),
    (GlassTintColor::Red, "Red"),
    (GlassTintColor::Cyan, "Cyan"),
    (GlassTintColor::Teal, "Teal"),
    (GlassTintColor::Indigo, "Indigo"),
];

const ICON_TILES: &[(IconName, GlassTileTint, &str)] = &[
    (IconName::Check, GlassTileTint::Green, "Success"),
    (IconName::Sparkle, GlassTileTint::Purple, "AI"),
    (IconName::Folder, GlassTileTint::Blue, "Files"),
    (IconName::AlertTriangle, GlassTileTint::Amber, "Alerts"),
];

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let tint_card = |color: GlassTintColor, name: &'static str| {
        let tint = theme.glass.tints.get(color);
        glass_effect(
            div()
                .w(px(110.0))
                .h(px(72.0))
                .flex()
                .items_center()
                .justify_center(),
            theme,
            Glass::Regular.tint(Some(tint.bg)),
            Shape::Default,
            Elevation::Resting,
        )
        .child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text)
                .child(name),
        )
    };

    div()
        .id("liquid-glass-pane")
        .bg(theme.glass.root_bg)
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Liquid Glass"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "macOS 26 Tahoe ships Liquid Glass as the system material. \
                     This page exercises the tinted variants and `GlassIconTile`. \
                     Translucency reads best with a colourful desktop wallpaper.",
                ),
        )
        // ── Tinted glass palette ──────────────────────────────────────
        .child(
            div()
                .pt(theme.spacing_md)
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Tinted glass palette"),
        )
        .child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child("Green, Blue, Purple, Amber, Red, Cyan, Teal, Indigo."),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(theme.spacing_md)
                .children(TINTS.iter().map(|(c, n)| tint_card(*c, n))),
        )
        // ── Accent-tinted glass ──────────────────────────────────────
        .child(
            div()
                .pt(theme.spacing_lg)
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Accent-tinted glass"),
        )
        .child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child(
                    "`accent_tinted_glass_surface` pulls from the active accent \
                     so toolbars and primary action surfaces match the system tint.",
                ),
        )
        .child(
            glass_effect(
                div()
                    .w(px(360.0))
                    .h(px(96.0))
                    .flex()
                    .items_center()
                    .justify_center(),
                theme,
                Glass::Regular.tint(Some(theme.glass.accent_tint.bg)),
                Shape::Default,
                Elevation::Resting,
            )
            .child(
                div()
                    .text_style(TextStyle::Headline, theme)
                    .text_color(theme.text)
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Accent surface"),
            ),
        )
        // ── Glass icon tiles ──────────────────────────────────────────
        .child(
            div()
                .pt(theme.spacing_lg)
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Glass icon tiles"),
        )
        .child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child(
                    "`GlassIconTile` mirrors Apple's Liquid Glass app-icon template \
                     with a 22.5% corner radius, category-tinted fill, and \
                     `IconStyle::LiquidGlass` rendering.",
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(theme.spacing_md)
                .children(ICON_TILES.iter().map(|(icon, tint, label)| {
                    GlassIconTile::new(*icon)
                        .tint(*tint)
                        .icon_size(px(36.0))
                        .label(*label)
                })),
        )
        .into_any_element()
}
