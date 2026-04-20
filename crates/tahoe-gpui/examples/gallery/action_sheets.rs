//! ActionSheet demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::presentation::action_sheet::{
    ActionSheet, ActionSheetItem, ActionSheetStyle,
};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    _state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    div()
        .id("action-sheets-pane")
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
                        .child("Action Sheets"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "An action sheet presents a set of alternatives \
                             for how to proceed with a task.",
                        ),
                )
                .child(div().h(theme.spacing_sm))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Default Style"),
                )
                .child(
                    ActionSheet::new("demo-action-sheet")
                        .items(vec![
                            ActionSheetItem::new("Copy"),
                            ActionSheetItem::new("Move to Trash")
                                .style(ActionSheetStyle::Destructive),
                        ])
                        .open(true),
                )
                .child(div().h(px(16.0)))
                .child(
                    div()
                        .text_style(TextStyle::Title3, theme)
                        .text_color(theme.text)
                        .child("Destructive Emphasis"),
                )
                .child(
                    ActionSheet::new("demo-action-sheet-destructive")
                        .items(vec![
                            ActionSheetItem::new("Share Link"),
                            ActionSheetItem::new("Duplicate"),
                            ActionSheetItem::new("Delete Permanently")
                                .style(ActionSheetStyle::Destructive),
                        ])
                        .cancel_text("Dismiss")
                        .open(true),
                ),
        )
        .into_any_element()
}
