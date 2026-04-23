//! Hex and HSB colour parsers, plus the inverse formatters used by
//! `components/color_well`.
//!
//! Promotes the private helpers that used to live inside `color_well.rs`
//! to the public API so any component (or user) can parse `#RRGGBB`
//! strings and HSB triples into [`Color`] without reaching into a
//! component's implementation.

use gpui::{Hsla, Rgba};

use super::{ResolvedColor, token::Color};

/// Error returned by [`Color::hex`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseColorError {
    /// The string does not fit `#RGB` / `#RGBA` / `#RRGGBB` / `#RRGGBBAA`
    /// (with optional leading `#`).
    InvalidFormat,
    /// The string contains non-ASCII / non-hex characters.
    InvalidCharacter,
}

/// Parse `#RGB`, `#RGBA`, `#RRGGBB`, or `#RRGGBBAA` into raw RGBA bytes.
///
/// Whitespace around the leading `#` is tolerated; the leading `#` itself
/// is optional. Non-ASCII input is rejected (UTF-8 safety — see the
/// regression tests cited by issue #60). Returns `(r, g, b, a)` each
/// 0..=255.
pub fn hex_to_rgba_bytes(input: &str) -> Result<(u8, u8, u8, u8), ParseColorError> {
    let trimmed = input.trim().trim_start_matches('#');
    if !trimmed.is_ascii() {
        return Err(ParseColorError::InvalidCharacter);
    }
    let (r, g, b, a) = match trimmed.len() {
        3 => {
            let r = decode_nibble_pair(&trimmed[0..1])?;
            let g = decode_nibble_pair(&trimmed[1..2])?;
            let b = decode_nibble_pair(&trimmed[2..3])?;
            (r, g, b, 255)
        }
        4 => {
            let r = decode_nibble_pair(&trimmed[0..1])?;
            let g = decode_nibble_pair(&trimmed[1..2])?;
            let b = decode_nibble_pair(&trimmed[2..3])?;
            let a = decode_nibble_pair(&trimmed[3..4])?;
            (r, g, b, a)
        }
        6 => {
            let r = decode_byte(&trimmed[0..2])?;
            let g = decode_byte(&trimmed[2..4])?;
            let b = decode_byte(&trimmed[4..6])?;
            (r, g, b, 255)
        }
        8 => {
            let r = decode_byte(&trimmed[0..2])?;
            let g = decode_byte(&trimmed[2..4])?;
            let b = decode_byte(&trimmed[4..6])?;
            let a = decode_byte(&trimmed[6..8])?;
            (r, g, b, a)
        }
        _ => return Err(ParseColorError::InvalidFormat),
    };
    Ok((r, g, b, a))
}

fn decode_nibble_pair(s: &str) -> Result<u8, ParseColorError> {
    u8::from_str_radix(&s.repeat(2), 16).map_err(|_| ParseColorError::InvalidCharacter)
}

fn decode_byte(s: &str) -> Result<u8, ParseColorError> {
    u8::from_str_radix(s, 16).map_err(|_| ParseColorError::InvalidCharacter)
}

/// Parse a hex string into [`Hsla`]. Thin wrapper over
/// [`hex_to_rgba_bytes`] for callers still working with the GPUI primitive.
pub fn hex_to_hsla(input: &str) -> Result<Hsla, ParseColorError> {
    let (r, g, b, a) = hex_to_rgba_bytes(input)?;
    let rgba = Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    };
    Ok(rgba.into())
}

/// Render an `Hsla` colour as `#RRGGBB` (or `#RRGGBBAA` when alpha < 1).
pub fn hsla_to_hex(color: Hsla) -> String {
    let rgba = Rgba::from(color);
    let r = (rgba.r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (rgba.g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (rgba.b * 255.0).round().clamp(0.0, 255.0) as u8;
    let a = (rgba.a * 255.0).round().clamp(0.0, 255.0) as u8;
    if a == 255 {
        format!("#{:02X}{:02X}{:02X}", r, g, b)
    } else {
        format!("#{:02X}{:02X}{:02X}{:02X}", r, g, b, a)
    }
}

/// Return the R, G, B (each 0-255) components of an Hsla colour.
pub fn hsla_to_rgb_bytes(color: Hsla) -> (u8, u8, u8) {
    let rgba = Rgba::from(color);
    let r = (rgba.r * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = (rgba.g * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = (rgba.b * 255.0).round().clamp(0.0, 255.0) as u8;
    (r, g, b)
}

/// Convert HSL (0..=1 hue/sat/light) to HSB (hue in degrees 0-360,
/// saturation 0-100, brightness 0-100). Matches `NSColorPanel`'s HSB tab.
pub fn hsla_to_hsb(color: Hsla) -> (u32, u32, u32) {
    let h = (color.h * 360.0).round().clamp(0.0, 360.0) as u32;
    let l = color.l.clamp(0.0, 1.0);
    let s = color.s.clamp(0.0, 1.0);
    // HSL → HSV conversion. V = L + S · min(L, 1 - L)
    let v = l + s * l.min(1.0 - l);
    let sv = if v == 0.0 { 0.0 } else { 2.0 * (1.0 - l / v) };
    let s_pct = (sv * 100.0).round().clamp(0.0, 100.0) as u32;
    let b_pct = (v * 100.0).round().clamp(0.0, 100.0) as u32;
    (h, s_pct, b_pct)
}

/// Convert HSB (h in 0..=1 turns, s/b in 0..=1, a in 0..=1) to an Hsla
/// value via the canonical HSV→HSL identity `L = V·(1 − S/2)`.
///
/// Mirrors SwiftUI `Color(hue:saturation:brightness:opacity:)`. Inputs are
/// clamped to their canonical ranges; non-finite values collapse to 0.
pub fn hsb_to_hsla(hue: f32, saturation: f32, brightness: f32, alpha: f32) -> Hsla {
    let h = clamp_canonical(hue);
    let s_hsv = clamp_canonical(saturation);
    let v = clamp_canonical(brightness);
    let a = clamp_canonical(alpha);
    let l = v * (1.0 - s_hsv / 2.0);
    let s_hsl = if l > 0.0 && l < 1.0 {
        (v - l) / l.min(1.0 - l)
    } else {
        0.0
    };
    Hsla {
        h,
        s: s_hsl.clamp(0.0, 1.0),
        l: l.clamp(0.0, 1.0),
        a,
    }
}

fn clamp_canonical(v: f32) -> f32 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

impl Color {
    /// Parse a hex string (`#RGB` / `#RGBA` / `#RRGGBB` / `#RRGGBBAA` —
    /// leading `#` optional) into a [`Color`]. Mirrors the private hex
    /// parser that used to live in `components/color_well`.
    ///
    /// The result is eager — stored as `Color::resolved(...)` — so
    /// [`Color::into_hsla`] is cheap and the GPUI bridge never panics.
    pub fn hex(input: &str) -> Result<Self, ParseColorError> {
        let (r, g, b, a) = hex_to_rgba_bytes(input)?;
        Ok(Color::resolved(ResolvedColor::from_srgb(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )))
    }

    /// SwiftUI-parity `Color(hue: saturation: brightness: opacity:)`.
    ///
    /// Inputs: `hue` is in 0..=1 turns (wrap at 1.0), `saturation` and
    /// `brightness` are 0..=1, `alpha` is 0..=1. Non-finite values collapse
    /// to 0.
    pub fn hsb(hue: f32, saturation: f32, brightness: f32, alpha: f32) -> Self {
        Color::from_hsla(hsb_to_hsla(hue, saturation, brightness, alpha))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    // ─── hex_to_rgba_bytes ─────────────────────────────────────────────

    #[test]
    fn hex_rrggbb_full_channels() {
        assert_eq!(hex_to_rgba_bytes("#FF0000"), Ok((255, 0, 0, 255)));
        assert_eq!(hex_to_rgba_bytes("#00FF00"), Ok((0, 255, 0, 255)));
        assert_eq!(hex_to_rgba_bytes("#0000FF"), Ok((0, 0, 255, 255)));
    }

    #[test]
    fn hex_rrggbbaa_full_channels() {
        assert_eq!(hex_to_rgba_bytes("#FF0000AA"), Ok((255, 0, 0, 170)));
    }

    #[test]
    fn hex_short_form_expands() {
        assert_eq!(hex_to_rgba_bytes("#F00"), Ok((255, 0, 0, 255)));
        assert_eq!(hex_to_rgba_bytes("#F00A"), Ok((255, 0, 0, 170)));
    }

    #[test]
    fn hex_leading_hash_optional() {
        assert_eq!(hex_to_rgba_bytes("FF0000"), hex_to_rgba_bytes("#FF0000"));
    }

    #[test]
    fn hex_whitespace_tolerated() {
        assert_eq!(hex_to_rgba_bytes("  #FF0000  "), Ok((255, 0, 0, 255)));
    }

    #[test]
    fn hex_lowercase_accepted() {
        assert_eq!(hex_to_rgba_bytes("#ff00aa"), Ok((255, 0, 170, 255)));
    }

    #[test]
    fn hex_rejects_bad_length() {
        assert_eq!(
            hex_to_rgba_bytes("#12345"),
            Err(ParseColorError::InvalidFormat)
        );
    }

    #[test]
    fn hex_rejects_multibyte_len_3() {
        assert!(hex_to_rgba_bytes("#0é").is_err());
    }

    #[test]
    fn hex_rejects_multibyte_len_6() {
        assert!(hex_to_rgba_bytes("#0é000").is_err());
    }

    #[test]
    fn hex_rejects_emoji_len_8() {
        assert!(hex_to_rgba_bytes("#🎨0000").is_err());
    }

    #[test]
    fn hex_rejects_non_hex_ascii() {
        assert!(hex_to_rgba_bytes("#GHIJKL").is_err());
    }

    // ─── Color::hex ────────────────────────────────────────────────────

    #[test]
    fn color_hex_produces_resolved_variant() {
        let c = Color::hex("#FF0000").unwrap();
        // A resolved Color round-trips through the bridge without panicking.
        let h: Hsla = c.into();
        assert!((h.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn color_hex_roundtrips_via_hsla_to_hex() {
        let c = Color::hex("#4080C0").unwrap();
        let h: Hsla = c.into();
        let back = hsla_to_hex(h);
        assert_eq!(back, "#4080C0");
    }

    #[test]
    fn color_hex_invalid_returns_err() {
        assert!(Color::hex("not hex").is_err());
        assert!(Color::hex("#XX0000").is_err());
    }

    // ─── hsb_to_hsla ───────────────────────────────────────────────────

    #[test]
    fn hsb_round_trip_primary_red() {
        // HSB pure red: (0, 1, 1, 1) → should match SystemColor::Red in sRGB.
        let h = hsb_to_hsla(0.0, 1.0, 1.0, 1.0);
        assert!(h.h.abs() < 1e-4);
        assert!((h.s - 1.0).abs() < 1e-4);
        assert!((h.l - 0.5).abs() < 1e-4);
        assert!((h.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hsb_round_trip_primary_green() {
        let h = hsb_to_hsla(1.0 / 3.0, 1.0, 1.0, 1.0);
        assert!((h.h - 1.0 / 3.0).abs() < 1e-4);
        assert!((h.s - 1.0).abs() < 1e-4);
        assert!((h.l - 0.5).abs() < 1e-4);
    }

    #[test]
    fn hsb_zero_brightness_is_black() {
        let h = hsb_to_hsla(0.5, 1.0, 0.0, 1.0);
        assert!(h.l.abs() < 1e-6);
    }

    #[test]
    fn hsb_zero_saturation_is_gray() {
        let h = hsb_to_hsla(0.5, 0.0, 0.5, 1.0);
        assert!(h.s.abs() < 1e-6);
        assert!((h.l - 0.5).abs() < 1e-4);
    }

    #[test]
    fn hsb_non_finite_collapses_to_zero() {
        let h = hsb_to_hsla(f32::NAN, 0.5, 0.5, 1.0);
        assert_eq!(h.h, 0.0);
    }

    #[test]
    fn hsla_to_hsb_round_trip_within_tolerance() {
        // Take a known HSL, convert to HSB, convert back via hsb_to_hsla,
        // and check the round-trip stays within 1 degree / 1% of the
        // original.
        let input = Hsla {
            h: 0.4,
            s: 0.6,
            l: 0.5,
            a: 1.0,
        };
        let (h_deg, s_pct, b_pct) = hsla_to_hsb(input);
        let back = hsb_to_hsla(
            h_deg as f32 / 360.0,
            s_pct as f32 / 100.0,
            b_pct as f32 / 100.0,
            1.0,
        );
        assert!((back.h - input.h).abs() < 1.0 / 360.0 + 1e-3);
        assert!((back.l - input.l).abs() < 0.01);
    }

    // ─── hsla_to_hex ───────────────────────────────────────────────────

    #[test]
    fn hsla_to_hex_opaque() {
        let c: Hsla = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
        .into();
        assert_eq!(hsla_to_hex(c), "#FF0000");
    }

    #[test]
    fn hsla_to_hex_alpha_suffix_on_partial_alpha() {
        let c: Hsla = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        }
        .into();
        let hex = hsla_to_hex(c);
        assert!(hex.starts_with("#FF0000"));
        assert_eq!(hex.len(), 9, "partial alpha should emit #RRGGBBAA");
    }

    // ─── hsla_to_rgb_bytes ─────────────────────────────────────────────

    #[test]
    fn rgb_bytes_match_known_colour() {
        let yellow: Hsla = Rgba {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }
        .into();
        assert_eq!(hsla_to_rgb_bytes(yellow), (255, 255, 0));
    }
}
