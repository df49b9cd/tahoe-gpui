//! Materials demo — shows Liquid Glass surface sizes and material thickness levels.
//!
//! GPUI renders glass surfaces as semi-transparent fills that composite against
//! the macOS window blur (WindowBackgroundAppearance::Blurred). The translucency
//! is visible when looking through the glass to the desktop wallpaper — NOT to
//! sibling elements within the same window. This demo places glass samples
//! directly on the window's blurred root background so the effect is visible.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::foundations::materials::{
    Elevation, Glass, MaterialThickness, Shape, glass_effect,
};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let label = |text: &'static str| {
        div()
            .text_size(px(11.0))
            .text_color(theme.text_muted)
            .pt(px(6.0))
            .child(text)
    };

    // Glass samples sit directly on the window root (no intermediate bg)
    // so the macOS window blur shows through the semi-transparent fills.
    let glass_elevation_card = |elevation: Elevation, name: &'static str| {
        div()
            .flex()
            .flex_col()
            .items_center()
            .child(
                glass_effect(
                    div()
                        .w(px(100.0))
                        .h(px(70.0))
                        .rounded(px(14.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(theme.text_muted)
                                .child(name),
                        ),
                    theme,
                    Glass::Regular,
                    Shape::Default,
                    elevation,
                )
                .into_any_element(),
            )
            .child(label(name))
    };

    let thickness_card = |thickness: MaterialThickness, name: &'static str, theme: &TahoeTheme| {
        let pct = format!("{:.0}%", theme.glass.material_bg(thickness).a * 100.0);
        div()
            .flex()
            .flex_col()
            .items_center()
            .child(
                gpui::div()
                    .w(px(100.0))
                    .h(px(70.0))
                    .rounded(px(14.0))
                    .bg(theme.glass.material_bg(thickness))
                    .shadow(Elevation::Elevated.shadows(theme).to_vec())
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.text)
                            .child(pct),
                    )
                    .into_any_element(),
            )
            .child(label(name))
    };

    // Use transparent root so the window blur is visible behind glass surfaces.
    // The gallery's main pane has theme.background as bg — we override it here
    // with a very low opacity so the window blur shows through.
    let root_bg = theme.glass.root_bg;

    div()
        .id("materials-pane")
        .bg(root_bg)
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        // Title
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Materials"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Liquid Glass surfaces are semi-transparent — they let the macOS \
                     desktop wallpaper show through via the window's blur effect. \
                     Move this window over a colorful wallpaper to see the translucency.",
                ),
        )
        // ── Surface elevations ────────────────────────────────────
        .child(
            div()
                .pt(theme.spacing_md)
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Liquid Glass surface elevations"),
        )
        .child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child(
                    "Resting for toolbars and buttons. Elevated for sidebars and cards. \
                     Floating for sheets and modals. Each has a different shadow intensity.",
                ),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_lg)
                .child(glass_elevation_card(Elevation::Resting, "Resting"))
                .child(glass_elevation_card(Elevation::Elevated, "Elevated"))
                .child(glass_elevation_card(Elevation::Floating, "Floating")),
        )
        // ── Material thickness ───────────────────────────────────
        .child(
            div()
                .pt(theme.spacing_lg)
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Material thickness levels"),
        )
        .child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .text_color(theme.text_muted)
                .child(
                    "Per HIG: thicker materials are more opaque. Ultra Thin \
                     (10–36% dark–light) shows most of the wallpaper through, \
                     while Ultra Thick (50–84%) is nearly solid. The effect is \
                     best seen with a colorful desktop wallpaper behind the window.",
                ),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(thickness_card(
                    MaterialThickness::UltraThin,
                    "Ultra Thin",
                    theme,
                ))
                .child(thickness_card(MaterialThickness::Thin, "Thin", theme))
                .child(thickness_card(MaterialThickness::Regular, "Regular", theme))
                .child(thickness_card(MaterialThickness::Thick, "Thick", theme))
                .child(thickness_card(
                    MaterialThickness::UltraThick,
                    "Ultra Thick",
                    theme,
                )),
        )
        // ── How it works ─────────────────────────────────────────
        .child(
            div()
                .pt(theme.spacing_lg)
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("How Liquid Glass works in GPUI"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "GPUI uses macOS WindowBackgroundAppearance::Blurred to enable \
                     window-level blur. Glass surfaces apply semi-transparent fills \
                     (via bg()) that let the system compositor blend the desktop \
                     wallpaper through. The specular edge effect comes from \
                     multi-layer box shadows, not CSS borders. This approach matches \
                     Apple's Liquid Glass design: translucent surfaces that respond to \
                     their environment rather than simulating blur inline.",
                ),
        )
        .into_any_element()
}
