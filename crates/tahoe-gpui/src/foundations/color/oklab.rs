//! OKLab colour space conversions.
//!
//! Implements the canonical Björn Ottosson OKLab matrix for perceptually
//! uniform colour interpolation. Used by [`super::token::Color::mix`].

#![allow(clippy::excessive_precision)]

use super::srgb::{linear_to_srgb, srgb_to_linear};

/// Convert gamma-encoded sRGB `[0..1]³` to OKLab `[L, a, b]`.
///
/// L is in `[0..1]` for sRGB-gamut colours; a and b are approximately
/// `[-0.5..0.5]` but can extend beyond for wide-gamut inputs.
pub fn srgb_to_oklab(srgb: [f32; 3]) -> [f32; 3] {
    let [r, g, b] = [
        srgb_to_linear(srgb[0]),
        srgb_to_linear(srgb[1]),
        srgb_to_linear(srgb[2]),
    ];

    // Linear sRGB → LMS (M1)
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;

    // Cube root
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();

    // LMS' → OKLab (M2)
    let ok_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
    let ok_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
    let ok_b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

    [ok_l, ok_a, ok_b]
}

/// Convert OKLab `[L, a, b]` to gamma-encoded sRGB `[0..1]³`.
///
/// Output channels are clamped to `[0, 1]` after the round-trip.
pub fn oklab_to_srgb(lab: [f32; 3]) -> [f32; 3] {
    let [ok_l, ok_a, ok_b] = lab;

    // Inverse M2: OKLab → LMS'
    let l_ = ok_l + 0.3963377774 * ok_a + 0.2158037573 * ok_b;
    let m_ = ok_l - 0.1055613458 * ok_a - 0.0638541728 * ok_b;
    let s_ = ok_l - 0.0894841775 * ok_a - 1.2914855480 * ok_b;

    // Cube
    let l = l_ * l_ * l_;
    let m = m_ * m_ * m_;
    let s = s_ * s_ * s_;

    // Inverse M1: LMS → linear sRGB
    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    [
        linear_to_srgb(r).clamp(0.0, 1.0),
        linear_to_srgb(g).clamp(0.0, 1.0),
        linear_to_srgb(b).clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::{oklab_to_srgb, srgb_to_oklab};
    use core::prelude::v1::test;

    #[test]
    fn srgb_oklab_round_trip_within_1e_4() {
        let cases = [
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.5, 0.5, 0.5],
            [0.9, 0.1, 0.3],
            [0.1, 0.3, 0.9],
        ];
        for &input in &cases {
            let lab = srgb_to_oklab(input);
            let round = oklab_to_srgb(lab);
            for i in 0..3 {
                assert!(
                    (round[i] - input[i]).abs() < 1e-4,
                    "round-trip failed for sRGB {:?}: got {:?} (channel {i} drift {})",
                    input,
                    round,
                    (round[i] - input[i]).abs()
                );
            }
        }
    }

    #[test]
    fn oklab_mid_grey_has_l_near_half() {
        let [l, a, b] = srgb_to_oklab([0.5, 0.5, 0.5]);
        assert!(
            (l - 0.5).abs() < 0.10,
            "mid-grey L should be near 0.5, got {l}"
        );
        assert!(a.abs() < 0.01, "mid-grey a should be ~0, got {a}");
        assert!(b.abs() < 0.01, "mid-grey b should be ~0, got {b}");
    }
}
