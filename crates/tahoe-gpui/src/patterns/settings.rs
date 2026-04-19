//! Settings pattern aligned with HIG.
//!
//! Settings let people tailor an app's behaviour to their preferences.
//! HIG: organise settings into scannable groups, provide sensible
//! defaults, keep destructive operations off the settings screen, apply
//! changes immediately, avoid requiring restart, and surface important
//! settings inside the app (not only in the macOS Settings app).
//!
//! macOS convention: the Settings window opens via the `⌘,` shortcut and
//! the application-menu "Settings…" item (renamed from "Preferences…"
//! in macOS 13 Ventura).
//!
//! # Action + shortcut
//!
//! [`settings_keybindings`] returns the `⌘,` binding mapped to the
//! [`OpenSettings`] action. Hosts install the binding at app init and
//! route the action to whichever surface opens their settings window
//! (a `Sheet`, a dedicated GPUI window, or a menu-only command).
//!
//! ```ignore
//! cx.bind_keys(tahoe_gpui::patterns::settings::settings_keybindings());
//! // Then register an on_action handler for `OpenSettings`.
//! ```
//!
//! # See also
//!
//! - [`crate::components::selection_and_input::toggle::Toggle`] — the
//!   canonical on/off setting.
//! - [`crate::components::selection_and_input::picker::Picker`] — for
//!   enum-typed settings (theme, accent colour).
//! - [`crate::components::selection_and_input::stepper::Stepper`] — for
//!   bounded integer settings (font scale, tab width).
//! - [`crate::components::selection_and_input::slider::Slider`] — for
//!   continuous settings (volume, opacity).
//! - [`crate::components::layout_and_organization::disclosure_group::DisclosureGroup`]
//!   — to group advanced or rarely-touched settings behind a header.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/settings>

use gpui::{KeyBinding, actions};

actions!(
    settings,
    [
        /// Show the app's Settings window (⌘, on macOS).
        OpenSettings,
    ]
);

/// Returns the standard macOS Settings shortcut binding.
///
/// Binds `⌘,` → [`OpenSettings`]. Register during app init:
///
/// ```ignore
/// cx.bind_keys(tahoe_gpui::patterns::settings::settings_keybindings());
/// ```
pub fn settings_keybindings() -> Vec<KeyBinding> {
    vec![KeyBinding::new("cmd-,", OpenSettings, None)]
}

#[cfg(test)]
mod tests {
    use super::settings_keybindings;
    use core::prelude::v1::test;

    #[test]
    fn settings_keybinding_registered() {
        let bindings = settings_keybindings();
        assert_eq!(bindings.len(), 1);
    }
}
