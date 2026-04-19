//! Loading pattern aligned with HIG.
//!
//! The HIG recommends showing content as soon as possible and using
//! progressive loading to avoid blank screens. This module provides a
//! state machine for managing loading states across components.
//!
//! When the elapsed time is known, prefer [`LoadingState::LoadingWithProgress`]
//! with a 0.0–1.0 fraction so the host can render a determinate
//! [`ProgressIndicator`](crate::components::status::progress_indicator::ProgressIndicator).
//! When the duration is unbounded or unknown, use the indeterminate
//! [`LoadingState::Loading`] and render a
//! [`Shimmer`](crate::components::status::shimmer::Shimmer) or
//! [`ActivityIndicator`](crate::components::status::activity_indicator::ActivityIndicator)
//! instead.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/loading>

/// Loading state for content display per HIG.
///
/// Use to manage progressive content loading. Components can check the
/// loading state to decide whether to show placeholders, shimmer effects,
/// or real content.
///
/// Two loading variants are provided:
///
/// - [`Loading`](Self::Loading) — **indeterminate**, for work without a
///   known duration. Render with
///   [`Shimmer`](crate::components::status::shimmer::Shimmer) or
///   [`ActivityIndicator`](crate::components::status::activity_indicator::ActivityIndicator).
/// - [`LoadingWithProgress(f32)`](Self::LoadingWithProgress) —
///   **determinate**, carrying a 0.0–1.0 progress fraction. Render with
///   [`ProgressIndicator`](crate::components::status::progress_indicator::ProgressIndicator).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LoadingState {
    /// Content has not started loading.
    #[default]
    Idle,
    /// Content is loading without a known duration — show placeholder or
    /// shimmer.
    Loading,
    /// Content is loading with known progress (0.0u20131.0). Host components
    /// should render a determinate progress indicator.
    ///
    /// The fraction is clamped by [`LoadingState::progress`].
    LoadingWithProgress(f32),
    /// Content loaded successfully — show real content.
    Loaded,
    /// Loading failed — show error state.
    Failed,
}

impl LoadingState {
    /// Returns true if the state is [`Idle`](Self::Idle) (loading not yet
    /// started).
    pub fn is_idle(self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Returns true if content is still loading — covers both the
    /// indeterminate and determinate variants.
    pub fn is_loading(self) -> bool {
        matches!(self, Self::Loading | Self::LoadingWithProgress(_))
    }

    /// Returns true if content has loaded (successfully or not).
    pub fn is_complete(self) -> bool {
        matches!(self, Self::Loaded | Self::Failed)
    }

    /// Returns true if loading failed.
    pub fn is_failed(self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Returns the determinate progress fraction (0.0u20131.0) when the state
    /// is [`LoadingWithProgress`](Self::LoadingWithProgress), or `None`
    /// otherwise. Values are clamped into `0.0..=1.0` so callers can feed
    /// the result straight into a
    /// [`ProgressIndicator`](crate::components::status::progress_indicator::ProgressIndicator)
    /// without a separate clamp.
    pub fn progress(self) -> Option<f32> {
        match self {
            Self::LoadingWithProgress(p) => Some(p.clamp(0.0, 1.0)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LoadingState;
    use core::prelude::v1::test;

    #[test]
    fn idle_is_idle_and_not_loading() {
        assert!(LoadingState::Idle.is_idle());
        assert!(!LoadingState::Idle.is_loading());
    }

    #[test]
    fn indeterminate_loading_is_loading_without_progress() {
        assert!(LoadingState::Loading.is_loading());
        assert!(!LoadingState::Loading.is_idle());
        assert!(!LoadingState::Loading.is_complete());
        assert_eq!(LoadingState::Loading.progress(), None);
    }

    #[test]
    fn determinate_loading_exposes_clamped_progress() {
        let state = LoadingState::LoadingWithProgress(0.42);
        assert!(state.is_loading());
        assert_eq!(state.progress(), Some(0.42));
    }

    #[test]
    fn progress_clamped_below_zero() {
        assert_eq!(
            LoadingState::LoadingWithProgress(-0.5).progress(),
            Some(0.0)
        );
    }

    #[test]
    fn progress_clamped_above_one() {
        assert_eq!(LoadingState::LoadingWithProgress(1.7).progress(), Some(1.0));
    }

    #[test]
    fn complete_and_failed_predicates() {
        assert!(LoadingState::Loaded.is_complete());
        assert!(!LoadingState::Loaded.is_failed());
        assert!(LoadingState::Failed.is_complete());
        assert!(LoadingState::Failed.is_failed());
        assert_eq!(LoadingState::Loaded.progress(), None);
        assert_eq!(LoadingState::Failed.progress(), None);
    }
}
