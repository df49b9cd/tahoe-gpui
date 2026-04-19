//! Shimmer demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::status::shimmer::Shimmer;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    div()
        .id("shimmers-pane")
        .child(
            div()
                .p(theme.spacing_xl)
                .flex()
                .flex_col()
                .gap(theme.spacing_lg)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::LargeTitle, theme)
                        .text_color(theme.text)
                        .child("Shimmers"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A shimmer provides a loading placeholder animation \
                             that indicates content is being fetched.",
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Content placeholders"),
                )
                .child(
                    // Skeleton placeholder dimensions (px(20.0), px(14.0),
                    // px(48.0)) are intentional fixed sizes — they stand in
                    // for a Title text run, two body lines, and an avatar
                    // tile, NOT a spacing-token bypass. Do not migrate these
                    // to `theme.spacing_*`: a shimmer's size is the size of
                    // the content it replaces, not a layout gap.
                    div()
                        .flex()
                        .flex_col()
                        .gap(theme.spacing_md)
                        .child(
                            Shimmer::new("sh-title")
                                .width(px(200.0))
                                .height(px(20.0))
                                .label("Title loading"),
                        )
                        .child(
                            Shimmer::new("sh-body-1")
                                .width(px(320.0))
                                .height(px(14.0))
                                .label("Body line 1"),
                        )
                        .child(
                            Shimmer::new("sh-body-2")
                                .width(px(280.0))
                                .height(px(14.0))
                                .label("Body line 2"),
                        )
                        .child(
                            Shimmer::new("sh-avatar")
                                .width(px(48.0))
                                .height(px(48.0))
                                .label("Avatar"),
                        ),
                ),
        )
        .into_any_element()
}
