//! Visual gallery of every `ButtonVariant` in every state.
//!
//! Renders the canonical macOS 26 (Tahoe) "Push Buttons" reference grid:
//! one row per variant, showing idle / hover-target / disabled / round /
//! icon-only states. Use this to diff the implementation 1:1 against the
//! Figma Buttons page in the macOS 26 (Community) UI Kit.

use gpui::prelude::*;
use gpui::{
    App, Bounds, FontWeight, Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, div,
    px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::foundations::icons::{EmbeddedIconAssets, Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

const VARIANTS: &[(ButtonVariant, &str)] = &[
    (ButtonVariant::Outline, "Outline (Default)"),
    (ButtonVariant::Secondary, "Secondary"),
    (ButtonVariant::Primary, "Primary (Colored)"),
    (ButtonVariant::Filled, "Filled (Tahoe CTA)"),
    (ButtonVariant::Destructive, "Destructive"),
    (ButtonVariant::Ghost, "Ghost (Borderless)"),
];

struct ButtonGallery;

impl ButtonGallery {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ButtonGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let theme = &theme;

        let mut grid = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_lg)
            .p(theme.spacing_xl);

        // Header
        grid = grid
            .child(
                div()
                    .text_style_emphasized(TextStyle::LargeTitle, theme)
                    .text_color(theme.text)
                    .child("Buttons"),
            )
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child("A button initiates an instantaneous action."),
            );

        // Column header row
        grid = grid.child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .pb(theme.spacing_xs)
                .border_b_1()
                .border_color(theme.border)
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .font_weight(FontWeight::MEDIUM)
                .child(div().w(px(180.0)).child("Variant"))
                .child(div().w(px(120.0)).child("Idle"))
                .child(div().w(px(120.0)).child("Disabled"))
                .child(div().w(px(120.0)).child("Round"))
                .child(div().w(px(80.0)).child("Icon"))
                .child(div().w(px(80.0)).child("Icon Sm")),
        );

        // One row per variant
        for (variant, label) in VARIANTS {
            let v = *variant;
            let id_root = format!(
                "btn-{}",
                match v {
                    ButtonVariant::Primary => "primary",
                    ButtonVariant::Secondary => "secondary",
                    ButtonVariant::Outline => "outline",
                    ButtonVariant::Ghost => "ghost",
                    ButtonVariant::Destructive => "destructive",
                    ButtonVariant::Glass => "glass",
                    ButtonVariant::GlassProminent => "glass-prom",
                    ButtonVariant::Filled => "filled",
                    ButtonVariant::Help => "help",
                    ButtonVariant::Disclosure => "disclosure",
                    ButtonVariant::Gradient => "gradient",
                    ButtonVariant::Link => "link",
                    // `ButtonVariant` is `#[non_exhaustive]`; any future
                    // variant lands here until the gallery adds an entry.
                    _ => "unknown",
                }
            );

            grid = grid.child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_md)
                    .py(theme.spacing_sm)
                    // Label column
                    .child(
                        div()
                            .w(px(180.0))
                            .text_style(TextStyle::Body, theme)
                            .text_color(theme.text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(*label),
                    )
                    // Idle
                    .child(
                        div().w(px(120.0)).child(
                            Button::new(format!("{id_root}-idle"))
                                .label("Label")
                                .variant(v)
                                .size(ButtonSize::Regular),
                        ),
                    )
                    // Disabled
                    .child(
                        div().w(px(120.0)).child(
                            Button::new(format!("{id_root}-disabled"))
                                .label("Label")
                                .variant(v)
                                .size(ButtonSize::Regular)
                                .disabled(true),
                        ),
                    )
                    // Round (capsule)
                    .child(
                        div().w(px(120.0)).child(
                            Button::new(format!("{id_root}-round"))
                                .label("Label")
                                .variant(v)
                                .size(ButtonSize::Regular)
                                .round(true),
                        ),
                    )
                    // Icon-only (md)
                    .child(
                        div().w(px(80.0)).child(
                            Button::new(format!("{id_root}-icon"))
                                .icon(Icon::new(IconName::Sparkle).size(px(16.0)))
                                .variant(v)
                                .size(ButtonSize::Icon),
                        ),
                    )
                    // Icon-only (sm)
                    .child(
                        div().w(px(80.0)).child(
                            Button::new(format!("{id_root}-icon-sm"))
                                .icon(Icon::new(IconName::Sparkle).size(px(14.0)))
                                .variant(v)
                                .size(ButtonSize::IconSmall),
                        ),
                    ),
            );
        }

        // Composition example: matches the HIG button pair pattern
        grid = grid
            .child(
                div()
                    .pt(theme.spacing_xl)
                    .text_style_emphasized(TextStyle::Title3, theme)
                    .text_color(theme.text)
                    .child("Action pair"),
            )
            .child(
                div()
                    .flex()
                    .gap(theme.spacing_sm)
                    .child(
                        Button::new("pair-cancel")
                            .label("Cancel")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Regular),
                    )
                    .child(
                        Button::new("pair-save")
                            .label("Save")
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Regular),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(theme.spacing_sm)
                    .child(
                        Button::new("pair-cancel-2")
                            .label("Cancel")
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Regular),
                    )
                    .child(
                        Button::new("pair-delete")
                            .label("Delete")
                            .variant(ButtonVariant::Destructive)
                            .size(ButtonSize::Regular),
                    ),
            );

        div()
            .id("button-gallery-scroll")
            .size_full()
            .bg(theme.background)
            .overflow_y_scroll()
            .child(grid)
    }
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            let theme = TahoeTheme::liquid_glass_light();
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            let bounds = Bounds::centered(None, size(px(900.0), px(800.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| cx.new(ButtonGallery::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
