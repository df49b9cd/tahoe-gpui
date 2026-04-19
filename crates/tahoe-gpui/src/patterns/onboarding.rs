//! Onboarding pattern aligned with HIG.
//!
//! HIG: show, don't tell. Onboarding belongs inside the primary surface
//! (inline coachmarks, empty-state callouts) rather than a separate
//! multi-screen tour; anything longer than one step should be
//! interruptible (Skip + persistent entry point to re-run). Avoid
//! requiring sign-in before the app is usable.
//!
//! # Flow primitive
//!
//! [`OnboardingFlow`] tracks an ordered list of [`OnboardingStep`]s,
//! enforces the Skip contract, and fires a completion callback. The
//! primitive intentionally has no rendering surface — hosts render each
//! step's `title` / `body` with their own layout (sheet, callout,
//! inline banner) so the pattern composes with Apple-native surfaces
//! rather than imposing its own chrome.
//!
//! ```ignore
//! use tahoe_gpui::patterns::onboarding::{OnboardingFlow, OnboardingStep};
//!
//! let mut flow = OnboardingFlow::new()
//!     .step(OnboardingStep::new("Welcome", "Sign in any time"))
//!     .step(OnboardingStep::new("Connect", "Pick a workspace"))
//!     .skip_allowed(true);
//!
//! while !flow.is_complete() {
//!     let step = flow.current_step().unwrap();
//!     render_step(step);
//!     flow.advance();
//! }
//! ```
//!
//! # See also
//!
//! - [`crate::components::content::avatar::Avatar`] — identity
//!   placeholder for first-run persona setup.
//! - [`crate::components::presentation::sheet::Sheet`] — modal container
//!   for multi-step guided flows that shouldn't disrupt the window
//!   chrome.
//! - [`crate::patterns::launching`] — paired pattern covering the app's
//!   first post-launch impression.
//! - Zed's `onboarding` crate (`crates/onboarding/`) — full-crate
//!   example of a production onboarding surface on GPUI.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/onboarding>

use gpui::SharedString;

/// A single onboarding step — title and explanatory body. Rendering is
/// left to the host so the step can appear as a sheet, callout, or
/// inline banner without the flow primitive knowing which.
#[derive(Debug, Clone)]
pub struct OnboardingStep {
    /// Short heading (e.g. "Welcome to Claude").
    pub title: SharedString,
    /// One-to-two-sentence explanatory body. HIG: keep copy short and
    /// action-oriented.
    pub body: SharedString,
}

impl OnboardingStep {
    /// Create a new step with the given title and body.
    pub fn new(title: impl Into<SharedString>, body: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }
}

/// State machine for a multi-step onboarding flow.
///
/// The flow carries an ordered `Vec<OnboardingStep>` plus a cursor
/// (`current`) that advances as the user accepts each step. When
/// `skip_allowed` is `true`, [`skip`](Self::skip) jumps to completion.
/// [`on_complete`](Self::on_complete) optionally wires a callback fired
/// when the flow terminates (either through `advance` past the last
/// step or through `skip`).
pub struct OnboardingFlow {
    steps: Vec<OnboardingStep>,
    current: usize,
    skip_allowed: bool,
    completed: bool,
    on_complete: Option<Box<dyn FnOnce() + Send + Sync + 'static>>,
}

impl OnboardingFlow {
    /// Create an empty flow. Add steps via [`Self::step`].
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            current: 0,
            skip_allowed: false,
            completed: false,
            on_complete: None,
        }
    }

    /// Append a step to the flow.
    pub fn step(mut self, step: OnboardingStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Allow the user to skip the remaining steps. HIG: "make it easy
    /// to skip." Default is `false` (the host must opt in).
    pub fn skip_allowed(mut self, allowed: bool) -> Self {
        self.skip_allowed = allowed;
        self
    }

    /// Register a completion callback fired after the final step is
    /// accepted or the flow is skipped.
    pub fn on_complete(mut self, f: impl FnOnce() + Send + Sync + 'static) -> Self {
        self.on_complete = Some(Box::new(f));
        self
    }

    /// The step currently in focus, or `None` when the flow is
    /// complete.
    pub fn current_step(&self) -> Option<&OnboardingStep> {
        if self.completed {
            None
        } else {
            self.steps.get(self.current)
        }
    }

    /// Zero-based index of the current step.
    pub fn current_index(&self) -> usize {
        self.current
    }

    /// Total number of steps in the flow.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// `true` once the flow has terminated (via `advance` past the last
    /// step or via `skip`).
    pub fn is_complete(&self) -> bool {
        self.completed
    }

    /// Whether the user may skip the remaining steps.
    pub fn can_skip(&self) -> bool {
        self.skip_allowed && !self.completed
    }

    /// Accept the current step and advance to the next. Marks the flow
    /// complete when there is no next step.
    pub fn advance(&mut self) {
        if self.completed {
            return;
        }
        self.current += 1;
        if self.current >= self.steps.len() {
            self.finish();
        }
    }

    /// Skip the remaining steps. No-op when `skip_allowed` is `false`.
    pub fn skip(&mut self) {
        if self.can_skip() {
            self.finish();
        }
    }

    fn finish(&mut self) {
        self.completed = true;
        if let Some(cb) = self.on_complete.take() {
            cb();
        }
    }
}

impl Default for OnboardingFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{OnboardingFlow, OnboardingStep};
    use core::prelude::v1::test;
    use std::sync::{Arc, Mutex};

    fn sample_flow() -> OnboardingFlow {
        OnboardingFlow::new()
            .step(OnboardingStep::new("Welcome", "Say hi"))
            .step(OnboardingStep::new("Connect", "Pick a workspace"))
    }

    #[test]
    fn new_flow_is_empty_not_complete() {
        let flow = OnboardingFlow::new();
        assert_eq!(flow.step_count(), 0);
        // Empty flow is considered complete because there is nothing to do.
        // We only expose `current_step()` == None in both the empty and
        // completed cases; the numeric `step_count == 0` is the
        // disambiguator.
        assert!(flow.current_step().is_none());
    }

    #[test]
    fn advance_walks_steps_then_completes() {
        let mut flow = sample_flow();
        assert_eq!(flow.current_step().map(|s| s.title.as_ref()), Some("Welcome"));
        flow.advance();
        assert_eq!(flow.current_step().map(|s| s.title.as_ref()), Some("Connect"));
        flow.advance();
        assert!(flow.is_complete());
        assert!(flow.current_step().is_none());
    }

    #[test]
    fn skip_disabled_by_default() {
        let mut flow = sample_flow();
        assert!(!flow.can_skip());
        flow.skip();
        assert!(!flow.is_complete());
    }

    #[test]
    fn skip_allowed_terminates_flow() {
        let mut flow = sample_flow().skip_allowed(true);
        assert!(flow.can_skip());
        flow.skip();
        assert!(flow.is_complete());
        assert!(!flow.can_skip());
    }

    #[test]
    fn on_complete_fires_once() {
        let counter = Arc::new(Mutex::new(0u32));
        let counter_cb = counter.clone();
        let mut flow = sample_flow()
            .skip_allowed(true)
            .on_complete(move || *counter_cb.lock().unwrap() += 1);
        flow.skip();
        flow.skip(); // should not refire
        flow.advance(); // should not refire
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn advance_past_end_is_idempotent() {
        let mut flow = sample_flow();
        flow.advance();
        flow.advance();
        assert!(flow.is_complete());
        // Extra advances should not panic or re-enter finish() logic.
        flow.advance();
        assert!(flow.is_complete());
    }
}
