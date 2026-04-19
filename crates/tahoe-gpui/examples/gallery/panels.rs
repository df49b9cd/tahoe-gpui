//! Panels demo — wires the live `Panel` component (issue #156 F-05).

use gpui::prelude::*;
use gpui::{AnyElement, App, Context, Window, div, px};

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::presentation::panel::{Panel, PanelPosition, PanelStyle};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let is_open = state.panel_open;

    let entity = cx.entity().downgrade();
    let dismiss = move |_window: &mut Window, cx: &mut App| {
        if let Some(this) = entity.upgrade() {
            this.update(cx, |this, cx| {
                this.panel_open = false;
                cx.notify();
            });
        }
    };

    let header = div()
        .px(theme.spacing_xl)
        .pt(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .flex_col()
        .gap(theme.spacing_xs)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Panels"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A panel is an `NSPanel`-style floating surface (Inspector, \
                     Fonts, Colors). It docks to the leading or trailing edge of \
                     its container. Click \u{201C}Open inspector\u{201D} to slide \
                     one in from the right.",
                ),
        );

    let toolbar = div()
        .px(theme.spacing_xl)
        .pb(theme.spacing_lg)
        .flex()
        .gap(theme.spacing_sm)
        .child(
            Button::new("panel-show")
                .label("Open inspector")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Md)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.panel_open = true;
                    cx.notify();
                })),
        );

    let panel: AnyElement = if is_open {
        let dismiss_for_close = dismiss.clone();
        let body = div()
            .p(theme.spacing_lg)
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_style_emphasized(TextStyle::Title3, theme)
                    .text_color(theme.text)
                    .child("Inspector"),
            )
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text_muted)
                    .child(
                        "The Panel component renders a glass surface anchored to \
                         the leading or trailing edge with an optional backdrop dim. \
                         HUD style omits the dim for tool-palette overlays.",
                    ),
            )
            .child(
                div()
                    .pt(theme.spacing_md)
                    .child(
                        Button::new("panel-close")
                            .label("Close")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Md)
                            .on_click(move |_, window, cx| dismiss_for_close(window, cx)),
                    ),
            );

        Panel::new("gallery-panel")
            .open(true)
            .position(PanelPosition::Right)
            .style(PanelStyle::Standard)
            .width(px(320.0))
            .on_dismiss(move |window, cx| dismiss(window, cx))
            .child(body)
            .into_any_element()
    } else {
        div().into_any_element()
    };

    div()
        .id("panels-pane")
        .relative()
        .size_full()
        .bg(theme.glass.root_bg)
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(header)
                .child(toolbar),
        )
        .child(panel)
        .into_any_element()
}
