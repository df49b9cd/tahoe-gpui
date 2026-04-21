//! Accessibility configuration aligned with HIG.
//!
//! Provides accessibility mode bitflags, accessibility tokens for
//! Liquid Glass surfaces, and the [`AccessibilityProps`] / [`AccessibleExt`]
//! scaffolding used by components to declare VoiceOver labels, roles, and
//! values.
//!
//! # VoiceOver status (GPUI upstream gap)
//!
//! GPUI `0.2.2` (tag `v0.231.1-pre`) does not yet expose
//! `accessibility_label` / `accessibility_role` APIs on `Div` /
//! `Stateful<Div>`. Verified 2026-04-18 by grepping the upstream source
//! at that tag for `accessibility`, `AXRole`, `NSAccessibility`, and
//! `VoiceOver` — no matches outside settings strings. Components store
//! their labels via [`AccessibilityProps`] and attach them through
//! [`AccessibleExt`] so that when GPUI lands the upstream API the single
//! `apply_accessibility` entry point below can wire labels to the AX
//! tree without any per-component changes.
//!
//! Tracked in <https://github.com/df49b9cd/tahoe-gpui/issues/47>; file a GPUI
//! upstream issue in zed-industries/zed if one does not yet exist.
//!
//! For keyboard graph navigation that does work today (per-component
//! focus rings, Tab-order cycling), see
//! [`crate::workflow::WorkflowCanvas::cycle_node_focus`] — the keyboard
//! half of the HIG accessibility story that doesn't depend on the missing AX API.

pub mod focus_group;
pub mod modes;
pub mod tokens;
pub mod voiceover;

pub use focus_group::{FocusGroup, FocusGroupExt, FocusGroupMode};
pub use modes::AccessibilityMode;
pub use tokens::{
    AccessibilityTokens, apply_focus_ring, apply_high_contrast_border, effective_duration,
    reduce_motion_substitute_ms,
};
pub use voiceover::{AccessibilityProps, AccessibilityRole, AccessibleExt};
