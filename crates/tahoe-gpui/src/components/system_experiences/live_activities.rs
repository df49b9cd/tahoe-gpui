//! Live Activities (HIG Live Activities).
//!
//! Live Activities are a **system-owned** surface (Lock Screen, Dynamic
//! Island, menu bar) backed by the ActivityKit framework. Apps push
//! updates via `Activity<ContentState>`; the system renders the UI. A
//! GPUI component cannot draw this surface.
//!
//! This module exists so HIG audits find an anchor here; there is no
//! drawable widget to ship.
//!
//! # Host integration
//!
//! Hosts start activities with `Activity.request(…)` and render the
//! lock-screen view in SwiftUI. No GPUI code lives on that path.
//!
//! # See also
//!
//! - [`crate::components::content::badge::Badge`] — in-app pendant for
//!   ongoing activities (`BadgeVariant::Notification { count }`).
//! - [`crate::components::status::activity_indicator::ActivityIndicator`]
//!   — in-app visual for indeterminate in-progress work.
//! - [`crate::components::status::progress_indicator::ProgressIndicator`]
//!   — in-app progress bar mirroring a live activity's content state.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/live-activities>
//!
//! Tracked by `docs/hig/components/system-experiences.md:370`.
