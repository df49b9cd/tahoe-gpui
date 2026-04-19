//! Window configuration aligned with HIG `#windows` (macOS anatomy).
//!
//! Provides types and utilities for configuring window appearance per Apple
//! HIG. macOS 26 Tahoe distinguishes **primary** windows (main document,
//! settings) from **auxiliary** windows (single-task surfaces with a Close
//! button, no minimize/zoom) and from **panels** (`NSPanel`-backed floating
//! surfaces — inspector, Fonts, Colors, HUD — surfaced separately via
//! [`Panel`](super::panel::Panel)).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/windows>

use crate::foundations::layout::MACOS_TITLE_BAR_HEIGHT;

/// Window style per HIG `#windows` macOS anatomy.
///
/// macOS recognises two high-level categories:
/// - **Primary** windows — `Document`, `Settings`, `About`, `Welcome` —
///   full title bar (28 pt) with close / minimize / zoom traffic lights.
/// - **Auxiliary** windows — `Auxiliary` — single-task surface with only
///   a Close button. No minimize/zoom traffic lights.
///
/// Floating utility surfaces (inspector, Fonts, Colors, HUD) are modeled
/// separately via [`Panel`](super::panel::Panel).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WindowStyle {
    /// Standard document window with full 28 pt title bar. Corresponds
    /// to `NSWindow` with `NSWindowStyleMask::Titled | .Closable |
    /// .Miniaturizable | .Resizable`.
    #[default]
    Document,
    /// Single-task auxiliary window (HIG: "An auxiliary window supports
    /// the functionality of a primary window"). Has a Close button, no
    /// minimize/zoom. Appropriate for compose windows, import sheets
    /// elevated into windows, secondary editors.
    Auxiliary,
    /// Preferences/Settings window (fixed size, centered, non-resizable).
    Settings,
    /// About window (compact, non-resizable).
    About,
    /// Welcome/onboarding window (centered, larger).
    Welcome,
}

impl WindowStyle {
    /// Suggested minimum width for this window style per HIG.
    pub fn min_width(self) -> f32 {
        match self {
            Self::Document => 480.0,
            Self::Auxiliary => 320.0,
            Self::Settings => 540.0,
            Self::About => 300.0,
            Self::Welcome => 600.0,
        }
    }

    /// Suggested minimum height for this window style per HIG.
    pub fn min_height(self) -> f32 {
        match self {
            Self::Document => 320.0,
            Self::Auxiliary => 200.0,
            Self::Settings => 400.0,
            Self::About => 200.0,
            Self::Welcome => 400.0,
        }
    }

    /// Title bar height in points per HIG macOS anatomy. All window
    /// styles use the regular 28 pt title bar; `NSPanel`'s 22 pt variant
    /// is owned by [`Panel`](super::panel::Panel) instead.
    pub fn title_bar_height(self) -> f32 {
        MACOS_TITLE_BAR_HEIGHT
    }

    /// Whether the window has a minimize / zoom traffic-light pair.
    ///
    /// Primary document windows expose all three traffic lights
    /// (close, minimize, zoom). Auxiliary and settings-style windows
    /// show only Close, per HIG `#windows` macOS anatomy.
    pub fn has_minimize_zoom(self) -> bool {
        matches!(self, Self::Document | Self::Welcome)
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::WindowStyle;
    use crate::foundations::layout::MACOS_TITLE_BAR_HEIGHT;

    #[test]
    fn window_style_defaults() {
        let style = WindowStyle::default();
        assert_eq!(style, WindowStyle::Document);
    }

    #[test]
    fn window_style_sizes() {
        assert!(WindowStyle::Document.min_width() > WindowStyle::Auxiliary.min_width());
        assert!(WindowStyle::Settings.min_width() > WindowStyle::About.min_width());
        assert!(WindowStyle::Auxiliary.min_width() >= WindowStyle::About.min_width());
    }

    #[test]
    fn primary_windows_use_regular_title_bar() {
        assert!(
            (WindowStyle::Document.title_bar_height() - MACOS_TITLE_BAR_HEIGHT).abs()
                < f32::EPSILON
        );
        assert!(
            (WindowStyle::Auxiliary.title_bar_height() - MACOS_TITLE_BAR_HEIGHT).abs()
                < f32::EPSILON
        );
        assert!(
            (WindowStyle::Settings.title_bar_height() - MACOS_TITLE_BAR_HEIGHT).abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn only_document_and_welcome_expose_minimize_zoom() {
        assert!(WindowStyle::Document.has_minimize_zoom());
        assert!(WindowStyle::Welcome.has_minimize_zoom());
        assert!(!WindowStyle::Auxiliary.has_minimize_zoom());
        assert!(!WindowStyle::Settings.has_minimize_zoom());
        assert!(!WindowStyle::About.has_minimize_zoom());
    }
}
