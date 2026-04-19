//! Home Screen quick action stub (HIG Home Screen quick actions).
//!
//! Not yet implemented. iOS / iPadOS Home Screen long-press menu
//! triggered via `UIApplicationShortcutItem`. macOS uses Dock menus
//! ([`crate::components::menus_and_actions::dock_menu`]) for the
//! equivalent quick-action pattern.
//!
//! Platform: **iOS / iPadOS**.
//!
//! ## Icon constraint
//!
//! HIG: quick-action icons **must be monochromatic SF Symbols**. Emoji and
//! multicolor icons are rejected by the springboard. When this stub
//! graduates to a real component, the icon slot will be typed as
//! `IconName` (which already maps to SF Symbols 7 assets — see
//! [`crate::foundations::icons::IconName::system_name`]).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/home-screen-quick-actions>
//!
//! Tracked by `docs/hig/components/menus-and-actions.md:381`.

// Note: Home Screen quick actions (iOS/iPadOS only) use monochromatic
// SF Symbols — see HIG #home-screen-quick-actions.
