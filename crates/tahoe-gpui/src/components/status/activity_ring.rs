//! Activity Ring component (HIG circular progress ring).
//!
//! Renders a circular progress arc using GPUI's canvas API, inspired by
//! Apple's Activity Rings. The `ActivityRingSet` composite stacks the three
//! canonical Apple Fitness rings (Move / Exercise / Stand) on a black
//! background per HIG.
//!
//! # HIG reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/activity-rings>

use std::f32::consts::PI;

use gpui::prelude::*;
use gpui::{App, Bounds, Hsla, Pixels, SharedString, Window, canvas, fill, hsla, point, px, size};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::ActiveTheme;

/// Default ring diameter in pixels.
const DEFAULT_SIZE: f32 = 44.0;

/// Default stroke width in pixels.
const DEFAULT_STROKE: f32 = 6.0;

/// Number of segments used to approximate the progress arc.
const ARC_SEGMENTS: usize = 64;

/// Canonical *Move* ring color from Apple Health / Fitness — RGB 250/17/79.
///
/// HIG: "Never change the colors of the rings."
pub const ACTIVITY_RING_MOVE: Hsla = Hsla {
    // H derived from RGB(250, 17, 79) — red with slight pink cast.
    h: 349.0 / 360.0,
    s: 0.957,
    l: 0.524,
    a: 1.0,
};

/// Canonical *Exercise* ring color from Apple Health / Fitness — RGB 166/255/0.
pub const ACTIVITY_RING_EXERCISE: Hsla = Hsla {
    // H derived from RGB(166, 255, 0) — yellow-green.
    h: 79.0 / 360.0,
    s: 1.0,
    l: 0.5,
    a: 1.0,
};

/// Canonical *Stand* ring color from Apple Health / Fitness — RGB 0/255/246.
pub const ACTIVITY_RING_STAND: Hsla = Hsla {
    // H derived from RGB(0, 255, 246) — cyan.
    h: 178.0 / 360.0,
    s: 1.0,
    l: 0.5,
    a: 1.0,
};

/// A circular progress ring following the Apple Fitness Activity Ring pattern.
#[derive(IntoElement)]
pub struct ActivityRing {
    /// Progress value. 0.0 = empty, 1.0 = full circle. Values > 1.0 render
    /// a wrap-around arc at reduced opacity.
    value: f32,
    color: Option<Hsla>,
    track_color: Option<Hsla>,
    size: Option<Pixels>,
    stroke_width: Option<f32>,
    label: Option<SharedString>,
}

impl ActivityRing {
    /// Create a new activity ring with the given progress value. Values
    /// greater than `1.0` are preserved and render a wrap-around arc.
    pub fn new(value: f32) -> Self {
        Self {
            value,
            color: None,
            track_color: None,
            size: None,
            stroke_width: None,
            label: None,
        }
    }

    /// Set the fill color. Defaults to `theme.accent`.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Set the track (background ring) color. Defaults to a dim version of
    /// the fill color, or — under `REDUCE_TRANSPARENCY` — an opaque dark
    /// ring so the shape is preserved on the forced black background.
    pub fn track_color(mut self, color: Hsla) -> Self {
        self.track_color = Some(color);
        self
    }

    /// Set the overall ring diameter. Defaults to 44px.
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = Some(size);
        self
    }

    /// Set the stroke width. Defaults to 6px.
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = Some(width);
        self
    }

    /// Override the VoiceOver label (default: `"Activity ring"`).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl RenderOnce for ActivityRing {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let display_size = self.size.unwrap_or(px(DEFAULT_SIZE));
        let stroke = self.stroke_width.unwrap_or(DEFAULT_STROKE);
        let fill_color = self.color.unwrap_or(theme.accent);
        let reduce_transparency = theme.accessibility_mode.reduce_transparency();
        let track_color = self.track_color.unwrap_or_else(|| {
            if reduce_transparency {
                // Opaque dark track so the ring shape survives on the
                // HIG-mandated black background when transparency is reduced.
                hsla(0.0, 0.0, 0.15, 1.0)
            } else {
                Hsla {
                    a: 0.20,
                    ..fill_color
                }
            }
        });

        let raw = if self.value.is_finite() {
            self.value
        } else {
            0.0
        };
        let base_progress = raw.clamp(0.0, 1.0);
        let wrap_progress = if raw > 1.0 { (raw - 1.0).min(1.0) } else { 0.0 };

        let percent = (raw * 100.0).round() as i32;
        let a11y_label: SharedString = self
            .label
            .clone()
            .unwrap_or_else(|| SharedString::from("Activity ring"));
        let a11y_props = AccessibilityProps::new()
            .label(a11y_label)
            .role(AccessibilityRole::ProgressIndicator)
            .value(SharedString::from(format!("{percent} percent")));

        let canvas_el = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let s = f32::from(display_size);
                let cx_f = f32::from(bounds.origin.x) + s / 2.0;
                let cy_f = f32::from(bounds.origin.y) + s / 2.0;
                let radius = (s - stroke) / 2.0;

                // Background track: full circle border.
                let track_bounds = Bounds {
                    origin: point(
                        px(cx_f - radius - stroke / 2.0),
                        px(cy_f - radius - stroke / 2.0),
                    ),
                    size: size(px(radius * 2.0 + stroke), px(radius * 2.0 + stroke)),
                };
                window.paint_quad(
                    fill(
                        track_bounds,
                        Hsla {
                            a: 0.0,
                            ..fill_color
                        },
                    )
                    .corner_radii(px(s / 2.0))
                    .border_widths(px(stroke))
                    .border_color(track_color),
                );

                paint_arc(
                    window,
                    cx_f,
                    cy_f,
                    radius,
                    stroke,
                    base_progress,
                    fill_color,
                );

                // HIG v2: when the value exceeds 100%, the ring wraps around
                // past full circle with a second overlapping arc in a
                // lighter tint. HIG leaves the exact tint unspecified; we
                // use 60% alpha to read as clearly secondary.
                if wrap_progress > 0.0 {
                    let wrap_color = Hsla {
                        a: 0.60,
                        ..fill_color
                    };
                    paint_arc(
                        window,
                        cx_f,
                        cy_f,
                        radius,
                        stroke,
                        wrap_progress,
                        wrap_color,
                    );
                }
            },
        )
        .size(display_size);

        gpui::div().with_accessibility(&a11y_props).child(canvas_el)
    }
}

fn paint_arc(
    window: &mut Window,
    cx_f: f32,
    cy_f: f32,
    radius: f32,
    stroke: f32,
    proportion: f32,
    color: Hsla,
) {
    if proportion <= 0.0 {
        return;
    }
    let total_angle = proportion * 2.0 * PI;
    let start_angle = -PI / 2.0; // top of circle (12 o'clock)
    let dot_size = stroke;
    let segments = ((ARC_SEGMENTS as f32 * proportion).ceil() as usize).max(1);

    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let angle = start_angle + t * total_angle;
        let qx = cx_f + radius * angle.cos();
        let qy = cy_f + radius * angle.sin();
        let dot_bounds = Bounds {
            origin: point(px(qx - dot_size / 2.0), px(qy - dot_size / 2.0)),
            size: size(px(dot_size), px(dot_size)),
        };
        window.paint_quad(fill(dot_bounds, color).corner_radii(px(dot_size / 2.0)));
    }

    let end_angle = start_angle + total_angle;
    let end_x = cx_f + radius * end_angle.cos();
    let end_y = cy_f + radius * end_angle.sin();
    let cap_bounds = Bounds {
        origin: point(px(end_x - dot_size / 2.0), px(end_y - dot_size / 2.0)),
        size: size(px(dot_size), px(dot_size)),
    };
    window.paint_quad(fill(cap_bounds, color).corner_radii(px(dot_size / 2.0)));
}

/// Three-ring composite matching Apple Fitness. HIG: "In watchOS, the
/// Activity ring element always contains three rings, whose colors and
/// meanings match those the Activity app provides" on a black background.
#[derive(IntoElement)]
pub struct ActivityRingSet {
    move_progress: f32,
    exercise_progress: f32,
    stand_progress: f32,
    size: Pixels,
    stroke_width: f32,
}

impl ActivityRingSet {
    /// Build a three-ring composite with the canonical HIG colors.
    pub fn fitness(move_progress: f32, exercise_progress: f32, stand_progress: f32) -> Self {
        Self {
            move_progress,
            exercise_progress,
            stand_progress,
            size: px(DEFAULT_SIZE * 2.0),
            stroke_width: DEFAULT_STROKE,
        }
    }

    /// Override outer diameter (default 88pt).
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    /// Override stroke width (default 6pt).
    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_width = w;
        self
    }
}

impl RenderOnce for ActivityRingSet {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let stroke = self.stroke_width;
        let gap = stroke / 2.0;
        let outer = self.size;
        let middle = px(f32::from(outer) - 2.0 * (stroke + gap));
        let inner = px(f32::from(outer) - 4.0 * (stroke + gap));

        let reduce_transparency = theme.accessibility_mode.reduce_transparency();
        let track_color = if reduce_transparency {
            hsla(0.0, 0.0, 0.15, 1.0)
        } else {
            hsla(0.0, 0.0, 0.25, 0.35)
        };

        let overall_percent =
            ((self.move_progress + self.exercise_progress + self.stand_progress) / 3.0 * 100.0)
                .round() as i32;
        let a11y_props = AccessibilityProps::new()
            .label(SharedString::from("Activity rings"))
            .role(AccessibilityRole::ProgressIndicator)
            .value(SharedString::from(format!(
                "{overall_percent} percent average"
            )));

        // HIG: "Always display Activity rings on a black background." An
        // opaque black panel with an outer margin equal to the stroke
        // enforces the rule at the component level.
        gpui::div()
            .relative()
            .bg(gpui::black())
            .rounded(px(f32::from(outer) / 2.0 + stroke))
            .p(px(stroke + gap))
            .with_accessibility(&a11y_props)
            .child(
                gpui::div()
                    .relative()
                    .size(outer)
                    .child(
                        gpui::div().absolute().top_0().left_0().size(outer).child(
                            ActivityRing::new(self.move_progress)
                                .color(ACTIVITY_RING_MOVE)
                                .track_color(track_color)
                                .size(outer)
                                .stroke_width(stroke)
                                .label("Move"),
                        ),
                    )
                    .child(
                        gpui::div()
                            .absolute()
                            .top(px(stroke + gap))
                            .left(px(stroke + gap))
                            .size(middle)
                            .child(
                                ActivityRing::new(self.exercise_progress)
                                    .color(ACTIVITY_RING_EXERCISE)
                                    .track_color(track_color)
                                    .size(middle)
                                    .stroke_width(stroke)
                                    .label("Exercise"),
                            ),
                    )
                    .child(
                        gpui::div()
                            .absolute()
                            .top(px(2.0 * (stroke + gap)))
                            .left(px(2.0 * (stroke + gap)))
                            .size(inner)
                            .child(
                                ActivityRing::new(self.stand_progress)
                                    .color(ACTIVITY_RING_STAND)
                                    .track_color(track_color)
                                    .size(inner)
                                    .stroke_width(stroke)
                                    .label("Stand"),
                            ),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::components::status::activity_ring::{
        ACTIVITY_RING_EXERCISE, ACTIVITY_RING_MOVE, ACTIVITY_RING_STAND, ActivityRing,
        ActivityRingSet,
    };
    use core::prelude::v1::test;
    use gpui::px;

    #[test]
    fn activity_ring_new_stores_value() {
        let ring = ActivityRing::new(0.5);
        assert!((ring.value - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn activity_ring_defaults_are_none() {
        let ring = ActivityRing::new(0.0);
        assert!(ring.color.is_none());
        assert!(ring.track_color.is_none());
        assert!(ring.size.is_none());
        assert!(ring.stroke_width.is_none());
        assert!(ring.label.is_none());
    }

    #[test]
    fn activity_ring_builder_color() {
        let color = gpui::hsla(0.3, 0.8, 0.5, 1.0);
        let ring = ActivityRing::new(0.5).color(color);
        assert_eq!(ring.color, Some(color));
    }

    #[test]
    fn activity_ring_builder_size() {
        let ring = ActivityRing::new(0.5).size(px(60.0));
        assert_eq!(ring.size, Some(px(60.0)));
    }

    #[test]
    fn activity_ring_value_exceeding_one_is_preserved() {
        let ring = ActivityRing::new(1.5);
        assert!((ring.value - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn activity_ring_label_builder() {
        let ring = ActivityRing::new(0.3).label("Move");
        assert_eq!(ring.label.as_ref().map(|s| s.as_ref()), Some("Move"));
    }

    #[test]
    fn canonical_fitness_colors_are_opaque() {
        assert!((ACTIVITY_RING_MOVE.a - 1.0).abs() < f32::EPSILON);
        assert!((ACTIVITY_RING_EXERCISE.a - 1.0).abs() < f32::EPSILON);
        assert!((ACTIVITY_RING_STAND.a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn canonical_fitness_colors_are_distinct() {
        assert!((ACTIVITY_RING_MOVE.h - ACTIVITY_RING_EXERCISE.h).abs() > 0.1);
        assert!((ACTIVITY_RING_EXERCISE.h - ACTIVITY_RING_STAND.h).abs() > 0.1);
        assert!((ACTIVITY_RING_STAND.h - ACTIVITY_RING_MOVE.h).abs() > 0.1);
    }

    #[test]
    fn activity_ring_set_fitness_stores_progress() {
        let set = ActivityRingSet::fitness(0.5, 0.25, 0.75);
        assert!((set.move_progress - 0.5).abs() < f32::EPSILON);
        assert!((set.exercise_progress - 0.25).abs() < f32::EPSILON);
        assert!((set.stand_progress - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn activity_ring_set_size_builder() {
        let set = ActivityRingSet::fitness(0.5, 0.5, 0.5).size(px(120.0));
        assert_eq!(set.size, px(120.0));
    }
}
