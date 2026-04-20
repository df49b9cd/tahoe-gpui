//! Buttons demo for the primitive gallery — mirrors the macOS 26
//! "Buttons" page from the Apple Tahoe UI Kit.

use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::foundations::icons::{Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

const VARIANTS: &[(ButtonVariant, &str)] = &[
    (ButtonVariant::Outline, "Outline (Default)"),
    (ButtonVariant::Secondary, "Secondary"),
    (ButtonVariant::Primary, "Primary (Colored)"),
    (ButtonVariant::Filled, "Filled (Tahoe CTA)"),
    (ButtonVariant::Destructive, "Destructive"),
    (ButtonVariant::Ghost, "Ghost (Borderless)"),
];

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    let mut grid = div()
        .flex()
        .flex_col()
        .gap(theme.spacing_lg)
        .p(theme.spacing_xl);

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
            .child(div().w(px(110.0)).child("Idle"))
            .child(div().w(px(110.0)).child("Pressed"))
            .child(div().w(px(110.0)).child("Disabled"))
            .child(div().w(px(110.0)).child("Round"))
            .child(div().w(px(120.0)).child("Lg"))
            .child(div().w(px(70.0)).child("Icon"))
            .child(div().w(px(70.0)).child("Icon Sm")),
    );

    for (variant, label) in VARIANTS {
        let v = *variant;
        let id_root = format!("btn-{label}").replace([' ', '(', ')', '/'], "-");
        grid = grid.child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .py(theme.spacing_sm)
                .child(
                    div()
                        .w(px(180.0))
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .font_weight(FontWeight::MEDIUM)
                        .child(*label),
                )
                .child(
                    div().w(px(110.0)).child(
                        Button::new(format!("{id_root}-idle"))
                            .label("Label")
                            .variant(v)
                            .size(ButtonSize::Regular),
                    ),
                )
                // Pressed-state preview: `focused(true)` paints the focus
                // ring so the column communicates the active/keyboard-active
                // chrome at a glance. The actual press fill kicks in when
                // the user clicks an Idle button — the focused chrome here
                // approximates the same emphasis statically.
                .child(
                    div().w(px(110.0)).child(
                        Button::new(format!("{id_root}-pressed"))
                            .label("Label")
                            .variant(v)
                            .size(ButtonSize::Regular)
                            .focused(true),
                    ),
                )
                .child(
                    div().w(px(110.0)).child(
                        Button::new(format!("{id_root}-disabled"))
                            .label("Label")
                            .variant(v)
                            .size(ButtonSize::Regular)
                            .disabled(true),
                    ),
                )
                .child(
                    div().w(px(110.0)).child(
                        Button::new(format!("{id_root}-round"))
                            .label("Label")
                            .variant(v)
                            .size(ButtonSize::Regular)
                            .round(true),
                    ),
                )
                .child(
                    div().w(px(120.0)).child(
                        Button::new(format!("{id_root}-lg"))
                            .label("Label")
                            .variant(v)
                            .size(ButtonSize::Large),
                    ),
                )
                .child(
                    div().w(px(70.0)).child(
                        Button::new(format!("{id_root}-icon"))
                            .icon(Icon::new(IconName::Sparkle).size(px(16.0)))
                            .variant(v)
                            .size(ButtonSize::Icon),
                    ),
                )
                .child(
                    div().w(px(70.0)).child(
                        Button::new(format!("{id_root}-icon-sm"))
                            .icon(Icon::new(IconName::Sparkle).size(px(14.0)))
                            .variant(v)
                            .size(ButtonSize::IconSmall),
                    ),
                ),
        );
    }

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

    div().id("buttons-scroll").child(grid).into_any_element()
}
