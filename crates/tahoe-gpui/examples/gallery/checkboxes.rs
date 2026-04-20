//! Checkboxes (Checkbox) demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::selection_and_input::checkbox::{Checkbox, CheckboxState};
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
    let current = state.checkbox_state;

    div()
        .id("checkboxes-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Checkboxes"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A checkbox is a small square that can indicate on, off, or \
                     mixed. People use checkboxes to choose one or more options \
                     from a list.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(180.0))
                        .child("Interactive"),
                )
                .child({
                    let entity = entity.clone();
                    Checkbox::new("cb-interactive")
                        .state(current)
                        .label("Remember me")
                        .on_change(move |new_state, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.checkbox_state = new_state;
                                cx.notify();
                            });
                        })
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(180.0))
                        .child("Disabled unchecked"),
                )
                .child(
                    Checkbox::new("cb-disabled-unchecked")
                        .state(CheckboxState::Unchecked)
                        .label("Subscribe to newsletter")
                        .disabled(true),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(180.0))
                        .child("Disabled checked"),
                )
                .child(
                    Checkbox::new("cb-disabled-checked")
                        .state(CheckboxState::Checked)
                        .label("Terms accepted")
                        .disabled(true),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(180.0))
                        .child("Disabled mixed"),
                )
                .child(
                    Checkbox::new("cb-disabled-mixed")
                        .state(CheckboxState::Mixed)
                        .label("Select all children")
                        .disabled(true),
                ),
        )
        .into_any_element()
}
