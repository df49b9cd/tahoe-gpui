//! Multitasking pattern aligned with HIG.
//!
//! HIG (macOS): support multiple open documents via standard window
//! semantics (Cmd+N, Cmd+W), split views for side-by-side work, and
//! Mission Control / Spaces. Preserve per-window state across launches
//! AND across Space / full-screen transitions so the user's in-progress
//! task survives context switches.
//!
//! # State-checkpoint contract
//!
//! [`WindowStateCheckpoint`] is the opt-in contract hosts implement to
//! snapshot window-level state at the points macOS can lose it:
//!
//! - `on_will_miniaturize` — the window is about to slide into the Dock.
//! - `on_space_did_change` — the user switched Spaces away from this
//!   window.
//! - `on_will_enter_full_screen` / `on_did_exit_full_screen` — HIG
//!   specifically calls out full-screen entry/exit as a natural save
//!   point.
//!
//! The trait is deliberately AppKit-flavoured in its hook names so
//! hosts can wire it to `NSWindow`/`NSWorkspace` notifications when
//! they ship a real macOS shell. Embedded GPUI panels typically
//! participate in the host window's multitasking rather than managing
//! their own, so the trait is best implemented at the host-application
//! layer and left as documentation for library consumers.
//!
//! # See also
//!
//! - [`crate::patterns::launching`] — the `StateRestoration` trait
//!   applies the saved state back at launch.
//! - [`crate::components::layout_and_organization::split_view`] —
//!   resizable side-by-side layout.
//! - [`crate::components::navigation_and_search::tab_bar::TabBar`] —
//!   in-window multitasking across documents / sections.
//! - [`crate::components::navigation_and_search::sidebar::Sidebar`] —
//!   primary navigation pane.
//! - `gpui::Window` — host-level windowing (create/close/focus).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/multitasking>

/// Hook points a multitasking-aware host should implement so per-window
/// state survives macOS context switches.
///
/// The methods all default to no-ops so hosts can implement only the
/// hooks they need. This is a documentation contract more than a
/// runtime surface: GPUI does not yet forward AppKit window
/// notifications, so host applications wire these hooks into their own
/// `NSWindowDelegate` / `NSWorkspace.notificationCenter` observers and
/// call through to their `WindowStateCheckpoint` implementation.
pub trait WindowStateCheckpoint {
    /// Called immediately before the window is miniaturised into the
    /// Dock. Save transient UI state (scroll position, inline editor
    /// contents) here.
    fn on_will_miniaturize(&mut self) {}

    /// Called after the user switches Spaces away from this window.
    /// HIG: preserve window frame + selection so returning to the
    /// Space looks identical.
    fn on_space_did_change(&mut self) {}

    /// Called before the window enters full-screen mode.
    fn on_will_enter_full_screen(&mut self) {}

    /// Called after the window exits full-screen mode.
    fn on_did_exit_full_screen(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::WindowStateCheckpoint;
    use core::prelude::v1::test;

    #[derive(Default)]
    struct Counters {
        mini: u32,
        space: u32,
        enter_fs: u32,
        exit_fs: u32,
    }

    impl WindowStateCheckpoint for Counters {
        fn on_will_miniaturize(&mut self) {
            self.mini += 1;
        }
        fn on_space_did_change(&mut self) {
            self.space += 1;
        }
        fn on_will_enter_full_screen(&mut self) {
            self.enter_fs += 1;
        }
        fn on_did_exit_full_screen(&mut self) {
            self.exit_fs += 1;
        }
    }

    #[test]
    fn default_impls_compile_and_are_noops() {
        struct Bare;
        impl WindowStateCheckpoint for Bare {}
        let mut b = Bare;
        b.on_will_miniaturize();
        b.on_space_did_change();
        b.on_will_enter_full_screen();
        b.on_did_exit_full_screen();
    }

    #[test]
    fn implementors_receive_hook_calls() {
        let mut c = Counters::default();
        c.on_will_miniaturize();
        c.on_space_did_change();
        c.on_will_enter_full_screen();
        c.on_did_exit_full_screen();
        assert_eq!(c.mini, 1);
        assert_eq!(c.space, 1);
        assert_eq!(c.enter_fs, 1);
        assert_eq!(c.exit_fs, 1);
    }
}
