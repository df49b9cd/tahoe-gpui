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
//! The glyph arrow is purely decorative — it has no accessibility
//! label because VoiceOver already announces "popover" as a role hint
//! when the focused content surface is acquired. Consumers should
//! supply semantic labeling on the `content` they pass in; the popover
//! shell adds no announceable text of its own. VoiceOver behaviour is
//! not yet verified end-to-end in automated tests, so manual AX passes
//! on macOS remain the source of truth for this surface.

use crate::foundations::layout::POPOVER_MAX_WIDTH;
use crate::foundations::motion::accessible_transition_animation;
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor, child_id};
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{
    AnimationExt, AnyElement, App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, Window,
    div, point, px,
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

impl PopoverPlacement {
    /// Whether this placement renders the popover above the trigger.
    fn is_above(self) -> bool {
        matches!(self, Self::AboveLeft | Self::AboveRight)
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

    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
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
            && !handle.is_focused(window)
        {
            // Auto-focus the content surface on open so Escape-dismiss
            // fires without a prior click, matching Alert/Modal. Done
            // before re-borrowing the theme below so the mutable-borrow
            // on `cx` is released before the immutable global read.
            handle.focus(window, cx);
        }

        let theme = cx.theme();

        let trigger = div()
            .debug_selector(|| "popover-trigger".into())
            .child(self.trigger);

        let overlay_id = child_id(&self.id, "overlay");

        let mut overlay = AnchoredOverlay::new(overlay_id, trigger).anchor(match self.placement {
            PopoverPlacement::BelowLeft => OverlayAnchor::BelowLeft,
            PopoverPlacement::BelowRight => OverlayAnchor::BelowRight,
            PopoverPlacement::AboveLeft => OverlayAnchor::AboveLeft,
            PopoverPlacement::AboveRight => OverlayAnchor::AboveRight,
        });

        // Gap between trigger edge and popover body. For below-placements
        // the anchored positioner offsets the content downward; for
        // above-placements upward. We key the direction off the preferred
        // placement; if `AnchoredOverlay` later flips the realised anchor
        // in prepaint, the offset still points in the right general
        // direction because the `anchored()` corner flips with it.
        let gap: gpui::Pixels = theme.spacing_xs;
        let gap_y = if self.placement.is_above() { -gap } else { gap };
        overlay = overlay.offset(point(px(0.0), gap_y));

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
                let handler = std::rc::Rc::new(handler);
                let key_handler = handler.clone();
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
            let motion = theme.glass.motion.clone();
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

    #[test]
    fn popover_is_above_classifies_placements() {
        assert!(PopoverPlacement::AboveLeft.is_above());
        assert!(PopoverPlacement::AboveRight.is_above());
        assert!(!PopoverPlacement::BelowLeft.is_above());
        assert!(!PopoverPlacement::BelowRight.is_above());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    use super::{Popover, PopoverPlacement};
    use crate::foundations::layout::DROPDOWN_SNAP_MARGIN;
    use crate::test_helpers::helpers::{
        InteractionExt, LocatorExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const POPOVER_TRIGGER: &str = "popover-trigger";
    const POPOVER_CONTENT: &str = "popover-content";

    // Trigger and content sizes used by the harness. Kept as named
    // constants so the padding derivation below stays readable.
    const TRIGGER_W: f32 = 80.0;
    const TRIGGER_H: f32 = 32.0;
    const CONTENT_W: f32 = 120.0;
    const CONTENT_H: f32 = 60.0;

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
            //    >2× the space of the preferred side. The default test
            //    display is 1920×1080 (see `TestDisplay::new`), so
            //    `trigger.origin.y >= (1080 - TRIGGER_H) / 3 ≈ 349pt`
            //    keeps the preferred anchor stable.
            //
            // 400pt covers both constraints with headroom; left padding
            // leaves room for the "Right"-aligned content surface.
            let top_pad = px(400.0);
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

    /// Regression harness: nest the popover inside a small
    /// `overflow_hidden()` container and verify the floating content is
    /// painted at window-absolute coordinates that fall OUTSIDE the
    /// clipping container's bounds. This is the primary invariant
    /// `AnchoredOverlay` was introduced to provide.
    struct ClippedPopoverHarness {
        is_open: bool,
    }

    impl Render for ClippedPopoverHarness {
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
    async fn overlay_content_escapes_parent_overflow_hidden_clip(cx: &mut TestAppContext) {
        let (_host, cx) =
            setup_test_window(cx, |_window, _cx| ClippedPopoverHarness { is_open: true });

        let clip = cx.get_element("clip-region");
        let content = cx.get_element(POPOVER_CONTENT);

        // Content is wider than the 80pt clip region — if it weren't
        // escaping via `deferred(anchored(...))`, its painted right edge
        // would be clamped to the clip's right edge. We assert it
        // extends past.
        assert!(
            content.bounds.right() > clip.bounds.right(),
            "content.right() {:?} should exceed clip.right() {:?}",
            content.bounds.right(),
            clip.bounds.right(),
        );
        // Content starts below the trigger (clip is trigger-sized); its
        // top should be at or past the clip's bottom edge.
        assert!(
            content.bounds.top() >= clip.bounds.bottom(),
            "content.top() {:?} should be at or below clip.bottom() {:?}",
            content.bounds.top(),
            clip.bounds.bottom(),
        );
    }
}
