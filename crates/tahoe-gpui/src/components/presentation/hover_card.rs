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
const HOVER_CARD_CLOSE_DEBOUNCE_MS: u64 = 80;

/// Positions the HoverCard renders relative to the trigger.
///
/// Mirrors [`super::popover::PopoverPlacement`] so callers that know
/// Popover's placement vocabulary can reuse it. Boundary clamping: if
/// the requested placement would spill outside the viewport, callers
/// should pick an alternate placement that stays inside bounds.
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

impl HoverCardPlacement {
    fn is_above(self) -> bool {
        matches!(self, Self::AboveLeft | Self::AboveRight)
    }

    fn aligns_left(self) -> bool {
        matches!(self, Self::AboveLeft | Self::BelowLeft)
    }

    /// Returns a fallback placement that flips vertically or
    /// horizontally when the caller detects a viewport edge conflict.
    ///
    /// The matrix is: flip the vertical side when `near_top_edge`;
    /// flip the horizontal alignment when `near_right_edge`. Both
    /// flags together flip both axes.
    pub fn clamp_for_edges(self, near_top_edge: bool, near_right_edge: bool) -> Self {
        let is_above = self.is_above();
        let aligns_left = self.aligns_left();
        let resolved_above = if near_top_edge { false } else { is_above };
        let resolved_left = if near_right_edge { true } else { aligns_left };
        match (resolved_above, resolved_left) {
            (true, true) => Self::AboveLeft,
            (true, false) => Self::AboveRight,
            (false, true) => Self::BelowLeft,
            (false, false) => Self::BelowRight,
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
    /// Unique id for element identification.
    id: ElementId,
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
    /// Timestamp of the most recent trigger enter, used to defer the
    /// open-visibility flip by `open_delay`.
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
}

impl HoverCard {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            is_open: false,
            trigger_hovered: false,
            content_hovered: false,
            id: id.into(),
            trigger: None,
            content: None,
            placement: HoverCardPlacement::default(),
            open_delay: Duration::from_millis(HOVER_CARD_DEFAULT_DELAY_MS),
            max_width: px(HOVER_CARD_MAX_WIDTH),
            trigger_entered_at: None,
            pending_open: None,
            pending_close: None,
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

    fn update_visibility(&mut self, cx: &mut Context<Self>) {
        let should_open = self.trigger_hovered || self.content_hovered;

        if !should_open {
            // Hover left both regions. Drop any pending open so a
            // lingering timer doesn't re-open the card after the user
            // has moved away, then debounce the close so pointer
            // traversal of the gap between the trigger and card (which
            // leaves both hit regions briefly) doesn't dismiss the
            // surface. When the card isn't open, just reset bookkeeping.
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
                        // region during the debounce window.
                        let still_hovered = this.trigger_hovered || this.content_hovered;
                        if !still_hovered && this.is_open {
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
        if self.open_delay > Duration::ZERO
            && let Some(entered_at) = self.trigger_entered_at
        {
            let elapsed = entered_at.elapsed();
            if elapsed < self.open_delay {
                let remaining = self.open_delay - elapsed;
                self.pending_open = Some(cx.spawn(async move |this, cx| {
                    cx.background_executor().timer(remaining).await;
                    this.update(cx, |this, cx| {
                        this.pending_open = None;
                        this.update_visibility(cx);
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let spacing_xs = theme.spacing_xs;

        let trigger_el = self.trigger.as_ref().map(|b| b(cx));
        let content_el = if self.is_open {
            self.content.as_ref().map(|b| b(cx))
        } else {
            None
        };

        let trigger_id =
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "hc-trigger".into());
        let content_id =
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "hc-content".into());
        let overlay_id =
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "hc-overlay".into());

        let trigger_div = div()
            .id(trigger_id)
            .on_hover(cx.listener(|this, &hovered: &bool, _window, cx| {
                this.trigger_hovered = hovered;
                if hovered {
                    this.trigger_entered_at = Some(Instant::now());
                } else {
                    this.trigger_entered_at = None;
                }
                this.update_visibility(cx);
            }))
            .children(trigger_el);

        // `AnchoredOverlay::gap` signs the gap against the realised anchor,
        // so if `realise_anchor` flips the preferred side in prepaint the
        // gap lands on the side the card actually renders on.
        let mut overlay = AnchoredOverlay::new(overlay_id, trigger_div)
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
            .id(content_id)
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
    use super::{HOVER_CARD_DEFAULT_DELAY_MS, HoverCardPlacement};
    use core::prelude::v1::test;

    #[test]
    fn placement_default_is_above_left() {
        assert_eq!(HoverCardPlacement::default(), HoverCardPlacement::AboveLeft);
    }

    #[test]
    fn placement_classifiers() {
        assert!(HoverCardPlacement::AboveLeft.is_above());
        assert!(HoverCardPlacement::AboveRight.is_above());
        assert!(!HoverCardPlacement::BelowLeft.is_above());
        assert!(HoverCardPlacement::AboveLeft.aligns_left());
        assert!(!HoverCardPlacement::AboveRight.aligns_left());
    }

    #[test]
    fn clamp_flips_vertical_near_top_edge() {
        assert_eq!(
            HoverCardPlacement::AboveLeft.clamp_for_edges(true, false),
            HoverCardPlacement::BelowLeft
        );
        assert_eq!(
            HoverCardPlacement::AboveRight.clamp_for_edges(true, false),
            HoverCardPlacement::BelowRight
        );
    }

    #[test]
    fn clamp_flips_horizontal_near_right_edge() {
        assert_eq!(
            HoverCardPlacement::AboveRight.clamp_for_edges(false, true),
            HoverCardPlacement::AboveLeft
        );
        assert_eq!(
            HoverCardPlacement::BelowRight.clamp_for_edges(false, true),
            HoverCardPlacement::BelowLeft
        );
    }

    #[test]
    fn clamp_flips_both_axes() {
        // AboveRight near both edges (trigger is at top-right of the
        // viewport): vertical flips from Above → Below (would spill
        // above the top), horizontal flips from Right → Left (would
        // spill past the right edge). Result: BelowLeft.
        assert_eq!(
            HoverCardPlacement::AboveRight.clamp_for_edges(true, true),
            HoverCardPlacement::BelowLeft
        );
    }

    #[test]
    fn clamp_is_noop_when_already_safe() {
        // A BelowLeft card near the top edge is already safe — we
        // don't flip Below to Above, we just keep the card below.
        assert_eq!(
            HoverCardPlacement::BelowLeft.clamp_for_edges(true, false),
            HoverCardPlacement::BelowLeft
        );
        assert_eq!(
            HoverCardPlacement::AboveLeft.clamp_for_edges(false, true),
            HoverCardPlacement::AboveLeft
        );
    }

    #[test]
    fn default_delay_is_300ms() {
        assert_eq!(HOVER_CARD_DEFAULT_DELAY_MS, 300);
    }
}
