//! Going full screen pattern aligned with HIG.
//!
//! Full-screen mode gives the active document or activity the entire
//! display. HIG (macOS): keep the menu bar reachable via hover-reveal;
//! when leaving full-screen, restore the prior window frame and toolbar
//! state. Embedded GPUI panels typically delegate full-screen to the
//! host window.
//!
//! # See also
//!
//! - [`crate::foundations::icons::IconName::Maximize`] — canonical
//!   enter-full-screen glyph for toolbar buttons.
//! - [`crate::components::navigation_and_search::toolbar::Toolbar`] —
//!   window toolbar; hide/compact it when the host enters full-screen
//!   via `gpui::Window` APIs.
//! - [`crate::components::presentation::modal::Modal`] — full-screen
//!   modal content that pre-dates OS full-screen (use sparingly).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/going-full-screen>
