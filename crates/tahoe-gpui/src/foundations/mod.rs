//! Design-system primitives aligned with the HIG Foundations.
//!
//! Exposes the 13 concrete foundation modules implemented today plus stubs
//! for 6 additional HIG foundation pages (Branding, Privacy, Immersive
//! experiences, Inclusion, Spatial layout, Writing) so the audit of HIG
//! coverage stays grep-friendly.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/foundations>

pub mod accessibility;
pub mod app_icon;
pub mod blink_manager;
pub mod branding;
pub mod color;
pub mod dark_mode;
pub mod icons;
pub mod images;
pub mod immersive_experiences;
pub mod inclusion;
pub mod keyboard;
pub mod keyboard_shortcuts;
pub mod layout;
pub mod materials;
pub mod motion;
pub mod privacy;
pub mod right_to_left;
pub mod sf_symbols;
pub mod spatial_layout;
pub mod surface_scope;
pub mod theme;
pub mod typography;
pub mod writing;

pub use accessibility::{FocusGroup, FocusGroupExt, FocusGroupMode};
pub use surface_scope::{
    GlassSurfaceGuard, GlassSurfaceScope, GlassSurfaceScopeElement, is_on_glass_surface,
};
pub use theme::TahoeTheme;
