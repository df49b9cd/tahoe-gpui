//! HoverCard component for rich hover content.
//!
//! Unlike Tooltip (text-only, managed by GPUI), HoverCard renders arbitrary
//! interactive content that stays visible while the user hovers over either
//! the trigger or the card itself.
//!
//! **Why stateful?** Unlike other primitives (Popover, Modal, Tooltip) which are
//! stateless with parent-managed visibility, HoverCard internally tracks hover
//! state across both the trigger and content regions. This avoids burdening
//! parents with coordinating two separate hover signals.

use std::time::{Duration, Instant};

use crate::foundations::layout::HOVER_CARD_MAX_WIDTH;
use crate::foundations::materials::{Elevation, Glass, Shape, glass_effect_lens};
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
use crate::foundations::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::{AnyElement, App, Context, ElementId, Task, Window, div, px};

/// Default hover-in delay (300 ms). Matches HIG guidance that
/// rich hover surfaces should not appear during pointer traversal.
pub const HOVER_CARD_DEFAULT_DELAY_MS: u64 = 300;

/// Base grace window (ms) used when computing the close debounce.
///
/// The debounce bridges the dead-zone between the trigger edge and the
/// card (the configured `gap`) so a pointer traversal of the gap — which
/// briefly leaves both hit regions — does not dismiss the card. The
/// effective debounce is `BASE + PER_PT * gap_pt`, so a theme that bumps
/// `spacing_xs` (the gap source) scales the debounce proportionally
/// rather than under-debouncing a wider gap.
///
/// Default theme pairs a 4 pt gap with an 80 ms debounce (60 ms base +
/// 5 ms/pt * 4 pt), matching the prior hand-tuned constant. At a 16 pt
/// gap it scales to 140 ms; at 0 pt (no gap) it floors at the base.
/// Shortening below ~50 ms risks spurious closes during gap traversal;
/// lengthening beyond ~150 ms starts to feel sticky on re-entry — the
/// base is chosen so the scaled result stays in that band over the
/// realistic `spacing_xs` range.
const HOVER_CARD_CLOSE_DEBOUNCE_BASE_MS: u64 = 60;
const HOVER_CARD_CLOSE_DEBOUNCE_PER_PT_MS: u64 = 5;

/// Compute the close-debounce duration for a given gap magnitude. Pulled
/// out so the relationship between `theme.spacing_xs` and the debounce
/// window is testable in isolation.
fn close_debounce_for_gap(gap: gpui::Pixels) -> Duration {
    let gap_pt: f32 = gap.into();
    let gap_pt = gap_pt.max(0.0);
    let extra = (gap_pt as u64).saturating_mul(HOVER_CARD_CLOSE_DEBOUNCE_PER_PT_MS);
    Duration::from_millis(HOVER_CARD_CLOSE_DEBOUNCE_BASE_MS.saturating_add(extra))
}

/// Positions the HoverCard renders relative to the trigger.
///
/// Mirrors [`super::popover::PopoverPlacement`] so callers that know
/// Popover's placement vocabulary can reuse it. The preferred side may
/// flip at render time: `AnchoredOverlay::realise_anchor` swaps
/// Below↔Above when the opposite side has materially more room, so
/// callers don't need to pre-compute edge clamping themselves.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HoverCardPlacement {
    /// Above the trigger, aligned to the left edge.
    #[default]
    AboveLeft,
    /// Above the trigger, aligned to the right edge.
    AboveRight,
    /// Below the trigger, aligned to the left edge.
    BelowLeft,
    /// Below the trigger, aligned to the right edge.
    BelowRight,
}

impl From<HoverCardPlacement> for OverlayAnchor {
    fn from(placement: HoverCardPlacement) -> Self {
        match placement {
            HoverCardPlacement::AboveLeft => OverlayAnchor::AboveLeft,
            HoverCardPlacement::AboveRight => OverlayAnchor::AboveRight,
            HoverCardPlacement::BelowLeft => OverlayAnchor::BelowLeft,
            HoverCardPlacement::BelowRight => OverlayAnchor::BelowRight,
        }
    }
}

/// A stateful hover card that shows rich content when the trigger is hovered.
///
/// The card stays open while the mouse is over the trigger or the card content.
///
/// Unlike other primitives in this module which are stateless (`RenderOnce`),
/// HoverCard is an `Entity<Self>` because it internally manages hover tracking
/// across both trigger and content regions.
use crate::callback_types::AppElementBuilder;
pub struct HoverCard {
    /// Whether the card is currently visible.
    is_open: bool,
    /// Whether the mouse is over the trigger.
    trigger_hovered: bool,
    /// Whether the mouse is over the content.
    content_hovered: bool,
    /// Whether keyboard focus is on the trigger. Opened on focus-in so
    /// keyboard-only users (no pointer, VoiceOver, switch control) can
    /// surface the card's contents; mirrors the hover-in path but uses
    /// `focus_open_delay` (not `open_delay`) because focus-in is itself
    /// a deliberate act and doesn't need a traversal-guard.
    trigger_focused: bool,
    /// Set when Escape dismisses an open, focused card. Gates re-opens
    /// until the user actually leaves the trigger (focus-out or
    /// hover-out clears it). Keeps `trigger_focused` honest with the
    /// underlying AppKit focus state — the old `trigger_focused = false`
    /// shortcut lied about focus and forced future code to reason about
    /// a desynced flag.
    dismissed_while_focused: bool,
    /// Cached child-element IDs computed once in `new()` so `render()`
    /// only does a cheap `Arc` refcount bump instead of allocating a new
    /// `Arc<SharedString>` on each frame.
    trigger_id: ElementId,
    content_id: ElementId,
    overlay_id: ElementId,
    /// Builder for the trigger element.
    trigger: AppElementBuilder,
    /// Builder for the card content.
    content: AppElementBuilder,
    /// Preferred placement. `AnchoredOverlay::realise_anchor` may flip
    /// Below↔Above at render time if the opposite side has materially
    /// more room; callers don't need to pre-compute edge clamping.
    placement: HoverCardPlacement,
    /// Hover-in delay; prevents the card from appearing during brief
    /// pointer traversals. Applies only to the hover path — the focus
    /// path uses [`HoverCard::focus_open_delay`] because keyboard focus
    /// is intentional, not accidental.
    open_delay: Duration,
    /// Keyboard focus-in delay. Defaults to `Duration::ZERO`. The hover
    /// `open_delay` exists to filter accidental pointer traversals; a
    /// keyboard user who tabs to a control is already committed, so the
    /// card should surface immediately by default. Raise this if a
    /// specific app wants focus-triggered cards to also dwell.
    focus_open_delay: Duration,
    /// Max content width (px). Defaults to [`HOVER_CARD_MAX_WIDTH`].
    max_width: gpui::Pixels,
    /// Timestamp of the most recent trigger enter (hover or focus),
    /// used to defer the open-visibility flip by `open_delay`.
    trigger_entered_at: Option<Instant>,
    /// Pending deferred wake that will re-run [`Self::update_visibility`]
    /// once `open_delay` has elapsed since `trigger_entered_at`. Storing
    /// (rather than detaching) the task means a subsequent hover-out
    /// drops this handle, cancelling the queued future — so the card
    /// doesn't pop open after the user has already moved away.
    pending_open: Option<Task<()>>,
    /// Pending close timer that fires after [`close_debounce_for_gap`]
    /// elapses if neither region is re-entered. Dropped (cancelled)
    /// the moment hover resumes on either region.
    pending_close: Option<Task<()>>,
    /// Focus handle attached to the trigger so the card has a keyboard
    /// activation path (Tab into trigger → card opens after
    /// `open_delay`; Tab out → close debounce fires).
    focus_handle: gpui::FocusHandle,
    /// Focus-in/out subscriptions, installed lazily on first render
    /// because [`Context::on_focus_in`]/[`on_focus_out`] require a
    /// `&mut Window` which isn't available inside [`Self::new`].
    /// `None` until render runs once.
    _focus_subscriptions: Option<[gpui::Subscription; 2]>,
}

impl HoverCard {
    pub fn new(id: impl Into<ElementId>, cx: &mut Context<Self>) -> Self {
        let id = id.into();
        let id_arc = std::sync::Arc::new(id);
        let trigger_id = ElementId::NamedChild(id_arc.clone(), "hc-trigger".into());
        let content_id = ElementId::NamedChild(id_arc.clone(), "hc-content".into());
        let overlay_id = ElementId::NamedChild(id_arc, "hc-overlay".into());
        Self {
            is_open: false,
            trigger_hovered: false,
            content_hovered: false,
            trigger_focused: false,
            dismissed_while_focused: false,
            trigger_id,
            content_id,
            overlay_id,
            trigger: None,
            content: None,
            placement: HoverCardPlacement::default(),
            open_delay: Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS),
            focus_open_delay: Duration::ZERO,
            max_width: px(HOVER_CARD_MAX_WIDTH),
            trigger_entered_at: None,
            pending_open: None,
            pending_close: None,
            focus_handle: cx.focus_handle(),
            _focus_subscriptions: None,
        }
    }

    /// Override the keyboard-focus open delay (default `Duration::ZERO`).
    /// Useful for apps that want focus-triggered hover cards to dwell the
    /// same way hover-triggered ones do, e.g. to avoid flashing content
    /// during rapid Tab-through during navigation-heavy workflows.
    pub fn set_focus_open_delay(&mut self, delay: Duration, cx: &mut Context<Self>) {
        self.focus_open_delay = delay;
        cx.notify();
    }

    /// Set the trigger element builder.
    pub fn set_trigger(
        &mut self,
        builder: impl Fn(&App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) {
        self.trigger = Some(Box::new(builder));
        cx.notify();
    }

    /// Set the card content builder.
    pub fn set_content(
        &mut self,
        builder: impl Fn(&App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) {
        self.content = Some(Box::new(builder));
        cx.notify();
    }

    /// Set the preferred placement relative to the trigger.
    pub fn set_placement(&mut self, placement: HoverCardPlacement, cx: &mut Context<Self>) {
        self.placement = placement;
        cx.notify();
    }

    /// Set the hover-in delay. Pass `Duration::ZERO` to open
    /// immediately (the prior behaviour).
    pub fn set_open_delay(&mut self, delay: Duration, cx: &mut Context<Self>) {
        self.open_delay = delay;
        cx.notify();
    }

    /// Override the default content max-width.
    pub fn set_max_width(&mut self, max_width: gpui::Pixels, cx: &mut Context<Self>) {
        self.max_width = max_width;
        cx.notify();
    }

    /// Install the focus-in/out subscriptions that drive the keyboard
    /// activation path. Idempotent — a no-op after the first call.
    ///
    /// `Context::on_focus_in` / `on_focus_out` require a `&mut Window`
    /// which isn't available inside [`Self::new`], so the subscriptions
    /// are wired lazily from [`Render::render`] on the first frame.
    /// This is a GPUI API constraint, not a design choice. The practical
    /// risk is minimal: the first render runs in the same event-loop turn
    /// as construction, so the window between `new()` and the first
    /// `render()` is never observable by user interaction.
    fn install_focus_subscriptions(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self._focus_subscriptions.is_some() {
            return;
        }
        let focus_in = cx.on_focus_in(&self.focus_handle, window, |this, _window, cx| {
            this.trigger_focused = true;
            // Clear the dismissal sentinel if focus is arriving fresh
            // (we cleared it on focus-out, so a new focus-in means the
            // user has deliberately re-engaged).
            this.dismissed_while_focused = false;
            // Guard the restamp so a focus-in arriving mid-hover-delay
            // doesn't erase real dwell time the hover path already
            // accumulated. Mirrors the `is_none()` guard on the hover
            // branch.
            if this.trigger_entered_at.is_none() {
                this.trigger_entered_at = Some(Instant::now());
            }
            this.update_visibility(cx);
        });
        let focus_out = cx.on_focus_out(&self.focus_handle, window, |this, _event, _window, cx| {
            this.trigger_focused = false;
            // Focus genuinely left — clear any Escape-dismissal sentinel
            // so a subsequent re-focus is free to reopen.
            this.dismissed_while_focused = false;
            if !this.trigger_hovered {
                // Focus left and the pointer isn't on the trigger —
                // drop the entered-at stamp so a subsequent re-entry
                // restarts the delay rather than firing instantly.
                this.trigger_entered_at = None;
            }
            this.update_visibility(cx);
        });
        self._focus_subscriptions = Some([focus_in, focus_out]);
    }

    /// Dismiss the card as if the user pressed Escape. Centralises the
    /// state reset so both the trigger and the content surface can share
    /// the same Escape path (focused descendants inside the content
    /// dismiss via this helper too).
    fn dismiss_from_escape(&mut self, cx: &mut Context<Self>) {
        if !self.is_open {
            return;
        }
        self.is_open = false;
        self.trigger_entered_at = None;
        self.pending_open = None;
        self.pending_close = None;
        // Only suppress re-open while focus is still live on the
        // trigger. A content-side Escape may fire while the content has
        // focus (not the trigger handle) — in that case focus-out on
        // the trigger already happened, so the sentinel stays off and
        // the next focus-in reopens normally.
        if self.trigger_focused {
            self.dismissed_while_focused = true;
        }
        cx.notify();
    }

    fn update_visibility(&mut self, cx: &mut Context<Self>) {
        let hover_active = self.trigger_hovered || self.content_hovered;
        let should_open = hover_active || self.trigger_focused;

        if !should_open {
            // Hover and focus both left the relevant regions. Drop any
            // pending open so a lingering timer doesn't re-open the
            // card after the user has moved away, then debounce the
            // close so pointer traversal of the gap between the trigger
            // and card (which leaves both hit regions briefly) doesn't
            // dismiss the surface. When the card isn't open, just
            // reset bookkeeping.
            self.pending_open = None;
            if !self.is_open {
                self.pending_close = None;
                return;
            }
            if self.pending_close.is_none() {
                // Scale the debounce against the actually-applied gap
                // so a theme that bumps `spacing_xs` to a wider dead-
                // zone gets a proportionally wider grace window (rather
                // than a flicker on traversal).
                let debounce = close_debounce_for_gap(cx.theme().spacing_xs);
                self.pending_close = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(debounce).await;
                    this.update(cx, |this, cx| {
                        this.pending_close = None;
                        // Re-check: the user may have re-entered either
                        // region or re-focused the trigger during the
                        // debounce window.
                        let still_active =
                            this.trigger_hovered || this.content_hovered || this.trigger_focused;
                        if !still_active && this.is_open {
                            this.is_open = false;
                            cx.notify();
                        }
                    })
                    .ok();
                }));
            }
            return;
        }

        // User dismissed with Escape while the trigger still had focus;
        // don't reopen until focus actually leaves (focus-out clears
        // the sentinel). Also cancel any scheduled opens so a pending
        // timer doesn't override the dismissal.
        if self.dismissed_while_focused && self.trigger_focused {
            self.pending_open = None;
            self.pending_close = None;
            return;
        }

        // Opening path: hover or focus resumed, so cancel any pending
        // close that was about to dismiss us.
        self.pending_close = None;

        if self.is_open {
            return;
        }

        // Pick the active delay. Hover traversal is the accidental
        // case — it needs the full dwell guard. Keyboard focus is
        // deliberate, so it uses `focus_open_delay` (default zero).
        let active_delay = if hover_active {
            self.open_delay
        } else {
            self.focus_open_delay
        };

        // Respect the active delay. If `trigger_entered_at` is still
        // inside the delay window, schedule a deferred wake so the flip
        // fires even when no subsequent pointer event arrives — without
        // this, a user who enters the trigger and holds still for longer
        // than the delay would never see the card open.
        //
        // Reduce-motion suppresses decorative transitions at the
        // material/overlay level; the dwell delay is about preventing
        // accidental opens during pointer traversal, not about animation,
        // so it is always honoured.
        if active_delay > Duration::ZERO
            && let Some(entered_at) = self.trigger_entered_at
        {
            let elapsed = entered_at.elapsed();
            if elapsed < active_delay {
                let remaining = active_delay - elapsed;
                // The timer IS the delay — once it fires we trust that
                // the delay has elapsed and flip `is_open` directly
                // instead of re-running `update_visibility` (which would
                // compare `entered_at.elapsed()` against the delay again,
                // and under a mocked background-executor clock that
                // wall-time comparison would keep rescheduling forever).
                // If the user leaves the trigger before the timer fires,
                // the hover-out branch drops `pending_open`, cancelling
                // the future.
                self.pending_open = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(remaining).await;
                    this.update(cx, |this, cx| {
                        this.pending_open = None;
                        let still_should =
                            this.trigger_hovered || this.content_hovered || this.trigger_focused;
                        // Re-check the dismissal sentinel at fire time:
                        // the user may have pressed Escape during the
                        // wait.
                        let suppressed = this.dismissed_while_focused && this.trigger_focused;
                        if still_should && !suppressed && !this.is_open {
                            this.is_open = true;
                            cx.notify();
                        }
                    })
                    .ok();
                }));
                return;
            }
        }

        self.pending_open = None;
        self.is_open = true;
        cx.notify();
    }
}

impl Render for HoverCard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.install_focus_subscriptions(window, cx);

        let theme = cx.theme();
        let spacing_xs = theme.spacing_xs;

        let trigger_el = self.trigger.as_ref().map(|b| b(cx));
        let content_el = if self.is_open {
            self.content.as_ref().map(|b| b(cx))
        } else {
            None
        };

        let trigger_div = div()
            .id(self.trigger_id.clone())
            // Keyboard-focusable so Tab reaches the card. `.focusable()`
            // plus `.track_focus(&handle)` is the crate's canonical
            // tabbable-trigger pattern (see popup_button.rs).
            .focusable()
            .track_focus(&self.focus_handle)
            .on_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                    if this.is_open && crate::foundations::keyboard::is_escape_key(event) {
                        this.dismiss_from_escape(cx);
                    }
                }),
            )
            .on_hover(cx.listener(|this, &hovered: &bool, _window, cx| {
                this.trigger_hovered = hovered;
                if hovered {
                    if this.trigger_entered_at.is_none() {
                        this.trigger_entered_at = Some(Instant::now());
                    }
                } else if !this.trigger_focused {
                    // Keep the entered-at stamp while focus is still on
                    // the trigger — hover-out shouldn't erase the timer
                    // the focus path is relying on.
                    this.trigger_entered_at = None;
                }
                this.update_visibility(cx);
            }))
            .children(trigger_el);

        // `AnchoredOverlay::gap` signs the gap against the realised anchor,
        // so if `realise_anchor` flips the preferred side in prepaint the
        // gap lands on the side the card actually renders on.
        let mut overlay = AnchoredOverlay::new(self.overlay_id.clone(), trigger_div)
            .anchor(self.placement.into())
            .gap(spacing_xs);

        if let Some(content) = content_el {
            // Hover cards share popover layering: Elevated tier — one
            // depth level above content, one below full-screen sheets.
            let card = glass_effect_lens(
                theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Elevated,
                None,
            )
            .overflow_hidden()
            .max_w(self.max_width)
            .id(self.content_id.clone())
            .on_hover(cx.listener(|this, &hovered: &bool, _window, cx| {
                this.content_hovered = hovered;
                this.update_visibility(cx);
            }))
            // Bubble-phase Escape on the content surface so a focused
            // descendant (e.g. a link inside rich card content) can
            // dismiss the card without bubbling up through the
            // deferred-rendered trigger. Mirrors Popover's content-
            // surface Escape handler.
            .on_key_down(
                cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                    if this.is_open && crate::foundations::keyboard::is_escape_key(event) {
                        this.dismiss_from_escape(cx);
                    }
                }),
            )
            .child(content);
            overlay = overlay.content(card);
        }

        overlay
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HOVER_CARD_CLOSE_DEBOUNCE_BASE_MS, HOVER_CARD_CLOSE_DEBOUNCE_PER_PT_MS,
        HOVER_CARD_DEFAULT_DELAY_MS, HoverCard, HoverCardPlacement, close_debounce_for_gap,
    };
    use crate::test_helpers::helpers::setup_test_window;
    use core::prelude::v1::test;
    use gpui::{IntoElement, TestAppContext, div, px};
    use std::time::{Duration, Instant};

    /// Debounce matching the default theme's `spacing_xs` (4 pt). Used by
    /// the close-debounce tests so the timing stays anchored to the
    /// helper's derivation rather than a hardcoded number that would
    /// drift if the base or per-pt constants move.
    fn default_close_debounce() -> Duration {
        close_debounce_for_gap(px(4.0))
    }

    #[test]
    fn placement_default_is_above_left() {
        assert_eq!(HoverCardPlacement::default(), HoverCardPlacement::AboveLeft);
    }

    #[test]
    fn default_delay_is_300ms() {
        assert_eq!(HOVER_CARD_DEFAULT_DELAY_MS, 300);
    }

    #[test]
    fn close_debounce_scales_with_gap_magnitude() {
        // Base floor when no gap: only the BASE_MS portion.
        assert_eq!(
            close_debounce_for_gap(px(0.0)),
            Duration::from_millis(HOVER_CARD_CLOSE_DEBOUNCE_BASE_MS)
        );
        // Default theme gap (4 pt): base + 4 * per-pt.
        assert_eq!(
            close_debounce_for_gap(px(4.0)),
            Duration::from_millis(
                HOVER_CARD_CLOSE_DEBOUNCE_BASE_MS + 4 * HOVER_CARD_CLOSE_DEBOUNCE_PER_PT_MS
            )
        );
        // Wider theme override (16 pt): linear scale, no clamp.
        assert_eq!(
            close_debounce_for_gap(px(16.0)),
            Duration::from_millis(
                HOVER_CARD_CLOSE_DEBOUNCE_BASE_MS + 16 * HOVER_CARD_CLOSE_DEBOUNCE_PER_PT_MS
            )
        );
        // Negative input (shouldn't happen at render time but defensive):
        // clamped to base, not sign-extended into a large duration.
        assert_eq!(
            close_debounce_for_gap(px(-4.0)),
            Duration::from_millis(HOVER_CARD_CLOSE_DEBOUNCE_BASE_MS)
        );
    }

    /// Builds a HoverCard with trivial trigger/content so `update_visibility`
    /// has something to observe. The tests below drive state by poking
    /// `trigger_hovered` / `content_hovered` / `trigger_focused` directly
    /// (private fields — same module) and then calling `update_visibility`
    /// as the real hover/focus listeners do.
    fn build_hover_card(cx: &mut gpui::Context<HoverCard>) -> HoverCard {
        let mut hc = HoverCard::new("test-hc", cx);
        hc.set_trigger(|_app| div().into_any_element(), cx);
        hc.set_content(|_app| div().into_any_element(), cx);
        hc
    }

    #[gpui::test]
    async fn hover_in_opens_card_after_delay(cx: &mut TestAppContext) {
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));

        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(!hc.is_open, "card must not open inside the delay window");
            assert!(hc.pending_open.is_some(), "open task should be scheduled");
        });

        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS + 20));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(hc.is_open, "card must open after delay elapses");
            assert!(hc.pending_open.is_none(), "pending open must clear");
        });
    }

    #[gpui::test]
    async fn hover_out_before_delay_cancels_open(cx: &mut TestAppContext) {
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));

        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(hc.pending_open.is_some());
        });

        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = false;
            hc.trigger_entered_at = None;
            hc.update_visibility(cx);
            assert!(
                hc.pending_open.is_none(),
                "hover-out must drop the pending open task"
            );
        });

        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS + 100));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(!hc.is_open, "card must never open after early hover-out");
        });
    }

    #[gpui::test]
    async fn hover_out_closes_card_after_debounce(cx: &mut TestAppContext) {
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));

        hc.update(cx, |hc, cx| {
            hc.set_open_delay(Duration::ZERO, cx);
            hc.trigger_hovered = true;
            hc.update_visibility(cx);
            assert!(hc.is_open, "zero delay opens immediately");
        });

        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = false;
            hc.update_visibility(cx);
            assert!(hc.is_open, "card stays open during debounce");
            assert!(hc.pending_close.is_some(), "close debounce must schedule");
        });

        let debounce = default_close_debounce();
        cx.executor()
            .advance_clock(debounce + Duration::from_millis(20));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(!hc.is_open, "card must close after debounce elapses");
            assert!(hc.pending_close.is_none());
        });
    }

    #[gpui::test]
    async fn re_enter_during_debounce_cancels_close(cx: &mut TestAppContext) {
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));

        hc.update(cx, |hc, cx| {
            hc.set_open_delay(Duration::ZERO, cx);
            hc.trigger_hovered = true;
            hc.update_visibility(cx);
            assert!(hc.is_open);
        });

        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = false;
            hc.update_visibility(cx);
            assert!(hc.pending_close.is_some());
        });

        // Simulate the pointer bridging the gap into the card while the
        // close debounce is still pending.
        let debounce = default_close_debounce();
        cx.executor().advance_clock(debounce / 2);
        cx.run_until_parked();

        hc.update(cx, |hc, cx| {
            hc.content_hovered = true;
            hc.update_visibility(cx);
            assert!(
                hc.pending_close.is_none(),
                "re-entry must cancel pending close"
            );
            assert!(hc.is_open);
        });

        // Advance past the original debounce deadline — card must persist.
        cx.executor().advance_clock(debounce);
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(hc.is_open, "card must remain open after re-entry");
        });
    }

    #[gpui::test]
    async fn open_delay_applies_to_hover_even_without_reduce_motion(cx: &mut TestAppContext) {
        // `update_visibility` does not consult the theme or
        // `AccessibilityMode`; the dwell delay is about filtering
        // accidental pointer traversal, which is a hover concern
        // independent of motion settings. This test guards that
        // invariant (a regression that made the delay theme-dependent
        // would start bypassing the timer under certain accessibility
        // modes). If the dwell logic ever does start reading the theme,
        // this test should grow a REDUCE_MOTION branch.
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));

        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(!hc.is_open, "hover path must respect open_delay");
            assert!(hc.pending_open.is_some(), "open timer must be scheduled");
        });

        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS + 20));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(hc.is_open, "card must open after delay elapses");
        });
    }

    #[gpui::test]
    async fn focus_in_opens_card_immediately_by_default(cx: &mut TestAppContext) {
        // Keyboard focus is a deliberate act, not accidental traversal.
        // `focus_open_delay` defaults to `Duration::ZERO`, so a focus-in
        // should flip `is_open` without spawning a timer.
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));
        cx.run_until_parked();

        hc.update(cx, |hc, cx| {
            hc.trigger_focused = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(hc.is_open, "zero focus_open_delay opens immediately");
            assert!(
                hc.pending_open.is_none(),
                "no open timer should be spawned at zero delay"
            );
        });
    }

    #[gpui::test]
    async fn focus_in_respects_non_zero_focus_open_delay(cx: &mut TestAppContext) {
        // Consumers that want the focus-triggered path to dwell can opt
        // in via `set_focus_open_delay`. The delay is honoured just like
        // the hover `open_delay`, including the deferred wake that fires
        // without a subsequent event.
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));
        cx.run_until_parked();

        hc.update(cx, |hc, cx| {
            hc.set_focus_open_delay(Duration::from_millis(150), cx);
            hc.trigger_focused = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(!hc.is_open);
            assert!(hc.pending_open.is_some());
        });

        cx.executor().advance_clock(Duration::from_millis(170));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(hc.is_open);
        });
    }

    #[gpui::test]
    async fn focus_in_preserves_in_flight_hover_entered_at(cx: &mut TestAppContext) {
        // If hover already stamped `trigger_entered_at` and a focus-in
        // arrives during the dwell window, focus-in must NOT restamp
        // the timer (that would erase accumulated dwell time). This
        // mirrors the `is_none()` guard on the hover path.
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));
        cx.run_until_parked();

        let earlier = Instant::now() - Duration::from_millis(200);
        let stamped = hc.update(cx, |hc, _cx| {
            hc.trigger_hovered = true;
            hc.trigger_entered_at = Some(earlier);
            hc.trigger_entered_at
        });

        // Simulate the focus-in listener: arrives while `trigger_entered_at`
        // is already set, must not overwrite it.
        hc.update(cx, |hc, _cx| {
            hc.trigger_focused = true;
            hc.dismissed_while_focused = false;
            if hc.trigger_entered_at.is_none() {
                hc.trigger_entered_at = Some(Instant::now());
            }
            assert_eq!(
                hc.trigger_entered_at, stamped,
                "focus-in must preserve the hover's entered_at stamp"
            );
        });
    }

    #[gpui::test]
    async fn escape_dismisses_focused_card_and_suppresses_reopen(cx: &mut TestAppContext) {
        // Escape dismisses an open, focused card, resets all pending
        // timers, and gates re-opens via `dismissed_while_focused` until
        // focus actually leaves.
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));
        cx.run_until_parked();

        hc.update(cx, |hc, cx| {
            hc.trigger_focused = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(hc.is_open);
        });

        hc.update(cx, |hc, cx| {
            hc.dismiss_from_escape(cx);
            assert!(!hc.is_open, "escape must close the card");
            assert!(hc.trigger_entered_at.is_none());
            assert!(hc.pending_open.is_none());
            assert!(hc.pending_close.is_none());
            assert!(hc.trigger_focused, "escape must NOT lie about focus state");
            assert!(hc.dismissed_while_focused, "sentinel must gate re-opens");
        });

        // Simulate the user re-hovering the trigger while focus is
        // still live: the card must NOT reopen because the sentinel
        // is set.
        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(!hc.is_open, "sentinel must block reopen while focused");
        });

        // Simulate focus leaving (focus-out clears the sentinel).
        hc.update(cx, |hc, cx| {
            hc.trigger_focused = false;
            hc.dismissed_while_focused = false;
            hc.update_visibility(cx);
        });

        // Zero-delay reopen via hover now that sentinel is clear.
        hc.update(cx, |hc, cx| {
            hc.set_open_delay(Duration::ZERO, cx);
            hc.update_visibility(cx);
            assert!(hc.is_open, "hover reopens once sentinel is cleared");
        });
    }

    #[gpui::test]
    async fn escape_on_unfocused_trigger_does_not_arm_sentinel(cx: &mut TestAppContext) {
        // A content-side Escape may fire while focus is on a descendant
        // inside the card, not on the trigger handle. The dismissal
        // sentinel should only arm when the trigger itself is focused
        // — otherwise subsequent focus-in on the trigger must be free
        // to reopen.
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));
        cx.run_until_parked();

        hc.update(cx, |hc, cx| {
            hc.set_open_delay(Duration::ZERO, cx);
            hc.trigger_hovered = true;
            hc.update_visibility(cx);
            assert!(hc.is_open);
            // Simulate focus moving to a descendant inside content:
            // trigger handle no longer focused.
            hc.trigger_focused = false;
        });

        hc.update(cx, |hc, cx| {
            hc.dismiss_from_escape(cx);
            assert!(!hc.is_open);
            assert!(
                !hc.dismissed_while_focused,
                "sentinel must NOT arm when the trigger is not focused"
            );
        });
    }
}
