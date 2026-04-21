//! Focus Groups demo — exercises `FocusGroup` in Cycle mode.
//!
//! Three focusable "option" rows are registered with a shared
//! `FocusGroup::cycle()`. Arrow-down / Arrow-up (or Tab / Shift+Tab via
//! GPUI's native tab-stop map) cycle focus between them, wrapping at the
//! edges. This is the pattern host apps should reach for when building
//! radio groups, segmented pickers, or custom navigation rails — instead
//! of hand-rolling the wrap math every time.

use gpui::prelude::*;
use gpui::{AnyElement, Context, KeyDownEvent, Stateful, Window, div, px};

use tahoe_gpui::foundations::accessibility::FocusGroupExt;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    // Idempotent: safe to call every render.
    for handle in &state.focus_group_handles {
        state.focus_group.register(handle);
    }

    let group = state.focus_group.clone();
    let handles = state.focus_group_handles.clone();

    let option_row = |index: usize, label: &str| -> Stateful<gpui::Div> {
        let focused = handles[index].is_focused(window);
        let bg = if focused {
            theme.selected_bg
        } else {
            theme.surface
        };
        div()
            .id(("focus-group-option", index))
            .focus_group(&state.focus_group, &handles[index])
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .bg(bg)
            .rounded(theme.radius_md)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child(label.to_string()),
            )
    };

    div()
        .id("focus-groups-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .on_key_down(
            move |event: &KeyDownEvent, window, cx| match event.keystroke.key.as_str() {
                "down" | "right" => group.focus_next(window, cx),
                "up" | "left" => group.focus_previous(window, cx),
                "home" => group.focus_first(window, cx),
                "end" => group.focus_last(window, cx),
                _ => {}
            },
        )
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Focus Groups"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child(
                    "A FocusGroup bundles FocusHandles into a single focus-graph \
                     cluster. Three modes are available: Open (no edge behavior, \
                     default), Cycle (wrap at edges for arrow-key nav), and Trap \
                     (swallow Tab so focus cannot escape — used by Modal). \
                     Tab through the rows or use Arrow keys to cycle.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(option_row(0, "Option 1 — first"))
        .child(option_row(1, "Option 2 — middle"))
        .child(option_row(2, "Option 3 — last"))
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .child(format!(
                    "Mode: {:?} · Members: {} · Wraps at edges: {}",
                    state.focus_group.mode(),
                    state.focus_group.len(),
                    !matches!(
                        state.focus_group.mode(),
                        tahoe_gpui::foundations::accessibility::FocusGroupMode::Open
                    ),
                )),
        )
        .w(px(560.0))
        .into_any_element()
}
