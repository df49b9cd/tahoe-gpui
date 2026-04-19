//! Collaboration and sharing pattern aligned with HIG.
//!
//! Collaboration surfaces attribute who did what. HIG: credit each
//! contributor with an avatar + name stack, show live presence via
//! subtle status indicators (dot badges, coloured cursors), and keep
//! share actions close to the content rather than hiding them behind a
//! menu. Respect privacy — let the user choose what identity is exposed.
//!
//! # See also
//!
//! - [`crate::components::content::avatar::Avatar`] — per-contributor
//!   identity glyph; stack via `FacePile` for multi-party attribution.
//! - [`crate::components::content::badge::Badge`] — presence / role
//!   chips (e.g. `"Editor"`, `"Viewer"`). Use `BadgeVariant::Dot` for
//!   live-presence indicators.
//! - [`crate::components::menus_and_actions::activity_view`] —
//!   iOS/iPadOS share sheet (stub; macOS uses a context-menu share
//!   instead).
//! - [`crate::code::commit`] — commit cards attribute author + message
//!   for collaborative code review.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/collaboration-and-sharing>
