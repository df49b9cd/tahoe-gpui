//! FKA (Full Keyboard Access) attachment for chart data points.

use gpui::prelude::*;
use gpui::{FocusHandle, SharedString, Window, px};

use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup, FocusGroupExt,
};
use crate::foundations::layout::{ControlSize, hit_region};
use crate::foundations::materials::apply_focus_ring;

/// Context shared between the bar and point FKA attachment paths.
///
/// Per-point VoiceOver labels live in `labels` — a slice indexed by the
/// same contiguous `index` passed to [`attach_fka`]. Precomputing the
/// strings in `Chart::render` keeps the format! calls out of the paint
/// path, so scrolling a 100-point multi-series chart doesn't rebuild
/// every label on each redraw.
pub(crate) struct FkaAttachContext<'a> {
    pub group: &'a FocusGroup,
    pub handles: &'a [FocusHandle],
    pub prefix: &'a SharedString,
    pub total: usize,
    pub theme: &'a crate::foundations::theme::TahoeTheme,
    pub labels: &'a [SharedString],
}

/// Wire a bar or point div up for Full Keyboard Access: per-value element
/// id, focus-group registration, per-value VoiceOver label, focus ring,
/// and arrow/Home/End key handling.
pub(crate) fn attach_fka(
    el: gpui::Div,
    ctx: &FkaAttachContext,
    index: usize,
    window: &Window,
) -> gpui::AnyElement {
    let is_focused = ctx.handles[index].is_focused(window);
    // C2: Use DataPoint role instead of Button — chart data points are not
    // activatable buttons. C3: Populate posinset/setsize so VoiceOver can
    // announce "row 1 of 5" structurally.
    let a11y = AccessibilityProps::new()
        .label(ctx.labels[index].clone())
        .role(AccessibilityRole::DataPoint)
        .posinset(index + 1)
        .setsize(ctx.total);

    // C4: Expand the hit target to at least the platform's minimum control
    // size so focus rings render at a reasonable dimension and pointer
    // users can click comfortably.
    let min_target = px(ctx.theme.control_height(ControlSize::Small));
    let group_for_keys = ctx.group.clone();
    let el = hit_region(
        min_target,
        el.id((ctx.prefix.clone(), index))
            .focus_group(ctx.group, &ctx.handles[index])
            .with_accessibility(&a11y)
            .on_key_down(move |ev: &gpui::KeyDownEvent, window, cx| {
                match ev.keystroke.key.as_str() {
                    "left" | "up" => {
                        group_for_keys.focus_previous(window, cx);
                        cx.stop_propagation();
                    }
                    "right" | "down" => {
                        group_for_keys.focus_next(window, cx);
                        cx.stop_propagation();
                    }
                    "home" => {
                        group_for_keys.focus_first(window, cx);
                        cx.stop_propagation();
                    }
                    "end" => {
                        group_for_keys.focus_last(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }),
    );
    apply_focus_ring(el, ctx.theme, is_focused, &[]).into_any_element()
}
