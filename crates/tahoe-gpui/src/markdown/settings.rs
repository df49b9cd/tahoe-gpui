//! Configuration for the streaming markdown renderer.

use std::time::Duration;

use gpui::Hsla;

use super::animation::{AnimationKind, Easing};
use super::caret::CaretKind;

/// Configuration for the streaming markdown renderer.
#[derive(Debug, Clone)]
pub struct StreamSettings {
    /// The animation to apply to newly revealed words.
    pub animation: AnimationKind,
    /// How long each word's animation takes.
    pub animation_duration: Duration,
    /// Delay between each word's animation start.
    pub animation_stagger: Duration,
    /// Easing function for the animation curve.
    pub easing: Easing,
    /// The caret to show at the insertion point, if any.
    pub caret: Option<CaretKind>,
    /// Override color for the caret. If `None`, uses the text color.
    pub caret_color: Option<Hsla>,
    /// How fast the caret blinks (half-period).
    pub caret_blink_interval: Duration,
    /// Override text color for animated spans. If `None`, uses theme text color.
    pub text_color: Option<Hsla>,
    /// Respect `AccessibilityMode::REDUCE_MOTION`: suppress per-word fade-in
    /// and reveal words immediately. Callers should initialize this from
    /// `TahoeTheme::accessibility_mode.reduce_motion()`. Default: `false`
    /// so non-themed callers keep the existing animated behavior.
    pub reduce_motion: bool,
}

impl Default for StreamSettings {
    fn default() -> Self {
        Self {
            animation: AnimationKind::FadeIn,
            animation_duration: Duration::from_millis(300),
            animation_stagger: Duration::from_millis(30),
            easing: Easing::EaseOut,
            caret: Some(CaretKind::Block),
            caret_color: None,
            caret_blink_interval: Duration::from_millis(530),
            text_color: None,
            reduce_motion: false,
        }
    }
}

impl StreamSettings {
    /// Creates new settings with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the animation kind.
    pub fn animation(mut self, kind: AnimationKind) -> Self {
        self.animation = kind;
        self
    }

    /// Sets the animation duration per word.
    pub fn animation_duration(mut self, duration: Duration) -> Self {
        self.animation_duration = duration;
        self
    }

    /// Sets the stagger between word animation starts.
    pub fn animation_stagger(mut self, stagger: Duration) -> Self {
        self.animation_stagger = stagger;
        self
    }

    /// Sets the easing function.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the caret kind.
    pub fn caret(mut self, kind: Option<CaretKind>) -> Self {
        self.caret = kind;
        self
    }

    /// Sets the caret color.
    pub fn caret_color(mut self, color: Option<Hsla>) -> Self {
        self.caret_color = color;
        self
    }

    /// Sets the caret blink interval.
    pub fn caret_blink_interval(mut self, interval: Duration) -> Self {
        self.caret_blink_interval = interval;
        self
    }

    /// Enable or disable the Reduce Motion override. Intended to be set
    /// from `TahoeTheme::accessibility_mode.reduce_motion()` at construction.
    pub fn reduce_motion(mut self, reduce_motion: bool) -> Self {
        self.reduce_motion = reduce_motion;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::StreamSettings;
    use crate::markdown::animation::{AnimationKind, Easing};
    use core::prelude::v1::test;
    use std::time::Duration;

    #[test]
    fn default_settings() {
        let s = StreamSettings::default();
        assert_eq!(s.animation, AnimationKind::FadeIn);
        assert_eq!(s.easing, Easing::EaseOut);
        assert!(s.caret.is_some());
    }

    #[test]
    fn builder_pattern() {
        let s = StreamSettings::new()
            .animation(AnimationKind::None)
            .caret(None)
            .animation_duration(Duration::from_millis(100));
        assert_eq!(s.animation, AnimationKind::None);
        assert!(s.caret.is_none());
        assert_eq!(s.animation_duration, Duration::from_millis(100));
    }

    #[test]
    fn caret_color_override() {
        let accent = crate::foundations::theme::TahoeTheme::default().accent;
        let s = StreamSettings::new().caret_color(Some(accent));
        assert!(s.caret_color.is_some());
    }

    #[test]
    fn new_equals_default() {
        let a = StreamSettings::new();
        let b = StreamSettings::default();
        assert_eq!(a.animation, b.animation);
        assert_eq!(a.easing, b.easing);
        assert_eq!(a.caret, b.caret);
    }

    #[test]
    fn caret_blink_interval_default_is_apple_nstextview_530ms() {
        // Apple's NSTextView insertion-point blink interval is 500–530ms
        // (the toolkit uses a half-period in this range). Defaulting to
        // 530ms keeps the streaming caret feeling native.
        let s = StreamSettings::default();
        assert_eq!(s.caret_blink_interval, Duration::from_millis(530));
    }

    #[test]
    fn reduce_motion_builder_flag_flows_through() {
        let s = StreamSettings::new().reduce_motion(true);
        assert!(s.reduce_motion);
        let s = StreamSettings::new().reduce_motion(false);
        assert!(!s.reduce_motion);
    }
}
