//! Blinking caret for the streaming insertion point.
//!
//! Uses a global epoch for synchronized blinking across multiple carets.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{Hsla, Pixels, div, px};

/// Global epoch for synchronized caret blinking.
static EPOCH: OnceLock<Instant> = OnceLock::new();

/// The kind of caret to display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaretKind {
    /// A thin vertical bar that matches NSTextView's insertion point. This
    /// is the HIG default: 1.5pt wide, full line-height tall, drawn as a
    /// dedicated rectangle so it does not occupy a text glyph slot (and
    /// therefore does not shift layout around streaming word reveals).
    Block,
    /// A circular dot. Used by some hosts to signal a reasoning or
    /// "thinking" state distinct from the standard insertion caret.
    Circle,
}

/// Width of the thin vertical bar caret, in points. Apple's NSTextView
/// insertion point is 1–2pt wide depending on display scale; 1.5pt reads
/// cleanly on both 1x and 2x displays.
const BAR_WIDTH_PT: f32 = 1.5;

/// Returns `true` if the caret should be visible at `now` for the given
/// blink interval. `blink_interval == Duration::ZERO` forces the caret on
/// (HIG Reduce Motion — "omit motion that does not convey
/// information"; a blink is decorative).
///
/// `epoch` anchors the blink phase to the global [`EPOCH`] so carets
/// rendered on the same frame share timing. Extracted from
/// [`render_caret`] so the blink-phase logic can be tested without a
/// GPUI render context.
pub(crate) fn caret_visible_at(epoch: Instant, now: Instant, blink_interval: Duration) -> bool {
    let elapsed_ms = now.duration_since(epoch).as_millis();
    let cycle_ms = blink_interval.as_millis() * 2;
    cycle_ms == 0 || (elapsed_ms % cycle_ms) < blink_interval.as_millis()
}

/// Renders a blinking caret element.
///
/// Pass `Duration::ZERO` for `blink_interval` to force a steady (always-on)
/// caret — the required behaviour when HIG Reduce Motion is active.
/// `line_height` sets the bar's height so the caret matches the trailing
/// line's leading regardless of the active `TextStyle`.
pub fn render_caret(
    kind: CaretKind,
    color: Hsla,
    now: Instant,
    blink_interval: Duration,
    line_height: Pixels,
) -> impl IntoElement {
    let epoch = *EPOCH.get_or_init(Instant::now);
    let visible = caret_visible_at(epoch, now, blink_interval);
    let opacity = if visible { 1.0 } else { 0.0 };

    match kind {
        CaretKind::Block => div()
            .w(px(BAR_WIDTH_PT))
            .h(line_height)
            .bg(color)
            .opacity(opacity),
        CaretKind::Circle => {
            // Reasoning-state dot: render as a filled circle sized to the
            // line height so it reads as a bullet rather than a text glyph.
            let diameter = f32::from(line_height) * 0.5;
            div()
                .w(px(diameter))
                .h(px(diameter))
                .rounded_full()
                .bg(color)
                .opacity(opacity)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CaretKind, EPOCH, caret_visible_at};
    use core::prelude::v1::test;
    use std::time::{Duration, Instant};

    #[test]
    fn caret_kind_equality() {
        assert_eq!(CaretKind::Block, CaretKind::Block);
        assert_eq!(CaretKind::Circle, CaretKind::Circle);
        assert_ne!(CaretKind::Block, CaretKind::Circle);
    }

    #[test]
    fn caret_kind_copy() {
        let k = CaretKind::Block;
        let k2 = k;
        assert_eq!(k, k2);
    }

    #[test]
    fn epoch_is_consistent() {
        let e1 = *EPOCH.get_or_init(Instant::now);
        let e2 = *EPOCH.get_or_init(Instant::now);
        assert_eq!(e1, e2);
    }

    #[test]
    fn zero_interval_is_always_visible() {
        // Reduce Motion substitutes `Duration::ZERO` so the caret stays
        // on continuously rather than blinking.
        let t0 = Instant::now();
        for offset_ms in [0u64, 50, 250, 500, 1_000, 5_000] {
            let later = t0 + Duration::from_millis(offset_ms);
            assert!(
                caret_visible_at(t0, later, Duration::ZERO),
                "expected visible at +{offset_ms}ms with zero blink interval"
            );
        }
    }

    #[test]
    fn non_zero_interval_oscillates() {
        let epoch = Instant::now();
        let interval = Duration::from_millis(500);
        // First half-cycle: visible.
        assert!(caret_visible_at(epoch, epoch, interval));
        assert!(caret_visible_at(epoch, epoch + Duration::from_millis(250), interval));
        // Second half-cycle: hidden.
        assert!(!caret_visible_at(
            epoch,
            epoch + Duration::from_millis(600),
            interval
        ));
        assert!(!caret_visible_at(
            epoch,
            epoch + Duration::from_millis(999),
            interval
        ));
        // Back to visible after a full cycle (1000ms).
        assert!(caret_visible_at(
            epoch,
            epoch + Duration::from_millis(1_100),
            interval
        ));
    }
}
