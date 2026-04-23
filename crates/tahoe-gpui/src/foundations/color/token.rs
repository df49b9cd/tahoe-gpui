//! [`Color`] — the deferred-resolution colour token that mirrors
//! SwiftUI `Color`.
//!
//! A `Color` is not paintable on its own. Call [`Color::resolve`] with a
//! [`gpui::App`] (or [`Color::resolve_in`] with an explicit
//! [`ColorEnvironment`]) to obtain a concrete [`ResolvedColor`]; call
//! [`Color::into_hsla`] to cross the bridge into GPUI's existing paint
//! surface.
//!
//! ## Design
//!
//! `Color` is a small `Copy` struct carrying two fields:
//!
//! - `repr: ColorRepr` — one of `Literal { … }` / `System(SystemColor)` /
//!   `SystemGray(SystemGray)` / `Semantic(SemanticToken)` /
//!   `Resolved(ResolvedColor)`.
//! - `opacity_multiplier: f32` — multiplicative alpha applied at resolve
//!   time. `1.0` means "no modification"; `.opacity(0.5).opacity(0.4)`
//!   composes to `0.2`.
//!
//! Keeping `Color` `Copy` is load-bearing for the Phase 3 field swap:
//! `.bg(theme.accent)` must work without `.clone()` or `&`, and
//! `theme.accent` is accessed through a shared `&TahoeTheme`.
//!
//! Two `Color`s constructed by different paths that *resolve* to the same
//! pixel value are **not** structurally equal — [`Color`] deliberately
//! omits [`PartialEq`]. Token identity ≠ value identity.
//!
//! ## What's in Phase 2
//!
//! - Constructors: `rgb`, `rgba`, `white`, `resolved`, `from_hsla`
//! - Modifiers: `opacity` (multiplicative, inline)
//! - Resolution: `resolve(&App)`, `resolve_in(&ColorEnvironment)`
//! - Bridge: `into_hsla(&App)` (see `gpui_bridge.rs`)
//!
//! ## What's deferred
//!
//! - `hex` / `hsb` constructors — Phase 4
//! - `mix` / `darken` / `lighten` — Phase 5 (these will likely need a
//!   separate richer non-`Copy` type, since mix is a binary operation that
//!   does not fit an inline multiplier)
//! - `gradient()` — Phase 6
//! - `IntoElement for Color` — Phase 7

use gpui::{App, Hsla};

use super::{ResolvedColor, RgbColorSpace, SystemColor, SystemGray, environment::ColorEnvironment};
use crate::foundations::theme::{ActiveTheme, SemanticColors};

/// Interpolation space for [`Color::mix`] (Phase 5 — the enum ships now so
/// the API surface is stable).
///
/// Matches SwiftUI's `ColorMixingColorSpace`:
/// - `Perceptual` (default) — OKLab perceptual mixing.
/// - `Device` — linear-sRGB, Apple's "device working space".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MixColorSpace {
    #[default]
    Perceptual,
    Device,
}

/// Symbolic name for a semantic colour resolved against a
/// [`ColorEnvironment`]. Mirrors the HIG semantic-token set plus the
/// ambient accent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticToken {
    // Labels — 5 tiers
    Label,
    SecondaryLabel,
    TertiaryLabel,
    QuaternaryLabel,
    QuinaryLabel,
    // System backgrounds
    SystemBackground,
    SecondarySystemBackground,
    TertiarySystemBackground,
    // Grouped backgrounds
    SystemGroupedBackground,
    SecondarySystemGroupedBackground,
    TertiarySystemGroupedBackground,
    // Elevated backgrounds (popovers / sheets)
    ElevatedSystemBackground,
    ElevatedSecondarySystemBackground,
    // System fills — 5 tiers
    SystemFill,
    SecondarySystemFill,
    TertiarySystemFill,
    QuaternarySystemFill,
    QuinarySystemFill,
    // Separators, placeholders, links
    Separator,
    OpaqueSeparator,
    PlaceholderText,
    Link,
    // Ambient tokens
    AccentColor,
    Info,
    Ai,
}

/// Private colour representation. `Copy` so [`Color`] stays `Copy`.
#[derive(Debug, Clone, Copy)]
enum ColorRepr {
    Literal {
        space: RgbColorSpace,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    },
    System(SystemColor),
    SystemGray(SystemGray),
    Semantic(SemanticToken),
    Resolved(ResolvedColor),
}

/// Deferred colour token. Mirrors SwiftUI `Color`.
///
/// `Copy` — size is ~28 bytes (tagged union + `opacity_multiplier`).
/// Not paintable until [`Color::resolve`] runs.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    repr: ColorRepr,
    /// Multiplicative opacity applied at resolve time. `1.0` = pass-through.
    /// Callers rarely touch this directly; `.opacity(f)` composes into it
    /// so the struct stays `Copy`.
    opacity_multiplier: f32,
}

impl Color {
    // ───── Named palette (SwiftUI `.red` / `.blue` / …) ─────────────────
    pub const RED: Color = Color::from_repr(ColorRepr::System(SystemColor::Red));
    pub const ORANGE: Color = Color::from_repr(ColorRepr::System(SystemColor::Orange));
    pub const YELLOW: Color = Color::from_repr(ColorRepr::System(SystemColor::Yellow));
    pub const GREEN: Color = Color::from_repr(ColorRepr::System(SystemColor::Green));
    pub const MINT: Color = Color::from_repr(ColorRepr::System(SystemColor::Mint));
    pub const TEAL: Color = Color::from_repr(ColorRepr::System(SystemColor::Teal));
    pub const CYAN: Color = Color::from_repr(ColorRepr::System(SystemColor::Cyan));
    pub const BLUE: Color = Color::from_repr(ColorRepr::System(SystemColor::Blue));
    pub const INDIGO: Color = Color::from_repr(ColorRepr::System(SystemColor::Indigo));
    pub const PURPLE: Color = Color::from_repr(ColorRepr::System(SystemColor::Purple));
    pub const PINK: Color = Color::from_repr(ColorRepr::System(SystemColor::Pink));
    pub const BROWN: Color = Color::from_repr(ColorRepr::System(SystemColor::Brown));

    // ───── HIG gray scale ───────────────────────────────────────────────
    pub const GRAY: Color = Color::from_repr(ColorRepr::SystemGray(SystemGray::Gray));
    pub const GRAY_2: Color = Color::from_repr(ColorRepr::SystemGray(SystemGray::Gray2));
    pub const GRAY_3: Color = Color::from_repr(ColorRepr::SystemGray(SystemGray::Gray3));
    pub const GRAY_4: Color = Color::from_repr(ColorRepr::SystemGray(SystemGray::Gray4));
    pub const GRAY_5: Color = Color::from_repr(ColorRepr::SystemGray(SystemGray::Gray5));
    pub const GRAY_6: Color = Color::from_repr(ColorRepr::SystemGray(SystemGray::Gray6));

    // ───── Black / white / clear (SwiftUI `.black` / `.white` / `.clear`) ─
    pub const WHITE: Color = Color::from_repr(ColorRepr::Literal {
        space: RgbColorSpace::Srgb,
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    });
    pub const BLACK: Color = Color::from_repr(ColorRepr::Literal {
        space: RgbColorSpace::Srgb,
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    pub const CLEAR: Color = Color::from_repr(ColorRepr::Literal {
        space: RgbColorSpace::Srgb,
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    });

    // ───── Semantic tokens ───────────────────────────────────────────────
    pub const LABEL: Color = Color::from_repr(ColorRepr::Semantic(SemanticToken::Label));
    pub const SECONDARY_LABEL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::SecondaryLabel));
    pub const TERTIARY_LABEL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::TertiaryLabel));
    pub const QUATERNARY_LABEL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::QuaternaryLabel));
    pub const QUINARY_LABEL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::QuinaryLabel));

    pub const SYSTEM_BACKGROUND: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::SystemBackground));
    pub const SECONDARY_SYSTEM_BACKGROUND: Color = Color::from_repr(ColorRepr::Semantic(
        SemanticToken::SecondarySystemBackground,
    ));
    pub const TERTIARY_SYSTEM_BACKGROUND: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::TertiarySystemBackground));

    pub const SYSTEM_GROUPED_BACKGROUND: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::SystemGroupedBackground));
    pub const SECONDARY_SYSTEM_GROUPED_BACKGROUND: Color = Color::from_repr(ColorRepr::Semantic(
        SemanticToken::SecondarySystemGroupedBackground,
    ));
    pub const TERTIARY_SYSTEM_GROUPED_BACKGROUND: Color = Color::from_repr(ColorRepr::Semantic(
        SemanticToken::TertiarySystemGroupedBackground,
    ));

    pub const ELEVATED_SYSTEM_BACKGROUND: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::ElevatedSystemBackground));
    pub const ELEVATED_SECONDARY_SYSTEM_BACKGROUND: Color = Color::from_repr(ColorRepr::Semantic(
        SemanticToken::ElevatedSecondarySystemBackground,
    ));

    pub const SYSTEM_FILL: Color = Color::from_repr(ColorRepr::Semantic(SemanticToken::SystemFill));
    pub const SECONDARY_SYSTEM_FILL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::SecondarySystemFill));
    pub const TERTIARY_SYSTEM_FILL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::TertiarySystemFill));
    pub const QUATERNARY_SYSTEM_FILL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::QuaternarySystemFill));
    pub const QUINARY_SYSTEM_FILL: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::QuinarySystemFill));

    pub const SEPARATOR: Color = Color::from_repr(ColorRepr::Semantic(SemanticToken::Separator));
    pub const OPAQUE_SEPARATOR: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::OpaqueSeparator));
    pub const PLACEHOLDER_TEXT: Color =
        Color::from_repr(ColorRepr::Semantic(SemanticToken::PlaceholderText));
    pub const LINK: Color = Color::from_repr(ColorRepr::Semantic(SemanticToken::Link));
    pub const INFO: Color = Color::from_repr(ColorRepr::Semantic(SemanticToken::Info));
    pub const AI: Color = Color::from_repr(ColorRepr::Semantic(SemanticToken::Ai));

    /// SwiftUI `Color.accentColor` / HIG `tintColor` — resolves to
    /// `theme.accent` (the palette colour chosen via
    /// [`crate::foundations::color::AccentColor`]).
    pub const ACCENT: Color = Color::from_repr(ColorRepr::Semantic(SemanticToken::AccentColor));
    /// SwiftUI `Color.primary` — alias for [`Color::LABEL`].
    pub const PRIMARY: Color = Self::LABEL;
    /// SwiftUI `Color.secondary` — alias for [`Color::SECONDARY_LABEL`].
    pub const SECONDARY: Color = Self::SECONDARY_LABEL;

    // ───── Constructors ─────────────────────────────────────────────────

    const fn from_repr(repr: ColorRepr) -> Self {
        Color {
            repr,
            opacity_multiplier: 1.0,
        }
    }

    /// Build an opaque colour from channel values in the given space.
    pub const fn rgb(space: RgbColorSpace, r: f32, g: f32, b: f32) -> Self {
        Self::rgba(space, r, g, b, 1.0)
    }

    /// Build a colour from channel values and an explicit alpha.
    pub const fn rgba(space: RgbColorSpace, r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::from_repr(ColorRepr::Literal { space, r, g, b, a })
    }

    /// SwiftUI-style grayscale constructor: `white = lightness` across
    /// R/G/B channels (sRGB space).
    pub const fn white(lightness: f32, alpha: f32) -> Self {
        Self::rgba(RgbColorSpace::Srgb, lightness, lightness, lightness, alpha)
    }

    /// Wrap a pre-resolved [`ResolvedColor`] as a `Color`. Used by
    /// [`Color::from_hsla`] and by `TahoeTheme` builders when the stored
    /// value is already concrete.
    pub const fn resolved(value: ResolvedColor) -> Self {
        Self::from_repr(ColorRepr::Resolved(value))
    }

    /// Lift an existing [`gpui::Hsla`] into a `Color`. Eager: performs the
    /// `Hsla → ResolvedColor` conversion at construction time, so
    /// [`Color::into_hsla`] is cheap on the result.
    pub fn from_hsla(h: Hsla) -> Self {
        Self::resolved(ResolvedColor::from_hsla(h))
    }

    // ───── Modifiers ────────────────────────────────────────────────────

    /// Multiply the colour's opacity by `factor`. Mirrors SwiftUI's
    /// `.opacity(_:)`. `factor` is clamped to `[0, 1]`; non-finite inputs
    /// are treated as `1.0` so a single bad caller cannot poison the
    /// pipeline (parity with
    /// [`crate::foundations::color::opacity`]).
    ///
    /// Composes multiplicatively: `c.opacity(0.5).opacity(0.4)` resolves
    /// to the same alpha as `c.opacity(0.2)`.
    pub fn opacity(mut self, factor: f32) -> Self {
        self.opacity_multiplier =
            (self.opacity_multiplier * normalize_factor(factor)).clamp(0.0, 1.0);
        self
    }

    // ───── Resolution ───────────────────────────────────────────────────

    /// Resolve this colour using the theme registered as a GPUI global on
    /// `cx`. Equivalent to `self.resolve_in(&cx.theme().color_environment())`.
    pub fn resolve(&self, cx: &App) -> ResolvedColor {
        self.resolve_in(&cx.theme().color_environment())
    }

    /// Resolve this colour against an explicit [`ColorEnvironment`]. Used
    /// by tests that don't stand up a GPUI `App`.
    pub fn resolve_in(&self, env: &ColorEnvironment<'_>) -> ResolvedColor {
        let mut base = resolve_repr(&self.repr, env);
        if self.opacity_multiplier < 1.0 {
            base.opacity = (base.opacity * self.opacity_multiplier).clamp(0.0, 1.0);
        }
        base
    }

    // ───── Private helpers used by the GPUI bridge ──────────────────────

    /// Try to collapse to an [`Hsla`] without consulting an environment.
    /// Succeeds on literal / resolved variants (with `opacity_multiplier`
    /// applied); returns `Err` on any variant that needs appearance data.
    pub(super) fn try_into_hsla_eager(&self) -> Result<Hsla, &'static str> {
        let mut h = match &self.repr {
            ColorRepr::Literal { space, r, g, b, a } => {
                literal_resolved(*space, *r, *g, *b, *a).to_hsla()
            }
            ColorRepr::Resolved(r) => r.to_hsla(),
            ColorRepr::System(_) => return Err("Color::SystemColor needs a ColorEnvironment"),
            ColorRepr::SystemGray(_) => return Err("Color::SystemGray needs a ColorEnvironment"),
            ColorRepr::Semantic(_) => return Err("Color::Semantic needs a ColorEnvironment"),
        };
        if self.opacity_multiplier < 1.0 {
            h.a = (h.a * self.opacity_multiplier).clamp(0.0, 1.0);
        }
        Ok(h)
    }
}

fn resolve_repr(repr: &ColorRepr, env: &ColorEnvironment<'_>) -> ResolvedColor {
    match repr {
        ColorRepr::Literal { space, r, g, b, a } => literal_resolved(*space, *r, *g, *b, *a),
        ColorRepr::System(sc) => ResolvedColor::from_hsla(sc.resolve(env.appearance)),
        ColorRepr::SystemGray(sg) => ResolvedColor::from_hsla(sg.resolve(env.appearance)),
        ColorRepr::Semantic(token) => ResolvedColor::from_hsla(resolve_semantic(*token, env)),
        ColorRepr::Resolved(r) => *r,
    }
}

fn literal_resolved(space: RgbColorSpace, r: f32, g: f32, b: f32, a: f32) -> ResolvedColor {
    match space {
        RgbColorSpace::Srgb => ResolvedColor::from_srgb(r, g, b, a),
        RgbColorSpace::SrgbLinear => ResolvedColor::from_linear_srgb(r, g, b, a),
        // DisplayP3 is tagged through storage today — GPUI paints sRGB so
        // the values flow as-is until the wide-gamut paint path lands.
        RgbColorSpace::DisplayP3 => ResolvedColor::new(RgbColorSpace::DisplayP3, r, g, b, a),
    }
}

fn resolve_semantic(token: SemanticToken, env: &ColorEnvironment<'_>) -> Hsla {
    let sem: &SemanticColors = env.semantic;
    match token {
        SemanticToken::Label => sem.label,
        SemanticToken::SecondaryLabel => sem.secondary_label,
        SemanticToken::TertiaryLabel => sem.tertiary_label,
        SemanticToken::QuaternaryLabel => sem.quaternary_label,
        SemanticToken::QuinaryLabel => sem.quinary_label,
        SemanticToken::SystemBackground => sem.system_background,
        SemanticToken::SecondarySystemBackground => sem.secondary_system_background,
        SemanticToken::TertiarySystemBackground => sem.tertiary_system_background,
        SemanticToken::SystemGroupedBackground => sem.system_grouped_background,
        SemanticToken::SecondarySystemGroupedBackground => sem.secondary_system_grouped_background,
        SemanticToken::TertiarySystemGroupedBackground => sem.tertiary_system_grouped_background,
        SemanticToken::ElevatedSystemBackground => sem.elevated_system_background,
        SemanticToken::ElevatedSecondarySystemBackground => {
            sem.elevated_secondary_system_background
        }
        SemanticToken::SystemFill => sem.system_fill,
        SemanticToken::SecondarySystemFill => sem.secondary_system_fill,
        SemanticToken::TertiarySystemFill => sem.tertiary_system_fill,
        SemanticToken::QuaternarySystemFill => sem.quaternary_system_fill,
        SemanticToken::QuinarySystemFill => sem.quinary_system_fill,
        SemanticToken::Separator => sem.separator,
        SemanticToken::OpaqueSeparator => sem.opaque_separator,
        SemanticToken::PlaceholderText => sem.placeholder_text,
        SemanticToken::Link => sem.link,
        SemanticToken::Info => sem.info,
        SemanticToken::Ai => sem.ai,
        SemanticToken::AccentColor => env.accent,
    }
}

fn normalize_factor(factor: f32) -> f32 {
    if factor.is_finite() {
        factor.clamp(0.0, 1.0)
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Appearance, ResolvedColor, RgbColorSpace, SystemPalette};
    use super::*;
    use crate::foundations::theme::SemanticColors;
    use core::prelude::v1::test;

    fn test_env(appearance: Appearance) -> (SystemPalette, SemanticColors, Hsla) {
        let palette = SystemPalette::new(appearance);
        let semantic = SemanticColors::new(appearance);
        let accent = palette.blue;
        (palette, semantic, accent)
    }

    /// Allow 1e-3 drift — the Hsla → ResolvedColor → Hsla round-trip goes
    /// through linear-sRGB + Rgba, which introduces f32 error at the
    /// boundaries (e.g. `1.0` comes back as `0.99999994`). The tolerance
    /// matches the plan's explicit 1e-4/1e-3 floor for semantic-token
    /// value comparisons.
    fn assert_hsla_close(got: Hsla, expected: Hsla, ctx: &str) {
        let eps = 1e-3;
        assert!(
            (got.h - expected.h).abs() < eps || (got.h - expected.h).abs() > 1.0 - eps,
            "{ctx}: hue drift {} vs {}",
            got.h,
            expected.h
        );
        assert!(
            (got.s - expected.s).abs() < eps,
            "{ctx}: saturation drift {} vs {}",
            got.s,
            expected.s
        );
        assert!(
            (got.l - expected.l).abs() < eps,
            "{ctx}: lightness drift {} vs {}",
            got.l,
            expected.l
        );
        assert!(
            (got.a - expected.a).abs() < 1e-6,
            "{ctx}: alpha drift {} vs {}",
            got.a,
            expected.a
        );
    }

    #[test]
    fn red_resolves_to_palette_red_in_all_four_appearances() {
        for appearance in [
            Appearance::Light,
            Appearance::Dark,
            Appearance::LightHighContrast,
            Appearance::DarkHighContrast,
        ] {
            let (palette, semantic, accent) = test_env(appearance);
            let env = ColorEnvironment::new(appearance, accent, &semantic, &palette);
            let got = Color::RED.resolve_in(&env).to_hsla();
            assert_hsla_close(
                got,
                SystemColor::Red.resolve(appearance),
                &format!("Color::RED/{appearance:?}"),
            );
        }
    }

    #[test]
    fn label_resolves_to_semantic_label_in_all_four_appearances() {
        for appearance in [
            Appearance::Light,
            Appearance::Dark,
            Appearance::LightHighContrast,
            Appearance::DarkHighContrast,
        ] {
            let (palette, semantic, accent) = test_env(appearance);
            let env = ColorEnvironment::new(appearance, accent, &semantic, &palette);
            let got = Color::LABEL.resolve_in(&env).to_hsla();
            assert_hsla_close(got, semantic.label, &format!("Color::LABEL/{appearance:?}"));
        }
    }

    #[test]
    fn accent_resolves_to_theme_accent() {
        let (palette, semantic, _) = test_env(Appearance::Dark);
        // Build an env where `accent` is something non-default (purple) to
        // prove ACCENT uses env.accent, not palette.blue.
        let non_default_accent = palette.purple;
        let env = ColorEnvironment::new(Appearance::Dark, non_default_accent, &semantic, &palette);
        assert_hsla_close(
            Color::ACCENT.resolve_in(&env).to_hsla(),
            non_default_accent,
            "Color::ACCENT",
        );
    }

    #[test]
    fn opacity_modifier_multiplies_final_alpha() {
        let (palette, semantic, accent) = test_env(Appearance::Light);
        let env = ColorEnvironment::new(Appearance::Light, accent, &semantic, &palette);
        let base = Color::BLUE.resolve_in(&env).opacity;
        let halved = Color::BLUE.opacity(0.5).resolve_in(&env).opacity;
        assert!((halved - base * 0.5).abs() < 1e-6);
    }

    #[test]
    fn opacity_clamps_factor() {
        let (palette, semantic, accent) = test_env(Appearance::Light);
        let env = ColorEnvironment::new(Appearance::Light, accent, &semantic, &palette);
        // negative clamps to 0, >1 clamps to 1, NaN → leave alpha unchanged.
        assert!(Color::RED.opacity(-0.5).resolve_in(&env).opacity.abs() < 1e-6);
        let full = Color::RED.resolve_in(&env).opacity;
        assert!((Color::RED.opacity(2.0).resolve_in(&env).opacity - full).abs() < 1e-6);
        assert!((Color::RED.opacity(f32::NAN).resolve_in(&env).opacity - full).abs() < 1e-6);
    }

    #[test]
    fn opacity_is_compositional() {
        // .opacity(a).opacity(b) ≡ .opacity(a * b)
        let (palette, semantic, accent) = test_env(Appearance::Light);
        let env = ColorEnvironment::new(Appearance::Light, accent, &semantic, &palette);
        let combined = Color::RED
            .opacity(0.5)
            .opacity(0.4)
            .resolve_in(&env)
            .opacity;
        let single = Color::RED.opacity(0.2).resolve_in(&env).opacity;
        assert!((combined - single).abs() < 1e-6);
    }

    #[test]
    fn constants_are_const_evaluable() {
        // Associated const resolution should not need runtime state.
        const _: Color = Color::RED;
        const _: Color = Color::LABEL;
        const _: Color = Color::ACCENT;
        const _: Color = Color::WHITE;
        const _: Color = Color::BLACK;
        const _: Color = Color::CLEAR;
        const _: Color = Color::GRAY_6;
        const _: Color = Color::rgb(RgbColorSpace::Srgb, 0.1, 0.2, 0.3);
        const _: Color = Color::rgba(RgbColorSpace::SrgbLinear, 0.1, 0.2, 0.3, 0.5);
        const _: Color = Color::white(0.5, 1.0);
    }

    #[test]
    fn primary_aliases_label() {
        let (palette, semantic, accent) = test_env(Appearance::Light);
        let env = ColorEnvironment::new(Appearance::Light, accent, &semantic, &palette);
        assert_eq!(
            Color::PRIMARY.resolve_in(&env).to_hsla(),
            Color::LABEL.resolve_in(&env).to_hsla(),
        );
        assert_eq!(
            Color::SECONDARY.resolve_in(&env).to_hsla(),
            Color::SECONDARY_LABEL.resolve_in(&env).to_hsla(),
        );
    }

    #[test]
    fn resolved_variant_survives_roundtrip() {
        let (palette, semantic, accent) = test_env(Appearance::Dark);
        let env = ColorEnvironment::new(Appearance::Dark, accent, &semantic, &palette);
        let raw = ResolvedColor::from_srgb(0.2, 0.4, 0.6, 0.8);
        let c = Color::resolved(raw);
        let back = c.resolve_in(&env);
        assert_eq!(back, raw);
    }

    #[test]
    fn literal_srgb_decodes_on_resolve() {
        let (palette, semantic, accent) = test_env(Appearance::Light);
        let env = ColorEnvironment::new(Appearance::Light, accent, &semantic, &palette);
        let c = Color::rgb(RgbColorSpace::Srgb, 0.5, 0.5, 0.5);
        let via_resolve = c.resolve_in(&env);
        let direct = ResolvedColor::from_srgb(0.5, 0.5, 0.5, 1.0);
        assert_eq!(via_resolve, direct);
    }

    #[test]
    fn every_semantic_token_resolves_without_nan_in_every_appearance() {
        let tokens = [
            SemanticToken::Label,
            SemanticToken::SecondaryLabel,
            SemanticToken::TertiaryLabel,
            SemanticToken::QuaternaryLabel,
            SemanticToken::QuinaryLabel,
            SemanticToken::SystemBackground,
            SemanticToken::SecondarySystemBackground,
            SemanticToken::TertiarySystemBackground,
            SemanticToken::SystemGroupedBackground,
            SemanticToken::SecondarySystemGroupedBackground,
            SemanticToken::TertiarySystemGroupedBackground,
            SemanticToken::ElevatedSystemBackground,
            SemanticToken::ElevatedSecondarySystemBackground,
            SemanticToken::SystemFill,
            SemanticToken::SecondarySystemFill,
            SemanticToken::TertiarySystemFill,
            SemanticToken::QuaternarySystemFill,
            SemanticToken::QuinarySystemFill,
            SemanticToken::Separator,
            SemanticToken::OpaqueSeparator,
            SemanticToken::PlaceholderText,
            SemanticToken::Link,
            SemanticToken::Info,
            SemanticToken::Ai,
            SemanticToken::AccentColor,
        ];
        for appearance in [
            Appearance::Light,
            Appearance::Dark,
            Appearance::LightHighContrast,
            Appearance::DarkHighContrast,
        ] {
            let (palette, semantic, accent) = test_env(appearance);
            let env = ColorEnvironment::new(appearance, accent, &semantic, &palette);
            for token in tokens {
                let c = Color::from_repr(ColorRepr::Semantic(token));
                let r = c.resolve_in(&env);
                assert!(
                    r.linear_red.is_finite()
                        && r.linear_green.is_finite()
                        && r.linear_blue.is_finite()
                        && r.opacity.is_finite(),
                    "{token:?} produced NaN in {appearance:?}"
                );
            }
        }
    }

    #[test]
    fn try_into_hsla_eager_succeeds_on_resolved_literal_and_their_opacity() {
        let hsla = Color::from_hsla(gpui::Hsla {
            h: 0.25,
            s: 0.6,
            l: 0.5,
            a: 1.0,
        })
        .try_into_hsla_eager();
        assert!(hsla.is_ok());

        let lit = Color::rgb(RgbColorSpace::Srgb, 0.5, 0.0, 0.0).try_into_hsla_eager();
        assert!(lit.is_ok());

        let composed = Color::from_hsla(gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
        })
        .opacity(0.3)
        .try_into_hsla_eager();
        assert!(composed.is_ok());
        assert!((composed.unwrap().a - 0.3).abs() < 1e-6);
    }

    #[test]
    fn try_into_hsla_eager_errors_on_deferred_tokens() {
        assert!(Color::RED.try_into_hsla_eager().is_err());
        assert!(Color::GRAY_3.try_into_hsla_eager().is_err());
        assert!(Color::ACCENT.try_into_hsla_eager().is_err());
        assert!(Color::LABEL.try_into_hsla_eager().is_err());
    }

    #[test]
    fn color_is_copy() {
        // Phase 3 load-bearing: `.bg(theme.accent)` needs Color to be Copy
        // so call sites don't have to clone or borrow. Guard the trait
        // bounds with a const so accidental Arc/Box reintroduction breaks
        // the build instead of the Phase 3 sweep.
        const fn assert_copy<T: Copy>() {}
        assert_copy::<Color>();
        assert_copy::<ColorRepr>();
        // A theme field is accessed through `&TahoeTheme`: simulate the
        // move-out to prove it works.
        let holder = Color::BLUE;
        let a = holder;
        let b = holder; // would fail to compile if Color weren't Copy
        let _ = (a, b);
    }

    #[test]
    fn inline_opacity_multiplier_composes() {
        // .opacity(0.5).opacity(0.4) → multiplier = 0.2 (same as .opacity(0.2))
        let c = Color::BLUE.opacity(0.5).opacity(0.4);
        assert!((c.opacity_multiplier - 0.2).abs() < 1e-6);
    }
}
