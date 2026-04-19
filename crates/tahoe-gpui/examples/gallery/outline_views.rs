//! OutlineView demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::layout_and_organization::outline_view::{OutlineNode, OutlineView};
use tahoe_gpui::foundations::icons::IconName;
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
    let expanded = &state.outline_expanded;

    let tree = vec![
        OutlineNode::new("src", "src")
            .icon(IconName::Folder)
            .expanded(expanded.contains("src"))
            .children(vec![
                OutlineNode::new("main", "main.rs").icon(IconName::File),
                OutlineNode::new("lib", "lib.rs").icon(IconName::File),
                OutlineNode::new("components", "components")
                    .icon(IconName::Folder)
                    .expanded(expanded.contains("components"))
                    .children(vec![
                        OutlineNode::new("button", "button.rs").icon(IconName::File),
                        OutlineNode::new("modal", "modal.rs").icon(IconName::File),
                    ]),
            ]),
        OutlineNode::new("cargo", "Cargo.toml").icon(IconName::File),
        OutlineNode::new("readme", "README.md").icon(IconName::File),
    ];

    div()
        .id("outline-views-pane")
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
                        .child("Outline Views"),
                )
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child(
                            "An outline view displays hierarchical data in a \
                             tree structure with expandable nodes. Click a \
                             folder to expand or collapse it.",
                        ),
                )
                .child(OutlineView::new("demo-outline").nodes(tree).on_toggle(
                    move |node_id, _window, cx| {
                        entity.update(cx, |this, cx| {
                            let id = node_id.to_string();
                            if this.outline_expanded.contains(&id) {
                                this.outline_expanded.remove(&id);
                            } else {
                                this.outline_expanded.insert(id);
                            }
                            cx.notify();
                        });
                    },
                )),
        )
        .into_any_element()
}
