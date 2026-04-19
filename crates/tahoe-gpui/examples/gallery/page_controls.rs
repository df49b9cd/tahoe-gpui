//! PageControls demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::presentation::page_controls::PageControls;
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

    let page_current = state.page_current;

    div()
        .id("page-controls-pane")
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
                        .child("Page Controls"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A page control displays a row of dots indicating \
                             the current page within a flat page sequence.",
                        ),
                )
                // Interactive example
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child(format!("Interactive (page {})", page_current + 1)),
                )
                .child(
                    PageControls::new("pc-interactive")
                        .total(5)
                        .current(page_current)
                        .on_change(move |index, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.page_current = index;
                                cx.notify();
                            });
                        }),
                )
                // Static examples
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("5 pages, current = 0"),
                )
                .child(PageControls::new("pc-5-0").total(5).current(0))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("5 pages, current = 2"),
                )
                .child(PageControls::new("pc-5-2").total(5).current(2))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("5 pages, current = 4"),
                )
                .child(PageControls::new("pc-5-4").total(5).current(4))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("3 pages, focused"),
                )
                .child(
                    PageControls::new("pc-3-focused")
                        .total(3)
                        .current(1)
                        .focused(true),
                ),
        )
        .into_any_element()
}
