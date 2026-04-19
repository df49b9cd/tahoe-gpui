//! Popover component for floating content (HIG `#popovers`).
//!
//! Uses absolute positioning to render content below/beside a trigger.
//! The parent manages the `is_visible` state.
//!
//! The popover renders a directional arrow pointing at the trigger
//! (HIG: "Make sure a popover's arrow points as directly as possible
//! to the element that revealed it") and caps its width at
//! [`POPOVER_MAX_WIDTH`] so it stays within the HIG sizing envelope.

use std::time::Duration;

use crate::foundations::layout::POPOVER_MAX_WIDTH;
use crate::foundations::motion::REDUCE_MOTION_CROSSFADE;
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme};
use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, AnyElement, App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent,
    Window, div, px,
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

    /// Whether this placement aligns the popover to the left edge.
    fn aligns_left(self) -> bool {
        matches!(self, Self::BelowLeft | Self::AboveLeft)
    }
}

/// A popover that shows floating content relative to a trigger.
///
/// The parent manages `is_visible` and toggles it via hover/click.
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    is_visible: bool,
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
            is_visible: false,
            trigger: trigger.into_any_element(),
            content: content.into_any_element(),
            placement: PopoverPlacement::default(),
            focus_handle: None,
            on_dismiss: None,
            max_width: None,
            arrow: true,
        }
    }

    pub fn visible(mut self, visible: bool) -> Self {
        self.is_visible = visible;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn with_focus_handle(mut self, handle: FocusHandle) -> Self {
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
        let mut container = div()
            .id(self.id)
            .debug_selector(|| "popover-root".into())
            .relative()
            .child(
                div()
                    .debug_selector(|| "popover-trigger".into())
                    .child(self.trigger),
            );

        if self.is_visible
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

        if self.is_visible {
            // Wrapper uses padding instead of margin to keep the hover zone
            // contiguous between trigger and content (prevents flicker).
            let mut wrapper = div().absolute();
            match self.placement {
                PopoverPlacement::BelowLeft => {
                    wrapper = wrapper.top_full().left_0().pt(theme.spacing_xs);
                }
                PopoverPlacement::BelowRight => {
                    wrapper = wrapper.top_full().right_0().pt(theme.spacing_xs);
                }
                PopoverPlacement::AboveLeft => {
                    wrapper = wrapper.bottom_full().left_0().pb(theme.spacing_xs);
                }
                PopoverPlacement::AboveRight => {
                    wrapper = wrapper.bottom_full().right_0().pb(theme.spacing_xs);
                }
            }

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

            // HIG: "Make sure a popover's arrow points as directly as
            // possible to the element that revealed it." The arrow is
            // drawn as a diamond (rotated square) clipped by a
            // containing div so only one triangular half is visible on
            // the edge adjacent to the trigger.
            let arrow_el = if self.arrow {
                Some(build_arrow(theme, self.placement))
            } else {
                None
            };

            // Layout order: arrow above content for below-placement,
            // below content for above-placement.
            let mut content_col = div().flex().flex_col();
            if !self.placement.is_above() {
                if let Some(arrow) = arrow_el {
                    content_col = content_col.child(arrow);
                }
                content_col = content_col.child(content_div);
            } else {
                content_col = content_col.child(content_div);
                if let Some(arrow) = build_arrow_if(self.arrow, theme, self.placement) {
                    content_col = content_col.child(arrow);
                }
            }

            // Present-transition: HIG calls for popovers to scale from the
            // anchor. GPUI's style API doesn't yet expose a transform-origin
            // scale, so we approximate with a short fade plus a small
            // vertical translate toward the anchor. Under Reduce Motion
            // we drop the translate per `foundations.md:1100`.
            let reduce_motion = theme.accessibility_mode.reduce_motion();
            let (anim_duration, translate_px) = if reduce_motion {
                (REDUCE_MOTION_CROSSFADE, 0.0)
            } else {
                (
                    Duration::from_millis(theme.glass.motion.lift_duration_ms),
                    6.0,
                )
            };
            let is_above = self.placement.is_above();
            let anim_id = ElementId::Name("popover-present".into());
            let populated_wrapper = wrapper.child(content_col);
            let animated_wrapper = populated_wrapper.with_animation(
                anim_id,
                Animation::new(anim_duration),
                move |el, delta| {
                    let offset = translate_px * (1.0 - delta);
                    let signed = if is_above { offset } else { -offset };
                    el.opacity(delta).mt(gpui::px(signed))
                },
            );

            container = container.child(animated_wrapper);
        }

        container
    }
}

fn build_arrow_if(
    enabled: bool,
    theme: &TahoeTheme,
    placement: PopoverPlacement,
) -> Option<gpui::Div> {
    if enabled {
        Some(build_arrow(theme, placement))
    } else {
        None
    }
}

/// Render a directional callout arrow pointing at the trigger.
///
/// GPUI does not yet expose per-side border colors (required for the
/// classic CSS "triangle via borders" trick) nor transform-origin
/// rotation. Instead we render a small Unicode glyph — ▼ when the
/// popover is above the trigger (arrow points down at the trigger) and
/// ▲ when the popover is below the trigger (arrow points up). The
/// glyph takes the glass surface color so it reads as an extension of
/// the panel rather than a free-floating mark.
fn build_arrow(theme: &TahoeTheme, placement: PopoverPlacement) -> gpui::Div {
    let arrow_bg = theme
        .glass
        .accessible_bg(GlassSize::Medium, theme.accessibility_mode);
    let glyph = if placement.is_above() { "▼" } else { "▲" };

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
    if placement.aligns_left() {
        wrapper = wrapper.justify_start().pl(theme.spacing_sm);
    } else {
        wrapper = wrapper.justify_end().pr(theme.spacing_sm);
    }
    wrapper.child(pointer)
}

#[cfg(test)]
mod tests {
    use super::{Popover, PopoverPlacement};
    use core::prelude::v1::test;

    #[test]
    fn popover_default_is_not_visible() {
        let popover = Popover::new("test", gpui::div(), gpui::div());
        assert!(!popover.is_visible);
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

    #[test]
    fn popover_aligns_left_classifies_placements() {
        assert!(PopoverPlacement::AboveLeft.aligns_left());
        assert!(PopoverPlacement::BelowLeft.aligns_left());
        assert!(!PopoverPlacement::AboveRight.aligns_left());
        assert!(!PopoverPlacement::BelowRight.aligns_left());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    use super::{Popover, PopoverPlacement};
    use crate::test_helpers::helpers::{
        InteractionExt, LocatorExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const POPOVER_TRIGGER: &str = "popover-trigger";
    const POPOVER_CONTENT: &str = "popover-content";

    struct PopoverHarness {
        focus_handle: FocusHandle,
        visible: bool,
        dismiss_count: usize,
        placement: PopoverPlacement,
    }

    impl PopoverHarness {
        fn new(cx: &mut Context<Self>, visible: bool, placement: PopoverPlacement) -> Self {
            Self {
                focus_handle: cx.focus_handle(),
                visible,
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
            Popover::new(
                "popover",
                div().w(px(80.0)).h(px(32.0)).child("Trigger"),
                div().w(px(120.0)).h(px(60.0)).child("Content"),
            )
            .visible(self.visible)
            .placement(self.placement)
            .with_focus_handle(self.focus_handle.clone())
            .on_dismiss(move |_, cx| {
                entity.update(cx, |this, cx| {
                    this.dismiss_count += 1;
                    this.visible = false;
                    cx.notify();
                });
            })
        }
    }

    #[gpui::test]
    async fn hidden_and_visible_states_render_expected_content(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            PopoverHarness::new(cx, false, PopoverPlacement::BelowLeft)
        });

        assert_element_absent(cx, POPOVER_CONTENT);
        host.update_in(cx, |host, _window, cx| {
            host.visible = true;
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
            assert!(!host.visible);
        });
        assert_element_absent(cx, POPOVER_CONTENT);
    }
}
