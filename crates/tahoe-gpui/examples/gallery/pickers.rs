//! Picker demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::selection_and_input::picker::{Picker, PickerItem};
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
    let picker_selected = state.picker_selected.clone();
    let picker_open = state.picker_open;

    let status_text = match &picker_selected {
        Some(v) => format!("Selected: {v}"),
        None => "No selection".to_string(),
    };

    div()
        .id("pickers-pane")
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
                        .child("Pickers"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A picker lets users choose from a list of \
                             mutually exclusive values in a dropdown.",
                        ),
                )
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child(status_text),
                )
                .child({
                    let entity_change = entity.clone();
                    let entity_toggle = entity.clone();
                    Picker::new("pk-interactive")
                        .items(vec![
                            PickerItem::new("Apple", "apple"),
                            PickerItem::new("Banana", "banana"),
                            PickerItem::new("Cherry", "cherry"),
                            PickerItem::new("Date", "date"),
                        ])
                        .selected(picker_selected)
                        .open(picker_open)
                        .on_change(move |value, _window, cx| {
                            entity_change.update(cx, |this, cx| {
                                this.picker_selected = Some(value.clone());
                                this.picker_open = false;
                                cx.notify();
                            });
                        })
                        .on_toggle(move |open, _window, cx| {
                            entity_toggle.update(cx, |this, cx| {
                                this.picker_open = open;
                                cx.notify();
                            });
                        })
                })
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Placeholder (no selection)"),
                )
                .child(
                    Picker::new("pk-placeholder")
                        .items(vec![
                            PickerItem::new("Small", "sm"),
                            PickerItem::new("Medium", "md"),
                            PickerItem::new("Large", "lg"),
                        ])
                        .placeholder("Choose a size"),
                ),
        )
        .into_any_element()
}
