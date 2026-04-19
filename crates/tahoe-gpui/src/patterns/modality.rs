//! Modality pattern aligned with HIG.
//!
//! Modality interrupts a user's workflow to focus attention on a single
//! task. HIG: use modality sparingly and reserve it for situations where
//! the user's full attention is required (error recovery, destructive
//! confirmations, content that requires complete review). Non-modal
//! alternatives (tooltips, inline errors, banners) are preferred for
//! advisory content.
//!
//! HIG lists three best practices explicitly:
//!
//! 1. **Use modal presentations sparingly.** Prefer inline or non-modal
//!    alternatives when they work.
//! 2. **Always provide a way to dismiss a modal view.** Every modal
//!    must expose at least one exit path (Cancel / Done / ⎋).
//! 3. **Avoid nesting modals.** Stacking a second modal on top of a
//!    presented modal traps the user and contradicts the "single task"
//!    framing modal presentation promises.
//!
//! # Runtime guard
//!
//! [`ModalGuard`] implements HIG best practice #3 at runtime: it keeps
//! a reference-counted depth so callers can detect nested presentation
//! attempts. A debug build panics when depth > 1; a release build
//! silently logs via `debug_assert!` and lets the nested presentation
//! proceed. Hosts wrap their modal presenters:
//!
//! ```ignore
//! use tahoe_gpui::patterns::modality::ModalGuard;
//!
//! let guard = ModalGuard::global();
//! let _active = guard.present();  // `_active` drops → guard decrements
//! ```
//!
//! # See also
//!
//! - [`crate::components::presentation::alert::Alert`] — modal dialogs
//!   with action buttons. Default for destructive confirmations.
//! - [`crate::components::presentation::sheet::Sheet`] — slide-up modal
//!   for task flows (form entry, picker detail).
//! - [`crate::components::presentation::modal::Modal`] — generic
//!   full-screen modal container.
//! - [`crate::components::presentation::popover::Popover`] — lightweight
//!   non-modal anchored overlay; use instead of a modal when the user
//!   should still see surrounding context.
//! - [`crate::components::presentation::action_sheet::ActionSheet`] —
//!   bottom-anchored action list, modal on iOS, non-modal on macOS.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/modality>

use std::sync::{Arc, Mutex, OnceLock};

/// RAII guard that tracks modal-presentation depth so callers can
/// enforce HIG "don't nest modals" at runtime.
///
/// Construct via [`ModalGuard::new`] (per-host instance) or
/// [`ModalGuard::global`] (process-wide singleton — the common case for
/// a single-window macOS app).
#[derive(Clone)]
pub struct ModalGuard {
    depth: Arc<Mutex<u32>>,
}

impl ModalGuard {
    /// Create a fresh guard with depth 0.
    pub fn new() -> Self {
        Self {
            depth: Arc::new(Mutex::new(0)),
        }
    }

    /// Return the process-wide singleton guard. Suitable for apps that
    /// only present one modal surface at a time.
    pub fn global() -> Self {
        static GLOBAL: OnceLock<ModalGuard> = OnceLock::new();
        GLOBAL.get_or_init(ModalGuard::new).clone()
    }

    /// Begin presenting a modal. The returned [`ActiveModal`] decrements
    /// the guard when dropped.
    ///
    /// In debug builds, a nested presentation (depth becomes > 1) fires
    /// a `debug_assert!` so the divergence from HIG surfaces during
    /// development. Release builds let the nested presentation proceed
    /// so a bug doesn't trap the user.
    #[must_use = "drop the ActiveModal to release the depth counter"]
    pub fn present(&self) -> ActiveModal {
        let new_depth = {
            let mut depth = self
                .depth
                .lock()
                .expect("ModalGuard mutex poisoned — a previous `present()` panicked");
            *depth += 1;
            *depth
        };
        debug_assert!(
            new_depth <= 1,
            "HIG §Modality: nested modal presentation (depth={new_depth}). \
             Dismiss the existing modal before presenting another — stacking \
             modals traps the user."
        );
        ActiveModal {
            depth: self.depth.clone(),
        }
    }

    /// Current presentation depth. Useful for tests and diagnostics.
    pub fn depth(&self) -> u32 {
        *self
            .depth
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// `true` when at least one modal is currently presented.
    pub fn is_active(&self) -> bool {
        self.depth() > 0
    }
}

impl Default for ModalGuard {
    fn default() -> Self {
        Self::new()
    }
}

/// Drop-guard returned by [`ModalGuard::present`]. Decrements the
/// presentation depth when the value goes out of scope.
pub struct ActiveModal {
    depth: Arc<Mutex<u32>>,
}

impl Drop for ActiveModal {
    fn drop(&mut self) {
        if let Ok(mut depth) = self.depth.lock()
            && *depth > 0
        {
            *depth -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ModalGuard;
    use core::prelude::v1::test;

    #[test]
    fn new_guard_has_depth_zero() {
        let guard = ModalGuard::new();
        assert_eq!(guard.depth(), 0);
        assert!(!guard.is_active());
    }

    #[test]
    fn present_increments_depth() {
        let guard = ModalGuard::new();
        let _active = guard.present();
        assert_eq!(guard.depth(), 1);
        assert!(guard.is_active());
    }

    #[test]
    fn drop_decrements_depth() {
        let guard = ModalGuard::new();
        {
            let _active = guard.present();
            assert_eq!(guard.depth(), 1);
        }
        assert_eq!(guard.depth(), 0);
        assert!(!guard.is_active());
    }

    #[test]
    fn sequential_presentations_track_independently() {
        let guard = ModalGuard::new();
        {
            let _a = guard.present();
            assert_eq!(guard.depth(), 1);
        }
        {
            let _b = guard.present();
            assert_eq!(guard.depth(), 1);
        }
        assert_eq!(guard.depth(), 0);
    }

    // In debug builds a nested present() fires `debug_assert!`, which
    // panics and aborts the test. Use #[should_panic] only in debug;
    // release builds silently allow it so the check would pass trivially.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "HIG §Modality")]
    fn nested_present_panics_in_debug() {
        let guard = ModalGuard::new();
        let _outer = guard.present();
        let _inner = guard.present();
    }

    #[test]
    fn global_returns_shared_instance() {
        let a = ModalGuard::global();
        let b = ModalGuard::global();
        let _active = a.present();
        assert_eq!(b.depth(), a.depth());
        drop(_active);
    }
}
