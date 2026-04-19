//! App Shortcuts (HIG App Shortcuts).
//!
//! App Shortcuts are a **system-owned** surface: Siri / Spotlight /
//! Shortcuts.app render the shortcut catalogue backed by the App Intents
//! framework. A GPUI component cannot draw this surface itself —
//! registration happens in the host app via `AppIntent` declarations.
//!
//! This module exists so HIG audits find an anchor here; there is no
//! drawable widget to ship.
//!
//! # Host integration
//!
//! Hosts declare App Intents in their app target (Swift or Objective-C)
//! and ship shortcut phrase definitions via `AppShortcutsProvider`. No
//! GPUI code lives on that path.
//!
//! # See also
//!
//! - [`crate::components::menus_and_actions::command_palette`] — the
//!   in-app equivalent: a keyboard-first action surface consumers can
//!   embed anywhere in the GPUI tree.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/app-shortcuts>
//!
//! Tracked by `docs/hig/components/system-experiences.md:19`.
