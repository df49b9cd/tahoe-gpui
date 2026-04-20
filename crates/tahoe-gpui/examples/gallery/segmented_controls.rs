//! Segmented Controls demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::selection_and_input::segmented_control::{
    SegmentItem, SegmentedControl,
};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

const SEGMENT_LABELS: &[&str] = &["All", "Open", "Closed", "Draft", "Archived"];

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let entity = cx.entity().clone();
    let selected = state.segmented_index;

    let selected_label = SEGMENT_LABELS.get(selected).unwrap_or(&"?");

    div()
        .id("segmented-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Segmented Controls"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A segmented control is a linear set of two or more segments, \
                     each of which functions as a button.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(format!("Selected: {selected_label}")),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child("Five segments (interactive):"),
                )
                .child(
                    div().w(px(420.0)).child(
                        SegmentedControl::new("sc-interactive")
                            .items(
                                SEGMENT_LABELS
                                    .iter()
                                    .map(|s| SegmentItem::new(*s))
                                    .collect(),
                            )
                            .selected(selected)
                            .on_change(move |new_idx, _window, cx| {
                                entity.update(cx, |this, cx| {
                                    this.segmented_index = new_idx;
                                    cx.notify();
                                });
                            }),
                    ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child("Two segments (static):"),
                )
                .child(
                    div().w(px(220.0)).child(
                        SegmentedControl::new("sc-2")
                            .items(vec![SegmentItem::new("Code"), SegmentItem::new("Preview")])
                            .selected(0),
                    ),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap(theme.spacing_md)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .child("Disabled:"),
                )
                .child(
                    div().w(px(420.0)).child(
                        SegmentedControl::new("sc-disabled")
                            .items(
                                SEGMENT_LABELS
                                    .iter()
                                    .map(|s| SegmentItem::new(*s))
                                    .collect(),
                            )
                            .selected(1)
                            .disabled(true)
                            .on_change(|_, _, _| {}),
                    ),
                ),
        )
        .into_any_element()
}
