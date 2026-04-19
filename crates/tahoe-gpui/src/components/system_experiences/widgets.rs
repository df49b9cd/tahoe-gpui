//! Widgets (HIG Widgets).
//!
//! Widgets are a **system-owned** surface (Home Screen, Today View,
//! Desktop, Lock Screen) backed by the WidgetKit framework. Widget
//! content is rendered by the system from a SwiftUI timeline supplied
//! by the host extension. A GPUI component cannot draw widget content.
//!
//! This module exists so HIG audits find an anchor here; no drawable
//! widget lives here.
//!
//! # Host integration
//!
//! Hosts ship a widget-extension target that implements `Widget`
//! (SwiftUI). The main GPUI app can update the widget's snapshot via
//! shared-container I/O. No GPUI code renders the widget itself.
//!
//! # See also
//!
//! - [`crate::foundations::materials`] — Liquid Glass tokens that match
//!   macOS 26 widget chrome when rendered inside the GPUI app.
//! - [`crate::components::content::badge::Badge`] and
//!   [`crate::code::artifact::Artifact`] — in-app containers with
//!   similar density characteristics to a widget tile.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/widgets>
//!
//! Tracked by `docs/hig/components/system-experiences.md:985`.
