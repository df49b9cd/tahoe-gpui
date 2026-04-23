//! Bridge between the deferred [`Color`] token and GPUI's paint surface.
//!
//! GPUI today accepts [`Hsla`] in `.bg(...)`, `.text_color(...)`, etc.
//! The migration path defined in the refactor plan (Phase 3) swaps
//! `TahoeTheme` fields from `Hsla` to `Color`, but relies on the
//! `impl From<Color> for Hsla` below so that `.bg(theme.accent)` — the
//! common case — compiles without textual change. Because every
//! `TahoeTheme` field is *eagerly resolved* at theme-build time, the
//! bridge only ever sees `Color(Resolved(...))` on the hot path.
//!
//! Deferred tokens (`Color::RED`, `Color::ACCENT`, `Color::LABEL`, …) still
//! need a [`gpui::App`] to resolve against, so the blanket `From` impl
//! **panics** on those. Call sites that start with a deferred token must
//! go through [`Color::into_hsla`] explicitly:
//!
//! ```ignore
//! .bg(Color::ACCENT.into_hsla(cx))
//! ```

use gpui::{App, Hsla};

use super::token::Color;

/// Unwrap a [`Color`] into a concrete [`Hsla`] *without* consulting a
/// theme. Succeeds on [`Color::resolved`] / [`Color::rgb`] /
/// [`Color::from_hsla`] and on `.opacity(...)` chains over those.
///
/// **Panics** on [`Color::RED`] / [`Color::GRAY`] / [`Color::LABEL`] /
/// [`Color::ACCENT`] and other deferred tokens — those need an
/// environment. Use [`Color::into_hsla`] to resolve them explicitly.
///
/// This is the bridge that makes the Phase 3 field-swap ergonomic: every
/// value stored on `TahoeTheme` is pre-resolved, so `.bg(theme.background)`
/// continues to compile unchanged.
impl From<Color> for Hsla {
    fn from(color: Color) -> Self {
        match color.try_into_hsla_eager() {
            Ok(h) => h,
            Err(reason) => panic!(
                "tried to convert a deferred Color to Hsla without resolving against a \
                 ColorEnvironment — {reason}. Use `Color::into_hsla(cx)` or \
                 `Color::resolve(cx)` instead."
            ),
        }
    }
}

/// Symmetric convenience — lets `Hsla` literals flow into `Color`-shaped
/// APIs without `.into()`. Always eager (wraps as `Color::resolved`).
impl From<Hsla> for Color {
    fn from(h: Hsla) -> Self {
        Color::from_hsla(h)
    }
}

impl Color {
    /// One-shot bridge to [`Hsla`], resolving against the theme registered
    /// on `cx`. Works for every variant (including deferred tokens) —
    /// prefer this over the panicking `From<Color> for Hsla` when the
    /// caller has a `&App`.
    pub fn into_hsla(self, cx: &App) -> Hsla {
        self.resolve(cx).to_hsla()
    }
}

#[cfg(test)]
mod tests {
    use super::super::{ResolvedColor, RgbColorSpace};
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn from_resolved_color_is_cheap_roundtrip() {
        let input = Hsla {
            h: 0.3,
            s: 0.7,
            l: 0.4,
            a: 0.9,
        };
        let c = Color::from_hsla(input);
        let back: Hsla = c.into();
        // Allow f32 ULP drift — the round-trip goes through linear-sRGB.
        assert!((back.h - input.h).abs() < 1e-3);
        assert!((back.l - input.l).abs() < 1e-3);
        assert!((back.a - input.a).abs() < 1e-6);
    }

    #[test]
    fn from_literal_srgb_color_works_without_env() {
        let c = Color::rgb(RgbColorSpace::Srgb, 0.5, 0.0, 0.0);
        let h: Hsla = c.into();
        assert!(h.a > 0.99, "literal sRGB should be opaque by default");
    }

    #[test]
    fn from_resolved_variant_works_without_env() {
        let rc = ResolvedColor::from_srgb(0.1, 0.2, 0.3, 0.5);
        let c = Color::resolved(rc);
        let h: Hsla = c.into();
        assert!((h.a - 0.5).abs() < 1e-6);
    }

    #[test]
    fn from_opacity_of_resolved_works_without_env() {
        let c = Color::from_hsla(Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
        })
        .opacity(0.25);
        let h: Hsla = c.into();
        assert!((h.a - 0.25).abs() < 1e-6);
    }

    #[test]
    #[should_panic(expected = "deferred Color")]
    fn from_system_color_panics() {
        let _: Hsla = Color::RED.into();
    }

    #[test]
    #[should_panic(expected = "deferred Color")]
    fn from_semantic_panics() {
        let _: Hsla = Color::ACCENT.into();
    }

    #[test]
    #[should_panic(expected = "deferred Color")]
    fn from_system_gray_panics() {
        let _: Hsla = Color::GRAY_3.into();
    }

    #[test]
    #[should_panic(expected = "deferred Color")]
    fn from_label_panics() {
        let _: Hsla = Color::LABEL.into();
    }
}
