//! Controls (HIG Controls).
//!
//! Control Center / Lock Screen controls are a **system-owned** surface
//! backed by the ControlKit framework. Apps register controls at the
//! host level; GPUI cannot render this surface and no drawable widget
//! lives here.
//!
//! This module exists so HIG audits find an anchor here.
//!
//! # Host integration
//!
//! Hosts declare control widgets with `ControlWidget` protocol
//! implementations (Swift). No GPUI code lives on that path.
//!
//! # See also
//!
//! - [`crate::components::menus_and_actions::button::Button`] +
//!   [`crate::components::selection_and_input::toggle::Toggle`] — the
//!   in-app equivalents for quick actions inside the GPUI surface.
//! - [`crate::components::selection_and_input::slider::Slider`] — the
//!   in-app continuous-control equivalent.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/controls>
//!
//! Tracked by `docs/hig/components/system-experiences.md:305`.
