//! Boxes (Panel) demo. macOS HIG calls these "boxes" — grouped surfaces.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::foundations::materials::glass_surface;
use tahoe_gpui::foundations::theme::{GlassSize, TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    // Helper: a box (grouped surface) with optional title
    let titled_box = |id: &'static str, title: &'static str, body: &'static str| {
        glass_surface(
            div()
                .w_full()
                .overflow_hidden()
                .rounded(theme.glass.radius(GlassSize::Medium)),
            theme,
            GlassSize::Medium,
        )
        .id(id)
        .child(
            div()
                .p(theme.spacing_md)
                .flex()
                .flex_col()
                .gap(theme.spacing_sm)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::Headline, theme)
                        .text_color(theme.text)
                        .child(title),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(body),
                ),
        )
    };

    div()
        .id("boxes-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Boxes"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A box (or group box) creates a visual grouping of related controls. \
                     Rendered as a glass surface with rounded corners.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        // Box with title and description
        .child(titled_box(
            "box-titled",
            "General",
            "Panels group related controls behind a subtle \
             glass surface, matching the macOS box pattern.",
        ))
        // Box without title (content only)
        .child(
            glass_surface(
                div()
                    .w_full()
                    .overflow_hidden()
                    .rounded(theme.glass.radius(GlassSize::Medium)),
                theme,
                GlassSize::Medium,
            )
            .id("box-no-title")
            .child(
                div()
                    .p(theme.spacing_md)
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child(
                        "A box without a title groups content with a subtle surface \
                         and no heading. Useful for secondary or auxiliary content.",
                    ),
            ),
        )
        // Box with border emphasis
        .child(
            glass_surface(
                div()
                    .w_full()
                    .overflow_hidden()
                    .rounded(theme.glass.radius(GlassSize::Medium))
                    .border_1()
                    .border_color(theme.border),
                theme,
                GlassSize::Medium,
            )
            .id("box-bordered")
            .child(
                div()
                    .p(theme.spacing_md)
                    .flex()
                    .flex_col()
                    .gap(theme.spacing_sm)
                    .child(
                        div()
                            .text_style_emphasized(TextStyle::Headline, theme)
                            .text_color(theme.text)
                            .child("Bordered"),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Body, theme)
                            .text_color(theme.text_muted)
                            .child(
                                "Adding a border provides extra visual separation \
                                 for high-contrast or accessibility scenarios.",
                            ),
                    ),
            ),
        )
        .into_any_element()
}
