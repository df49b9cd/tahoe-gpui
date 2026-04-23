//! WCAG 2.1 relative-luminance and contrast-ratio utilities.
//!
//! Used for accessibility checks against the HIG 3:1 / 4.5:1 thresholds
//! and by `ops::text_on_background` to pick white vs. black labels.

use gpui::Hsla;

use super::srgb::srgb_to_linear;

/// Computes the relative luminance of an HSLA color (simplified sRGB).
/// Uses the HSL-to-RGB conversion, then applies the sRGB luminance formula.
pub(crate) fn relative_luminance(c: Hsla) -> f32 {
    // Convert HSL to linear RGB
    let h = c.h * 360.0;
    let s = c.s;
    let l = c.l;
    let c_val = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c_val * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = l - c_val / 2.0;
    let (r1, g1, b1) = if h < 60.0 {
        (c_val, x, 0.0)
    } else if h < 120.0 {
        (x, c_val, 0.0)
    } else if h < 180.0 {
        (0.0, c_val, x)
    } else if h < 240.0 {
        (0.0, x, c_val)
    } else if h < 300.0 {
        (x, 0.0, c_val)
    } else {
        (c_val, 0.0, x)
    };
    0.2126 * srgb_to_linear(r1 + m)
        + 0.7152 * srgb_to_linear(g1 + m)
        + 0.0722 * srgb_to_linear(b1 + m)
}

/// Returns the WCAG 2.1 contrast ratio between two colors (1.0 to 21.0).
/// Does not account for alpha compositing — assumes both colors are opaque
/// or pre-composited against their background.
pub fn contrast_ratio(fg: Hsla, bg: Hsla) -> f32 {
    debug_assert!(
        fg.a > 0.99 && bg.a > 0.99,
        "contrast_ratio requires opaque colors; pre-composite against background first"
    );
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

/// Returns `true` if the foreground/background pair meets the specified
/// minimum contrast ratio per WCAG 2.1.
///
/// Common thresholds:
/// - 4.5:1 for normal text (AA)
/// - 3.0:1 for large text and non-text elements (AA)
/// - 7.0:1 for enhanced contrast (AAA)
pub fn meets_contrast(fg: Hsla, bg: Hsla, threshold: f32) -> bool {
    contrast_ratio(fg, bg) >= threshold
}
