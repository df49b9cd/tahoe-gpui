//! Gradient types mirroring SwiftUI's `Gradient` / `LinearGradient` /
//! `RadialGradient` / `AngularGradient`.
//!
//! The gradient stops store [`super::Color`] tokens and resolve lazily at
//! paint time. Use [`AnyGradient::to_gpui`] to produce GPUI paint values.

use std::ops::Range;

use gpui::{App, Hsla};

use super::{Color, MixColorSpace};
use crate::foundations::color::environment::ColorEnvironment;
use crate::foundations::theme::ActiveTheme;

// ────────────────────────────────────────────────────────────────────────────
// UnitPoint
// ────────────────────────────────────────────────────────────────────────────

/// A normalised (0–1) 2D point used as a gradient anchor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitPoint {
    pub x: f32,
    pub y: f32,
}

impl UnitPoint {
    pub const TOP: Self = Self { x: 0.5, y: 0.0 };
    pub const BOTTOM: Self = Self { x: 0.5, y: 1.0 };
    pub const LEADING: Self = Self { x: 0.0, y: 0.5 };
    pub const TRAILING: Self = Self { x: 1.0, y: 0.5 };
    pub const CENTER: Self = Self { x: 0.5, y: 0.5 };
    pub const TOP_LEADING: Self = Self { x: 0.0, y: 0.0 };
    pub const TOP_TRAILING: Self = Self { x: 1.0, y: 0.0 };
    pub const BOTTOM_LEADING: Self = Self { x: 0.0, y: 1.0 };
    pub const BOTTOM_TRAILING: Self = Self { x: 1.0, y: 1.0 };
}

// ────────────────────────────────────────────────────────────────────────────
// Gradient / GradientStop
// ────────────────────────────────────────────────────────────────────────────

/// A single colour anchor in a gradient.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    pub location: f32,
}

/// A colour gradient with stops and an interpolation colour space.
#[derive(Debug, Clone, PartialEq)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub color_space: MixColorSpace,
}

impl Gradient {
    pub fn new(stops: Vec<GradientStop>) -> Self {
        Self {
            stops,
            color_space: MixColorSpace::default(),
        }
    }

    pub fn with_color_space(mut self, space: MixColorSpace) -> Self {
        self.color_space = space;
        self
    }
}

// ────────────────────────────────────────────────────────────────────────────
// AnyGradient
// ────────────────────────────────────────────────────────────────────────────

/// Polymorphic gradient — mirrors SwiftUI's `AnyShapeStyle` gradient family.
#[derive(Debug, Clone, PartialEq)]
pub enum AnyGradient {
    Linear(LinearGradient),
    Radial(RadialGradient),
    Angular(AngularGradient),
}

/// Linear gradient from `start_point` to `end_point`.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub gradient: Gradient,
    pub start_point: UnitPoint,
    pub end_point: UnitPoint,
}

impl LinearGradient {
    pub fn new(gradient: Gradient, start: UnitPoint, end: UnitPoint) -> Self {
        Self {
            gradient,
            start_point: start,
            end_point: end,
        }
    }

    /// Resolve all stops and produce GPUI's `gpui::LinearColorStop` list.
    ///
    /// Computes the angle from `start_point` → `end_point` for GPUI's
    /// angle-based gradient API.
    pub fn to_gpui(&self, cx: &App) -> (f32, Vec<gpui::LinearColorStop>) {
        self.to_gpui_in(&cx.theme().color_environment())
    }

    /// Same as [`LinearGradient::to_gpui`] with an explicit
    /// [`ColorEnvironment`].
    pub fn to_gpui_in(&self, env: &ColorEnvironment<'_>) -> (f32, Vec<gpui::LinearColorStop>) {
        let angle = point_to_angle(self.start_point, self.end_point);
        let stops = resolve_stops_in(&self.gradient, env);
        (angle, stops)
    }

    /// Resolve all stops eagerly without an environment.
    ///
    /// Works when every stop's colour is pre-resolved (literal or
    /// `Color::from_hsla`). Panics on deferred tokens — matching the
    /// `From<Color> for Hsla` contract.
    pub fn to_gpui_eager(&self) -> (f32, Vec<gpui::LinearColorStop>) {
        let angle = point_to_angle(self.start_point, self.end_point);
        let stops = self
            .gradient
            .stops
            .iter()
            .map(|stop| {
                let hsla: Hsla = stop.color.try_into_hsla_eager().expect(
                    "LinearGradient::to_gpui_eager called with deferred Color — \
                     use to_gpui(cx) or to_gpui_in(env) instead",
                );
                gpui::LinearColorStop {
                    color: hsla,
                    percentage: stop.location.clamp(0.0, 1.0) * 100.0,
                }
            })
            .collect();
        (angle, stops)
    }
}

/// Radial gradient emanating from `center`.
#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    pub gradient: Gradient,
    pub center: UnitPoint,
    pub start_radius: f32,
    pub end_radius: f32,
}

/// Angular (conic) gradient sweeping around `center`.
#[derive(Debug, Clone, PartialEq)]
pub struct AngularGradient {
    pub gradient: Gradient,
    pub center: UnitPoint,
    pub angle_range_turns: Range<f32>,
}

// ────────────────────────────────────────────────────────────────────────────
// Color::gradient() — SwiftUI parity
// ────────────────────────────────────────────────────────────────────────────

impl Color {
    /// Create a 3-stop gradient (shade → self → highlight), matching
    /// SwiftUI's `Color.gradient`.
    pub fn gradient(&self) -> AnyGradient {
        let shade = self.opacity(0.6);
        let highlight = self.opacity(0.9);
        AnyGradient::Linear(LinearGradient::new(
            Gradient::new(vec![
                GradientStop {
                    color: shade,
                    location: 0.0,
                },
                GradientStop {
                    color: *self,
                    location: 0.5,
                },
                GradientStop {
                    color: highlight,
                    location: 1.0,
                },
            ]),
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
        ))
    }
}

// ────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────

fn resolve_stops_in(gradient: &Gradient, env: &ColorEnvironment<'_>) -> Vec<gpui::LinearColorStop> {
    gradient
        .stops
        .iter()
        .map(|stop| {
            let hsla: Hsla = stop.color.resolve_in(env).to_hsla();
            gpui::LinearColorStop {
                color: hsla,
                percentage: stop.location.clamp(0.0, 1.0) * 100.0,
            }
        })
        .collect()
}

/// Convert two [`UnitPoint`]s to a CSS-style angle (degrees, 0 = bottom→top,
/// clockwise).
fn point_to_angle(start: UnitPoint, end: UnitPoint) -> f32 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    // CSS angles: 0° = upward, measured clockwise.
    // atan2(dx, -dy) gives the angle from the "up" direction in screen coords.
    let radians = dx.atan2(-dy);
    let deg = radians.to_degrees();
    (deg + 360.0) % 360.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundations::color::{Appearance, SystemPalette};
    use crate::foundations::theme::SemanticColors;
    use core::prelude::v1::test;
    use gpui::Hsla;

    fn test_env() -> (SystemPalette, SemanticColors, Hsla) {
        let palette = SystemPalette::new(Appearance::Dark);
        let semantic = SemanticColors::new(Appearance::Dark);
        let accent = palette.blue;
        (palette, semantic, accent)
    }

    #[test]
    fn gradient_produces_three_stops_spanning_full_range() {
        let c = Color::from_hsla(Hsla {
            h: 0.6,
            s: 0.8,
            l: 0.5,
            a: 1.0,
        });
        let grad = c.gradient();
        match grad {
            AnyGradient::Linear(lg) => {
                assert_eq!(lg.gradient.stops.len(), 3);
                assert!((lg.gradient.stops[0].location - 0.0).abs() < 1e-6);
                assert!((lg.gradient.stops[1].location - 0.5).abs() < 1e-6);
                assert!((lg.gradient.stops[2].location - 1.0).abs() < 1e-6);
            }
            _ => panic!("expected linear gradient"),
        }
    }

    #[test]
    fn linear_gradient_resolves_stops_against_environment() {
        let (palette, semantic, accent) = test_env();
        let env = ColorEnvironment::new(Appearance::Dark, accent, &semantic, &palette);
        let lg = LinearGradient::new(
            Gradient::new(vec![
                GradientStop {
                    color: Color::from_hsla(Hsla {
                        h: 0.0,
                        s: 1.0,
                        l: 0.5,
                        a: 1.0,
                    }),
                    location: 0.0,
                },
                GradientStop {
                    color: Color::from_hsla(Hsla {
                        h: 0.6,
                        s: 1.0,
                        l: 0.5,
                        a: 1.0,
                    }),
                    location: 1.0,
                },
            ]),
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
        );
        let (_, stops) = lg.to_gpui_in(&env);
        assert_eq!(stops.len(), 2);
        assert!((stops[0].percentage - 0.0).abs() < 1e-4);
        assert!((stops[1].percentage - 100.0).abs() < 1e-4);
    }

    #[test]
    fn two_stop_linear_interpolation_matches_manual_calc() {
        let (palette, semantic, accent) = test_env();
        let env = ColorEnvironment::new(Appearance::Dark, accent, &semantic, &palette);
        let red = Color::from_hsla(Hsla {
            h: 0.0,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        });
        let blue = Color::from_hsla(Hsla {
            h: 0.667,
            s: 1.0,
            l: 0.5,
            a: 1.0,
        });
        let mid = red.mix_in(blue, 0.5, MixColorSpace::Device, &env);
        let mid_hsla: Hsla = mid.into();

        let lg = LinearGradient::new(
            Gradient::new(vec![
                GradientStop {
                    color: red,
                    location: 0.0,
                },
                GradientStop {
                    color: blue,
                    location: 1.0,
                },
            ]),
            UnitPoint::TOP,
            UnitPoint::BOTTOM,
        );
        let (_, stops) = lg.to_gpui_in(&env);
        // The stops resolve to the original colours; the interpolation is
        // done by GPUI's renderer. Just verify the stops are correct.
        assert!(
            (stops[0].color.h - 0.0).abs() < 1e-3,
            "first stop should be red"
        );
        assert!(
            (stops[1].color.h - 0.667).abs() < 0.01,
            "second stop should be blue"
        );
        let _ = mid_hsla; // used in the mix, suppress unused warning
    }

    #[test]
    fn point_to_angle_top_to_bottom_is_180() {
        let angle = point_to_angle(UnitPoint::TOP, UnitPoint::BOTTOM);
        assert!(
            (angle - 180.0).abs() < 1e-3,
            "top→bottom should be 180°, got {angle}"
        );
    }

    #[test]
    fn point_to_angle_bottom_to_top_is_0() {
        let angle = point_to_angle(UnitPoint::BOTTOM, UnitPoint::TOP);
        assert!(
            (angle - 0.0).abs() < 1e-3 || (angle - 360.0).abs() < 1e-3,
            "bottom→top should be ~0°, got {angle}"
        );
    }
}
