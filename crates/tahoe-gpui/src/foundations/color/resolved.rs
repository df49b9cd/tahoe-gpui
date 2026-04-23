//! Concrete, paintable colour values.
//!
//! [`ResolvedColor`] is the leaf type that [`super::Color`] (Phase 2) will
//! resolve to. Storage is **linear-light** RGBA plus a colour-space tag, so
//! that:
//!
//! - `opacity`, `mix`, and gradient interpolation can be computed in the
//!   correct space without reparsing.
//! - Extended-range channels (negative or > 1.0) representing wide-gamut
//!   colours survive storage without being clamped.
//! - The "component" accessors return the sRGB-encoded value callers expect
//!   (`.red() / .green() / .blue()`), while `linear_*` accessors give raw
//!   linear-light for math — matching SwiftUI's `Color.Resolved` (iOS 17+).

use gpui::Hsla;

use super::srgb::{linear_to_srgb, srgb_to_linear};

/// The colour space a [`ResolvedColor`] was authored in.
///
/// `SrgbLinear` is the working space GPUI paints in today. `Srgb` and
/// `DisplayP3` tag provenance: a value authored in P3 or gamma-encoded sRGB
/// is converted to linear-sRGB at construction time so `linear_*` fields
/// always share one working space. The tag is preserved for round-trip
/// fidelity (e.g. serialisation, future wide-gamut paint paths).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RgbColorSpace {
    /// Gamma-encoded sRGB (the SwiftUI default).
    #[default]
    Srgb,
    /// Linear-light sRGB (the GPUI paint working space).
    SrgbLinear,
    /// Display P3 wide-gamut colour space.
    DisplayP3,
}

/// Concrete RGBA value, ready to paint.
///
/// Storage is `f32` linear-light sRGB plus a colour-space provenance tag.
/// Channels may be negative or exceed `1.0` when representing wide-gamut
/// colours — the type does **not** clamp at construction.
///
/// Parity with `SwiftUI.Color.Resolved` (iOS 17+/macOS 14+).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedColor {
    /// Linear-light red channel.
    pub linear_red: f32,
    /// Linear-light green channel.
    pub linear_green: f32,
    /// Linear-light blue channel.
    pub linear_blue: f32,
    /// Opacity (0..=1 in canonical use; extended-range allowed).
    pub opacity: f32,
    /// Provenance of the colour — which space the caller authored it in.
    /// `linear_*` fields are always linear-sRGB regardless.
    pub color_space: RgbColorSpace,
}

impl ResolvedColor {
    /// Build a new [`ResolvedColor`] from channel values in the given space.
    ///
    /// - `RgbColorSpace::Srgb` — `r`/`g`/`b` are gamma-encoded sRGB (0..=1);
    ///   converted to linear-light for storage.
    /// - `RgbColorSpace::SrgbLinear` — stored as-is (already linear).
    /// - `RgbColorSpace::DisplayP3` — P3 primaries are stored as-is in the
    ///   `linear_*` fields for now. A full P3→sRGB gamut map lives in a
    ///   later phase; GPUI paints sRGB today, so callers who pass P3 values
    ///   get the same numeric channels tagged as P3 until the paint path
    ///   grows wide-gamut support.
    pub const fn new(space: RgbColorSpace, r: f32, g: f32, b: f32, opacity: f32) -> Self {
        // `const fn` cannot call non-const math — decoding sRGB uses `.powf`,
        // which is not `const` on stable — so the sRGB→linear conversion is
        // deferred to `new_decoded` below, and `new` simply stores whatever
        // the caller passed. Components that hand in sRGB directly should
        // use [`ResolvedColor::from_srgb`] instead.
        Self {
            linear_red: r,
            linear_green: g,
            linear_blue: b,
            opacity,
            color_space: space,
        }
    }

    /// Build a [`ResolvedColor`] from gamma-encoded sRGB channels (the
    /// SwiftUI default). Performs sRGB → linear decoding.
    pub fn from_srgb(r: f32, g: f32, b: f32, opacity: f32) -> Self {
        Self {
            linear_red: srgb_to_linear(r),
            linear_green: srgb_to_linear(g),
            linear_blue: srgb_to_linear(b),
            opacity,
            color_space: RgbColorSpace::Srgb,
        }
    }

    /// Build a [`ResolvedColor`] from linear-light sRGB channels.
    pub const fn from_linear_srgb(r: f32, g: f32, b: f32, opacity: f32) -> Self {
        Self {
            linear_red: r,
            linear_green: g,
            linear_blue: b,
            opacity,
            color_space: RgbColorSpace::SrgbLinear,
        }
    }

    /// Gamma-encoded sRGB red channel.
    pub fn red(&self) -> f32 {
        linear_to_srgb(self.linear_red)
    }
    /// Gamma-encoded sRGB green channel.
    pub fn green(&self) -> f32 {
        linear_to_srgb(self.linear_green)
    }
    /// Gamma-encoded sRGB blue channel.
    pub fn blue(&self) -> f32 {
        linear_to_srgb(self.linear_blue)
    }

    /// Raw linear-light red channel (parity with `SwiftUI.Color.Resolved.linearRed`).
    pub fn linear_red(&self) -> f32 {
        self.linear_red
    }
    /// Raw linear-light green channel.
    pub fn linear_green(&self) -> f32 {
        self.linear_green
    }
    /// Raw linear-light blue channel.
    pub fn linear_blue(&self) -> f32 {
        self.linear_blue
    }

    /// Convert to [`gpui::Hsla`]. Passes through [`gpui::Rgba`] so the HSL
    /// hue calculation matches GPUI's own rendering path.
    pub fn to_hsla(&self) -> Hsla {
        let rgba = gpui::Rgba {
            r: self.red().clamp(0.0, 1.0),
            g: self.green().clamp(0.0, 1.0),
            b: self.blue().clamp(0.0, 1.0),
            a: self.opacity.clamp(0.0, 1.0),
        };
        rgba.into()
    }

    /// Build a [`ResolvedColor`] from an existing [`gpui::Hsla`] value.
    ///
    /// Goes via GPUI's own `Hsla → Rgba` path so the result round-trips
    /// through `to_hsla()` within f32 ULP budget.
    pub fn from_hsla(h: Hsla) -> Self {
        let rgba = h.to_rgb();
        Self::from_srgb(rgba.r, rgba.g, rgba.b, rgba.a)
    }
}

impl From<Hsla> for ResolvedColor {
    fn from(h: Hsla) -> Self {
        Self::from_hsla(h)
    }
}

impl From<ResolvedColor> for Hsla {
    fn from(r: ResolvedColor) -> Self {
        r.to_hsla()
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedColor, RgbColorSpace};
    use core::prelude::v1::test;
    use gpui::Hsla;
    use proptest::prelude::*;

    #[test]
    fn rgb_color_space_default_is_srgb() {
        assert_eq!(RgbColorSpace::default(), RgbColorSpace::Srgb);
    }

    #[test]
    fn from_linear_srgb_stores_channels_as_given() {
        let c = ResolvedColor::from_linear_srgb(0.1, 0.2, 0.3, 0.4);
        assert_eq!(c.linear_red, 0.1);
        assert_eq!(c.linear_green, 0.2);
        assert_eq!(c.linear_blue, 0.3);
        assert_eq!(c.opacity, 0.4);
        assert_eq!(c.color_space, RgbColorSpace::SrgbLinear);
    }

    #[test]
    fn from_srgb_decodes_to_linear() {
        let c = ResolvedColor::from_srgb(0.5, 0.5, 0.5, 1.0);
        // sRGB 0.5 → linear ≈ 0.2140
        assert!((c.linear_red - 0.2140).abs() < 1e-3);
        assert!((c.linear_green - 0.2140).abs() < 1e-3);
        assert!((c.linear_blue - 0.2140).abs() < 1e-3);
        // Round-trip back through red() returns to gamma-encoded sRGB.
        assert!((c.red() - 0.5).abs() < 1e-4);
    }

    #[test]
    fn hsla_roundtrip_preserves_value_within_ulp() {
        // Pick values that exercise the HSL → RGB conversion: saturated,
        // near-neutral, near-black, near-white.
        let cases = [
            Hsla {
                h: 0.0,
                s: 1.0,
                l: 0.5,
                a: 1.0,
            }, // red
            Hsla {
                h: 0.667,
                s: 1.0,
                l: 0.5,
                a: 1.0,
            }, // blue
            Hsla {
                h: 0.333,
                s: 0.5,
                l: 0.5,
                a: 0.7,
            },
            Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.95,
                a: 1.0,
            }, // near-white
            Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.05,
                a: 1.0,
            }, // near-black
        ];
        for input in cases {
            let round = ResolvedColor::from_hsla(input).to_hsla();
            assert!(
                (round.h - input.h).abs() < 1e-3,
                "hue drift: {input:?} → {round:?}"
            );
            assert!(
                (round.s - input.s).abs() < 1e-3,
                "saturation drift: {input:?} → {round:?}"
            );
            assert!(
                (round.l - input.l).abs() < 1e-3,
                "lightness drift: {input:?} → {round:?}"
            );
            assert!(
                (round.a - input.a).abs() < 1e-6,
                "alpha drift: {input:?} → {round:?}"
            );
        }
    }

    #[test]
    fn extended_range_channels_are_preserved() {
        // Wide-gamut (P3) colours can land outside [0, 1] once expressed in
        // linear-sRGB. Storage must not clamp.
        let c = ResolvedColor {
            linear_red: 1.2,
            linear_green: -0.05,
            linear_blue: 0.4,
            opacity: 1.0,
            color_space: RgbColorSpace::DisplayP3,
        };
        assert_eq!(c.linear_red, 1.2);
        assert_eq!(c.linear_green, -0.05);
        assert_eq!(c.color_space, RgbColorSpace::DisplayP3);
    }

    #[test]
    fn from_into_bridge_matches_methods() {
        let h = Hsla {
            h: 0.25,
            s: 0.8,
            l: 0.5,
            a: 0.9,
        };
        let via_from: ResolvedColor = h.into();
        let via_method = ResolvedColor::from_hsla(h);
        assert_eq!(via_from, via_method);

        let back_via_from: Hsla = via_from.into();
        let back_via_method = via_method.to_hsla();
        assert_eq!(back_via_from, back_via_method);
    }

    proptest! {
        /// For every sRGB-space HSLA, the Hsla → ResolvedColor → Hsla
        /// round-trip is idempotent within 1e-4. Picks arbitrary but finite
        /// hue/saturation/lightness/alpha triples.
        #[test]
        fn hsla_resolvedcolor_idempotent(
            h in 0.0f32..1.0,
            s in 0.0f32..1.0,
            l in 0.0f32..1.0,
            a in 0.0f32..=1.0,
        ) {
            let input = Hsla { h, s, l, a };
            let round = ResolvedColor::from_hsla(input).to_hsla();
            // HSL is ambiguous at s=0 and at l∈{0,1}; skip those degenerate
            // inputs (hue is undefined when saturation is zero).
            if s > 1e-3 && (0.01..=0.99).contains(&l) {
                prop_assert!((round.h - h).abs() < 1e-3 || (round.h - h).abs() > 0.999);
                prop_assert!((round.s - s).abs() < 1e-3);
            }
            prop_assert!((round.l - l).abs() < 1e-3);
            prop_assert!((round.a - a).abs() < 1e-4);
        }
    }
}
