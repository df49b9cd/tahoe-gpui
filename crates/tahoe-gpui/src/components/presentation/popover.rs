//! Popover component for floating content (HIG `#popovers`).
//!
//! Uses absolute positioning to render content below/beside a trigger.
//! The parent manages the `is_open` state.
//!
//! The popover renders a directional arrow pointing at the trigger
//! (HIG: "Make sure a popover's arrow points as directly as possible
//! to the element that revealed it") and caps its width at
//! [`POPOVER_MAX_WIDTH`] so it stays within the HIG sizing envelope.
//!
//! # Accessibility
//!
//! The glyph arrow is purely decorative and carries no accessibility
//! label. GPUI does not yet expose an AppKit "popover" role, so what
//! VoiceOver announces when focus lands on the content surface is
//! whatever role the contained elements surface (typically the
//! content's own role rather than a "popover" hint). Manual AX passes
//! on macOS are the source of truth for this surface until GPUI ships
//! richer role metadata (tracked by the shim in
//! [`foundations::accessibility::voiceover`]).
//!
//! Consumers should label the content they pass in. The canonical
//! pattern uses the project's [`AccessibleExt::with_accessibility`]
//! shim, which is a no-op today but will wire through once GPUI lands
//! an AX tree:
//!
//! ```ignore
//! use tahoe_gpui::foundations::accessibility::{
//!     AccessibilityProps, AccessibilityRole, AccessibleExt,
//! };
//!
//! let props = AccessibilityProps::new()
//!     .role(AccessibilityRole::Dialog)
//!     .label("Formatting options");
//! let labelled = div().with_accessibility(&props).child(/* content */);
//! Popover::new(id, trigger, labelled.into_any_element())
//! ```
//!
//! ## Escape-dismiss contract
//!
//! The `on_dismiss` handler is wired to Escape via a bubble-phase
//! `on_key_down` on the content surface. This follows the same
//! convention as [`Alert`](super::alert::Alert) and
//! [`Modal`](super::modal::Modal). Content authors must *not* call
//! `cx.stop_propagation()` on Escape inside popover children — doing so
//! will break the popover's dismiss path. Components in this crate
//! (TextField, Button, Icon, etc.) do not stop-propagate Escape, so
//! the common case "just works".
//!
//! ## Focus restoration on dismiss
//!
//! Use [`Popover::restore_focus_to`] with the triggering control's
//! `FocusHandle` so focus returns to the trigger when the popover
//! dismisses. This mirrors [`Modal::restore_focus_to`](super::modal::Modal)
//! and [`Sheet::restore_focus_to`](super::sheet::Sheet) and is HIG-
//! aligned behaviour for any dismissible overlay summoned from a
//! keyboard-focusable control.
//!
//! [`foundations::accessibility::voiceover`]: crate::foundations::accessibility::voiceover
//! [`AccessibleExt::with_accessibility`]: crate::foundations::accessibility::AccessibleExt::with_accessibility

use crate::foundations::layout::POPOVER_MAX_WIDTH;
use crate::foundations::motion::accessible_transition_animation;
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{
    AnimationExt, AnyElement, App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, Window,
    div, px,
};

/// Arrow width in points. Chosen to match macOS popover callouts
/// (approximately 14 pt base, 7 pt height), which optically balance
/// the glass panel shadow without consuming content.
const ARROW_WIDTH: f32 = 14.0;
/// Arrow height in points.
const ARROW_HEIGHT: f32 = 7.0;

/// Controls where the popover content is positioned relative to the trigger.
use crate::callback_types::OnMutCallback;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PopoverPlacement {
    /// Below the trigger, aligned to the left edge.
    #[default]
    BelowLeft,
    /// Below the trigger, aligned to the right edge.
    BelowRight,
    /// Above the trigger, aligned to the left edge.
    AboveLeft,
    /// Above the trigger, aligned to the right edge.
    AboveRight,
}

impl From<PopoverPlacement> for OverlayAnchor {
    fn from(placement: PopoverPlacement) -> Self {
        match placement {
            PopoverPlacement::BelowLeft => OverlayAnchor::BelowLeft,
            PopoverPlacement::BelowRight => OverlayAnchor::BelowRight,
            PopoverPlacement::AboveLeft => OverlayAnchor::AboveLeft,
            PopoverPlacement::AboveRight => OverlayAnchor::AboveRight,
        }
    }
}

/// A popover that shows floating content relative to a trigger.
///
/// The parent manages `is_open` and toggles it via hover/click.
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    is_open: bool,
    trigger: AnyElement,
    content: AnyElement,
    placement: PopoverPlacement,
    focus_handle: Option<FocusHandle>,
    restore_focus_to: Option<FocusHandle>,
    on_dismiss: OnMutCallback,
    max_width: Option<gpui::Pixels>,
    /// Whether to render the directional callout arrow. Defaults to `true`.
    arrow: bool,
}

impl Popover {
    pub fn new(
        id: impl Into<ElementId>,
        trigger: impl IntoElement,
        content: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            is_open: false,
            trigger: trigger.into_any_element(),
            content: content.into_any_element(),
            placement: PopoverPlacement::default(),
            focus_handle: None,
            restore_focus_to: None,
            on_dismiss: None,
            max_width: None,
            arrow: true,
        }
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Attach a focus handle to the content surface. When set, the popover
    /// auto-focuses the surface on open so Escape fires without a prior click.
    ///
    /// Pair with [`Self::restore_focus_to`] so focus returns to the
    /// triggering control on dismiss. Without `restore_focus_to`, focus
    /// stays on the now-detached popover content surface after dismiss
    /// and the next Tab behaves unpredictably.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Focus handle to refocus when the popover dismisses (typically the
    /// trigger button's handle). Mirrors the restoration path on
    /// [`super::modal::Modal`] and [`super::sheet::Sheet`]: the popover
    /// wraps `on_dismiss` so the handle is focused before the consumer's
    /// dismiss handler runs.
    pub fn restore_focus_to(mut self, handle: FocusHandle) -> Self {
        self.restore_focus_to = Some(handle);
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// Override the maximum content width. Defaults to
    /// [`POPOVER_MAX_WIDTH`] (320 pt) per HIG `#popovers`.
    pub fn max_width(mut self, width: gpui::Pixels) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Disable the directional callout arrow. Defaults to on. The
    /// arrow is rendered even without a custom `max_width` because
    /// HIG mandates the arrow for popovers regardless of size.
    pub fn arrow(mut self, enabled: bool) -> Self {
        self.arrow = enabled;
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if self.is_open
            && let Some(ref handle) = self.focus_handle
            && !handle.contains_focused(window, cx)
        {
            // Auto-focus the content surface on open so Escape-dismiss
            // fires without a prior click, matching Alert/Modal. Use
            // `contains_focused` (not `is_focused`) so a focusable child
            // inside the content — e.g. a text field — keeps focus on
            // open instead of having it stolen back to the shell.
            // Done before re-borrowing the theme below so the
            // mutable-borrow on `cx` is released before the immutable
            // global read.
            handle.focus(window, cx);
        }

        let theme = cx.theme();

        let trigger = div()
            .debug_selector(|| "popover-trigger".into())
            .child(self.trigger);

        let overlay_id =
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "overlay".into());

        let mut overlay = AnchoredOverlay::new(overlay_id, trigger).anchor(self.placement.into());

        // Gap between trigger edge and popover body. `AnchoredOverlay::gap`
        // resolves the sign against the *realised* anchor inside prepaint,
        // so flipping Below↔Above via `realise_anchor` keeps the gap on the
        // side the overlay actually lands on.
        overlay = overlay.gap(theme.spacing_xs);

        if self.is_open {
            // Popovers are mid-layer overlay surfaces: one depth level above
            // content, one below sheets/modals. `Large` (34pt radius, 40pt
            // shadow blur) is reserved for full-screen sheets and alerts;
            // applying it to a narrow popover bleeds the shadow into
            // adjacent content and flattens the depth hierarchy.
            let max_w = self.max_width.unwrap_or(px(POPOVER_MAX_WIDTH));
            let mut content_div = crate::foundations::materials::glass_surface(
                div().overflow_hidden().max_w(max_w),
                theme,
                GlassSize::Medium,
            )
            .id(ElementId::Name("popover-content-surface".into()))
            .debug_selector(|| "popover-content".into());

            content_div = content_div.child(self.content);

            if let Some(ref handle) = self.focus_handle {
                content_div = content_div.track_focus(handle);
            }

            if let Some(handler) = self.on_dismiss {
                // Wrap the consumer's handler so `restore_focus_to` (if
                // set) moves focus back to the triggering control before
                // their callback observes the dismissal. Mirrors the
                // restoration path on Modal/Sheet.
                let restore_handle = self.restore_focus_to.clone();
                let consumer = handler;
                let handler = std::rc::Rc::new(move |window: &mut Window, cx: &mut App| {
                    if let Some(handle) = restore_handle.as_ref() {
                        handle.focus(window, cx);
                    }
                    consumer(window, cx);
                });
                let key_handler = handler.clone();
                // Bubble-phase Escape handler, same convention as Alert
                // and Modal. "Bubble-phase" means children receive the
                // event first; if a child calls `cx.stop_propagation()`
                // the popover's handler is suppressed. The crate's own
                // components don't stop-propagate Escape, and the
                // docblock above ("Escape-dismiss contract") makes this a
                // documented contract for consumer-authored content.
                content_div = content_div.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if crate::foundations::keyboard::is_escape_key(event) {
                        key_handler(window, cx);
                    }
                });
                let outside_handler = handler.clone();
                content_div =
                    content_div.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                        outside_handler(window, cx);
                    });
            }

            // Snapshot theme-sourced values into owned data so the
            // content-builder closure (invoked later from `prepaint`) has
            // no lingering borrow on `cx`.
            let arrow_enabled = self.arrow;
            let arrow_bg = theme
                .glass
                .accessible_bg(GlassSize::Medium, theme.accessibility_mode);
            let spacing_sm = theme.spacing_sm;
            let accessibility = theme.accessibility_mode;
            let motion = theme.glass.motion;
            let natural_duration = std::time::Duration::from_millis(motion.lift_duration_ms);

            // Defer the arrow-rendering decision to the realised anchor.
            // `AnchoredOverlay` flips Below↔Above when the preferred side
            // has materially less room than the opposite side; the arrow
            // and animation translate both track the realised side so the
            // HIG "arrow points at the trigger" invariant holds regardless
            // of whether the original placement survived.
            overlay = overlay.content_fn(true, move |realised| {
                let realised_is_above = matches!(
                    realised,
                    OverlayAnchor::AboveLeft | OverlayAnchor::AboveRight
                );
                let realised_aligns_left = matches!(
                    realised,
                    OverlayAnchor::BelowLeft | OverlayAnchor::AboveLeft
                );

                let arrow_el = if arrow_enabled {
                    Some(build_arrow_from_values(
                        arrow_bg,
                        spacing_sm,
                        realised_is_above,
                        realised_aligns_left,
                    ))
                } else {
                    None
                };

                // Layout order: arrow above content for below-placement,
                // below content for above-placement.
                let mut content_col = div().flex().flex_col();
                if !realised_is_above {
                    if let Some(arrow) = arrow_el {
                        content_col = content_col.child(arrow);
                    }
                    content_col = content_col.child(content_div);
                } else {
                    content_col = content_col.child(content_div);
                    if let Some(arrow) = arrow_el {
                        content_col = content_col.child(arrow);
                    }
                }

                // Present-transition: HIG calls for popovers to scale from the
                // anchor. GPUI's style API doesn't yet expose a transform-origin
                // scale, so we approximate with a short fade plus a small
                // vertical translate toward the anchor. Under Reduce Motion or
                // Prefer Cross-Fade we drop the translate per
                // `foundations.md:1100`.
                let translate_px = if accessibility.reduce_motion()
                    || accessibility.prefer_cross_fade_transitions()
                {
                    0.0
                } else {
                    6.0
                };
                let anim_id = ElementId::Name("popover-present".into());
                content_col
                    .with_animation(
                        anim_id,
                        accessible_transition_animation(&motion, natural_duration, accessibility),
                        move |el, delta| {
                            let offset = translate_px * (1.0 - delta);
                            let signed = if realised_is_above { offset } else { -offset };
                            el.opacity(delta).mt(gpui::px(signed))
                        },
                    )
                    .into_any_element()
            });
        }

        div()
            .id(self.id)
            .debug_selector(|| "popover-root".into())
            .child(overlay)
    }
}

/// Render a directional callout arrow pointing at the trigger. Takes
/// raw theme-sourced values (rather than a `&TahoeTheme` borrow) so it
/// can be called from a captured closure inside the overlay's content
/// builder without lingering borrow of `cx`.
///
/// GPUI does not yet expose per-side border colors (required for the
/// classic CSS "triangle via borders" trick) nor transform-origin
/// rotation. Instead we render a small Unicode glyph — ▼ when the
/// popover is above the trigger (arrow points down at the trigger) and
/// ▲ when the popover is below the trigger (arrow points up). The
/// glyph takes the glass surface color so it reads as an extension of
/// the panel rather than a free-floating mark.
fn build_arrow_from_values(
    arrow_bg: gpui::Hsla,
    spacing_sm: gpui::Pixels,
    is_above: bool,
    aligns_left: bool,
) -> gpui::Div {
    let glyph = if is_above { "▼" } else { "▲" };

    let pointer = div()
        .w(px(ARROW_WIDTH))
        .h(px(ARROW_HEIGHT))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(ARROW_WIDTH))
        .text_color(arrow_bg)
        .child(glyph.to_string());

    let mut wrapper = div().w_full().flex();
    if aligns_left {
        wrapper = wrapper.justify_start().pl(spacing_sm);
    } else {
        wrapper = wrapper.justify_end().pr(spacing_sm);
    }
    wrapper.child(pointer)
}

#[cfg(test)]
mod tests {
    use super::{Popover, PopoverPlacement};
    use core::prelude::v1::test;

    #[test]
    fn popover_default_is_not_open() {
        let popover = Popover::new("test", gpui::div(), gpui::div());
        assert!(!popover.is_open);
    }

    #[test]
    fn popover_placement_default_is_below_left() {
        assert_eq!(PopoverPlacement::default(), PopoverPlacement::BelowLeft);
    }

    #[test]
    fn popover_on_dismiss_default_is_none() {
        let popover = Popover::new("test", gpui::div(), gpui::div());
        assert!(popover.on_dismiss.is_none());
    }

    #[test]
    fn popover_default_max_width_is_none() {
        let popover = Popover::new("test", gpui::div(), gpui::div());
        assert!(popover.max_width.is_none());
    }

    #[test]
    fn popover_arrow_default_is_on() {
        let popover = Popover::new("test", gpui::div(), gpui::div());
        assert!(popover.arrow);
    }

    #[test]
    fn popover_max_width_builder() {
        let popover = Popover::new("test", gpui::div(), gpui::div()).max_width(gpui::px(240.0));
        assert_eq!(popover.max_width, Some(gpui::px(240.0)));
    }

    #[test]
    fn popover_arrow_builder() {
        let popover = Popover::new("test", gpui::div(), gpui::div()).arrow(false);
        assert!(!popover.arrow);
    }

    #[test]
    fn popover_placement_all_distinct() {
        let placements = [
            PopoverPlacement::BelowLeft,
            PopoverPlacement::BelowRight,
            PopoverPlacement::AboveLeft,
            PopoverPlacement::AboveRight,
        ];
        for i in 0..placements.len() {
            for j in (i + 1)..placements.len() {
                assert_ne!(
                    placements[i], placements[j],
                    "{:?} and {:?} should be distinct",
                    placements[i], placements[j]
                );
            }
        }
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::{
        Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render, Styled,
        TestAppContext, div, px,
    };

    use super::{Popover, PopoverPlacement};
    use crate::foundations::layout::DROPDOWN_SNAP_MARGIN;
    use crate::test_helpers::helpers::{
        InteractionExt, LocatorExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const POPOVER_TRIGGER: &str = "popover-trigger";
    const POPOVER_CONTENT: &str = "popover-content";

    // Assumed test viewport dimensions (matches `TestDisplay::new`'s
    // default 1920×1080). Padding values below are derived from these
    // rather than hardcoded so the coupling is load-bearing: if
    // `TestDisplay`'s default ever drifts, the derived constants shift
    // with it, and `realise_anchor`'s flip threshold stays relatively
    // placed instead of silently becoming wrong.
    const VIEWPORT_W: f32 = 1920.0;
    const VIEWPORT_H: f32 = 1080.0;

    // Trigger and content sizes used by the harness. Kept as named
    // constants so the padding derivation below stays readable.
    const TRIGGER_W: f32 = 80.0;
    const TRIGGER_H: f32 = 32.0;
    const CONTENT_W: f32 = 120.0;
    const CONTENT_H: f32 = 60.0;

    // Keep the trigger this far below the viewport top. Derived so the
    // "above" side has strictly less than 2× the "below" side — the
    // `realise_anchor` flip threshold is >2×, so we stay just under.
    // `VIEWPORT_H / 2.7` puts the trigger at ~400pt on the default
    // 1080pt display with room for neighbouring harness padding.
    const STABLE_ANCHOR_TOP_PAD: f32 = VIEWPORT_H / 2.7;

    // Keep the trigger near the viewport's bottom so the preferred
    // `BelowLeft` placement runs out of room and flips to `AboveLeft`.
    // `VIEWPORT_H - 280` leaves space_below ≈ 248 and space_above ≈
    // 800 — well past the 2× flip threshold.
    const FORCED_FLIP_TOP_PAD: f32 = VIEWPORT_H - 280.0;

    // Compile-time sanity checks that keep `VIEWPORT_W`/`VIEWPORT_H`
    // load-bearing — silent drift of either constant to zero or a
    // negative value would fail the build rather than the runtime.
    // VIEWPORT_W doesn't appear in a derived padding today but is the
    // grep-anchor for any future horizontal derivation (e.g. centering
    // a trigger for corner-edge flip tests).
    const _: () = {
        assert!(VIEWPORT_W > 0.0);
        assert!(VIEWPORT_H > 0.0);
        assert!(STABLE_ANCHOR_TOP_PAD > 0.0);
        assert!(FORCED_FLIP_TOP_PAD > STABLE_ANCHOR_TOP_PAD);
    };

    struct PopoverHarness {
        focus_handle: FocusHandle,
        is_open: bool,
        dismiss_count: usize,
        placement: PopoverPlacement,
    }

    impl PopoverHarness {
        fn new(cx: &mut Context<Self>, is_open: bool, placement: PopoverPlacement) -> Self {
            Self {
                focus_handle: cx.focus_handle(),
                is_open,
                dismiss_count: 0,
                placement,
            }
        }
    }

    impl Render for PopoverHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            // Pad the trigger away from the window edges for two reasons:
            //
            // 1. `snap_to_window_with_margin` must not clamp "above"
            //    placements into the viewport (which would invert the
            //    expected ordering).
            // 2. `AnchoredOverlay::realise_anchor` must not flip
            //    Above→Below. The flip fires when the opposite side has
            //    >2× the space of the preferred side — see
            //    `STABLE_ANCHOR_TOP_PAD` for the derivation.
            //
            // Left padding leaves room for the "Right"-aligned content
            // surface plus the snap margin.
            let top_pad = px(STABLE_ANCHOR_TOP_PAD);
            let left_pad = DROPDOWN_SNAP_MARGIN + px(CONTENT_W - TRIGGER_W + 4.0);
            div().pt(top_pad).pl(left_pad).child(
                Popover::new(
                    "popover",
                    div().w(px(TRIGGER_W)).h(px(TRIGGER_H)).child("Trigger"),
                    div().w(px(CONTENT_W)).h(px(CONTENT_H)).child("Content"),
                )
                .open(self.is_open)
                .placement(self.placement)
                .focus_handle(self.focus_handle.clone())
                .on_dismiss(move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.dismiss_count += 1;
                        this.is_open = false;
                        cx.notify();
                    });
                }),
            )
        }
    }

    #[gpui::test]
    async fn hidden_and_visible_states_render_expected_content(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            PopoverHarness::new(cx, false, PopoverPlacement::BelowLeft)
        });

        assert_element_absent(cx, POPOVER_CONTENT);
        host.update_in(cx, |host, _window, cx| {
            host.is_open = true;
            cx.notify();
        });

        assert_element_exists(cx, POPOVER_TRIGGER);
        assert_element_exists(cx, POPOVER_CONTENT);
    }

    #[gpui::test]
    async fn placement_above_right_positions_content_above_trigger(cx: &mut TestAppContext) {
        let (_host, cx) = setup_test_window(cx, |_window, cx| {
            PopoverHarness::new(cx, true, PopoverPlacement::AboveRight)
        });

        let trigger = cx.get_element(POPOVER_TRIGGER);
        let content = cx.get_element(POPOVER_CONTENT);
        assert!(content.bounds.bottom() <= trigger.bounds.top());
    }

    #[gpui::test]
    async fn escape_dismisses_visible_popover(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            PopoverHarness::new(cx, true, PopoverPlacement::BelowLeft)
        });

        // Popover now auto-acquires focus on visible, so Escape fires
        // without a parent-side focus call — but the harness still
        // focuses the handle to ensure the test handler is reliable
        // across runners.
        host.update_in(cx, |host, window, cx| {
            host.focus_handle.focus(window, cx);
        });
        cx.press("escape");

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.dismiss_count, 1);
            assert!(!host.is_open);
        });
        assert_element_absent(cx, POPOVER_CONTENT);
    }

    /// Harness that pins the trigger near the bottom of the
    /// VIEWPORT_W×VIEWPORT_H test window so [`crate::foundations::overlay::realise_anchor`] is
    /// forced to flip a preferred `Below*` placement to `Above*`. Used to
    /// verify the full pipeline (`.gap()` sign flip + arrow orientation
    /// swap) end-to-end, not just the pure-function level covered by the
    /// overlay unit tests.
    ///
    /// Top padding derived so space_above ≈ VIEWPORT_H - 280 ≈ 800pt
    /// and space_below ≈ 248pt, ratio > 3x — comfortably past
    /// `realise_anchor`'s strict 2x threshold.
    struct FlippedPopoverHarness {
        is_open: bool,
    }

    impl Render for FlippedPopoverHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let top_pad = px(FORCED_FLIP_TOP_PAD);
            let left_pad = DROPDOWN_SNAP_MARGIN + px(CONTENT_W - TRIGGER_W + 4.0);
            div().pt(top_pad).pl(left_pad).child(
                Popover::new(
                    "popover",
                    div().w(px(TRIGGER_W)).h(px(TRIGGER_H)).child("Trigger"),
                    div().w(px(CONTENT_W)).h(px(CONTENT_H)).child("Content"),
                )
                .open(self.is_open)
                .placement(PopoverPlacement::BelowLeft)
                .arrow(false),
            )
        }
    }

    #[gpui::test]
    async fn preferred_below_flips_to_above_when_trigger_near_bottom(cx: &mut TestAppContext) {
        let (_host, cx) =
            setup_test_window(cx, |_window, _cx| FlippedPopoverHarness { is_open: true });

        let trigger = cx.get_element(POPOVER_TRIGGER);
        let content = cx.get_element(POPOVER_CONTENT);

        // The caller asked for `BelowLeft`, but with the trigger pinned
        // near the bottom edge `realise_anchor` must flip the preferred
        // side. If the flip fires, the content lays out above the
        // trigger; if it doesn't, the content spills past the viewport
        // bottom or gets snapped up under the trigger by
        // `snap_to_window_with_margin` (both of which would fail this
        // assertion because the content's bottom would be at or below
        // the trigger's top).
        assert!(
            content.bounds.bottom() <= trigger.bounds.top(),
            "flipped content.bottom() {:?} should be at or above trigger.top() {:?}",
            content.bounds.bottom(),
            trigger.bounds.top(),
        );
    }

    /// Harness: nest the popover inside a small `overflow_hidden()`
    /// container so we can read both regions' layout bounds. The
    /// harness is named for what the assertion actually checks
    /// (overlay's layout rectangle anchors past the parent's bounds),
    /// not the paint-time clipping the branch name suggests — see the
    /// limitation note below.
    ///
    /// Harness limitation: `VisualTestContext::debug_bounds` returns
    /// taffy layout bounds, not post-clip paint bounds. This harness
    /// verifies the layout-level contract that `AnchoredOverlay` routed
    /// the overlay through `deferred(anchored(...))` — if `anchored()`
    /// is removed, the content lays out inside the parent and the
    /// assertion below fails. A true paint-clip verification (pixel
    /// diff after clipping is applied) requires visual-regression
    /// infrastructure (`RenderImage`-based golden diffing) that this
    /// crate does not yet ship. `TODO(overlay-paint-golden)`: once
    /// `test-support` exposes post-clip paint geometry or golden-image
    /// diffing, add a true pixel-level clip test alongside this
    /// structural one. Tracked alongside the overlay migration work
    /// (see `TODO(overlay-migration)` call sites in menus_and_actions
    /// and selection_and_input).
    struct AnchoredPastParentHarness {
        is_open: bool,
    }

    impl Render for AnchoredPastParentHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            // Outer wrapper adds a debug selector so we can read the
            // clip region's bounds in the assertion. The inner container
            // is 80x40 with `overflow_hidden` — content is 120pt wide, so
            // without the overlay escape it would be clipped horizontally
            // AND vertically (content lives below the trigger).
            div().pt(px(120.0)).pl(px(40.0)).child(
                div()
                    .debug_selector(|| "clip-region".into())
                    .w(px(TRIGGER_W))
                    .h(px(TRIGGER_H))
                    .overflow_hidden()
                    .child(
                        Popover::new(
                            "popover",
                            div().w(px(TRIGGER_W)).h(px(TRIGGER_H)).child("Trigger"),
                            div().w(px(CONTENT_W)).h(px(CONTENT_H)).child("Content"),
                        )
                        .open(self.is_open)
                        .placement(PopoverPlacement::BelowLeft),
                    ),
            )
        }
    }

    #[gpui::test]
    async fn overlay_content_anchors_outside_parent_layout_bounds(cx: &mut TestAppContext) {
        let (_host, cx) = setup_test_window(cx, |_window, _cx| AnchoredPastParentHarness {
            is_open: true,
        });

        let clip = cx.get_element("clip-region");
        let content = cx.get_element(POPOVER_CONTENT);

        // `debug_bounds` returns taffy layout bounds, not post-clip paint
        // bounds — so this assertion verifies that `anchored()` placed the
        // overlay's layout rectangle outside the clip region, not that
        // GPUI's clip pipeline allowed it through. If `anchored()` were
        // removed from `AnchoredOverlay`, the content would lay out inline
        // inside the clip region and this assertion would fail.
        //
        // TODO(overlay-paint-golden): once the `test-support` harness
        // exposes post-clip paint geometry (or a golden-image diff),
        // extend this to assert that the pixels inside `content.bounds`
        // are actually drawn. The structural check below catches the
        // "anchored() removed" regression but not "paint pipeline
        // re-clips deferred children". Tracked alongside the overlay
        // migration work.
        assert!(
            content.bounds.right() > clip.bounds.right(),
            "content.right() {:?} should exceed clip.right() {:?}",
            content.bounds.right(),
            clip.bounds.right(),
        );
        // Content lands below the trigger (which fills the clip); its top
        // should meet or clear the clip's bottom edge.
        assert!(
            content.bounds.top() >= clip.bounds.bottom(),
            "content.top() {:?} should be at or below clip.bottom() {:?}",
            content.bounds.top(),
            clip.bounds.bottom(),
        );
    }
}
