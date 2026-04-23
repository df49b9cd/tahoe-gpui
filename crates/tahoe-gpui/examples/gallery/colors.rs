//! Colors demo — mirrors the macOS 26 (Community) "Colors" foundation page
//! from the Apple Tahoe UI Kit.

use gpui::prelude::*;
use gpui::{AnyElement, Context, FontWeight, Hsla, Window, div, px};

use tahoe_gpui::foundations::color::gradient::LinearGradient;
use tahoe_gpui::foundations::color::{Color, Gradient, GradientStop, MixColorSpace, UnitPoint};
use tahoe_gpui::foundations::materials::{Elevation, Glass, Shape, glass_effect};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

fn swatch(color: Hsla, label: &'static str, theme: &TahoeTheme) -> impl IntoElement + use<> {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(div().size(px(48.0)).rounded_full().bg(color))
        .child(
            div()
                .text_size(px(10.0))
                .text_color(theme.text_muted)
                .child(label),
        )
}

fn fill_swatch(color: Hsla, label: &'static str, theme: &TahoeTheme) -> impl IntoElement + use<> {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(
            div()
                .w(px(72.0))
                .h(px(48.0))
                .rounded(px(6.0))
                .bg(color)
                .border_1()
                .border_color(theme.border),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(theme.text_muted)
                .child(label),
        )
}

fn text_swatch(color: Hsla, label: &'static str, theme: &TahoeTheme) -> impl IntoElement + use<> {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.0))
        .child(
            div()
                .w(px(72.0))
                .h(px(48.0))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(28.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color)
                .child("A"),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(theme.text_muted)
                .child(label),
        )
}

fn section_header(text: &'static str, theme: &TahoeTheme) -> impl IntoElement + use<> {
    div()
        .text_style_emphasized(TextStyle::Headline, theme)
        .text_color(theme.text)
        .pt(theme.spacing_lg)
        .child(text)
}

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let p = &theme.palette;
    let sem = &theme.semantic;

    let accents = vec![
        (p.red, "Red"),
        (p.orange, "Orange"),
        (p.yellow, "Yellow"),
        (p.green, "Green"),
        (p.mint, "Mint"),
        (p.teal, "Teal"),
        (p.cyan, "Cyan"),
        (p.blue, "Blue"),
        (p.indigo, "Indigo"),
        (p.purple, "Purple"),
        (p.pink, "Pink"),
        (p.brown, "Brown"),
        (p.gray, "Gray"),
    ];

    div()
        .id("colors-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Colors"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Judicious use of color can enhance communication, evoke your \
                     brand, provide visual continuity, communicate status and feedback, \
                     and help people understand information.",
                ),
        )
        .child(section_header("Accents", theme))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap(theme.spacing_md)
                .children(accents.iter().map(|(c, name)| swatch(*c, name, theme))),
        )
        .child(section_header("Grays", theme))
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(swatch(p.gray, "Gray", theme))
                .child(swatch(p.gray2, "Gray 2", theme))
                .child(swatch(p.gray3, "Gray 3", theme))
                .child(swatch(p.gray4, "Gray 4", theme))
                .child(swatch(p.gray5, "Gray 5", theme))
                .child(swatch(p.gray6, "Gray 6", theme)),
        )
        .child(section_header("Fills (5 levels)", theme))
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(fill_swatch(sem.system_fill.into(), "Primary", theme))
                .child(fill_swatch(
                    sem.secondary_system_fill.into(),
                    "Secondary",
                    theme,
                ))
                .child(fill_swatch(
                    sem.tertiary_system_fill.into(),
                    "Tertiary",
                    theme,
                ))
                .child(fill_swatch(
                    sem.quaternary_system_fill.into(),
                    "Quaternary",
                    theme,
                ))
                .child(fill_swatch(
                    sem.quinary_system_fill.into(),
                    "Quinary",
                    theme,
                )),
        )
        .child(section_header("Text (5 levels)", theme))
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(text_swatch(sem.label.into(), "Primary", theme))
                .child(text_swatch(sem.secondary_label.into(), "Secondary", theme))
                .child(text_swatch(sem.tertiary_label.into(), "Tertiary", theme))
                .child(text_swatch(
                    sem.quaternary_label.into(),
                    "Quaternary",
                    theme,
                ))
                .child(text_swatch(sem.quinary_label.into(), "Quinary", theme)),
        )
        .child(section_header("Vibrant Glass Labels (Dim)", theme))
        .child(
            glass_effect(
                div().w_full().rounded(theme.radius_lg).overflow_hidden(),
                theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Elevated,
            )
            .id("vibrant-dim")
            .child(
                div()
                    .flex()
                    .gap(theme.spacing_md)
                    .p(theme.spacing_md)
                    .child(text_swatch(
                        theme.glass.labels_dim.primary.into(),
                        "Primary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_dim.secondary.into(),
                        "Secondary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_dim.tertiary.into(),
                        "Tertiary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_dim.quaternary.into(),
                        "Quaternary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_dim.quinary.into(),
                        "Quinary",
                        theme,
                    )),
            ),
        )
        .child(section_header("Vibrant Glass Labels (Bright)", theme))
        .child(
            glass_effect(
                div().w_full().rounded(theme.radius_lg).overflow_hidden(),
                theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Elevated,
            )
            .id("vibrant-bright")
            .child(
                div()
                    .flex()
                    .gap(theme.spacing_md)
                    .p(theme.spacing_md)
                    .child(text_swatch(
                        theme.glass.labels_bright.primary.into(),
                        "Primary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_bright.secondary.into(),
                        "Secondary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_bright.tertiary.into(),
                        "Tertiary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_bright.quaternary.into(),
                        "Quaternary",
                        theme,
                    ))
                    .child(text_swatch(
                        theme.glass.labels_bright.quinary.into(),
                        "Quinary",
                        theme,
                    )),
            ),
        )
        .child(section_header("Color Mixing (OKLab)", theme))
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(swatch(p.red, "Red", theme))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(Color::from_hsla(p.red).mix(
                                    Color::from_hsla(p.blue),
                                    0.5,
                                    MixColorSpace::Perceptual,
                                    cx,
                                )),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.text_muted)
                                .child("50%"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(Color::from_hsla(p.red).mix(
                                    Color::from_hsla(p.blue),
                                    0.3,
                                    MixColorSpace::Perceptual,
                                    cx,
                                )),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.text_muted)
                                .child("30%"),
                        ),
                )
                .child(swatch(p.blue, "Blue", theme)),
        )
        .child(section_header("Opacity Modifier", theme))
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(swatch(p.red, "100%", theme))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(Color::from_hsla(p.red).opacity(0.75)),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.text_muted)
                                .child("75%"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(Color::from_hsla(p.red).opacity(0.5)),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.text_muted)
                                .child("50%"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .size(px(48.0))
                                .rounded_full()
                                .bg(Color::from_hsla(p.red).opacity(0.25)),
                        )
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.text_muted)
                                .child("25%"),
                        ),
                ),
        )
        .child(section_header("Gradient from Color", theme))
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child({
                            let c = Color::from_hsla(p.blue);
                            let grad = c.gradient();
                            match &grad {
                                tahoe_gpui::foundations::color::AnyGradient::Linear(lg) => {
                                    let (angle, stops) = lg.to_gpui_eager();
                                    div()
                                        .w(px(120.0))
                                        .h(px(48.0))
                                        .rounded(px(6.0))
                                        .bg(gpui::linear_gradient(angle, stops[0], stops[1]))
                                }
                                _ => div(),
                            }
                        })
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.text_muted)
                                .child("Blue"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child({
                            let lg = LinearGradient::new(
                                Gradient::new(vec![
                                    GradientStop {
                                        color: Color::from_hsla(p.red),
                                        location: 0.0,
                                    },
                                    GradientStop {
                                        color: Color::from_hsla(p.purple),
                                        location: 0.5,
                                    },
                                    GradientStop {
                                        color: Color::from_hsla(p.blue),
                                        location: 1.0,
                                    },
                                ]),
                                UnitPoint::LEADING,
                                UnitPoint::TRAILING,
                            );
                            let (angle, stops) = lg.to_gpui_eager();
                            div()
                                .w(px(120.0))
                                .h(px(48.0))
                                .rounded(px(6.0))
                                .bg(gpui::linear_gradient(angle, stops[0], stops[1]))
                        })
                        .child(
                            div()
                                .text_size(px(10.0))
                                .text_color(theme.text_muted)
                                .child("Red→Purple→Blue"),
                        ),
                ),
        )
        .into_any_element()
}
