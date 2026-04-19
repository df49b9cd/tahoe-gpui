//! Branding foundation aligned with HIG Branding.
//!
//! Branding on Apple platforms is restrained: the HIG explicitly
//! discourages custom-font walls, logo persistence in navigation
//! chrome, and brand-colour replacements for system accents. A
//! well-branded app on macOS usually surfaces the brand only at launch
//! (splash / about panel) and lets the system theme drive everything
//! else.
//!
//! This module does **not** ship a separate "brand palette" type; by
//! design, brand tokens live alongside the rest of the design tokens
//! on [`crate::foundations::theme::TahoeTheme`] so a single accent /
//! typography decision propagates through every component. Consumers
//! that need stricter brand controls typically wrap `TahoeTheme` in a
//! host-owned builder rather than duplicating tokens here.
//!
//! # See also
//!
//! - [`crate::foundations::theme::TahoeTheme::with_accent`] — override
//!   the accent colour to match a brand hue while keeping the rest of
//!   the HIG palette intact.
//! - [`crate::foundations::color::AccentColor`] — the Apple-provided
//!   named accents that most apps should prefer over custom colours.
//! - [`crate::foundations::typography`] — font family / size tokens;
//!   the HIG discourages custom body fonts but allows custom display
//!   faces in hero content.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/branding>
//!
//! Tracked by `docs/hig/foundations.md:184`.
