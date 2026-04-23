//! Alpha / lightness operators on [`gpui::Hsla`] plus the
//! [`text_on_background`] label-contrast helper.
//!
//! `HslaAlphaExt` mirrors Zed's `Hsla::{alpha, opacity, fade_out}` so call
//! sites can chain `some_hsla.opacity(0.6)` instead of wrapping every alpha
//! tweak in a free function.

use gpui::Hsla;

use super::contrast::relative_luminance;

/// Return `color` with its alpha channel replaced.
///
/// Semantic mirror of Zed's `Hsla::alpha` — assigns the alpha directly
/// without consulting the existing value. Use [`opacity`] to multiply the
/// existing alpha, or [`fade_out`] to shift alpha towards transparency.
pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

/// Return `color` with its alpha multiplied by `factor`.
///
/// Mirrors Zed's `Hsla::opacity(f32)` — scales the existing alpha rather
/// than replacing it, so `#000 @ 60%` composed with `opacity(0.5)` yields
/// `#000 @ 30%`. `factor` is clamped to `[0.0, 1.0]`; non-finite inputs are
/// treated as `1.0` (no change) so a single bad caller cannot poison the
/// color pipeline.
pub fn opacity(color: Hsla, factor: f32) -> Hsla {
    let factor = if factor.is_finite() {
        factor.clamp(0.0, 1.0)
    } else {
        1.0
    };
    Hsla {
        a: (color.a * factor).clamp(0.0, 1.0),
        ..color
    }
}

/// Return `color` with its alpha multiplicatively faded towards zero.
///
/// Mirrors Zed's `Hsla::fade_out(f32)`: a `factor` of `1.0` produces fully
/// transparent, `0.0` leaves the color untouched. Equivalent to
/// `opacity(color, 1.0 - factor)` but phrased so callers can express
/// "fade this out by 30 %" directly.
pub fn fade_out(color: Hsla, factor: f32) -> Hsla {
    let factor = if factor.is_finite() {
        factor.clamp(0.0, 1.0)
    } else {
        0.0
    };
    opacity(color, 1.0 - factor)
}

/// Extension trait that adds Zed-style alpha helpers to [`Hsla`].
///
/// Exposing the helpers as methods lets call sites chain `some_hsla
/// .opacity(0.6)` the same way Zed does, instead of wrapping every alpha
/// tweak in `with_alpha(color, color.a * 0.6)`. Finding 10 in the
/// the Zed cross-reference audit Zed cross-reference audit tracks this gap.
pub trait HslaAlphaExt: Copy {
    /// Replace the alpha channel (see [`with_alpha`]).
    fn alpha(self, alpha: f32) -> Self;
    /// Multiply the alpha channel by `factor` (see [`opacity`]).
    fn opacity(self, factor: f32) -> Self;
    /// Fade towards transparency by `factor` (see [`fade_out`]).
    fn fade_out(self, factor: f32) -> Self;
}

impl HslaAlphaExt for Hsla {
    fn alpha(self, alpha: f32) -> Self {
        with_alpha(self, alpha)
    }

    fn opacity(self, factor: f32) -> Self {
        opacity(self, factor)
    }

    fn fade_out(self, factor: f32) -> Self {
        fade_out(self, factor)
    }
}

/// Darken a color by reducing lightness. Clamps to 0.0.
/// Returns the color unchanged if either `color.l` or `amount` is NaN/infinity.
pub fn darken(color: Hsla, amount: f32) -> Hsla {
    if !color.l.is_finite() || !amount.is_finite() {
        return color;
    }
    Hsla {
        l: (color.l - amount).clamp(0.0, 1.0),
        ..color
    }
}

/// Lighten a color by increasing lightness. Clamps to 1.0.
/// Returns the color unchanged if either `color.l` or `amount` is NaN/infinity.
pub fn lighten(color: Hsla, amount: f32) -> Hsla {
    if !color.l.is_finite() || !amount.is_finite() {
        return color;
    }
    Hsla {
        l: (color.l + amount).clamp(0.0, 1.0),
        ..color
    }
}

/// Choose white or black text for legibility over `bg`.
///
/// Uses WCAG 2.1 relative luminance rather than raw HSL lightness so saturated
/// accents (yellow, orange, cyan) pick the correct label — a naive `bg.l >
/// 0.55` heuristic over-rotates on warm hues where perceived brightness
/// diverges from HSL lightness (yellow at `l = 0.50` reads almost as bright
/// as white).
///
/// The threshold 0.179 is the WCAG "flip point" where white-on-color and
/// black-on-color achieve equal contrast ratio against sRGB midpoint; below
/// it, white gives higher contrast, above it, black does.
///
/// If `bg.l` is NaN, `relative_luminance` propagates NaN, and the comparison
/// `> 0.179` is false, so white is returned — the conservative default for
/// unknown backgrounds.
pub fn text_on_background(bg: Hsla) -> Hsla {
    if relative_luminance(bg) > 0.179 {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 1.0,
        } // black
    } else {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 1.0,
            a: 1.0,
        } // white
    }
}

#[cfg(test)]
mod tests {
    use super::{HslaAlphaExt, darken, fade_out, lighten, opacity, text_on_background, with_alpha};
    use core::prelude::v1::test;
    use gpui::Hsla;

    #[test]
    fn with_alpha_preserves_hsl() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.6,
            a: 1.0,
        };
        let r = with_alpha(c, 0.3);
        assert_eq!(r.h, 0.5);
        assert_eq!(r.s, 0.8);
        assert_eq!(r.l, 0.6);
        assert_eq!(r.a, 0.3);
    }

    #[test]
    fn opacity_multiplies_existing_alpha() {
        let c = Hsla {
            h: 0.25,
            s: 0.6,
            l: 0.5,
            a: 0.6,
        };
        let r = opacity(c, 0.5);
        assert!((r.a - 0.3).abs() < 1e-6, "opacity should scale alpha");
        assert_eq!((r.h, r.s, r.l), (c.h, c.s, c.l));
    }

    #[test]
    fn opacity_clamps_factor() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.5,
            a: 0.8,
        };
        assert!(opacity(c, -0.5).a.abs() < 1e-6);
        assert!((opacity(c, 2.0).a - 0.8).abs() < 1e-6);
        assert!((opacity(c, f32::NAN).a - 0.8).abs() < 1e-6);
    }

    #[test]
    fn fade_out_is_inverse_of_opacity() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.5,
            a: 1.0,
        };
        let faded = fade_out(c, 0.3);
        assert!((faded.a - 0.7).abs() < 1e-6);
        assert!((faded.a - opacity(c, 0.7).a).abs() < 1e-6);
    }

    #[test]
    fn hsla_alpha_ext_chain_matches_free_fns() {
        let c = Hsla {
            h: 0.1,
            s: 0.2,
            l: 0.3,
            a: 0.8,
        };
        assert_eq!(c.alpha(0.4), with_alpha(c, 0.4));
        assert_eq!(c.opacity(0.5), opacity(c, 0.5));
        assert_eq!(c.fade_out(0.25), fade_out(c, 0.25));
    }

    #[test]
    fn darken_reduces_lightness() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.6,
            a: 1.0,
        };
        let r = darken(c, 0.1);
        assert_eq!(r.h, 0.5);
        assert_eq!(r.s, 0.8);
        assert!((r.l - 0.5).abs() < 0.001);
        assert_eq!(r.a, 1.0);
    }

    #[test]
    fn darken_clamps_at_zero() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.05,
            a: 1.0,
        };
        let r = darken(c, 0.2);
        assert_eq!(r.l, 0.0);
    }

    #[test]
    fn lighten_increases_lightness() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.4,
            a: 1.0,
        };
        let r = lighten(c, 0.1);
        assert!((r.l - 0.5).abs() < 0.001);
        assert_eq!(r.a, 1.0);
    }

    #[test]
    fn lighten_clamps_at_one() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.95,
            a: 1.0,
        };
        let r = lighten(c, 0.2);
        assert_eq!(r.l, 1.0);
    }

    #[test]
    fn darken_with_negative_amount_clamps_at_one() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.8,
            a: 1.0,
        };
        let r = darken(c, -0.5);
        assert!(
            r.l <= 1.0,
            "darken with negative amount should clamp at 1.0, got {}",
            r.l
        );
    }

    #[test]
    fn lighten_with_negative_amount_clamps_at_zero() {
        let c = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.1,
            a: 1.0,
        };
        let r = lighten(c, -0.5);
        assert!(
            r.l >= 0.0,
            "lighten with negative amount should clamp at 0.0, got {}",
            r.l
        );
    }

    #[test]
    fn text_on_background_dark_bg_returns_white() {
        let dark = Hsla {
            h: 0.58,
            s: 1.0,
            l: 0.3,
            a: 1.0,
        };
        let result = text_on_background(dark);
        assert_eq!(result.l, 1.0, "dark bg should get white text");
    }

    #[test]
    fn text_on_background_light_bg_returns_black() {
        let light = Hsla {
            h: 0.58,
            s: 1.0,
            l: 0.7,
            a: 1.0,
        };
        let result = text_on_background(light);
        assert_eq!(result.l, 0.0, "light bg should get black text");
    }

    #[test]
    fn text_on_background_nan_defaults_to_white() {
        let nan_bg = Hsla {
            h: 0.0,
            s: 0.0,
            l: f32::NAN,
            a: 1.0,
        };
        let result = text_on_background(nan_bg);
        assert_eq!(result.l, 1.0, "NaN bg.l should default to white");
    }

    #[test]
    fn darken_nan_lightness_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: f32::NAN,
            a: 1.0,
        };
        let r = darken(c, 0.1);
        assert!(r.l.is_nan(), "NaN lightness should pass through unchanged");
        assert_eq!(r.h, 0.5);
    }

    #[test]
    fn lighten_infinity_lightness_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: f32::INFINITY,
            a: 1.0,
        };
        let r = lighten(c, 0.1);
        assert!(r.l.is_infinite());
    }

    #[test]
    fn darken_nan_amount_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.5,
            a: 1.0,
        };
        let r = darken(c, f32::NAN);
        assert_eq!(r.l, 0.5, "NaN amount should leave color unchanged");
    }

    #[test]
    fn lighten_nan_amount_returns_unchanged() {
        let c = Hsla {
            h: 0.5,
            s: 0.8,
            l: 0.5,
            a: 1.0,
        };
        let r = lighten(c, f32::NAN);
        assert_eq!(r.l, 0.5, "NaN amount should leave color unchanged");
    }
}
