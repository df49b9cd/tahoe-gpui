//! DigitEntry demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    // Wire the interactive entity's `on_change` back into gallery state
    // once (setup runs every render; handler replacement is cheap).
    let entity = cx.entity().clone();
    state.digit_interactive.update(cx, |entry, _cx| {
        entry.set_on_change(move |value, _window, cx| {
            let value = value.to_string();
            entity.update(cx, |this, cx| {
                this.digit_last_value = value.clone().into();
                cx.notify();
            });
        });
    });

    let digit_value = state.digit_last_value.clone();
    let status_text = if digit_value.is_empty() {
        "No digits entered".to_string()
    } else {
        format!("Entered: {digit_value}")
    };

    div()
        .id("digit-entries-pane")
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
                        .child("Digit Entries"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A digit entry provides individual cells for PIN \
                             or verification code input.",
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child(status_text),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("6-digit PIN (interactive)"),
                )
                .child(state.digit_interactive.clone())
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("4-digit PIN (static, pre-filled)"),
                )
                .child(state.digit_static.clone()),
        )
        .into_any_element()
}
