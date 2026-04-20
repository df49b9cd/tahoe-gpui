//! Sheets demo — wires the live `Sheet` component and presents it inside
//! the gallery pane (issue #156 F-05). The pane root is `relative + size_full`
//! so the Sheet's absolute backdrop covers only the demo pane rather than
//! the entire window.

use gpui::prelude::*;
use gpui::{AnyElement, App, Context, Window, div};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::presentation::sheet::{Sheet, SheetDetent, SheetPresentation};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let is_open = state.sheet_open;

    let entity = cx.entity().downgrade();
    let dismiss = move |_window: &mut Window, cx: &mut App| {
        if let Some(this) = entity.upgrade() {
            this.update(cx, |this, cx| {
                this.sheet_open = false;
                cx.notify();
            });
        }
    };

    let header = div()
        .px(theme.spacing_xl)
        .pt(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .flex_col()
        .gap(theme.spacing_xs)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Sheets"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A sheet is a modal interface element that floats over its parent. \
                     macOS renders cardlike centered sheets; touch platforms render \
                     bottom-anchored drawers with detent heights.",
                ),
        );

    let toolbar = div()
        .px(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .gap(theme.spacing_sm)
        .child(
            Button::new("sheet-show-cardlike")
                .label("Show macOS cardlike sheet")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Regular)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.sheet_open = true;
                    cx.notify();
                })),
        );

    let sheet: AnyElement = if is_open {
        let dismiss_for_button = dismiss.clone();
        let body = div()
            .p(theme.spacing_lg)
            .flex()
            .flex_col()
            .gap(theme.spacing_md)
            .child(
                div()
                    .text_style_emphasized(TextStyle::Title3, theme)
                    .text_color(theme.text)
                    .child("Compose new message"),
            )
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child(
                        "Sheet content renders inside a glass panel with the \
                         caller-supplied body. Click the dimmed area or press \
                         Escape to dismiss.",
                    ),
            )
            .child(
                div()
                    .pt(theme.spacing_md)
                    .flex()
                    .gap(theme.spacing_sm)
                    .justify_end()
                    .child(
                        Button::new("sheet-cancel")
                            .label("Cancel")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Regular)
                            .on_click(move |_event, window, cx| {
                                dismiss_for_button(window, cx);
                            }),
                    ),
            );

        Sheet::new("gallery-sheet", body)
            .open(true)
            .presentation(SheetPresentation::Cardlike)
            .detent(SheetDetent::Large)
            .on_dismiss(move |window, cx| dismiss(window, cx))
            .into_any_element()
    } else {
        div().into_any_element()
    };

    div()
        .id("sheets-pane")
        .relative()
        .size_full()
        .bg(theme.glass.root_bg)
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(header)
                .child(toolbar),
        )
        .child(sheet)
        .into_any_element()
}
