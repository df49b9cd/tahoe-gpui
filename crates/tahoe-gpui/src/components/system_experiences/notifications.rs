//! Notifications (HIG Notifications).
//!
//! System notifications (Notification Center banners, Lock Screen
//! entries) are a **system-owned** surface backed by
//! `UNUserNotificationCenter`. The host delivers notifications via the
//! user-notifications framework; the system draws them. GPUI cannot
//! render this surface itself.
//!
//! This module exists so HIG audits find an anchor here; no drawable
//! widget lives here. For in-app notifications, render an
//! [`Alert`](crate::components::presentation::alert::Alert) or a
//! badge on the relevant UI element instead.
//!
//! # Host integration
//!
//! Hosts schedule notifications via
//! `UNUserNotificationCenter.current().add(request)` in Swift. No GPUI
//! code lives on that path.
//!
//! # See also
//!
//! - [`crate::components::presentation::alert::Alert`] — in-app modal
//!   for advisory content that must be acknowledged.
//! - [`crate::components::content::badge::Badge`] with
//!   `BadgeVariant::Notification { count }` — unread-count marker.
//! - [`crate::patterns::managing_notifications`] — pattern guidance for
//!   how users configure notification preferences inside the app.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/notifications>
//!
//! Tracked by `docs/hig/components/system-experiences.md:687`.
