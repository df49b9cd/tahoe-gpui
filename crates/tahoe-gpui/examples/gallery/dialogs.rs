//! Dialogs demo — wires the live `Modal` component as a focused
//! confirmation dialog (issue #156 F-05/F-14). The HIG distinguishes a
//! dialog (single task, action buttons) from a generic modal; we use the
//! same Modal primitive but supply dialog-shaped content + buttons.

use gpui::prelude::*;
use gpui::{AnyElement, App, Context, Window, div, px};

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
    let is_open = state.dialog_open;

    let entity = cx.entity().downgrade();
    let dismiss = move |_window: &mut Window, cx: &mut App| {
        if let Some(this) = entity.upgrade() {
            this.update(cx, |this, cx| {
                this.dialog_open = false;
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
                .child("Dialogs"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A dialog presents a focused interaction tied to a specific \
                     task. The Save Dialog (Small) variant below uses the live \
                     `Modal` primitive with dialog-shaped content.",
                ),
        );

    let toolbar = div()
        .px(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .gap(theme.spacing_sm)
        .child(
            Button::new("dialog-show-save")
                .label("Show Save dialog")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.dialog_open = true;
                    cx.notify();
                })),
        );

    let dialog: AnyElement = if is_open {
        let dismiss_a = dismiss.clone();
        let dismiss_b = dismiss.clone();
        let dismiss_c = dismiss.clone();
        let body = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_md)
            .p(theme.spacing_lg)
            .child(
                div()
                    .text_style_emphasized(TextStyle::Headline, theme)
                    .text_color(theme.text)
                    .child("Do you want to keep this new document \u{201C}Untitled\u{201D}?"),
            )
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child(
                        "You can choose to save your changes, or delete this document \
                         immediately. You can\u{2019}t undo this action.",
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .child(
                        div()
                            .text_style(TextStyle::Body, theme)
                            .text_color(theme.text_muted)
                            .w(px(80.0))
                            .child("Save As:"),
                    )
                    .child(
                        div()
                            .flex_1()
                            .h(px(28.0))
                            .px(theme.spacing_sm)
                            .flex()
                            .items_center()
                            .border_1()
                            .border_color(theme.border)
                            .rounded(theme.radius_sm)
                            .text_style(TextStyle::Body, theme)
                            .text_color(theme.text)
                            .child("Untitled"),
                    ),
            )
            .child(
                div()
                    .pt(theme.spacing_md)
                    .flex()
                    .gap(theme.spacing_sm)
                    .justify_end()
                    .child(
                        Button::new("dlg-delete")
                            .label("Delete")
                            .variant(ButtonVariant::Destructive)
                            .size(ButtonSize::Md)
                            .on_click(move |_, window, cx| dismiss_a(window, cx)),
                    )
                    .child(
                        Button::new("dlg-cancel")
                            .label("Cancel")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Md)
                            .on_click(move |_, window, cx| dismiss_b(window, cx)),
                    )
                    .child(
                        Button::new("dlg-save")
                            .label("Save")
                            .variant(ButtonVariant::Primary)
                            .size(ButtonSize::Md)
                            .on_click(move |_, window, cx| dismiss_c(window, cx)),
                    ),
            );

        Modal::new("gallery-dialog", body)
            .open(true)
            .on_dismiss(move |window, cx| dismiss(window, cx))
            .into_any_element()
    } else {
        div().into_any_element()
    };

    div()
        .id("dialogs-pane")
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
        .child(dialog)
        .into_any_element()
}
