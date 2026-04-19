//! Example: visual gallery of all primitive components and theme tokens.

use tahoe_gpui::components::content::avatar::Avatar;
use tahoe_gpui::components::content::badge::{Badge, BadgeVariant};
use tahoe_gpui::components::layout_and_organization::disclosure_group::DisclosureGroup;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::status::shimmer::Shimmer;
use tahoe_gpui::foundations::icons::{Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};
use tahoe_gpui::markdown::code_block::CodeBlockView;
use gpui::prelude::*;
use gpui::{
    App, Bounds, Div, FontWeight, Hsla, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, hsla, px, size,
};
use gpui_platform::application;

struct ThemeGallery;

impl Render for ThemeGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .p(px(24.0))
            .gap(px(24.0))
            .id("theme-gallery-scroll")
            .overflow_y_scroll()
            // Title
            .child(
                div()
                    .text_style(TextStyle::Title1, theme)
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("AI Elements - Theme Gallery"),
            )
            // Buttons section
            .child(
                section("Buttons", theme).child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.0))
                        .child(
                            Button::new("b1")
                                .label("Primary")
                                .variant(ButtonVariant::Primary),
                        )
                        .child(
                            Button::new("b2")
                                .label("Ghost")
                                .variant(ButtonVariant::Ghost),
                        )
                        .child(
                            Button::new("b3")
                                .label("Outline")
                                .variant(ButtonVariant::Outline),
                        )
                        .child(
                            Button::new("b4")
                                .label("Destructive")
                                .variant(ButtonVariant::Destructive),
                        )
                        .child(
                            Button::new("b5")
                                .label("Disabled")
                                .variant(ButtonVariant::Primary)
                                .disabled(true),
                        )
                        .child(
                            Button::new("b6")
                                .icon(Icon::new(IconName::Copy))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Icon),
                        )
                        .child(
                            Button::new("b7")
                                .icon(Icon::new(IconName::Send))
                                .variant(ButtonVariant::Primary)
                                .size(ButtonSize::Icon),
                        )
                        .child(
                            Button::new("b8")
                                .label("Small")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Sm),
                        ),
                ),
            )
            // Badges section
            .child(
                section("Badges", theme).child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(Badge::new("Default"))
                        .child(Badge::new("Success").variant(BadgeVariant::Success))
                        .child(Badge::new("Warning").variant(BadgeVariant::Warning))
                        .child(Badge::new("Error").variant(BadgeVariant::Error))
                        .child(Badge::new("Muted").variant(BadgeVariant::Muted)),
                ),
            )
            // Avatars section
            .child(
                section("Avatars", theme).child(
                    div()
                        .flex()
                        .gap(px(12.0))
                        .items_center()
                        .child(Avatar::new("U").bg(hsla(0.58, 0.80, 0.65, 1.0)))
                        .child(Avatar::new("AI").bg(hsla(0.83, 0.60, 0.45, 1.0)))
                        .child(Avatar::new("S").bg(hsla(0.10, 0.80, 0.60, 1.0)))
                        .child(
                            Avatar::new("XL")
                                .bg(hsla(0.35, 0.72, 0.55, 1.0))
                                .size(px(40.0)),
                        ),
                ),
            )
            // Shimmer section
            .child(
                section("Shimmer / Loading", theme).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(Shimmer::new("s1").width(px(300.0)).height(px(16.0)))
                        .child(Shimmer::new("s2").width(px(200.0)).height(px(16.0)))
                        .child(Shimmer::new("s3").width(px(250.0)).height(px(16.0)))
                        .child(
                            Shimmer::new("s4")
                                .label("Thinking...")
                                .width(px(120.0))
                                .height(px(28.0)),
                        ),
                ),
            )
            // DisclosureGroup section
            .child(
                section("DisclosureGroup", theme).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.0))
                        .child(DisclosureGroup::new(
                            "c1",
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child("Click to expand (closed)"),
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text)
                                .child("This content is hidden."),
                        ))
                        .child(
                            DisclosureGroup::new(
                                "c2",
                                div()
                                    .text_style(TextStyle::Subheadline, theme)
                                    .text_color(theme.text_muted)
                                    .child("Expanded section"),
                                div()
                                    .text_style(TextStyle::Subheadline, theme)
                                    .text_color(theme.text)
                                    .child("This is visible because open=true."),
                            )
                            .open(true),
                        ),
                ),
            )
            // Code block section
            .child(
                section("Code Block", theme).child(
                    CodeBlockView::new("fn main() {\n    println!(\"Hello, GPUI!\");\n}")
                        .language(Some("rust".to_string()))
                        .show_line_numbers(true),
                ),
            )
            // Color palette
            .child(
                section("Color Palette", theme).child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.0))
                        .child(color_swatch("Text", theme.text, theme))
                        .child(color_swatch("Muted", theme.text_muted, theme))
                        .child(color_swatch("Accent", theme.accent, theme))
                        .child(color_swatch("Success", theme.success, theme))
                        .child(color_swatch("Warning", theme.warning, theme))
                        .child(color_swatch("Error", theme.error, theme))
                        .child(color_swatch("Surface", theme.surface, theme))
                        .child(color_swatch("Border", theme.border, theme)),
                ),
            )
    }
}

fn section(title: &str, theme: &TahoeTheme) -> Div {
    div().flex().flex_col().gap(px(8.0)).child(
        div()
            .text_style(TextStyle::Subheadline, theme)
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(theme.text_muted)
            .child(title.to_string()),
    )
}

fn color_swatch(label: &str, color: Hsla, theme: &TahoeTheme) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(
            div()
                .size(px(32.0))
                .rounded(theme.radius_md)
                .bg(color)
                .border_1()
                .border_color(theme.border),
        )
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(label.to_string()),
        )
}

fn main() {
    application().run(|cx: &mut App| {
        cx.set_global(TahoeTheme::dark());

        let bounds = Bounds::centered(None, size(px(900.), px(800.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_, cx| cx.new(|_| ThemeGallery),
        )
        .unwrap();
        cx.activate(true);
    });
}
