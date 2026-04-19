//! Design patterns aligned with HIG Patterns section.
//!
//! Patterns describe common user-facing flows (onboarding, searching,
//! entering data, drag-and-drop). Most are realised through component
//! composition rather than standalone types, so the modules here are
//! primarily *guidance* documents: each one carries the HIG anchor plus
//! a `See also` section pointing to the concrete components in this
//! crate that implement the pattern.
//!
//! Three entries carry real supporting types beyond documentation:
//! [`feedback`] (style + intensity enums), [`loading`]
//! (`LoadingState` machine), and [`undo_and_redo`] (which documents the
//! canvas-level `UndoStack` runtime). tvOS/watchOS-only patterns
//! (`live_viewing_apps`, `workouts`) are marked not-applicable.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/patterns>

pub mod charting_data;
pub mod collaboration_and_sharing;
pub mod drag_and_drop;
pub mod entering_data;
pub mod feedback;
pub mod file_management;
pub mod going_full_screen;
pub mod launching;
pub mod live_viewing_apps;
pub mod loading;
pub mod managing_accounts;
pub mod managing_notifications;
pub mod modality;
pub mod multitasking;
pub mod offering_help;
pub mod onboarding;
pub mod playing_audio;
pub mod playing_haptics;
pub mod playing_video;
pub mod printing;
pub mod ratings_and_reviews;
pub mod searching;
pub mod settings;
pub mod undo_and_redo;
pub mod workouts;
