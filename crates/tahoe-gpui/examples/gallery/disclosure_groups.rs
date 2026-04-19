//! DisclosureGroup demo (issue #156 F-09). Distinct from
//! `disclosure_controls.rs`, which exercises the bare `Disclosure` glyph;
//! `DisclosureGroup` couples a header + body with the keyboard cycle and
//! visual chrome HIG prescribes for collapsible sections.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::layout_and_organization::disclosure_group::DisclosureGroup;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;



pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let is_open = state.disclosure_open;

    let body = div()
        .pt(theme.spacing_xs)
        .pl(theme.spacing_lg)
        .flex()
        .flex_col()
        .gap(theme.spacing_xs)
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child("Camera roll"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child("Screenshots"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child("Selfies"),
        );

    let header = div()
        .text_style(TextStyle::Headline, theme)
        .text_color(theme.text)
        .child("Photos");

    div()
        .id("disclosure-groups-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Disclosure Groups"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A disclosure group reveals or hides a body section under a \
                     focusable header. Triangle direction is the sole open-state \
                     signal — the header chrome stays unchanged. Space / Return \
                     toggle, Right / Left arrows expand / collapse.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child({
            let entity = cx.entity().downgrade();
            DisclosureGroup::new("dg-photos", header, body)
                .open(is_open)
                .on_toggle(move |new_state, _window, cx| {
                    if let Some(this) = entity.upgrade() {
                        this.update(cx, |this, cx| {
                            this.disclosure_open = new_state;
                            cx.notify();
                        });
                    }
                })
        })
        .into_any_element()
}
