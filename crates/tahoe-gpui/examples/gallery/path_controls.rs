//! Path Controls demo for the primitive gallery.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::navigation_and_search::path_control::{PathControl, PathSegment};
use tahoe_gpui::foundations::icons::IconName;
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
        .id("path-controls-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Path Controls"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A path control displays a breadcrumb trail of segments. \
                     The last segment represents the current location; earlier \
                     segments are interactive.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Simple path"),
        )
        .child(PathControl::new("pc-simple").segments(vec![
            PathSegment::new("Home"),
            PathSegment::new("Documents"),
            PathSegment::new("Project"),
        ]))
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("With icons"),
        )
        .child(PathControl::new("pc-icons").segments(vec![
            PathSegment::new("Root").icon(IconName::Folder),
            PathSegment::new("src").icon(IconName::Folder),
            PathSegment::new("main.rs").icon(IconName::File),
        ]))
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Headline, theme)
                .text_color(theme.text)
                .child("Single segment"),
        )
        .child(PathControl::new("pc-single").segments(vec![PathSegment::new("Dashboard")]))
        .into_any_element()
}
