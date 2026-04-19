//! DatePicker demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::selection_and_input::date_picker::DatePicker;
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
    let date = state.date_picker_date;
    let date_open = state.date_picker_open;
    let viewing_year = state.date_viewing_year;
    let viewing_month = state.date_viewing_month;

    let status_text = match &date {
        Some(d) => format!("Selected: {}-{:02}-{:02}", d.year, d.month, d.day),
        None => "No date selected".to_string(),
    };

    div()
        .id("date-pickers-pane")
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
                        .child("Date Pickers"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "A date picker lets the user select a calendar \
                             date from a compact popover control.",
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
                    let entity_nav = entity.clone();
                    let mut dp = DatePicker::new("dp-interactive")
                        .open(date_open)
                        .viewing(viewing_year, viewing_month);
                    if let Some(d) = date {
                        dp = dp.selected(d);
                    }
                    dp.on_change(move |new_date, _window, cx| {
                        entity_change.update(cx, |this, cx| {
                            this.date_picker_date = Some(new_date);
                            this.date_picker_open = false;
                            cx.notify();
                        });
                    })
                    .on_toggle(move |open, _window, cx| {
                        entity_toggle.update(cx, |this, cx| {
                            this.date_picker_open = open;
                            cx.notify();
                        });
                    })
                    .on_navigate(move |year, month, _window, cx| {
                        entity_nav.update(cx, |this, cx| {
                            this.date_viewing_year = year;
                            this.date_viewing_month = month;
                            cx.notify();
                        });
                    })
                })
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("No selection (placeholder)"),
                )
                .child(DatePicker::new("dp-empty")),
        )
        .into_any_element()
}
