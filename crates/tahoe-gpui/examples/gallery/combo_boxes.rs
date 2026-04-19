//! Combo Boxes demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, SharedString, Window, div, px};

use tahoe_gpui::components::selection_and_input::combo_box::ComboBox;
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
    let combo_value = state.combo_value.clone();
    let combo_open = state.combo_open;

    let countries: Vec<SharedString> = [
        "Australia",
        "Brazil",
        "Canada",
        "Denmark",
        "Estonia",
        "Finland",
        "France",
        "Germany",
        "Ireland",
        "Japan",
    ]
    .iter()
    .map(|s| SharedString::from(*s))
    .collect();

    let status_text = if combo_value.is_empty() {
        "No selection".to_string()
    } else {
        format!("Value: {combo_value}")
    };

    div()
        .id("combo-boxes-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Combo Boxes"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A combo box is a text input combined with a filterable dropdown \
                     of suggestions.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(status_text),
        )
        .child(
            div()
                .flex()
                .items_start()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .w(px(140.0))
                        .child("Interactive"),
                )
                .child(div().w(px(280.0)).child({
                    let entity_select = entity.clone();
                    let entity_toggle = entity.clone();
                    let entity_input = entity.clone();
                    ComboBox::new("cb-interactive")
                        .placeholder("Select a country")
                        .value(combo_value.clone())
                        .items(countries)
                        .open(combo_open)
                        .on_select(move |selected, _window, cx| {
                            entity_select.update(cx, |this, cx| {
                                this.combo_value = selected.clone();
                                this.combo_open = false;
                                cx.notify();
                            });
                        })
                        .on_toggle(move |open, _window, cx| {
                            entity_toggle.update(cx, |this, cx| {
                                this.combo_open = open;
                                cx.notify();
                            });
                        })
                        .on_input(move |text, _window, cx| {
                            entity_input.update(cx, |this, cx| {
                                this.combo_value = text;
                                cx.notify();
                            });
                        })
                })),
        )
        .into_any_element()
}
