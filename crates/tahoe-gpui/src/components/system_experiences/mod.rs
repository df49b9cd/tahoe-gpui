//! System experiences components (HIG: Components > System experiences).
//!
//! This is the 8th HIG component subcategory. Every entry here is a
//! **system-owned** surface (Widgets, Notifications, Live Activities,
//! Controls, App Shortcuts, Status Bars) or a platform-excluded surface
//! (Top Shelf/tvOS, Watch Faces/watchOS, Complications/watchOS). None of
//! them can be drawn by a GPUI component — the system renders them from
//! host-registered extensions (WidgetKit, ActivityKit, `NSStatusItem`,
//! App Intents).
//!
//! Each module exists so HIG audits find an anchor here and so consumers
//! are pointed at the nearest in-app analogue (e.g. notifications →
//! [`crate::components::presentation::alert::Alert`]; Live Activities →
//! [`crate::components::status::activity_indicator::ActivityIndicator`]).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/components/system-experiences>

pub mod app_shortcuts;
pub mod complications;
pub mod controls;
pub mod live_activities;
pub mod notifications;
pub mod status_bars;
pub mod top_shelf;
pub mod watch_faces;
pub mod widgets;
