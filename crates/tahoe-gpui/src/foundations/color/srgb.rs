//! sRGB ↔ linear-light RGB conversion and linear-space black-tint compositing.
//!
//! Used by Liquid Glass surface composition (Layer 2 black tint) and by the
//! WCAG relative-luminance calculation in `contrast`.

use gpui::Hsla;

/// Convert a single sRGB channel value [0..1] to linear-light RGB.
pub(crate) fn srgb_to_linear(v: f32) -> f32 {
    if v <= 0.040_45 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

/// Convert a single linear-light RGB channel value [0..1] back to sRGB.
pub(crate) fn linear_to_srgb(v: f32) -> f32 {
    if v <= 0.003_130_8 {
        v * 12.92
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}

/// Composite an opaque black tint of `tint_alpha` over `bg` in linear-light
/// RGB space, preserving `bg.a`.
///
/// This is the correct math for Apple's Liquid Glass "Layer 2" tint
/// (`#000000 @ 20%` in Figma) composited over the base Layer 1 fill. A naive
/// blend in HSL lightness diverges from the Porter–Duff linear-light result
/// by up to ~17% on bright surfaces; this helper keeps the tinted surface
/// visually consistent across the full 0..1 lightness range.
///
/// Color channels: `out = (1 - tint_alpha) * srgb_to_linear(bg)` per channel,
/// re-encoded back to sRGB. Alpha is passed through unchanged so the
/// glass surface retains the same translucency it had before tinting.
pub fn compose_black_tint_linear(bg: Hsla, tint_alpha: f32) -> Hsla {
    let keep = 1.0 - tint_alpha.clamp(0.0, 1.0);
    let rgba = bg.to_rgb();
    let blend = |v: f32| linear_to_srgb(keep * srgb_to_linear(v));
    let tinted = gpui::Rgba {
        r: blend(rgba.r),
        g: blend(rgba.g),
        b: blend(rgba.b),
        a: rgba.a,
    };
    let mut out: Hsla = tinted.into();
    out.a = bg.a;
    out
}

#[cfg(test)]
mod tests {
    use super::{compose_black_tint_linear, linear_to_srgb, srgb_to_linear};
    use core::prelude::v1::test;
    use gpui::Hsla;

    #[test]
    fn srgb_linear_roundtrip_is_identity() {
        for &v in &[0.0f32, 0.04, 0.17, 0.5, 0.8, 0.969, 1.0] {
            let round = linear_to_srgb(srgb_to_linear(v));
            assert!(
                (round - v).abs() < 1e-4,
                "sRGB->linear->sRGB roundtrip failed for {v}: got {round}"
            );
        }
    }

    #[test]
    fn compose_black_tint_preserves_alpha() {
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.17,
            a: 0.80,
        };
        let out = compose_black_tint_linear(bg, 0.20);
        assert!(
            (out.a - bg.a).abs() < f32::EPSILON,
            "alpha must be preserved, got {}",
            out.a
        );
    }

    #[test]
    fn compose_black_tint_matches_linear_rgb() {
        // At L=0.969 (light small bg), sRGB ≈ 0.969. Linear ≈ 0.930.
        // 80% of linear ≈ 0.744. Back to sRGB ≈ 0.874.
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.969,
            a: 1.0,
        };
        let out = compose_black_tint_linear(bg, 0.20);
        assert!(
            (out.l - 0.874).abs() < 0.01,
            "expected ~0.874 from linear-RGB blend, got {}",
            out.l
        );
    }

    #[test]
    fn compose_black_tint_scales_linear_dark() {
        // At L=0.17, sRGB=0.17, linear ≈ 0.0225. 80% ≈ 0.018, back to sRGB ≈ 0.144.
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.17,
            a: 0.80,
        };
        let out = compose_black_tint_linear(bg, 0.20);
        assert!(
            (out.l - 0.144).abs() < 0.01,
            "expected ~0.144 from linear-RGB blend, got {}",
            out.l
        );
    }

    #[test]
    fn compose_black_tint_zero_alpha_is_identity() {
        let bg = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.5,
            a: 0.67,
        };
        let out = compose_black_tint_linear(bg, 0.0);
        assert!((out.l - bg.l).abs() < 1e-4);
        assert!((out.a - bg.a).abs() < f32::EPSILON);
    }

    #[test]
    fn compose_black_tint_full_alpha_drives_to_black() {
        let bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.8,
            a: 1.0,
        };
        let out = compose_black_tint_linear(bg, 1.0);
        assert!(
            out.l < 0.01,
            "full tint should collapse to black, got {}",
            out.l
        );
    }
}
