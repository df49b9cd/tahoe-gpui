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
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{AnyElement, App, Context, SharedString, Window, div, px};

/// Default hover-in delay (300 ms). Matches HIG guidance that
/// rich hover surfaces should not appear during pointer traversal.
pub const HOVER_CARD_DEFAULT_DELAY_MS: u64 = 300;

/// Positions the HoverCard renders relative to the trigger.
///
/// Mirrors [`super::popover::PopoverPlacement`] so callers that know
/// Popover's placement vocabulary can reuse it. Boundary clamping: if
/// the requested placement would spill outside the viewport,
/// [`Self::fallback_for_edge`] returns an alternate placement that
/// stays inside bounds.
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
    id: SharedString,
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
}

impl HoverCard {
    pub fn new(id: impl Into<SharedString>, _cx: &mut Context<Self>) -> Self {
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
        if should_open != self.is_open {
            // Respect the hover-in delay: if the trigger just entered,
            // defer open until `open_delay` has elapsed. Close events
            // are applied immediately to avoid lingering cards.
            if should_open && self.open_delay > Duration::ZERO {
                if let Some(entered_at) = self.trigger_entered_at {
                    if entered_at.elapsed() < self.open_delay {
                        return;
                    }
                }
            }
            self.is_open = should_open;
            cx.notify();
        }
    }
}

impl Render for HoverCard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let trigger_el = self.trigger.as_ref().map(|b| b(cx));
        let content_el = if self.is_open {
            self.content.as_ref().map(|b| b(cx))
        } else {
            None
        };

        let trigger_id = format!("hc-trigger-{}", self.id);
        let content_id = format!("hc-content-{}", self.id);

        let mut container = div().relative().child(
            div()
                .id(SharedString::from(trigger_id))
                .on_hover(cx.listener(|this, &hovered: &bool, _window, cx| {
                    this.trigger_hovered = hovered;
                    if hovered {
                        this.trigger_entered_at = Some(Instant::now());
                    } else {
                        this.trigger_entered_at = None;
                    }
                    this.update_visibility(cx);
                }))
                .children(trigger_el),
        );

        if let Some(content) = content_el {
            // Hover cards share popover layering: mid-depth overlay surface,
            // not a sheet/modal. `GlassSize::Large` over-shadows the card
            // and breaks the HIG depth hierarchy — use `Medium` instead.
            let mut card = crate::foundations::materials::glass_surface(
                {
                    let mut base = div().absolute().overflow_hidden().max_w(self.max_width);
                    // Position by placement. `absolute + bottom_full /
                    // top_full + left_0 / right_0` anchors to the
                    // corresponding corner of the trigger.
                    base = match self.placement {
                        HoverCardPlacement::AboveLeft => {
                            base.bottom_full().left_0().pb(theme.spacing_xs)
                        }
                        HoverCardPlacement::AboveRight => {
                            base.bottom_full().right_0().pb(theme.spacing_xs)
                        }
                        HoverCardPlacement::BelowLeft => {
                            base.top_full().left_0().pt(theme.spacing_xs)
                        }
                        HoverCardPlacement::BelowRight => {
                            base.top_full().right_0().pt(theme.spacing_xs)
                        }
                    };
                    base
                },
                theme,
                GlassSize::Medium,
            )
            .id(SharedString::from(content_id))
            .on_hover(cx.listener(|this, &hovered: &bool, _window, cx| {
                this.content_hovered = hovered;
                this.update_visibility(cx);
            }));

            card = card.child(content);
            container = container.child(card);
        }

        container
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
