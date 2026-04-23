//! [`ColorEnvironment`] — the read-only view of the active theme that
//! [`Color`] needs to resolve to a concrete [`ResolvedColor`].
//!
//! Split into its own file because the environment is referenced both by
//! `Color::resolve_in` and by `TahoeTheme::color_environment`, and the
//! struct needs a home that doesn't pull the full `color_type.rs` module
//! into the theme crate at construction time.
//!
//! [`Color`]: super::Color
//! [`ResolvedColor`]: super::ResolvedColor

use gpui::Hsla;

use super::{Appearance, SystemPalette};
use crate::foundations::theme::SemanticColors;

/// Borrowed snapshot of the parts of the active theme that the deferred
/// [`super::Color`] token needs in order to resolve.
///
/// Constructed via [`crate::TahoeTheme::color_environment`]. Tests that
/// don't want to stand up a full `App` can build one directly.
///
/// Carries four input axes:
/// - `appearance` — picks the palette column (Light / Dark / HC variants)
/// - `accent` — pre-resolved ambient accent colour (SwiftUI `Color.accentColor`)
/// - `semantic` — the semantic-token table for the active appearance
/// - `palette` — the 18 HIG palette colours pre-resolved for appearance
///
/// And two *hints* that future semantic lookups can key off (nothing in
/// Phase 2 reads them yet, but the fields are here so call sites don't
/// have to refactor once a semantic token becomes elevation-sensitive):
/// - `reduce_transparency` — macOS "Reduce Transparency" accessibility pref
/// - `elevated` — `true` on elevated surfaces (popovers, sheets) so
///   `ElevatedSystemBackground` can pick the right fill
#[derive(Debug, Clone, Copy)]
pub struct ColorEnvironment<'a> {
    pub appearance: Appearance,
    pub accent: Hsla,
    pub semantic: &'a SemanticColors,
    pub palette: &'a SystemPalette,
    pub reduce_transparency: bool,
    pub elevated: bool,
}

impl<'a> ColorEnvironment<'a> {
    /// Construct an environment from the raw parts. Prefer
    /// [`crate::TahoeTheme::color_environment`] — this form exists for tests
    /// that stand up a `ColorEnvironment` without building a full theme.
    pub fn new(
        appearance: Appearance,
        accent: Hsla,
        semantic: &'a SemanticColors,
        palette: &'a SystemPalette,
    ) -> Self {
        Self {
            appearance,
            accent,
            semantic,
            palette,
            reduce_transparency: false,
            elevated: false,
        }
    }

    /// Mark this environment as running on an elevated surface (popover,
    /// sheet, modal). Elevated-aware semantic tokens may pick a different
    /// fill.
    pub fn elevated(mut self, elevated: bool) -> Self {
        self.elevated = elevated;
        self
    }

    /// Toggle the Reduce Transparency hint. Currently informational — tokens
    /// do not branch on it yet.
    pub fn with_reduce_transparency(mut self, reduce: bool) -> Self {
        self.reduce_transparency = reduce;
        self
    }
}
