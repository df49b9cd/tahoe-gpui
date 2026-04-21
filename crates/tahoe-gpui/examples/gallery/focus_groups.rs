//! Focus Groups demo — exercises `FocusGroup` in Cycle mode.
//!
//! Three focusable "option" rows are registered with a shared
//! `FocusGroup::cycle()`. Arrow-up / Arrow-down cycle focus vertically,
//! wrapping at the edges; Tab / Shift+Tab fall through to GPUI's native
//! tab-stop map (Cycle mode doesn't trap Tab). This is the *focus-
//! movement* substrate host apps reach for when building segmented
//! pickers, tab bars, or custom navigation rails — instead of hand-
//! rolling the wrap math every time. APG radio groups additionally
//! require selection-follows-focus (the selected option changes as
//! focus moves); `FocusGroup` only moves focus, so the selection
//! side has to be wired by the host.
//!
//! Only the vertical axis is bound in this demo. Host apps laying out a
//! horizontal row should mirror this and bind `left`/`right` to
//! `focus_previous`/`focus_next` instead; binding both axes to the same
//! group is usually a sign that two groups are needed (one per axis).

use gpui::prelude::*;
use gpui::{AnyElement, Context, KeyDownEvent, Stateful, Window, div, px};

use tahoe_gpui::foundations::accessibility::FocusGroupExt;
use tahoe_gpui::foundations::materials::apply_focus_ring;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;

    // Set — not append — so hosts whose member identity changes across
    // renders get a fresh, correctly-sized group instead of a list that
    // grows every frame.
    state.focus_group.set_members(&state.focus_group_handles);

    // Focus the first member on the first render so arrow keys move focus
    // from the first interaction rather than waiting for the user to Tab
    // in. The latch prevents the next render from stealing focus back.
    if !state.focus_group_initial_focused {
        state.focus_group.focus_first(window, cx);
        state.focus_group_initial_focused = true;
    }

    let group = state.focus_group.clone();
    let handles = state.focus_group_handles.clone();

    let option_row = |index: usize, label: &str| -> Stateful<gpui::Div> {
        let focused = handles[index].is_focused(window);
        let row = div()
            .id(("focus-group-option", index))
            .focus_group(&state.focus_group, &handles[index])
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .child(label.to_string()),
            );
        apply_focus_ring(row, theme, focused, &[])
    };

    div()
        .id("focus-groups-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        // Vertical-only: the options stack top-to-bottom, so only the
        // up/down arrow keys drive the group. Home / End jump to the
        // first / last row. A horizontal variant should swap these
        // bindings for `left`/`right`.
        .on_key_down(
            move |event: &KeyDownEvent, window, cx| match event.keystroke.key.as_str() {
                "down" => group.focus_next(window, cx),
                "up" => group.focus_previous(window, cx),
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
                     default), Cycle (wrap at edges for arrow-key nav — this demo), \
                     and Trap (swallow Tab so focus cannot escape — used by Modal). \
                     Use Up/Down arrows to cycle through the rows; Home/End jump \
                     to the first/last. Tab falls through to GPUI's native \
                     tab-order in Cycle mode; use Trap (Modal) if focus must not \
                     escape the group.",
                ),
        )
        .child(div().h(theme.spacing_sm))
        .child(option_row(0, "Option 1 — first"))
        .child(option_row(1, "Option 2 — middle"))
        .child(option_row(2, "Option 3 — last"))
        // Illustration of the "disabled members must be deregistered"
        // contract: this row renders as a dimmed option but is deliberately
        // NOT tracked on a focus handle nor added to `set_members`. Arrow
        // keys skip it entirely — which is the host-side pattern for
        // disabling a member. `FocusGroup` has no enabled/disabled bit of
        // its own; omission from the group is the API.
        .child(
            div()
                .flex()
                .items_center()
                .gap(theme.spacing_sm)
                .px(theme.spacing_md)
                .py(theme.spacing_sm)
                .bg(theme.surface)
                .rounded(theme.radius_md)
                .opacity(0.45)
                .child(
                    div()
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text_muted)
                        .child("Option 4 — disabled (omitted from set_members)"),
                ),
        )
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
