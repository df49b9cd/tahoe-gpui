//! Status bars (HIG Status bars).
//!
//! The macOS menu bar extras area and iOS top status bar are
//! **system-owned** surfaces. Menu bar extras on macOS are registered
//! via `NSStatusItem` in AppKit; the iOS status bar is drawn by the
//! system and only accepts style overrides (light/dark). GPUI cannot
//! draw this surface directly.
//!
//! This module exists so HIG audits find an anchor here; no drawable
//! widget lives here.
//!
//! # Host integration
//!
//! For macOS menu bar extras, the host creates `NSStatusItem` instances
//! and populates their menu in AppKit. GPUI can render the *content*
//! of a popover attached to the status item, but the status item
//! itself is AppKit-owned.
//!
//! # See also
//!
//! - [`crate::components::status::activity_indicator::ActivityIndicator`]
//!   — spinner that Zed uses for language-server status in its own
//!   in-window status bar (`crates/activity_indicator/`).
//! - [`crate::components::content::badge::Badge`] — status chips for a
//!   custom in-app status footer.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/status-bars>
//!
//! Tracked by `docs/hig/components/system-experiences.md:812`.
