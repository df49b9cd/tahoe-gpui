//! Managing notifications pattern aligned with HIG.
//!
//! HIG: notifications must be meaningful, time-sensitive, and easy to
//! silence. Respect system quiet-hours and Focus modes; never duplicate
//! a notification inside the app surface if the system has already
//! shown one. Notification *management* UI (enable/disable, frequency)
//! belongs in the app's settings surface.
//!
//! # See also
//!
//! - [`crate::components::system_experiences::notifications`] — system
//!   notifications surface (stub; delivered by the host OS).
//! - [`crate::components::presentation::alert::Alert`] — in-app modal
//!   for time-sensitive announcements that require acknowledgement.
//! - [`crate::components::content::badge::Badge`] with
//!   `BadgeVariant::Notification { count }` — unread-count indicator
//!   for toolbar / sidebar items.
//! - [`crate::patterns::settings`] — where notification preferences
//!   belong in the settings surface.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/managing-notifications>
