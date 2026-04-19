//! Pull-down Buttons demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::pulldown_button::{
    PulldownButton, PulldownItem, PulldownItemStyle,
};
use tahoe_gpui::foundations::icons::{Icon, IconName};
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
    let pulldown_open = state.pulldown_open;

    let entity_toggle = entity.clone();

    div()
        .id("pulldown-buttons-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Pull-down Buttons"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A pull-down button reveals an action menu. Unlike a pop-up button, \
                     each item fires its own action rather than representing selection.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Interactive"),
        )
        .child(
            PulldownButton::new("pd-closed", "Actions")
                .open(pulldown_open)
                .on_toggle(move |open, _window, cx| {
                    entity_toggle.update(cx, |this, cx| {
                        this.pulldown_open = open;
                        cx.notify();
                    });
                })
                .item(PulldownItem::new("Cut"))
                .item(PulldownItem::new("Copy"))
                .item(PulldownItem::new("Paste")),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("With icon and mixed styles"),
        )
        .child(
            PulldownButton::new("pd-icon", "Edit")
                .icon(Icon::new(IconName::Pencil).size(px(16.0)))
                .item(PulldownItem::new("Rename").icon(IconName::Pencil))
                .item(PulldownItem::new("Archive").icon(IconName::Folder))
                .item(
                    PulldownItem::new("Delete")
                        .icon(IconName::Trash)
                        .style(PulldownItemStyle::Destructive),
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Disabled"),
        )
        .child(
            PulldownButton::new("pd-disabled", "Unavailable")
                .disabled(true)
                .item(PulldownItem::new("Action")),
        )
        .into_any_element()
}
