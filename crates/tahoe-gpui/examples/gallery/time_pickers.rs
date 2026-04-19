//! TimePicker demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::selection_and_input::time_picker::TimePicker;
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
    let hour = state.time_hour;
    let minute = state.time_minute;
    let time_open = state.time_picker_open;

    let am_pm = if hour < 12 { "AM" } else { "PM" };
    let display_hour = if hour == 0 {
        12
    } else if hour > 12 {
        hour - 12
    } else {
        hour
    };
    let status_text = format!("Selected: {display_hour}:{minute:02} {am_pm}");

    div()
        .id("time-pickers-pane")
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
                        .child("Time Pickers"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A time picker lets the user select an hour and \
                             minute value from a compact control.",
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
                    TimePicker::new("tp-interactive")
                        .hour(hour)
                        .minute(minute)
                        .open(time_open)
                        .on_change(move |new_hour, new_minute, _window, cx| {
                            entity_change.update(cx, |this, cx| {
                                this.time_hour = new_hour;
                                this.time_minute = new_minute;
                                this.time_picker_open = false;
                                cx.notify();
                            });
                        })
                        .on_toggle(move |open, _window, cx| {
                            entity_toggle.update(cx, |this, cx| {
                                this.time_picker_open = open;
                                cx.notify();
                            });
                        })
                })
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("2:15 PM (static)"),
                )
                .child(TimePicker::new("tp-1415").hour(14).minute(15)),
        )
        .into_any_element()
}
