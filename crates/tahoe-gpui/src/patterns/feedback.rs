//! Feedback pattern aligned with HIG.
//!
//! Feedback helps people understand the results of their actions. HIG lists
//! six best practices:
//!
//! 1. **Communicate status clearly** — keep people informed about changes
//!    relevant to their current task (background sync, save confirmation).
//! 2. **Time feedback correctly** — show it when the result occurs, not
//!    before; avoid flicker from feedback that appears and disappears too
//!    fast to perceive.
//! 3. **Make error messages constructive** — say what happened, why, and
//!    what the user can do about it. Avoid blame and jargon.
//! 4. **Use system-provided mechanisms** — alerts, HUD windows, progress
//!    indicators, haptics — before inventing bespoke surfaces.
//! 5. **Match feedback intensity to the importance of the event** — a
//!    success confirmation is subtle; a destructive error is prominent.
//! 6. **Avoid unnecessary interruptions** — prefer inline contextual
//!    feedback over modal alerts when a natural anchor exists.
//!
//! # See also
//!
//! - [`crate::components::presentation::alert::Alert`] — modal feedback for
//!   events that must block the workflow (destructive confirms, critical
//!   errors).
//! - [`crate::components::status::progress_indicator::ProgressIndicator`]
//!   — determinate task progress.
//! - [`crate::components::status::activity_indicator::ActivityIndicator`]
//!   — indeterminate background work.
//! - [`crate::components::selection_and_input::text_field::TextFieldValidation`]
//!   — inline field-level error feedback (constructive, anchored to the
//!   offending control).
//! - `Zed` `crates/workspace/src/notifications.rs` — transient overlay
//!   notification pattern for events without a natural inline location;
//!   treat as reference for a future toast implementation.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/feedback>

/// Feedback style for user actions per HIG.
///
/// Use to indicate the result of an action without a full alert/notification.
///
/// **Scaffold type.** Reserved for the future inline feedback widget
/// (tracked as `TODO(feedback-widget)` — see module docs). The enum is
/// kept public so consuming crates can pre-express style intent; the
/// actual banner/toast component is not yet implemented in this crate.
#[doc(hidden)] // TODO(feedback-widget): un-hide once the banner component lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FeedbackStyle {
    /// No explicit feedback (action is self-evident).
    #[default]
    None,
    /// Success feedback (checkmark flash, green highlight per
    /// `systemGreen`: `#34C759` light / `#30D158` dark).
    Success,
    /// Warning feedback (orange highlight per `systemOrange`:
    /// `#FF9500` light / `#FF9F0A` dark — macOS system warning color; the
    /// historical "amber" term was inaccurate).
    Warning,
    /// Error feedback (red highlight, shake animation per `systemRed`:
    /// `#FF3B30` light / `#FF453A` dark).
    Error,
    /// Informational feedback (blue highlight per `systemBlue`:
    /// `#007AFF` light / `#0A84FF` dark).
    Info,
}

/// Feedback intensity controls how prominently the feedback is displayed.
///
/// **Scaffold type.** Same caveat as [`FeedbackStyle`] — reserved for the
/// future inline feedback widget.
#[doc(hidden)] // TODO(feedback-widget): un-hide once the banner component lands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FeedbackIntensity {
    /// Subtle feedback (color change only).
    Subtle,
    /// Standard feedback (color change + brief animation).
    #[default]
    Standard,
    /// Prominent feedback (color change + animation + icon).
    Prominent,
}

#[cfg(test)]
mod tests {
    use super::{FeedbackIntensity, FeedbackStyle};
    use core::prelude::v1::test;

    #[test]
    fn feedback_style_default_is_none() {
        assert_eq!(FeedbackStyle::default(), FeedbackStyle::None);
    }

    #[test]
    fn feedback_intensity_default_is_standard() {
        assert_eq!(FeedbackIntensity::default(), FeedbackIntensity::Standard);
    }

    #[test]
    fn feedback_style_variants_distinct() {
        assert_ne!(FeedbackStyle::Success, FeedbackStyle::Warning);
        assert_ne!(FeedbackStyle::Warning, FeedbackStyle::Error);
        assert_ne!(FeedbackStyle::Error, FeedbackStyle::Info);
        assert_ne!(FeedbackStyle::Info, FeedbackStyle::None);
    }
}
