//! Rating Indicators demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::status::rating_indicator::RatingIndicator;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let entity = cx.entity().clone();
    let rating_value = state.rating_value;

    let row = |id: &'static str, label: &'static str, value: f32| {
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_md)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .w(px(140.0))
                    .child(label),
            )
            .child(RatingIndicator::new(id).value(value))
    };

    let rating_status = format!("{} stars", rating_value);

    div()
        .id("rating-indicators-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Rating Indicators"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A rating indicator uses a row of star icons to represent \
                     a value on a discrete scale.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Interactive"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child(rating_status),
                )
                .child(
                    RatingIndicator::new("r-interactive")
                        .value(rating_value)
                        .interactive(true)
                        .on_change(move |new_value, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.rating_value = new_value;
                                cx.notify();
                            });
                        }),
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Static examples"),
        )
        .child(row("r-0", "0 stars", 0.0))
        .child(row("r-1", "1 star", 1.0))
        .child(row("r-2.5", "2.5 stars (half)", 2.5))
        .child(row("r-4", "4 stars", 4.0))
        .child(row("r-5", "5 stars (max)", 5.0))
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Custom color"),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("3 stars (error)"),
                )
                .child(
                    RatingIndicator::new("r-custom")
                        .value(3.0)
                        .color(theme.error),
                ),
        )
        .into_any_element()
}
