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
//! ## Integrating with stateless builders
//!
//! [`Alert`](crate::components::presentation::alert::Alert),
//! [`Sheet`](crate::components::presentation::sheet::Sheet),
//! [`Modal`](crate::components::presentation::modal::Modal), and
//! [`ActionSheet`](crate::components::presentation::action_sheet::ActionSheet)
//! are stateless builders — they do not own `is_open` or the
//! [`ActiveModal`] slot themselves. Store both on the parent entity
//! that renders the presenter, and acquire / drop the guard in the
//! same `open` / `close` handlers that flip `is_open`:
//!
//! ```ignore
//! struct MyDialogHost {
//!     is_open: bool,
//!     modal_guard: Option<ActiveModal>,
//! }
//!
//! impl MyDialogHost {
//!     fn open(&mut self, _cx: &mut Context<Self>) {
//!         if self.modal_guard.is_none() {
//!             self.modal_guard = Some(ModalGuard::global().present());
//!             self.is_open = true;
//!         }
//!     }
//!
//!     fn close(&mut self, _cx: &mut Context<Self>) {
//!         self.is_open = false;
//!         self.modal_guard = None; // drops ActiveModal → decrements depth
//!     }
//! }
//! ```
//!
//! The example is marked `ignore` so it documents the pattern without
//! forcing a full GPUI test context into doctest compilation.
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
        // Construct the drop-guard BEFORE debug_assert! so a debug-build
        // panic on nested presentation unwinds through ActiveModal::drop,
        // decrementing the counter instead of leaking it.
        let active = ActiveModal {
            depth: self.depth.clone(),
        };
        let new_depth = {
            let mut depth = active
                .depth
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *depth += 1;
            *depth
        };
        debug_assert!(
            new_depth <= 1,
            "HIG §Modality: nested modal presentation (depth={new_depth}). \
             Dismiss the existing modal before presenting another — stacking \
             modals traps the user."
        );
        active
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
        let mut depth = self
            .depth
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *depth > 0 {
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
    #[cfg(debug_assertions)]
    fn nested_present_debug_panic_does_not_leak_depth() {
        use std::panic;
        let guard = ModalGuard::new();
        let _outer = guard.present();
        // Catch the debug_assert! panic triggered by the nested present.
        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
            let _inner = guard.present();
        }));
        assert!(result.is_err(), "nested present should panic in debug");
        // Outer is still active; inner unwound and decremented — depth
        // must be back to 1, not 2.
        assert_eq!(guard.depth(), 1);
        drop(_outer);
        assert_eq!(guard.depth(), 0);
    }

    #[test]
    fn global_returns_shared_instance() {
        let a = ModalGuard::global();
        let b = ModalGuard::global();
        let _active = a.present();
        assert_eq!(b.depth(), a.depth());
        drop(_active);
    }

    #[test]
    fn present_recovers_from_poisoned_mutex() {
        use std::thread;
        let guard = ModalGuard::new();
        // Poison the internal mutex by panicking while holding the lock on
        // another thread. Use a per-instance guard (not ModalGuard::global)
        // so the poisoned state doesn't leak into other tests.
        let depth = guard.depth.clone();
        let handle = thread::spawn(move || {
            let _locked = depth.lock().expect("fresh mutex should lock cleanly");
            panic!("intentional poison");
        });
        assert!(handle.join().is_err(), "thread should have panicked");
        // present() must succeed and depth() must reflect the increment.
        let active = guard.present();
        assert_eq!(guard.depth(), 1);
        // Drop must also decrement — PoisonError::into_inner does not clear
        // the poison flag, so drop has to recover on every lock attempt too.
        drop(active);
        assert_eq!(guard.depth(), 0, "drop must decrement even after poison");
    }
}
