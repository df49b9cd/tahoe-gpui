//! Typography demo — mirrors the macOS 26 (Community) "Typography" foundation
//! page. Shows every HIG text style (LargeTitle through Caption2) in
//! both regular and emphasized weights.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

const STYLES: &[(TextStyle, &str)] = &[
    (TextStyle::LargeTitle, "LargeTitle"),
    (TextStyle::Title1, "Title1"),
    (TextStyle::Title2, "Title2"),
    (TextStyle::Title3, "Title3"),
    (TextStyle::Headline, "Headline"),
    (TextStyle::Body, "Body"),
    (TextStyle::Callout, "Callout"),
    (TextStyle::Subheadline, "Subheadline"),
    (TextStyle::Footnote, "Footnote"),
    (TextStyle::Caption1, "Caption1"),
    (TextStyle::Caption2, "Caption2"),
];

fn column_label(text: &'static str, theme: &TahoeTheme) -> impl IntoElement + use<> {
    div()
        .w(px(220.0))
        .text_style(TextStyle::Caption1, theme)
        .text_color(theme.text_muted)
        .child(text)
}

fn style_row(
    style: TextStyle,
    label: &'static str,
    theme: &TahoeTheme,
) -> impl IntoElement + use<> {
    div()
        .flex()
        .items_baseline()
        .gap(px(40.0))
        .py(px(4.0))
        // Regular
        .child(
            div()
                .w(px(220.0))
                .text_style(style, theme)
                .text_color(theme.text)
                .child(label),
        )
        // Emphasized
        .child(
            div()
                .w(px(220.0))
                .text_style_emphasized(style, theme)
                .text_color(theme.text)
                .child(label),
        )
        // Spec metadata: size / leading
        .child({
            let attrs = style.attrs();
            // Round so a 13.5pt size displays as "14pt" instead of "13pt".
            // The plain `as i32` cast was a truncation that hid the half-
            // step values in the HIG type scale.
            let size = f32::from(attrs.size).round() as i32;
            let leading = f32::from(attrs.leading).round() as i32;
            div()
                .text_style(TextStyle::Caption2, theme)
                .text_color(theme.text_muted)
                .child(format!("{size}pt / {leading}pt leading"))
        })
}

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    div()
        .id("typography-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Typography"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "Apple provides two typeface families that support an extensive \
                     range of weights, sizes, styles, and languages: San Francisco \
                     and New York.",
                ),
        )
        // Column headers
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(40.0))
                .pt(theme.spacing_md)
                .border_b_1()
                .border_color(theme.border)
                .pb(theme.spacing_xs)
                .child(column_label("Regular", theme))
                .child(column_label("Emphasized", theme))
                .child(column_label("Spec", theme)),
        )
        // One row per style
        .children(STYLES.iter().map(|(s, label)| style_row(*s, label, theme)))
        .child(
            div()
                .pt(theme.spacing_lg)
                .text_size(px(11.0))
                .text_color(theme.text_muted)
                .child(
                    "Audit notes: All 11 HIG text styles are exposed via the \
                     TextStyle enum and theme.text_style() / text_style_emphasized() \
                     extension methods. Sizes match the macOS HIG type scale.",
                ),
        )
        .into_any_element()
}
