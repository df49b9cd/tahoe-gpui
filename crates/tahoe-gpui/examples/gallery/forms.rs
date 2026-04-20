//! Forms demo — composes primitives into a typical macOS form layout.

use gpui::prelude::*;
use gpui::{AnyElement, Context, SharedString, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::menus_and_actions::popup_button::{PopupButton, PopupItem};
use tahoe_gpui::components::selection_and_input::toggle::Toggle;
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
    let form_notifications = state.form_notifications;
    let form_sounds = state.form_sounds;
    let form_theme = state.form_theme.clone();

    let label = |text: &'static str| {
        div()
            .text_style(TextStyle::Body, theme)
            .text_color(theme.text)
            .w(px(140.0))
            .child(text)
    };

    let row = |label_el: gpui::Div, control: AnyElement| -> gpui::Div {
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_md)
            .py(theme.spacing_xs)
            .child(label_el)
            .child(control)
    };

    div()
        .id("forms-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Forms"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("Forms group related controls into right-aligned label/control rows."),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_lg)
                .p(theme.spacing_lg)
                .flex()
                .flex_col()
                .gap(theme.spacing_xs)
                .child(row(
                    label("Display name:"),
                    div()
                        .h(px(28.0))
                        .w(px(280.0))
                        .px(theme.spacing_sm)
                        .flex()
                        .items_center()
                        .border_1()
                        .border_color(theme.border)
                        .rounded(theme.radius_sm)
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child("Soeren Magnus Olesen")
                        .into_any_element(),
                ))
                .child(row(label("Theme:"), {
                    let entity = entity.clone();
                    PopupButton::new("form-theme")
                        .items(vec![
                            PopupItem::new("Light", "light"),
                            PopupItem::new("Dark", "dark"),
                            PopupItem::new("Auto", "auto"),
                        ])
                        .selected(form_theme)
                        .on_change(move |value: &SharedString, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.form_theme = value.clone();
                                cx.notify();
                            });
                        })
                        .into_any_element()
                }))
                .child(row(label("Notifications:"), {
                    let entity = entity.clone();
                    Toggle::new("form-notifications")
                        .checked(form_notifications)
                        .on_change(move |new_val, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.form_notifications = new_val;
                                cx.notify();
                            });
                        })
                        .into_any_element()
                }))
                .child(row(label("Sounds:"), {
                    let entity = entity.clone();
                    Toggle::new("form-sounds")
                        .checked(form_sounds)
                        .on_change(move |new_val, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.form_sounds = new_val;
                                cx.notify();
                            });
                        })
                        .into_any_element()
                }))
                .child(div().h(theme.spacing_sm))
                .child(
                    div()
                        .flex()
                        .gap(theme.spacing_sm)
                        .justify_end()
                        .child(
                            Button::new("form-cancel")
                                .label("Cancel")
                                .variant(ButtonVariant::Outline)
                                .size(ButtonSize::Regular),
                        )
                        .child(
                            Button::new("form-save")
                                .label("Save")
                                .variant(ButtonVariant::Primary)
                                .size(ButtonSize::Regular),
                        ),
                ),
        )
        .into_any_element()
}
