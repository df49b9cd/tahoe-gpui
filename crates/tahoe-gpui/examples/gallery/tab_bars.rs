//! Tab Bars demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div, px};

use tahoe_gpui::components::navigation_and_search::tab_bar::{TabBar, TabItem};
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
    let active = state.tab_active.clone();

    let body = |text: &'static str| {
        div()
            .p(theme.spacing_md)
            .text_style(TextStyle::Body, theme)
            .text_color(theme.text)
            .child(text)
    };

    div()
        .id("tab-bars-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Tab Bars"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A tab bar organizes content into switchable sections. \
                     The parent manages the active tab and provides an on_change callback.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child(format!("Active tab: {active}")),
        )
        .child(
            TabBar::new("tb-interactive")
                .items(vec![
                    TabItem::new("general", "General", body("General settings content.")),
                    TabItem::new("advanced", "Advanced", body("Advanced settings content.")),
                    TabItem::new("about", "About", body("About this app.")),
                ])
                .active(active)
                .on_change(move |new_tab, _window, cx| {
                    entity.update(cx, |this, cx| {
                        this.tab_active = new_tab;
                        cx.notify();
                    });
                }),
        )
        .child(div().h(px(16.0)))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("4 tabs (static, third active)"),
        )
        .child(
            TabBar::new("tb-4")
                .items(vec![
                    TabItem::new("inbox", "Inbox", body("Your inbox messages.")),
                    TabItem::new("sent", "Sent", body("Sent messages.")),
                    TabItem::new("drafts", "Drafts", body("Draft messages.")),
                    TabItem::new("trash", "Trash", body("Deleted messages.")),
                ])
                .active("drafts"),
        )
        .into_any_element()
}
