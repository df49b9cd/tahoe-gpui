//! Shared keyboard navigation actions for interactive components.
//!
//! These GPUI actions represent navigation operations shared by
//! components like TabBar, SegmentedControl, Slider, Stepper, and
//! other navigable controls per HIG Keyboards guidance.
//!
//! **Note:** Arrow-key navigation actions
//! ([`NavigateLeft`] / [`NavigateRight`] / [`NavigateUp`] /
//! [`NavigateDown`] / [`NavigateFirst`] / [`NavigateLast`] /
//! [`Activate`] / [`Dismiss`]) are defined but NOT globally bound.
//! Components that need navigation (TabBar, Slider, etc.) currently use
//! `on_key_down()` handlers for arrow keys because:
//! 1. Arrow keys have different meanings per component (tab switch vs slider increment)
//! 2. Stateless (`RenderOnce`) components can't register `on_action()` handlers
//! 3. Global arrow key bindings conflict with text cursor movement
//!
//! The action types are available for stateful (`Entity<T>`) components that
//! want to adopt GPUI's `on_action()` pattern with scoped keybindings.
//!
//! # macOS Edit-menu Find/Replace/GoTo (HIG)
//!
//! HIG §Edit menu mandates that content-editing apps expose **Find**,
//! **Find Next**, **Find Previous**, **Find and Replace**, and **Go To**
//! commands with standard shortcuts. Hosts that embed a search field or
//! navigable document should register [`find_keybindings`] so these
//! actions are reachable on the platform shortcuts. Components that
//! consume the resulting actions implement `on_action()` handlers — the
//! bindings themselves are scope-less so an app-level menu dispatcher
//! can re-route them as needed.
//!
//! ```ignore
//! cx.bind_keys(tahoe_gpui::navigation_actions::find_keybindings());
//! ```

use gpui::{KeyBinding, actions};

actions!(
    navigation,
    [
        NavigateLeft,
        NavigateRight,
        NavigateUp,
        NavigateDown,
        NavigateFirst,
        NavigateLast,
        Activate,
        Dismiss,
        /// Open the Find toolbar / palette (Cmd-F on macOS).
        Find,
        /// Advance to the next match (Cmd-G on macOS).
        FindNext,
        /// Back up to the previous match (Cmd-Shift-G on macOS).
        FindPrevious,
        /// Open find-and-replace (Cmd-Alt-F on macOS).
        Replace,
        /// Jump to line / location prompt (Cmd-L on macOS).
        GoTo,
    ]
);

/// Returns the HIG-standard macOS Edit-menu Find/Replace/GoTo bindings.
///
/// Register during app initialization when the host embeds a search
/// field, navigable document, or any surface for which HIG §Edit-menu
/// mandates these shortcuts:
///
/// ```ignore
/// cx.bind_keys(tahoe_gpui::navigation_actions::find_keybindings());
/// ```
///
/// The actions fire globally (no `KeyContext`) so an app-level
/// dispatcher — typically the same component that owns the search UI —
/// can route them to the appropriate target. Consuming components
/// implement `on_action()` handlers against the action types above.
pub fn find_keybindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("cmd-f", Find, None),
        KeyBinding::new("cmd-g", FindNext, None),
        KeyBinding::new("cmd-shift-g", FindPrevious, None),
        KeyBinding::new("cmd-alt-f", Replace, None),
        KeyBinding::new("cmd-l", GoTo, None),
    ]
}

#[cfg(test)]
mod tests {
    use super::find_keybindings;
    use core::prelude::v1::test;

    #[test]
    fn find_keybindings_expose_five_actions() {
        // HIG §Edit menu mandates exactly these five: Find / FindNext /
        // FindPrevious / Replace / GoTo. Guard against accidental
        // additions or removals.
        assert_eq!(find_keybindings().len(), 5);
    }
}
