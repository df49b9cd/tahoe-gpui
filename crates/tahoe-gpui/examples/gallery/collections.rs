//! Collections demo (issue #156 F-09). Exercises CollectionView's grid
//! layout, sections, and selection state.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::layout_and_organization::collection_view::{
    CollectionLayout, CollectionView,
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

    let tile = |label: &'static str| {
        div()
            .h(px(72.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(theme.radius_md)
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .text_style(TextStyle::Body, theme)
            .text_color(theme.text)
            .child(label)
    };

    div()
        .id("collections-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Collections"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A collection view displays items in a flexible grid. Items \
                     can have uniform or variable sizes; the layout adapts to the \
                     available width.",
                ),
        )
        .child(
            div()
                .pt(theme.spacing_sm)
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Grid (4 columns)"),
        )
        .child(
            CollectionView::new("collection-grid")
                .layout(CollectionLayout::Grid { columns: 4 })
                .child(tile("One"))
                .child(tile("Two"))
                .child(tile("Three"))
                .child(tile("Four"))
                .child(tile("Five"))
                .child(tile("Six"))
                .child(tile("Seven"))
                .child(tile("Eight")),
        )
        .child(
            div()
                .pt(theme.spacing_md)
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Flow (160 pt items)"),
        )
        .child(
            CollectionView::new("collection-flow")
                .layout(CollectionLayout::Flow {
                    item_width: px(160.0),
                })
                .child(tile("Photo"))
                .child(tile("Document"))
                .child(tile("Audio"))
                .child(tile("Video"))
                .child(tile("Note")),
        )
        .into_any_element()
}
