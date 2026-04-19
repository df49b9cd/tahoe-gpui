//! Word-level animation state for streaming content.
//!
//! Ported from `iced-ai-streamdown` (df49b9cd/iced#1). Uses a flat Vec of
//! reveal timestamps rather than per-word animation instances, keeping overhead
//! minimal even for thousands of words.

use std::time::{Duration, Instant};

/// What kind of animation to apply when revealing new words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationKind {
    /// Fade in from transparent to opaque.
    #[default]
    FadeIn,
    /// No animation; words appear instantly.
    None,
}

/// Easing functions for animation curves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Easing {
    Linear,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    EaseOutCubic,
}

/// Tracks word-level animation state for streaming content.
///
/// Uses a flat `Vec<Instant>` of reveal timestamps indexed by global word
/// position. Each word has a reveal timestamp; opacity is computed on-demand
/// per frame.
#[derive(Debug, Clone)]
pub struct AnimationState {
    word_reveal_times: Vec<Instant>,
    kind: AnimationKind,
    duration: Duration,
    stagger: Duration,
    easing: Easing,
    /// When true, `word_opacity` short-circuits to 1.0 (no fade-in).
    /// Set from `TahoeTheme::accessibility_mode.reduce_motion()` at the
    /// renderer level. Per HIG `foundations.md:1100`, animations that
    /// communicate status (streaming reveal) should collapse to an
    /// instant reveal when Reduce Motion is on.
    reduce_motion: bool,
}

impl AnimationState {
    /// Creates a new `AnimationState` with the given animation kind.
    pub fn new(kind: AnimationKind) -> Self {
        Self {
            word_reveal_times: Vec::new(),
            kind,
            duration: Duration::from_millis(300),
            stagger: Duration::from_millis(30),
            easing: Easing::EaseOut,
            reduce_motion: false,
        }
    }

    /// Sets the duration per word.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the stagger between word reveals.
    pub fn stagger(mut self, stagger: Duration) -> Self {
        self.stagger = stagger;
        self
    }

    /// Sets the easing function.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Enables or disables the Reduce Motion override.
    ///
    /// When enabled, `word_opacity` returns 1.0 for any registered word,
    /// producing an instant reveal even if `kind == AnimationKind::FadeIn`.
    pub fn reduce_motion(mut self, reduce_motion: bool) -> Self {
        self.reduce_motion = reduce_motion;
        self
    }

    /// Mutably toggle Reduce Motion after construction.
    pub fn set_reduce_motion(&mut self, reduce_motion: bool) {
        self.reduce_motion = reduce_motion;
    }

    /// Records `count` new words as revealed starting at `now`, each
    /// staggered by the configured stagger interval.
    pub fn reveal_words(&mut self, count: usize, now: Instant) {
        for i in 0..count {
            let reveal_at = now + self.stagger * i as u32;
            self.word_reveal_times.push(reveal_at);
        }
    }

    /// Computes the opacity (0.0..=1.0) for the word at the given global index.
    pub fn word_opacity(&self, global_word_index: usize, now: Instant) -> f32 {
        // `AnimationKind::None` semantics: no animation state is tracked,
        // so every index is opaque. Backward-compatible with the prior
        // behavior the streaming renderer relied on.
        if matches!(self.kind, AnimationKind::None) {
            return 1.0;
        }

        // Reduce Motion: registered words are immediately opaque, but
        // unregistered indices still report 0.0 so in-flight stream
        // layout (placeholder spans that haven't arrived yet) stays
        // hidden. Matches HIG `foundations.md:1100`.
        if self.reduce_motion {
            return if global_word_index < self.word_reveal_times.len() {
                1.0
            } else {
                0.0
            };
        }

        let Some(reveal_time) = self.word_reveal_times.get(global_word_index) else {
            return 0.0;
        };

        if now < *reveal_time {
            return 0.0;
        }

        let elapsed = now.duration_since(*reveal_time);
        let raw_t = if self.duration.as_nanos() == 0 {
            1.0
        } else {
            (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0)
        };

        apply_easing(raw_t, self.easing)
    }

    /// Returns `true` if any word is still mid-animation.
    pub fn is_animating(&self, now: Instant) -> bool {
        if matches!(self.kind, AnimationKind::None) || self.reduce_motion {
            return false;
        }

        self.word_reveal_times.last().is_some_and(|last| {
            now.checked_duration_since(*last)
                .is_none_or(|elapsed| elapsed < self.duration)
        })
    }

    /// Returns `true` once at least one word has been revealed.
    pub fn has_started(&self) -> bool {
        !self.word_reveal_times.is_empty()
    }

    /// Returns `true` when all revealed words have finished their animation.
    pub fn has_finished(&self, now: Instant) -> bool {
        if self.word_reveal_times.is_empty() {
            return false;
        }
        !self.is_animating(now)
    }

    /// Returns the total number of words that have been registered.
    pub fn word_count(&self) -> usize {
        self.word_reveal_times.len()
    }

    /// Returns the global word index up to which all words are fully opaque.
    ///
    /// Blocks whose entire word range falls below this threshold can skip
    /// per-word animation and use standard rendering.
    pub fn fully_revealed_watermark(&self, now: Instant) -> usize {
        if matches!(self.kind, AnimationKind::None) || self.reduce_motion {
            return self.word_reveal_times.len();
        }

        self.word_reveal_times.partition_point(|reveal_time| {
            now.checked_duration_since(*reveal_time)
                .is_some_and(|elapsed| elapsed >= self.duration)
        })
    }

    /// Resets all animation state.
    pub fn clear(&mut self) {
        self.word_reveal_times.clear();
    }
}

/// Applies the given easing to a linear `t` in 0.0..=1.0.
pub fn apply_easing(t: f32, easing: Easing) -> f32 {
    match easing {
        Easing::Linear => t,
        Easing::EaseInQuad => t * t,
        Easing::EaseOutQuad => t * (2.0 - t),
        Easing::EaseInOutQuad => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        Easing::EaseOut | Easing::EaseOutCubic => {
            let t1 = t - 1.0;
            1.0 + t1 * t1 * t1
        }
        Easing::EaseIn | Easing::EaseInCubic => t * t * t,
        Easing::EaseInOut => {
            if t < 0.5 {
                4.0 * t * t * t
            } else {
                let t1 = 2.0 * t - 2.0;
                0.5 * t1 * t1 * t1 + 1.0
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AnimationKind, AnimationState, Easing, apply_easing};
    use core::prelude::v1::test;
    use std::time::{Duration, Instant};

    fn now() -> Instant {
        Instant::now()
    }

    #[test]
    fn none_animation_always_opaque() {
        let state = AnimationState::new(AnimationKind::None);
        assert_eq!(state.word_opacity(0, now()), 1.0);
        assert_eq!(state.word_opacity(999, now()), 1.0);
    }

    #[test]
    fn unregistered_word_is_transparent() {
        let state = AnimationState::new(AnimationKind::FadeIn);
        assert_eq!(state.word_opacity(0, now()), 0.0);
    }

    #[test]
    fn word_fully_revealed_after_duration() {
        let mut state =
            AnimationState::new(AnimationKind::FadeIn).duration(Duration::from_millis(100));
        let t = now();
        state.reveal_words(1, t);
        let after = t + Duration::from_millis(200);
        assert_eq!(state.word_opacity(0, after), 1.0);
    }

    #[test]
    fn word_zero_before_reveal_time() {
        let mut state =
            AnimationState::new(AnimationKind::FadeIn).stagger(Duration::from_millis(1000));
        let t = now();
        state.reveal_words(2, t);
        assert_eq!(state.word_opacity(1, t), 0.0);
    }

    #[test]
    fn is_animating_while_mid_reveal() {
        let mut state =
            AnimationState::new(AnimationKind::FadeIn).duration(Duration::from_millis(500));
        let t = now();
        state.reveal_words(1, t);
        assert!(state.is_animating(t));
        assert!(!state.is_animating(t + Duration::from_secs(1)));
    }

    #[test]
    fn has_started_and_finished() {
        let mut state =
            AnimationState::new(AnimationKind::FadeIn).duration(Duration::from_millis(10));
        let t = now();
        assert!(!state.has_started());
        assert!(!state.has_finished(t));

        state.reveal_words(1, t);
        assert!(state.has_started());

        let later = t + Duration::from_secs(1);
        assert!(state.has_finished(later));
    }

    #[test]
    fn easing_boundaries() {
        for easing in [
            Easing::Linear,
            Easing::EaseIn,
            Easing::EaseOut,
            Easing::EaseInQuad,
            Easing::EaseOutQuad,
            Easing::EaseInOutQuad,
        ] {
            assert_eq!(apply_easing(0.0, easing), 0.0, "easing {:?} at 0", easing);
            assert!(
                (apply_easing(1.0, easing) - 1.0).abs() < 1e-6,
                "easing {:?} at 1",
                easing
            );
        }
    }

    #[test]
    fn easing_monotonic() {
        for easing in [Easing::Linear, Easing::EaseIn, Easing::EaseOut] {
            let mut prev = 0.0f32;
            for i in 0..=100 {
                let t = i as f32 / 100.0;
                let v = apply_easing(t, easing);
                assert!(
                    v >= prev - 1e-6,
                    "easing {:?} not monotonic at t={}",
                    easing,
                    t
                );
                prev = v;
            }
        }
    }

    #[test]
    fn fully_revealed_watermark_empty() {
        let state = AnimationState::new(AnimationKind::FadeIn);
        assert_eq!(state.fully_revealed_watermark(now()), 0);
    }

    #[test]
    fn fully_revealed_watermark_none_animation() {
        let mut state = AnimationState::new(AnimationKind::None);
        state.reveal_words(5, now());
        assert_eq!(state.fully_revealed_watermark(now()), 5);
    }

    #[test]
    fn fully_revealed_watermark_all_done() {
        let mut state = AnimationState::new(AnimationKind::FadeIn)
            .duration(Duration::from_millis(100))
            .stagger(Duration::from_millis(10));
        let t = now();
        state.reveal_words(10, t);
        assert_eq!(
            state.fully_revealed_watermark(t + Duration::from_millis(200)),
            10
        );
    }

    #[test]
    fn fully_revealed_watermark_partial() {
        let mut state = AnimationState::new(AnimationKind::FadeIn)
            .duration(Duration::from_millis(100))
            .stagger(Duration::from_millis(10));
        let t = now();
        state.reveal_words(10, t);
        assert_eq!(
            state.fully_revealed_watermark(t + Duration::from_millis(105)),
            1
        );
    }

    #[test]
    fn reduce_motion_reveals_registered_words_immediately() {
        let mut state = AnimationState::new(AnimationKind::FadeIn)
            .duration(Duration::from_millis(500))
            .stagger(Duration::from_millis(100))
            .reduce_motion(true);
        let t = now();
        state.reveal_words(3, t);
        // All registered words are at full opacity, despite FadeIn kind
        // and long duration/stagger.
        assert_eq!(state.word_opacity(0, t), 1.0);
        assert_eq!(state.word_opacity(2, t), 1.0);
        // Unregistered word remains transparent (index guard).
        assert_eq!(state.word_opacity(3, t), 0.0);
    }

    #[test]
    fn reduce_motion_is_animating_always_false() {
        let mut state = AnimationState::new(AnimationKind::FadeIn)
            .duration(Duration::from_millis(500))
            .reduce_motion(true);
        let t = now();
        state.reveal_words(1, t);
        assert!(!state.is_animating(t));
    }

    #[test]
    fn reduce_motion_watermark_counts_all_registered() {
        let mut state = AnimationState::new(AnimationKind::FadeIn)
            .duration(Duration::from_millis(500))
            .stagger(Duration::from_millis(100))
            .reduce_motion(true);
        let t = now();
        state.reveal_words(5, t);
        assert_eq!(state.fully_revealed_watermark(t), 5);
    }

    #[test]
    fn set_reduce_motion_can_be_toggled() {
        let mut state = AnimationState::new(AnimationKind::FadeIn)
            .duration(Duration::from_millis(500))
            .stagger(Duration::from_millis(100));
        let t = now();
        state.reveal_words(2, t);
        // Without reduce motion, word 1 is still 0.0 at t (100ms stagger)
        assert_eq!(state.word_opacity(1, t), 0.0);
        state.set_reduce_motion(true);
        assert_eq!(state.word_opacity(1, t), 1.0);
    }
}
