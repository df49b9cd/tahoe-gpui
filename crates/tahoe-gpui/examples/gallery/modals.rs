//! Modals demo — wires the live `Modal` component (issue #156 F-05/F-14).

use gpui::prelude::*;
use gpui::{AnyElement, App, Context, Window, div};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::presentation::modal::Modal;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let is_open = state.modal_open;

    let entity = cx.entity().downgrade();
    let dismiss = move |_window: &mut Window, cx: &mut App| {
        if let Some(this) = entity.upgrade() {
            this.update(cx, |this, cx| {
                this.modal_open = false;
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
                .child("Modals"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A modal presents focused content above its parent. Press \
                     Escape, click the dimmed backdrop, or use the action buttons \
                     to dismiss.",
                ),
        );

    let toolbar = div()
        .px(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .gap(theme.spacing_sm)
        .child(
            Button::new("modal-show")
                .label("Show modal")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.modal_open = true;
                    cx.notify();
                })),
        );

    let modal: AnyElement = if is_open {
        let dismiss_for_cancel = dismiss.clone();
        let dismiss_for_ok = dismiss.clone();
        let body = div()
            .p(theme.spacing_lg)
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_style_emphasized(TextStyle::Title3, theme)
                    .text_color(theme.text)
                    .child("Modal title"),
            )
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child(
                        "This is real modal content rendered through the live \
                         `Modal` component — backdrop, focus-trap, Escape key \
                         handling, and dismiss-on-outside-click are all wired.",
                    ),
            )
            .child(
                div()
                    .pt(theme.spacing_md)
                    .flex()
                    .gap(theme.spacing_sm)
                    .justify_end()
                    .child(
                        Button::new("modal-cancel")
                            .label("Cancel")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Md)
                            .on_click(move |_, window, cx| dismiss_for_cancel(window, cx)),
                    )
                    .child(
                        Button::new("modal-ok")
                            .label("OK")
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Md)
                            .on_click(move |_, window, cx| dismiss_for_ok(window, cx)),
                    ),
            );

        Modal::new("gallery-modal", body)
            .open(true)
            .on_dismiss(move |window, cx| dismiss(window, cx))
            .into_any_element()
    } else {
        div().into_any_element()
    };

    div()
        .id("modals-pane")
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
        .child(modal)
        .into_any_element()
}
