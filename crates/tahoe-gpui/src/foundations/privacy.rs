//! Privacy foundation aligned with HIG Privacy.
//!
//! HIG privacy guidance: collect only the data the feature needs, ask
//! for permission contextually (the moment the user invokes the
//! feature, not on launch), explain the *why* in human language, and
//! surface an easy path to revoke the grant. AI-SDK features that
//! capture voice, files, or clipboard must be especially explicit —
//! both about what gets sent and where it's processed.
//!
//! This module does not ship a generic "permission prompt" type — each
//! permission (microphone, camera, location, photos, accessibility,
//! automation) has platform-specific machinery that belongs in the
//! host app, not a UI library. Provided here as a documentation
//! anchor so HIG audits have a place to land.
//!
//! # In-app affordances the crate already provides
//!
//! - [`crate::components::presentation::alert::Alert`] — use for the
//!   pre-prompt explainer shown before triggering the system dialog.
//!   HIG calls out that a custom explainer improves grant rates vs
//!   showing the system dialog cold.
//! - [`crate::voice`] (requires `voice` feature) — microphone
//!   permission states are modelled via `SpeechInputState::{
//!   PermissionRequired, PermissionDenied }` so the UI can route
//!   denied users to settings.
//! - [`crate::components::content::badge::Badge`] with
//!   `BadgeVariant::Info` — inline privacy-indicator chips next to
//!   fields that touch protected data.
//!
//! # See also
//!
//! - [`crate::patterns::managing_accounts`] — sign-in flows that touch
//!   credential data.
//! - [`crate::patterns::offering_help`] — pattern for the "Why do we
//!   need this?" explainer attached to a permission prompt.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/privacy>
//!
//! Tracked by `docs/hig/foundations.md:1140`.
