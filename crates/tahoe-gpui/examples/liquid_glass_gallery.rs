//! Example: Liquid Glass design language showcase.
//!
//! Demonstrates Apple's glass morphism effects: translucent surfaces,
//! multi-shadow edge definition, tinted variants, and pill-shaped controls.
//! Uses Apple iOS 26 shadow system (no borders on glass surfaces).

use gpui::prelude::*;
use gpui::{App, Bounds, Div, FontWeight, Window, WindowBounds, WindowOptions, div, px, size};
use gpui_platform::application;
use tahoe_gpui::components::content::badge::{Badge, BadgeVariant};
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::foundations::icons::{Icon, IconName};
use tahoe_gpui::foundations::materials::GlassTintColor;
use tahoe_gpui::foundations::materials::{glass_surface, tinted_glass_surface};
use tahoe_gpui::foundations::theme::{GlassSize, GlassTint, TahoeTheme, TextStyle, TextStyledExt};

struct LiquidGlassGallery;

impl Render for LiquidGlassGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>();

        // Use the glass root_bg (semi-transparent) so the macOS window blur
        // (NSVisualEffectView) shows through, creating true glass depth.
        let root_bg = theme.glass.root_bg;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(root_bg)
            .p(px(32.0))
            .gap(px(32.0))
            .id("glass-gallery-scroll")
            .overflow_y_scroll()
            .text_color(theme.text)
            // Header
            .child(header(theme))
            // Glass cards section - show Small/Medium/Large side-by-side
            .child(
                section("Glass Cards (Small / Medium / Large)", theme).child(
                    div()
                        .flex()
                        .gap(px(12.0))
                        .child(glass_card(
                            "Small",
                            "Tab bars, toolbars, buttons",
                            theme,
                            GlassSize::Small,
                        ))
                        .child(glass_card(
                            "Medium",
                            "Cards, panels, containers",
                            theme,
                            GlassSize::Medium,
                        ))
                        .child(glass_card(
                            "Large",
                            "Modals, sheets, popovers",
                            theme,
                            GlassSize::Large,
                        )),
                ),
            )
            // Buttons section
            .child(
                section("Glass Buttons", theme).child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.0))
                        .child(
                            Button::new("gb1")
                                .label("Primary")
                                .variant(ButtonVariant::Primary),
                        )
                        .child(
                            Button::new("gb2")
                                .label("Ghost Glass")
                                .variant(ButtonVariant::Ghost),
                        )
                        .child(
                            Button::new("gb3")
                                .label("Outline Glass")
                                .variant(ButtonVariant::Outline),
                        )
                        .child(
                            Button::new("gb4")
                                .label("Destructive")
                                .variant(ButtonVariant::Destructive),
                        )
                        .child(
                            Button::new("gb5")
                                .label("Pill")
                                .variant(ButtonVariant::Ghost)
                                .round(true),
                        )
                        .child(
                            Button::new("gb6")
                                .icon(Icon::new(IconName::Copy))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Icon),
                        )
                        .child(
                            Button::new("gb7")
                                .label("Small")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Sm),
                        ),
                ),
            )
            // Badges section
            .child(
                section("Glass Badges", theme).child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(Badge::new("Default"))
                        .child(Badge::new("Success").variant(BadgeVariant::Success))
                        .child(Badge::new("Warning").variant(BadgeVariant::Warning))
                        .child(Badge::new("Error").variant(BadgeVariant::Error))
                        .child(Badge::new("Info").variant(BadgeVariant::Info))
                        .child(Badge::new("Muted").variant(BadgeVariant::Muted)),
                ),
            )
            // Tinted glass section
            .child(
                section("Tinted Glass", theme)
                    .child(div().flex().gap(px(12.0)).children(tinted_cards(theme))),
            )
    }
}

fn header(theme: &TahoeTheme) -> Div {
    let mut el = div()
        .flex()
        .flex_col()
        .items_center()
        .p(px(36.0))
        .gap(px(6.0));

    el = glass_surface(el, theme, GlassSize::Large).rounded(px(28.0));

    el.child(
        div()
            .text_size(px(38.0))
            .font_weight(FontWeight::EXTRA_LIGHT)
            .text_color(theme.text)
            .child("Liquid Glass"),
    )
    .child(
        div()
            .text_size(px(13.0))
            .font_weight(FontWeight::LIGHT)
            .text_color(theme.text_muted)
            .child("Apple-inspired translucent design language"),
    )
}

fn glass_card(title: &str, description: &str, theme: &TahoeTheme, size: GlassSize) -> Div {
    let mut card = div().flex().flex_col().gap(px(8.0)).p(px(16.0)).flex_1();

    card = glass_surface(card, theme, size);

    card.child(
        div()
            .text_style(TextStyle::Body, theme)
            .font_weight(FontWeight::MEDIUM)
            .child(title.to_string()),
    )
    .child(
        div()
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text_muted)
            .child(description.to_string()),
    )
}

fn tinted_cards(theme: &TahoeTheme) -> Vec<Div> {
    let glass = &theme.glass;
    let tints: Vec<(&str, Option<&GlassTint>, gpui::Hsla)> = vec![
        (
            "Green",
            Some(glass.tints.get(GlassTintColor::Green)),
            glass.tints.get(GlassTintColor::Green).bg,
        ),
        (
            "Blue",
            Some(glass.tints.get(GlassTintColor::Blue)),
            glass.tints.get(GlassTintColor::Blue).bg,
        ),
        (
            "Purple",
            Some(glass.tints.get(GlassTintColor::Purple)),
            glass.tints.get(GlassTintColor::Purple).bg,
        ),
        (
            "Amber",
            Some(glass.tints.get(GlassTintColor::Amber)),
            glass.tints.get(GlassTintColor::Amber).bg,
        ),
    ];

    tints
        .into_iter()
        .map(|(label, tint_opt, _bg)| {
            let mut card = div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(6.0))
                .p(px(14.0))
                .flex_1();

            if let Some(tint) = tint_opt {
                card = tinted_glass_surface(card, theme, tint, GlassSize::Small);
            } else {
                card = card.bg(_bg).rounded(theme.radius_lg).shadow_lg();
            }

            card.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text)
                    .child(label.to_string()),
            )
        })
        .collect()
}

fn section(title: &str, theme: &TahoeTheme) -> Div {
    div().flex().flex_col().gap(px(10.0)).child(
        div()
            .text_style(TextStyle::Subheadline, theme)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text_muted)
            .child(title.to_string()),
    )
}

fn main() {
    application().run(|cx: &mut App| {
        let theme = TahoeTheme::liquid_glass();
        let window_bg = theme.glass.window_background;
        cx.set_global(theme);
        cx.bind_keys(tahoe_gpui::all_keybindings());

        let bounds = Bounds::centered(None, size(px(950.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: window_bg,
                ..Default::default()
            },
            |_, cx| cx.new(|_| LiquidGlassGallery),
        )
        .unwrap();
        cx.activate(true);
    });
}
