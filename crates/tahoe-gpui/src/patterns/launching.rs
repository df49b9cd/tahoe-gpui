//! Launching pattern aligned with HIG.
//!
//! App launch should feel instant. HIG: show a placeholder that
//! approximates the eventual layout (shimmer skeleton) rather than a
//! static splash screen, and avoid modal first-run prompts — onboarding
//! belongs inline in the primary surface. Preserve the last open
//! document / selection across launches.
//!
//! # State restoration
//!
//! HIG §Launching: "Restore the previous state so people can continue
//! where they left off. If someone was in the middle of a task when they
//! last used your app, return them to that point."
//!
//! This crate exposes the [`StateRestoration`] trait as the HIG-aligned
//! contract hosts implement to preserve state across launches. A default
//! [`InMemoryStateRestoration`] is provided for tests and short-lived
//! processes; real applications plug in a disk-backed store (typically
//! `~/Library/Application Support/<bundle>/state.json` on macOS).
//!
//! ```ignore
//! use tahoe_gpui::patterns::launching::{
//!     InMemoryStateRestoration, StateRestoration,
//! };
//!
//! #[derive(Clone)]
//! struct AppState { open_document: Option<String>, selection: usize }
//!
//! let store = InMemoryStateRestoration::<AppState>::new();
//! store.save(AppState { open_document: Some("doc.md".into()), selection: 42 });
//! let restored = store.restore(); // Some(AppState { .. })
//! ```
//!
//! # See also
//!
//! - [`crate::components::status::shimmer::Shimmer`] — skeleton
//!   placeholder that pulses while content is loading; respects Reduce
//!   Motion.
//! - [`crate::components::status::activity_indicator::ActivityIndicator`]
//!   — 12-tick radial spinner for indeterminate startup work.
//! - [`crate::patterns::loading::LoadingState`] — state machine for
//!   `Idle → Loading → Loaded | Failed` transitions during launch.
//! - [`crate::patterns::onboarding`] — first-run guidance patterns.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/launching>

use std::sync::{Arc, Mutex};

/// Contract for saving and restoring host-level state across launches.
///
/// The trait is intentionally minimal — a single `save` accepting `&T`
/// and a `restore` returning `Option<T>`. Persistence, serialisation,
/// and error handling are delegated to implementers so hosts can plug in
/// whatever storage primitive they already ship (JSON on disk, SQLite,
/// `UserDefaults`, a custom protocol handler, …). Keep state objects
/// small and HIG-appropriate: last open document path, selection
/// indices, window frame — not anything the user would consider secret.
///
/// HIG §Launching: "Restore the previous state so people can continue
/// where they left off."
pub trait StateRestoration<T>: Send + Sync {
    /// Persist the given state so a later [`restore`](Self::restore)
    /// call can return it. Implementations may deduplicate, batch, or
    /// debounce — the trait imposes no durability guarantees beyond
    /// "the most recent `save` should be visible to the next
    /// `restore`."
    fn save(&self, value: T);

    /// Return the most recently saved state, or `None` if nothing has
    /// been saved (first launch) or the stored value could not be
    /// loaded (corrupt file, schema mismatch — implementers are free to
    /// discard rather than surface the error, since launch-time state
    /// is always a best-effort hint).
    fn restore(&self) -> Option<T>;

    /// Remove any previously saved state. Typical call sites: sign-out,
    /// "Reset to defaults" in settings, explicit user opt-out.
    fn clear(&self);
}

/// In-memory [`StateRestoration`] implementation backed by an
/// `Arc<Mutex<Option<T>>>`. Suitable for tests and short-lived
/// processes; for durable state across launches, implement
/// [`StateRestoration`] on a disk-backed store.
pub struct InMemoryStateRestoration<T> {
    slot: Arc<Mutex<Option<T>>>,
}

impl<T> InMemoryStateRestoration<T> {
    /// Create an empty store.
    pub fn new() -> Self {
        Self {
            slot: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T> Default for InMemoryStateRestoration<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for InMemoryStateRestoration<T> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot.clone(),
        }
    }
}

impl<T: Clone + Send + Sync> StateRestoration<T> for InMemoryStateRestoration<T> {
    fn save(&self, value: T) {
        // `Mutex::lock` only fails when another thread panicked while
        // holding the lock. State restoration is a best-effort surface
        // — drop the update rather than propagate a poisoned lock.
        if let Ok(mut guard) = self.slot.lock() {
            *guard = Some(value);
        }
    }

    fn restore(&self) -> Option<T> {
        self.slot.lock().ok().and_then(|guard| guard.clone())
    }

    fn clear(&self) {
        if let Ok(mut guard) = self.slot.lock() {
            *guard = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryStateRestoration, StateRestoration};
    use core::prelude::v1::test;

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        open_document: Option<String>,
        selection: usize,
    }

    #[test]
    fn restore_before_save_returns_none() {
        let store = InMemoryStateRestoration::<TestState>::new();
        assert!(store.restore().is_none());
    }

    #[test]
    fn save_then_restore_roundtrips() {
        let store = InMemoryStateRestoration::<TestState>::new();
        let state = TestState {
            open_document: Some("doc.md".into()),
            selection: 42,
        };
        store.save(state.clone());
        assert_eq!(store.restore(), Some(state));
    }

    #[test]
    fn save_replaces_previous_state() {
        let store = InMemoryStateRestoration::<TestState>::new();
        store.save(TestState {
            open_document: Some("a.md".into()),
            selection: 1,
        });
        store.save(TestState {
            open_document: Some("b.md".into()),
            selection: 2,
        });
        assert_eq!(
            store.restore().map(|s| s.open_document),
            Some(Some("b.md".into()))
        );
    }

    #[test]
    fn clear_removes_saved_state() {
        let store = InMemoryStateRestoration::<TestState>::new();
        store.save(TestState {
            open_document: Some("doc.md".into()),
            selection: 42,
        });
        store.clear();
        assert!(store.restore().is_none());
    }

    #[test]
    fn clone_shares_slot_across_handles() {
        let store_a = InMemoryStateRestoration::<TestState>::new();
        let store_b = store_a.clone();
        store_a.save(TestState {
            open_document: Some("shared.md".into()),
            selection: 7,
        });
        assert_eq!(
            store_b.restore().map(|s| s.open_document),
            Some(Some("shared.md".into()))
        );
    }
}
