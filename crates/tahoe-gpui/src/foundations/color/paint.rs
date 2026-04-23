//! `Paint` — polymorphic fill type mirroring SwiftUI's `ShapeStyle`.
//!
//! A `Paint` is either a flat [`Color`] or an [`AnyGradient`]. Components
//! that accept a fill can take `impl Into<Paint>` and handle both cases.
//!
//! Also provides [`IntoElement for Color`] so that `Color::BLUE` can be used
//! directly as a SwiftUI-style view (renders as a flex-1 div with the colour
//! as its background).

use gpui::{Div, IntoElement, Styled, div};

use super::Color;
use super::gradient::AnyGradient;

/// Polymorphic fill — mirrors SwiftUI's `ShapeStyle`.
#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Color(Color),
    Gradient(AnyGradient),
}

impl From<Color> for Paint {
    fn from(c: Color) -> Self {
        Paint::Color(c)
    }
}

impl From<AnyGradient> for Paint {
    fn from(g: AnyGradient) -> Self {
        Paint::Gradient(g)
    }
}

// ── IntoElement for Color (SwiftUI parity) ──────────────────────────────

/// Treat a [`Color`] as a SwiftUI-style view: a flex-1 div filled with the
/// colour. Mirrors `Color.blue` used as a view in SwiftUI.
impl IntoElement for Color {
    type Element = Div;

    fn into_element(self) -> Self::Element {
        div().flex_1().bg(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::Hsla;

    #[test]
    fn color_into_paint() {
        let c = Color::from_hsla(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.5,
            a: 1.0,
        });
        let p = Paint::from(c);
        assert!(matches!(p, Paint::Color(_)));
    }

    #[test]
    fn gradient_into_paint() {
        let c = Color::from_hsla(Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.5,
            a: 1.0,
        });
        let g = c.gradient();
        let p = Paint::from(g);
        assert!(matches!(p, Paint::Gradient(_)));
    }
}
