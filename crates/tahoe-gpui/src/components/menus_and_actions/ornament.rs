//! Ornament component stub (HIG Ornaments).
//!
//! Not yet implemented. visionOS-specific floating toolbar-like element
//! anchored relative to a parent window. Not applicable to macOS 26 desktop
//! targets but tracked for HIG taxonomy completeness.
//!
//! Platform: **visionOS only** — the `cfg(target_os = "visionos")` guard
//! below makes the implementation slot platform-scoped so the module map
//! stays complete on non-visionOS builds without exposing a type that
//! can't render.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/ornaments>
//!
//! Tracked by `docs/hig/components/menus-and-actions.md:479`.

#[cfg(target_os = "visionos")]
mod visionos {
    //! visionOS-only `Ornament` type would live here. GPUI does not
    //! target visionOS today (macOS/Windows/Linux only), so this module
    //! is empty but reserved.
}
