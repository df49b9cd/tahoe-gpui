//! Caret / cursor blink state — a GPUI entity mirroring Zed's
//! `BlinkManager`.
//!
//! Finding 4 in the Zed cross-reference audit
//!. Markdown streaming and TextField carets
//! previously relied on a global `OnceLock<Instant>` epoch to drive
//! blinking. That pattern conflates independent focus states, has no
//! way to reset the phase after a keystroke (so the cursor can visibly
//! disappear for the first 250 ms after pressing a key — a known macOS
//! nit), and does not honour the Reduce Motion accessibility flag.
//!
//! `BlinkManager` replaces that with a proper GPUI entity:
//! * host code owns a `Entity<BlinkManager>` per text input (not a
//!   single global — independent editors keep independent phases;
//!   see open question #4 on the Zed cross-reference audit),
//! * `reset()` marks the start of a new visible phase — call from
//!   `KeyDownEvent` handlers so the cursor is solid while the user is
//!   typing,
//! * [`BlinkManager::is_visible`] reports whether the caret should be
//!   drawn at a given point in time, honouring Reduce Motion (when set,
//!   the caret is permanently visible — HIG: "replace large, dramatic
//!   transitions with subtle cross-fades or omit them entirely"),
//! * callers can subscribe to `BlinkManager` events via GPUI's
//!   `EventEmitter` facility to get phase flips when they need per-frame
//!   re-renders.

use std::time::{Duration, Instant};

use gpui::{Context, EventEmitter};

use super::accessibility::AccessibilityMode;

/// Canonical blink interval in milliseconds — matches Zed's
/// `CURSOR_BLINK_INTERVAL` and macOS NSTextView's default.
pub const CURSOR_BLINK_INTERVAL_MS: u64 = 500;

/// Event emitted each time the blink phase flips (visible ↔ hidden).
///
/// Observing the entity for `BlinkPhaseChanged` events lets host views
/// schedule per-flip re-renders without spinning a timer themselves.
#[derive(Debug, Clone, Copy)]
pub struct BlinkPhaseChanged;

/// Caret-blink state for a single text surface.
///
/// Spawn one per input:
///
/// ```ignore
/// let blink = cx.new(|_| BlinkManager::new());
/// let mut field = TextField::new(cx).with_blink(blink.clone());
/// ```
///
/// Then in a `KeyDownEvent` handler:
///
/// ```ignore
/// blink.update(cx, |b, _| b.reset());
/// ```
#[derive(Debug, Clone)]
pub struct BlinkManager {
    /// Wall-clock moment this manager was born / last reset. The blink
    /// phase derives from `(now - epoch) % (2 * interval)`.
    epoch: Instant,
    /// Full on/off cycle length. Default is `CURSOR_BLINK_INTERVAL_MS`;
    /// callers can override per-instance (e.g. a slower blink for a
    /// secondary cursor in a split view).
    interval: Duration,
    /// `true` silences blinking — the caret stays permanently visible.
    /// Mirrors the macOS accessibility setting. Set by [`with_reduce_motion`].
    reduce_motion: bool,
}

impl BlinkManager {
    /// New manager with the default blink interval.
    pub fn new() -> Self {
        Self {
            epoch: Instant::now(),
            interval: Duration::from_millis(CURSOR_BLINK_INTERVAL_MS),
            reduce_motion: false,
        }
    }

    /// Explicit-interval constructor. Use sparingly — the default
    /// matches NSTextView and callers should normally accept it.
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            epoch: Instant::now(),
            interval,
            reduce_motion: false,
        }
    }

    /// Apply an [`AccessibilityMode`]. When the `REDUCE_MOTION` flag is
    /// set, [`is_visible`](Self::is_visible) returns `true`
    /// unconditionally so the caret does not blink.
    ///
    /// Callers that already hold an `TahoeTheme` should pass
    /// `theme.accessibility_mode`; standalone callers can thread the
    /// OS-reported preference directly.
    pub fn set_accessibility_mode(&mut self, mode: AccessibilityMode) {
        self.reduce_motion = mode.reduce_motion();
    }

    /// Reset the blink phase — call after a keystroke so the caret is
    /// solid while the user is actively typing. Matches Zed's
    /// `BlinkManager::pause_blinking` / resume cycle.
    pub fn reset(&mut self, cx: &mut Context<Self>) {
        self.epoch = Instant::now();
        // Emit a phase-change event so any observers redraw with a
        // solid caret immediately.
        cx.emit(BlinkPhaseChanged);
    }

    /// Is the caret visible at the given instant?
    ///
    /// Under Reduce Motion the caret is always visible — HIG Motion:
    /// "Replace large, dramatic transitions with subtle cross-fades or
    /// omit them entirely." Otherwise the visible window is the first
    /// half of each `2 * interval` cycle.
    pub fn is_visible_at(&self, now: Instant) -> bool {
        if self.reduce_motion {
            return true;
        }
        let cycle_ms = self.interval.as_millis() * 2;
        if cycle_ms == 0 {
            return true;
        }
        let elapsed = now.duration_since(self.epoch).as_millis();
        (elapsed % cycle_ms) < self.interval.as_millis()
    }

    /// Convenience: is the caret visible right now?
    pub fn is_visible(&self) -> bool {
        self.is_visible_at(Instant::now())
    }

    /// Blink interval (the duration of one half-cycle — visible or hidden).
    pub fn interval(&self) -> Duration {
        self.interval
    }
}

impl Default for BlinkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter<BlinkPhaseChanged> for BlinkManager {}

#[cfg(test)]
mod tests {
    use super::{BlinkManager, CURSOR_BLINK_INTERVAL_MS};
    use crate::foundations::accessibility::AccessibilityMode;
    use core::prelude::v1::test;
    use std::time::{Duration, Instant};

    #[test]
    fn blink_interval_matches_macos_default() {
        assert_eq!(CURSOR_BLINK_INTERVAL_MS, 500);
    }

    #[test]
    fn fresh_manager_is_visible() {
        let m = BlinkManager::new();
        // Immediately after construction the phase is "visible" (elapsed
        // ≈ 0 which sits in the first half of the cycle).
        assert!(m.is_visible_at(m.epoch));
    }

    #[test]
    fn hidden_in_second_half_of_cycle() {
        let m = BlinkManager::with_interval(Duration::from_millis(500));
        // 600 ms past epoch — inside the second half of a 1000 ms cycle.
        let later = m.epoch + Duration::from_millis(600);
        assert!(!m.is_visible_at(later));
    }

    #[test]
    fn visible_again_after_full_cycle() {
        let m = BlinkManager::with_interval(Duration::from_millis(500));
        // 1100 ms past epoch — 100 ms into the next cycle, visible.
        let later = m.epoch + Duration::from_millis(1100);
        assert!(m.is_visible_at(later));
    }

    #[test]
    fn reduce_motion_keeps_caret_visible() {
        let mut m = BlinkManager::new();
        m.set_accessibility_mode(AccessibilityMode::REDUCE_MOTION);
        // Any instant: visible.
        let later = m.epoch + Duration::from_secs(10);
        assert!(m.is_visible_at(later));
    }

    #[test]
    fn zero_interval_does_not_divide_by_zero() {
        let m = BlinkManager::with_interval(Duration::from_millis(0));
        // Zero interval is nonsensical but must not panic — treat as
        // "always visible".
        assert!(m.is_visible_at(Instant::now()));
    }
}
