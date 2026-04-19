//! Keyboard interaction helpers per HIG and GPUI best practices.
//!
//! ## GPUI keyboard handling patterns
//!
//! GPUI provides two keyboard handling mechanisms:
//!
//! ### 1. `on_key_down()` — for stateless (`RenderOnce`) components
//!
//! Stateless components use `on_key_down()` with manual key matching.
//! This is the **correct approach** for `RenderOnce` components because
//! they cannot register `on_action()` handlers (that requires `Entity<T>`).
//!
//! ```ignore
//! div()
//!     .on_key_down(|event: &KeyDownEvent, window, cx| {
//!         if is_activation_key(event) {
//!             // handle activation
//!         }
//!     })
//! ```
//!
//! ### 2. `on_action()` — for stateful (`Entity<T>` / `Render`) components
//!
//! Stateful components can use GPUI's action dispatch system with
//! `KeyBinding::new()` for customizable keybindings. This is preferred
//! for complex interactive components like TextField.
//!
//! ```ignore
//! // Register keybindings during init:
//! cx.bind_keys(vec![
//!     KeyBinding::new("cmd-c", Copy, Some("MyComponent")),
//! ]);
//!
//! // Handle in component:
//! element.on_action(cx.listener(Self::handle_copy))
//! ```
//!
//! ## Key matching helpers
//!
//! The helpers in this module support the `on_key_down()` pattern used
//! by the 28 stateless components in this crate. They check for standard
//! activation and dismiss keys per HIG and WCAG 2.1.

use gpui::KeyDownEvent;

/// Returns `true` when the keystroke is Enter or Space — the two keys
/// that activate a focusable widget per WCAG 2.1 and HIG.
///
/// Use with `on_key_down()` for stateless (`RenderOnce`) components.
/// Stateful components should use `on_action()` with the
/// [`Activate`](crate::navigation_actions::Activate) action type instead.
///
/// Call `cx.stop_propagation()` after handling the activation to prevent
/// the event from scrolling a parent container.
pub fn is_activation_key(event: &KeyDownEvent) -> bool {
    // Two independent reasons a keystroke activates:
    //  1) The named key is one of the standard activation keys.
    //  2) Some GPUI backends encode Space as an ASCII space in `key_char`
    //     rather than the "space" named key — accept either source.
    //
    // Expressed as a flat `matches! || ==` so a new named activation key
    // (e.g. a platform-specific alias) never accidentally shadows the
    // `key_char` fallback the way the previous `if ... return ...;` shape
    // could.
    matches!(event.keystroke.key.as_str(), "enter" | "return" | "space")
        || event.keystroke.key_char.as_deref() == Some(" ")
}

/// Returns true if the key event is an Escape key press.
///
/// Use with `on_key_down()` for stateless (`RenderOnce`) components.
/// Stateful components should use `on_action()` with the
/// [`Dismiss`](crate::navigation_actions::Dismiss) action type instead.
pub fn is_escape_key(event: &KeyDownEvent) -> bool {
    event.keystroke.key.as_str() == "escape"
}

/// Returns true if the key is an arrow key (left, right, up, down).
///
/// Use with `on_key_down()` for stateless components that need
/// directional navigation (TabBar, SegmentedControl, Stepper, etc.).
///
/// # RTL caveat
///
/// This helper is direction-agnostic: it reports the *physical* key that
/// was pressed. Per HIG Right-to-Left (Controls), components that provide
/// sequential navigation (TabBar, SegmentedControl, Stepper) must **swap
/// the semantics of Left and Right** when
/// `TahoeTheme::layout_direction == LayoutDirection::RightToLeft`.
/// Callers are responsible for the swap — the canonical pattern is to
/// gate on `theme.is_rtl()` inside the handler, mirroring
/// `Slider::handle_key_down` which translates Left/Right into
/// plus/minus-for-value based on direction.
pub fn is_arrow_key(event: &KeyDownEvent) -> bool {
    let key = event.keystroke.key.as_str();
    key == "left" || key == "right" || key == "up" || key == "down"
}

/// Returns true if the key is Home or End.
///
/// Use with `on_key_down()` for components that support
/// jump-to-first/last navigation (TabBar, SegmentedControl, etc.).
pub fn is_home_end_key(event: &KeyDownEvent) -> bool {
    let key = event.keystroke.key.as_str();
    key == "home" || key == "end"
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::{KeyDownEvent, Keystroke};

    use super::{is_activation_key, is_arrow_key, is_escape_key, is_home_end_key};

    fn make_event(key: &str) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke::parse(key).unwrap(),
            is_held: false,
            prefer_character_input: false,
        }
    }

    #[test]
    fn enter_is_activation() {
        assert!(is_activation_key(&make_event("enter")));
    }

    #[test]
    fn space_is_activation() {
        assert!(is_activation_key(&make_event("space")));
    }

    #[test]
    fn escape_is_not_activation() {
        assert!(!is_activation_key(&make_event("escape")));
    }

    #[test]
    fn tab_is_not_activation() {
        assert!(!is_activation_key(&make_event("tab")));
    }

    #[test]
    fn arrow_is_not_activation() {
        assert!(!is_activation_key(&make_event("down")));
    }

    #[test]
    fn escape_is_escape() {
        assert!(is_escape_key(&make_event("escape")));
    }

    #[test]
    fn enter_is_not_escape() {
        assert!(!is_escape_key(&make_event("enter")));
    }

    #[test]
    fn tab_is_not_escape() {
        assert!(!is_escape_key(&make_event("tab")));
    }

    #[test]
    fn arrow_keys() {
        assert!(is_arrow_key(&make_event("left")));
        assert!(is_arrow_key(&make_event("right")));
        assert!(is_arrow_key(&make_event("up")));
        assert!(is_arrow_key(&make_event("down")));
        assert!(!is_arrow_key(&make_event("enter")));
    }

    #[test]
    fn home_end_keys() {
        assert!(is_home_end_key(&make_event("home")));
        assert!(is_home_end_key(&make_event("end")));
        assert!(!is_home_end_key(&make_event("left")));
    }
}
