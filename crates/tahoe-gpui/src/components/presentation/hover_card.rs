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
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{AnyElement, App, Context, ElementId, Task, Window, div, px};

/// Default hover-in delay (300 ms). Matches HIG guidance that
/// rich hover surfaces should not appear during pointer traversal.
pub const HOVER_CARD_DEFAULT_DELAY_MS: u64 = 300;

/// Grace window between losing hover on both the trigger and the card
/// before the card actually closes. Bridges the dead-zone between the
/// trigger edge and the card (the configured `gap`) so a pointer
/// traversal of the gap — which briefly leaves both hit regions — does
/// not dismiss the card.
///
/// 80 ms ≈ 2× the pointer-travel time across a `spacing_xs` (8 pt) gap
/// at typical desktop velocities (~200 pt/s pointer moves clear the
/// gap in ~40 ms; doubling leaves a comfortable margin for slower
/// pointers and higher-DPI cursors). Shortening below ~50 ms risks
/// spurious closes during gap traversal; lengthening beyond ~150 ms
/// starts to feel sticky on re-entry.
const HOVER_CARD_CLOSE_DEBOUNCE_MS: u64 = 80;

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
    /// surface the card's contents; mirrors the hover-in path,
    /// including `open_delay` and close debounce.
    trigger_focused: bool,
    /// Unique id for element identification. Kept for structural identity
    /// even though render uses the cached child IDs; removing it would
    /// lose the HoverCard's canonical id.
    #[expect(dead_code)]
    id: ElementId,
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
    /// Placement preference. Callers can flip this based on viewport
    /// position to clamp to the visible bounds.
    placement: HoverCardPlacement,
    /// Hover-in delay; prevents the card from appearing during brief
    /// pointer traversals.
    open_delay: Duration,
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
    /// Pending close timer that fires after
    /// [`HOVER_CARD_CLOSE_DEBOUNCE_MS`] if neither region is re-entered.
    /// Dropped (cancelled) the moment hover resumes on either region.
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
        let trigger_id =
            ElementId::NamedChild(std::sync::Arc::new(id.clone()), "hc-trigger".into());
        let content_id =
            ElementId::NamedChild(std::sync::Arc::new(id.clone()), "hc-content".into());
        let overlay_id =
            ElementId::NamedChild(std::sync::Arc::new(id.clone()), "hc-overlay".into());
        Self {
            is_open: false,
            trigger_hovered: false,
            content_hovered: false,
            trigger_focused: false,
            id,
            trigger_id,
            content_id,
            overlay_id,
            trigger: None,
            content: None,
            placement: HoverCardPlacement::default(),
            open_delay: Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS),
            max_width: px(HOVER_CARD_MAX_WIDTH),
            trigger_entered_at: None,
            pending_open: None,
            pending_close: None,
            focus_handle: cx.focus_handle(),
            _focus_subscriptions: None,
        }
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
            this.trigger_entered_at = Some(Instant::now());
            this.update_visibility(cx);
        });
        let focus_out = cx.on_focus_out(&self.focus_handle, window, |this, _event, _window, cx| {
            this.trigger_focused = false;
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

    fn update_visibility(&mut self, cx: &mut Context<Self>) {
        let should_open = self.trigger_hovered || self.content_hovered || self.trigger_focused;

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
                self.pending_close = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor()
                        .timer(Duration::from_millis(HOVER_CARD_CLOSE_DEBOUNCE_MS))
                        .await;
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

        // Opening path: hover resumed on trigger or content, so cancel
        // any pending close that was about to dismiss us.
        self.pending_close = None;

        if self.is_open {
            return;
        }

        // Respect the hover-in delay. If `trigger_entered_at` is still
        // inside the delay window, schedule a deferred wake so the flip
        // fires even when no subsequent pointer event arrives — without
        // this, a user who enters the trigger and holds still for longer
        // than `open_delay` would never see the card open.
        //
        // Reduce-motion suppresses decorative transitions at the
        // material/overlay level; the dwell delay is about preventing
        // accidental opens during pointer traversal, not about animation,
        // so it is always honoured.
        if self.open_delay > Duration::ZERO
            && let Some(entered_at) = self.trigger_entered_at
        {
            let elapsed = entered_at.elapsed();
            if elapsed < self.open_delay {
                let remaining = self.open_delay - elapsed;
                // The timer IS the delay — once it fires we trust that
                // `open_delay` has elapsed and flip `is_open` directly
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
                        if still_should && !this.is_open {
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
                        this.is_open = false;
                        this.trigger_focused = false;
                        this.trigger_entered_at = None;
                        this.pending_open = None;
                        this.pending_close = None;
                        cx.notify();
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
            // Hover cards share popover layering: mid-depth overlay surface,
            // not a sheet/modal. `GlassSize::Large` over-shadows the card
            // and breaks the HIG depth hierarchy — use `Medium` instead.
            let card = crate::foundations::materials::glass_surface(
                div().overflow_hidden().max_w(self.max_width),
                theme,
                GlassSize::Medium,
            )
            .id(self.content_id.clone())
            .on_hover(cx.listener(|this, &hovered: &bool, _window, cx| {
                this.content_hovered = hovered;
                this.update_visibility(cx);
            }))
            .child(content);
            overlay = overlay.content(card);
        }

        overlay
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HOVER_CARD_CLOSE_DEBOUNCE_MS, HOVER_CARD_DEFAULT_DELAY_MS, HoverCard, HoverCardPlacement,
    };
    use crate::foundations::accessibility::AccessibilityMode;
    use crate::foundations::theme::TahoeTheme;
    use crate::test_helpers::helpers::setup_test_window;
    use core::prelude::v1::test;
    use gpui::{IntoElement, TestAppContext, div};
    use std::time::{Duration, Instant};

    #[test]
    fn placement_default_is_above_left() {
        assert_eq!(HoverCardPlacement::default(), HoverCardPlacement::AboveLeft);
    }

    #[test]
    fn default_delay_is_300ms() {
        assert_eq!(HOVER_CARD_DEFAULT_DELAY_MS, 300);
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

        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_CLOSE_DEBOUNCE_MS + 20));
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
        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_CLOSE_DEBOUNCE_MS / 2));
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
        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_CLOSE_DEBOUNCE_MS));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(hc.is_open, "card must remain open after re-entry");
        });
    }

    #[gpui::test]
    async fn reduce_motion_still_respects_open_delay(cx: &mut TestAppContext) {
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));

        // Swap in a theme with REDUCE_MOTION so `update_visibility` sees it.
        // Save the original theme to restore after assertions.
        let original_theme = cx.update(|_window, cx| {
            let original = cx.global::<TahoeTheme>().clone();
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
            cx.set_global(theme);
            original
        });

        hc.update(cx, |hc, cx| {
            hc.trigger_hovered = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            // Reduce-motion no longer collapses the delay — it only
            // suppresses decorative transitions at the material/overlay
            // level. The card must still wait for the open delay.
            assert!(!hc.is_open, "reduce_motion must still respect open_delay");
            assert!(hc.pending_open.is_some(), "open timer must be scheduled");
        });

        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS + 20));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(hc.is_open, "card must open after delay elapses");
        });

        // Restore original theme so subsequent tests aren't affected.
        cx.update(|_window, cx| {
            cx.set_global(original_theme);
        });
    }

    #[gpui::test]
    async fn focus_in_opens_card_after_delay(cx: &mut TestAppContext) {
        let (hc, cx) = setup_test_window(cx, |_w, cx| build_hover_card(cx));

        // Run until parked so the first render fires and focus
        // subscriptions are installed via `install_focus_subscriptions`.
        cx.run_until_parked();

        hc.update(cx, |hc, cx| {
            hc.trigger_focused = true;
            hc.trigger_entered_at = Some(Instant::now());
            hc.update_visibility(cx);
            assert!(!hc.is_open, "focus-in still respects open_delay");
            assert!(hc.pending_open.is_some());
        });

        cx.executor()
            .advance_clock(Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS + 20));
        cx.run_until_parked();

        hc.update(cx, |hc, _cx| {
            assert!(hc.is_open, "card opens once the focus-in delay elapses");
        });
    }
}
