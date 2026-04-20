//! Tooltips demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::presentation::tooltip::Tooltip;
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
        .id("tooltips-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Tooltips"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A tooltip provides contextual information about an interface element \
                     when people hover over it.",
                ),
        )
        .child(div().h(px(12.0)))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child("Hover over the buttons below to see tooltips:"),
        )
        .child(
            div()
                .flex()
                .gap(theme.spacing_md)
                .child(Tooltip::new(
                    "tt-save-wrap",
                    "Save the current document (\u{2318}S)",
                    Button::new("tt-save")
                        .label("Save")
                        .variant(ButtonVariant::Primary)
                        .size(ButtonSize::Regular),
                ))
                .child(Tooltip::new(
                    "tt-undo-wrap",
                    "Undo last action (\u{2318}Z)",
                    Button::new("tt-undo")
                        .label("Undo")
                        .variant(ButtonVariant::Outline)
                        .size(ButtonSize::Regular),
                )),
        )
        .into_any_element()
}
