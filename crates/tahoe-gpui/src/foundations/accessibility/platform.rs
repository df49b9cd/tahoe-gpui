//! Platform-specific accessibility setting detection.
//!
//! On macOS, reads `NSApplication.shared.isFullKeyboardAccessEnabled` and
//! `NSWorkspace.shared` accessibility display options (reduce motion, increase
//! contrast, reduce transparency, differentiate without colour). On other
//! platforms, or when the AppKit framework is not loaded (e.g. in unit tests),
//! returns [`AccessibilityMode::DEFAULT`].
//!
//! Call [`detect_system_accessibility_mode`] once at startup (or when the theme
//! is constructed) to seed [`TahoeTheme::accessibility_mode`] with the user's
//! current system preferences. For live updates, the host should observe
//! `NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification` and call
//! [`TahoeTheme::refresh_accessibility`].

use super::AccessibilityMode;

/// Read the current system accessibility settings.
///
/// On macOS this queries `NSApplication` and `NSWorkspace` via Objective-C
/// message sends. Returns [`AccessibilityMode::DEFAULT`] on other platforms or
/// when the AppKit framework is not loaded.
pub fn detect_system_accessibility_mode() -> AccessibilityMode {
    detect_platform()
}

#[cfg(target_os = "macos")]
fn detect_platform() -> AccessibilityMode {
    use objc2::msg_send;
    use objc2::runtime::AnyClass;

    let mut mode = AccessibilityMode::DEFAULT;

    // SAFETY: Both `NSApplication.sharedApplication` and
    // `NSWorkspace.sharedWorkspace` return process-singleton objects that are
    // always available on macOS when AppKit is loaded. The boolean property
    // getters are simple accessors that read from `CFPreferences` and cannot
    // raise exceptions. Theme construction always happens on the main thread
    // in GPUI, satisfying NSApplication's main-thread requirement.
    unsafe {
        let Some(ns_app) = AnyClass::get(c"NSApplication") else {
            return mode;
        };
        let Some(ns_workspace) = AnyClass::get(c"NSWorkspace") else {
            return mode;
        };

        let app: Option<&objc2::runtime::NSObject> =
            msg_send![ns_app as *const _, sharedApplication];
        let Some(app) = app else { return mode };

        let fka: bool = msg_send![app, isFullKeyboardAccessEnabled];
        if fka {
            mode |= AccessibilityMode::FULL_KEYBOARD_ACCESS;
        }

        let workspace: Option<&objc2::runtime::NSObject> =
            msg_send![ns_workspace as *const _, sharedWorkspace];
        let Some(workspace) = workspace else {
            return mode;
        };

        let reduce_motion: bool = msg_send![workspace, isAccessibilityDisplayShouldReduceMotion];
        if reduce_motion {
            mode |= AccessibilityMode::REDUCE_MOTION;
        }

        let increase_contrast: bool =
            msg_send![workspace, isAccessibilityDisplayShouldIncreaseContrast];
        if increase_contrast {
            mode |= AccessibilityMode::INCREASE_CONTRAST;
        }

        let reduce_transparency: bool =
            msg_send![workspace, isAccessibilityDisplayShouldReduceTransparency];
        if reduce_transparency {
            mode |= AccessibilityMode::REDUCE_TRANSPARENCY;
        }

        let differentiate: bool = msg_send![
            workspace,
            isAccessibilityDisplayShouldDifferentiateWithoutColor
        ];
        if differentiate {
            mode |= AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR;
        }
    }

    mode
}

#[cfg(not(target_os = "macos"))]
fn detect_platform() -> AccessibilityMode {
    AccessibilityMode::DEFAULT
}

#[cfg(test)]
mod tests {
    use super::detect_system_accessibility_mode;
    use core::prelude::v1::test;

    #[test]
    fn detect_system_returns_valid_mode() {
        // Cannot assert specific flag values because the test machine's
        // accessibility settings are unknown. Verifies the function runs
        // without panic on all platforms (including test runners where
        // AppKit may not be loaded).
        let mode = detect_system_accessibility_mode();
        let _ = mode.full_keyboard_access();
        let _ = mode.reduce_motion();
        let _ = mode.increase_contrast();
        let _ = mode.reduce_transparency();
        let _ = mode.differentiate_without_color();
    }
}
