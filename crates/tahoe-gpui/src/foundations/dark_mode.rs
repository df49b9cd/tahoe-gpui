//! Dark Mode support aligned with HIG.
//!
//! macOS, iOS, and other Apple platforms support a system-wide dark appearance.
//! The theme system handles dark mode automatically through the [`Appearance`] enum,
//! which components read via `cx.theme()` (or `cx.global::<TahoeTheme>()`
//! directly).
//!
//! # How dark mode works in this crate
//!
//! - [`Appearance`] has four variants: `Light`, `Dark`, `LightHighContrast`, `DarkHighContrast`
//! - [`SystemPalette`](super::color::SystemPalette) pre-resolves all system colors for the current appearance
//! - Components read semantic colors from `TahoeTheme` which are already adapted
//! - Liquid Glass materials adapt automatically via the glass style tokens
//!
//! # Best practices (from HIG)
//!
//! - Use semantic colors (`theme.text`, `theme.surface`) instead of hard-coded values
//! - Test both light and dark appearances
//! - Ensure sufficient contrast in both modes (use [`contrast_ratio`](super::color::contrast_ratio))
//! - Don't use pure black backgrounds in dark mode — prefer the system dark gray.
//!
//!   This rule applies to macOS, where window backgrounds always render against
//!   wallpaper or other window chrome and pure black would clip into the
//!   surrounding UI. iOS does use `#000000` for `systemBackground` on OLED
//!   panels (power savings) but its grouped/elevated layers still pull off
//!   absolute black; this codebase targets macOS first, so all themes — even
//!   the iOS-spec liquid-glass variant — follow the macOS substrate rule.

pub use super::color::Appearance;
pub use super::theme::TahoeTheme;
