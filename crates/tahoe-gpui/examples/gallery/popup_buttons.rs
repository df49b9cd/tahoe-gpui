//! Pop-up and Pull-down Buttons demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::menus_and_actions::popup_button::{PopupButton, PopupItem};
use tahoe_gpui::components::menus_and_actions::pulldown_button::{
    PulldownButton, PulldownItem, PulldownItemStyle,
};
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
    let popup2_open = state.popup2_open;
    let popup2_selected = state.popup2_selected.clone();
    let pulldown2_open = state.pulldown2_open;

    let entity_popup_toggle = entity.clone();
    let entity_popup_change = entity.clone();
    let entity_pulldown_toggle = entity.clone();

    div()
        .id("popup-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Pop-up and Pull-down Buttons"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A pop-up button shows a menu of mutually exclusive options. \
                     A pull-down button presents a menu of actions.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Pop-up Button"),
        )
        .child(
            PopupButton::new("popup-1")
                .items(vec![
                    PopupItem::new("Light", "light"),
                    PopupItem::new("Dark", "dark"),
                    PopupItem::new("Auto", "auto"),
                ])
                .selected(popup2_selected)
                .open(popup2_open)
                .on_toggle(move |open, _window, cx| {
                    entity_popup_toggle.update(cx, |this, cx| {
                        this.popup2_open = open;
                        cx.notify();
                    });
                })
                .on_change(move |value, _window, cx| {
                    entity_popup_change.update(cx, |this, cx| {
                        this.popup2_selected = value.clone();
                        this.popup2_open = false;
                        cx.notify();
                    });
                }),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style_emphasized(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Pull-down Button"),
        )
        .child(
            PulldownButton::new("pulldown-1", "Actions")
                .open(pulldown2_open)
                .on_toggle(move |open, _window, cx| {
                    entity_pulldown_toggle.update(cx, |this, cx| {
                        this.pulldown2_open = open;
                        cx.notify();
                    });
                })
                .item(PulldownItem::new("Open\u{2026}"))
                .item(PulldownItem::new("Save"))
                .item(PulldownItem::new("Save As\u{2026}"))
                .item(PulldownItem::new("Delete").style(PulldownItemStyle::Destructive)),
        )
        .into_any_element()
}
