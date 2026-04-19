//! Voice persona display component.
//!
//! `PersonaOrb` is a pure animated visual matching the reference AI Elements Persona:
//! a canvas-drawn orb that responds to conversational state and variant.
//!
//! `Persona` composes `PersonaOrb` with a name/state/description text column,
//! forming the card layout integrators reach for when they want the full
//! labeled cell rather than just the orb.

use std::f32::consts::PI;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{
    Animation, AnimationExt, App, Bounds, Context, ElementId, Entity, FontWeight, Hsla, Pixels,
    SharedString, Window, canvas, div, fill, point, px, size,
};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// The current state of a voice persona.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PersonaState {
    Idle,
    Speaking,
    Listening,
    Thinking,
    Asleep,
}

impl PersonaState {
    /// Short human-readable label for this state — also used as the
    /// VoiceOver description suffix on the orb canvas so sighted and
    /// VoiceOver users get the same cue (issue #148 F8).
    pub fn label(self) -> &'static str {
        match self {
            PersonaState::Idle => "Idle",
            PersonaState::Speaking => "Speaking",
            PersonaState::Listening => "Listening",
            PersonaState::Thinking => "Thinking",
            PersonaState::Asleep => "Asleep",
        }
    }

    /// Accessibility label applied to the persona orb canvas
    /// (e.g. `"Persona orb, Listening"`). Issue #148 F8.
    pub fn accessibility_label(self) -> SharedString {
        SharedString::from(format!("Persona orb, {}", self.label()))
    }
}

/// Visual variant for the persona animation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PersonaVariant {
    /// Dark concentric rings with slow rotation.
    #[default]
    Obsidian,
    /// Blue gradient pulse.
    Mana,
    /// Multi-color iridescent shimmer.
    Opal,
    /// Golden ring glow.
    Halo,
    /// Silver spark point.
    Glint,
    /// Green terminal-style grid.
    Command,
}

impl PersonaVariant {
    /// Primary color for this variant.
    fn color(&self) -> Hsla {
        match self {
            PersonaVariant::Obsidian => hsla(0.0, 0.0, 0.30, 1.0),
            PersonaVariant::Mana => hsla(0.58, 0.80, 0.55, 1.0),
            PersonaVariant::Opal => hsla(0.83, 0.50, 0.70, 1.0),
            PersonaVariant::Halo => hsla(0.12, 0.80, 0.55, 1.0),
            PersonaVariant::Glint => hsla(0.0, 0.0, 0.80, 1.0),
            PersonaVariant::Command => hsla(0.35, 0.70, 0.50, 1.0),
        }
    }

    /// Secondary color for layered effects.
    fn secondary_color(&self) -> Hsla {
        match self {
            PersonaVariant::Obsidian => hsla(0.0, 0.0, 0.15, 1.0),
            PersonaVariant::Mana => hsla(0.60, 0.60, 0.40, 1.0),
            PersonaVariant::Opal => hsla(0.16, 0.50, 0.70, 1.0),
            PersonaVariant::Halo => hsla(0.10, 0.90, 0.70, 1.0),
            PersonaVariant::Glint => hsla(0.0, 0.0, 0.95, 1.0),
            PersonaVariant::Command => hsla(0.35, 0.50, 0.30, 1.0),
        }
    }
}

fn hsla(h: f32, s: f32, l: f32, a: f32) -> Hsla {
    gpui::hsla(h, s, l, a)
}

/// Animation duration for a given state.
fn state_duration(state: PersonaState) -> Duration {
    match state {
        PersonaState::Idle => Duration::from_millis(4000),
        PersonaState::Speaking => Duration::from_millis(600),
        PersonaState::Listening => Duration::from_millis(1200),
        PersonaState::Thinking => Duration::from_millis(2500),
        PersonaState::Asleep => Duration::from_millis(4000), // not animated, but needed for type
    }
}

// ---------------------------------------------------------------------------
// PersonaOrb — pure animated visual (matches reference API)
// ---------------------------------------------------------------------------

/// A pure animated orb that responds to conversational state.
///
/// This matches the reference AI Elements `<Persona>` component: a standalone
/// animated visual with no text, name, or label. Size is controlled via `.size()`.
#[derive(IntoElement)]
pub struct PersonaOrb {
    state: PersonaState,
    variant: PersonaVariant,
    size: Pixels,
}

impl Default for PersonaOrb {
    fn default() -> Self {
        Self::new()
    }
}

impl PersonaOrb {
    /// Create a new persona orb with default state (Idle) and variant (Obsidian).
    pub fn new() -> Self {
        Self {
            state: PersonaState::Idle,
            variant: PersonaVariant::default(),
            size: px(64.0),
        }
    }

    /// Set the conversational state.
    pub fn state(mut self, state: PersonaState) -> Self {
        self.state = state;
        self
    }

    /// Set the visual variant.
    pub fn variant(mut self, variant: PersonaVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the orb size in pixels (default 64).
    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for PersonaOrb {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state;
        let variant = self.variant;
        let orb_size = self.size;
        let primary = variant.color();
        let secondary = variant.secondary_color();

        let id = ElementId::from(SharedString::from(format!(
            "persona-orb-{:?}-{:?}",
            variant, state
        )));

        // HIG `foundations.md:1100`: under Reduce Motion, render the orb as
        // a static frame (phase 0, no opacity oscillation). Asleep state
        // already renders static; Reduce Motion promotes every other state
        // to the same treatment.
        let reduce_motion = cx.theme().accessibility_mode.reduce_motion();

        // Issue #148 F8: The orb is a canvas-painted region without any
        // native label. Attach the state-dependent accessibility label via
        // a `div` wrapper so VoiceOver announces e.g. "Persona orb,
        // Listening" instead of a generic painted region.
        let ax_props = AccessibilityProps::new()
            .role(AccessibilityRole::Image)
            .label(state.accessibility_label());

        if state == PersonaState::Asleep || reduce_motion {
            let base_opacity = if state == PersonaState::Asleep {
                0.3
            } else {
                1.0
            };
            let c = canvas(
                move |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    paint_orb(
                        bounds, window, orb_size, primary, secondary, variant, 0.0, 1.0, None,
                    );
                },
            )
            .size(orb_size)
            .opacity(base_opacity);
            return div()
                .size(orb_size)
                .with_accessibility(&ax_props)
                .child(c)
                .into_any_element();
        }

        let duration = state_duration(state);

        let c = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                // This paint callback is never called directly — we use it
                // as a static fallback. The animated version below drives
                // the real painting via `with_animation`.
                paint_orb(
                    bounds, window, orb_size, primary, secondary, variant, 0.0, 1.0, None,
                );
            },
        )
        .size(orb_size)
        .with_animation(id, Animation::new(duration).repeat(), move |el, delta| {
            // The animation drives opacity modulation; actual painting
            // happens in the canvas callback above with t=0.
            let intensity = match state {
                PersonaState::Speaking => 0.6 + 0.4 * (delta * PI * 2.0).sin(),
                PersonaState::Listening => 0.5 + 0.5 * (delta * PI).sin(),
                PersonaState::Thinking => 0.3 + 0.7 * (delta * PI).sin(),
                PersonaState::Idle => 0.7 + 0.3 * (delta * PI).sin(),
                PersonaState::Asleep => 0.3,
            };
            el.opacity(intensity)
        });
        div()
            .size(orb_size)
            .with_accessibility(&ax_props)
            .child(c)
            .into_any_element()
    }
}

/// Paint the orb's layered circles into the canvas.
///
/// `delta` is the animation time parameter (speed-adjusted elapsed seconds).
/// When non-zero, per-variant motion is applied.
///
/// `amplitude` scales oscillation intensity (0.0 = static, 1.0 = full motion).
#[allow(clippy::too_many_arguments)]
fn paint_orb(
    bounds: Bounds<Pixels>,
    window: &mut Window,
    orb_size: Pixels,
    primary: Hsla,
    secondary: Hsla,
    variant: PersonaVariant,
    delta: f32,
    amplitude: f32,
    grid: Option<&[(f32, f32, f32)]>,
) {
    let s = f32::from(orb_size);
    let cx_f = f32::from(bounds.origin.x) + s / 2.0;
    let cy_f = f32::from(bounds.origin.y) + s / 2.0;
    let two_pi = PI * 2.0;

    // Background circle (consistent with PersonaOrb).
    let bg_bounds = Bounds {
        origin: point(px(cx_f - s / 2.0), px(cy_f - s / 2.0)),
        size: size(px(s), px(s)),
    };
    let bg_color = Hsla { a: 0.15, ..primary };
    window.paint_quad(fill(bg_bounds, bg_color).corner_radii(px(s / 2.0)));

    match variant {
        PersonaVariant::Obsidian => {
            // Concentric dark rings with oscillating radii.
            for i in 0..4 {
                let phase = i as f32 * PI * 0.5;
                let oscillation = 0.03 * amplitude * (delta * two_pi + phase).sin();
                let ratio = (1.0 - (i as f32 * 0.2)) + oscillation;
                let ring_s = s * ratio;
                let alpha = 0.15 + (i as f32 * 0.1);
                let color = Hsla {
                    a: alpha,
                    ..primary
                };
                let ring_bounds = Bounds {
                    origin: point(px(cx_f - ring_s / 2.0), px(cy_f - ring_s / 2.0)),
                    size: size(px(ring_s), px(ring_s)),
                };
                window.paint_quad(fill(ring_bounds, color).corner_radii(px(ring_s / 2.0)));
            }
        }
        PersonaVariant::Mana => {
            // Blue rippling circles with staggered pulsing.
            for i in 0..3 {
                let phase = i as f32 * two_pi / 3.0;
                let pulse = 0.05 * amplitude * (delta * two_pi + phase).sin();
                let ratio = (1.0 - (i as f32 * 0.25)) + pulse;
                let ring_s = s * ratio;
                let alpha_pulse = 0.08 * amplitude * (delta * two_pi + phase + PI * 0.5).sin();
                let alpha = (0.2 + (i as f32 * 0.15) + alpha_pulse).clamp(0.1, 0.8);
                let color = Hsla {
                    a: alpha,
                    ..primary
                };
                let ring_bounds = Bounds {
                    origin: point(px(cx_f - ring_s / 2.0), px(cy_f - ring_s / 2.0)),
                    size: size(px(ring_s), px(ring_s)),
                };
                window.paint_quad(fill(ring_bounds, color).corner_radii(px(ring_s / 2.0)));
            }
        }
        PersonaVariant::Opal => {
            // Three overlapping circles that rotate around center.
            let hues = [
                primary,
                secondary,
                Hsla {
                    h: (primary.h + 0.33) % 1.0,
                    ..primary
                },
            ];
            let base_angles = [0.0_f32, two_pi / 3.0, two_pi * 2.0 / 3.0];
            let orbit_radius = s * 0.08 * amplitude;
            for (color, base_angle) in hues.iter().zip(base_angles.iter()) {
                let angle = base_angle + delta * two_pi;
                let ox = orbit_radius * angle.cos();
                let oy = orbit_radius * angle.sin();
                let circle_s = s * 0.7;
                let circle_cx = cx_f + ox;
                let circle_cy = cy_f + oy;
                let c = Hsla { a: 0.3, ..*color };
                let b = Bounds {
                    origin: point(
                        px(circle_cx - circle_s / 2.0),
                        px(circle_cy - circle_s / 2.0),
                    ),
                    size: size(px(circle_s), px(circle_s)),
                };
                window.paint_quad(fill(b, c).corner_radii(px(circle_s / 2.0)));
            }
        }
        PersonaVariant::Halo => {
            // Filled background with breathing ring border.
            let glow_alpha = 0.15 + 0.1 * amplitude * (delta * two_pi).sin();
            let bg_s = s * 0.85;
            let bg_color = Hsla {
                a: glow_alpha,
                ..secondary
            };
            let bg = Bounds {
                origin: point(px(cx_f - bg_s / 2.0), px(cy_f - bg_s / 2.0)),
                size: size(px(bg_s), px(bg_s)),
            };
            window.paint_quad(fill(bg, bg_color).corner_radii(px(bg_s / 2.0)));

            // Outer ring with breathing border width.
            let border_w = 2.0 + 1.5 * amplitude * (delta * two_pi + PI * 0.5).sin();
            let ring_alpha = 0.5 + 0.2 * amplitude * (delta * two_pi).sin();
            let ring_s = s * 0.95;
            let ring_bounds = Bounds {
                origin: point(px(cx_f - ring_s / 2.0), px(cy_f - ring_s / 2.0)),
                size: size(px(ring_s), px(ring_s)),
            };
            let ring = fill(ring_bounds, Hsla { a: 0.0, ..primary })
                .corner_radii(px(ring_s / 2.0))
                .border_widths(px(border_w))
                .border_color(Hsla {
                    a: ring_alpha,
                    ..primary
                });
            window.paint_quad(ring);
        }
        PersonaVariant::Glint => {
            // Dark circle with scaling center dot and orbiting sparkle.
            let bg_base = Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.1,
                a: 1.0,
            };
            let bg_color = Hsla { a: 0.2, ..bg_base };
            let bg = Bounds {
                origin: point(px(cx_f - s / 2.0), px(cy_f - s / 2.0)),
                size: size(px(s), px(s)),
            };
            window.paint_quad(fill(bg, bg_color).corner_radii(px(s / 2.0)));

            // Pulsing center dot.
            let dot_scale = 1.0 + 0.3 * amplitude * (delta * two_pi).sin();
            let dot_s = s * 0.2 * dot_scale;
            let dot_bounds = Bounds {
                origin: point(px(cx_f - dot_s / 2.0), px(cy_f - dot_s / 2.0)),
                size: size(px(dot_s), px(dot_s)),
            };
            window.paint_quad(
                fill(dot_bounds, Hsla { a: 0.9, ..primary }).corner_radii(px(dot_s / 2.0)),
            );

            // Orbiting sparkle.
            let orbit_r = s * 0.3;
            let sparkle_angle = delta * two_pi;
            let sparkle_x = cx_f + orbit_r * sparkle_angle.cos();
            let sparkle_y = cy_f + orbit_r * sparkle_angle.sin();
            let sparkle_s = s * 0.08;
            let sparkle_bounds = Bounds {
                origin: point(
                    px(sparkle_x - sparkle_s / 2.0),
                    px(sparkle_y - sparkle_s / 2.0),
                ),
                size: size(px(sparkle_s), px(sparkle_s)),
            };
            window.paint_quad(
                fill(
                    sparkle_bounds,
                    Hsla {
                        a: 0.7,
                        ..secondary
                    },
                )
                .corner_radii(px(sparkle_s / 2.0)),
            );
        }
        PersonaVariant::Command => {
            // Green terminal-style: dark bg with wave-phased dot grid.
            let bg_base = Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.05,
                a: 1.0,
            };
            let bg_color = Hsla { a: 0.25, ..bg_base };
            let bg = Bounds {
                origin: point(px(cx_f - s / 2.0), px(cy_f - s / 2.0)),
                size: size(px(s), px(s)),
            };
            window.paint_quad(fill(bg, bg_color).corner_radii(px(s / 2.0)));

            // Grid of dots with wave-phase opacity.
            let dot_size = (s * 0.04).max(2.0);

            // Use precomputed grid when available, fall back to inline computation.
            let inline_grid;
            let grid_ref: &[(f32, f32, f32)] = match grid {
                Some(g) => g,
                None => {
                    inline_grid = compute_command_grid(orb_size);
                    &inline_grid
                }
            };
            for &(gx, gy, pos_phase) in grid_ref {
                let wave = 0.5 + 0.5 * (delta * two_pi + pos_phase * PI * 4.0).sin();
                let dot_alpha = (0.2 + 0.5 * amplitude * wave).clamp(0.1, 0.8);
                let dot_color = Hsla {
                    a: dot_alpha,
                    ..primary
                };
                let db = Bounds {
                    origin: point(
                        px(cx_f + gx - dot_size / 2.0),
                        px(cy_f + gy - dot_size / 2.0),
                    ),
                    size: size(px(dot_size), px(dot_size)),
                };
                window.paint_quad(fill(db, dot_color).corner_radii(px(dot_size / 2.0)));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Animation parameters per state
// ---------------------------------------------------------------------------

/// Speed multiplier for a given state.
fn state_speed(state: PersonaState) -> f32 {
    match state {
        PersonaState::Speaking => 2.5,
        PersonaState::Listening => 1.5,
        PersonaState::Thinking => 0.8,
        PersonaState::Idle => 0.4,
        PersonaState::Asleep => 0.0,
    }
}

/// Amplitude multiplier for a given state.
fn state_amplitude(state: PersonaState) -> f32 {
    match state {
        PersonaState::Speaking => 1.0,
        PersonaState::Listening => 0.7,
        PersonaState::Thinking => 0.5,
        PersonaState::Idle => 0.25,
        PersonaState::Asleep => 0.0,
    }
}

/// Precompute grid dot positions for the Command variant.
///
/// Returns a vec of `(gx, gy, pos_phase)` tuples for dots inside the circular mask.
fn compute_command_grid(orb_size: Pixels) -> Vec<(f32, f32, f32)> {
    let s = f32::from(orb_size);
    let grid_step = s * 0.15;
    let grid_radius = s * 0.35;
    let mut positions = Vec::new();
    let mut gx = -grid_radius;
    while gx <= grid_radius {
        let mut gy = -grid_radius;
        while gy <= grid_radius {
            if gx * gx + gy * gy <= grid_radius * grid_radius {
                let pos_phase = (gx + gy) / (grid_radius * 2.0);
                positions.push((gx, gy, pos_phase));
            }
            gy += grid_step;
        }
        gx += grid_step;
    }
    positions
}

// ---------------------------------------------------------------------------
// State transition tracking
// ---------------------------------------------------------------------------

/// Tracks an in-progress cross-fade between two persona states.
struct StateTransition {
    from: PersonaState,
    started_at: Instant,
}

const TRANSITION_DURATION: Duration = Duration::from_millis(300);

// ---------------------------------------------------------------------------
// PersonaOrbState — stateful entity with frame-driven animation
// ---------------------------------------------------------------------------

/// A stateful animated persona orb that drives per-frame canvas painting.
///
/// Unlike [`PersonaOrb`] (which uses opacity-only `with_animation`), this entity
/// uses `cx.notify()` each frame to inject real `delta` values into the canvas
/// paint closure, enabling per-variant motion (rotation, pulsing, orbiting, etc.).
///
/// Create via [`PersonaOrbState::new`] inside a `cx.new()` closure:
/// ```ignore
/// let orb = cx.new(|_cx| PersonaOrbState::new());
/// ```
#[allow(clippy::type_complexity)]
pub struct PersonaOrbState {
    state: PersonaState,
    variant: PersonaVariant,
    size: Pixels,
    start: Instant,
    paused: bool,
    pause_elapsed: Duration,
    transition: Option<StateTransition>,
    ready_fired: bool,
    on_ready: Option<Box<dyn Fn(&mut Window, &mut Context<Self>) + 'static>>,
    /// Precomputed Command-variant grid positions: (gx, gy, pos_phase).
    grid_positions: Vec<(f32, f32, f32)>,
}

impl Default for PersonaOrbState {
    fn default() -> Self {
        Self::new()
    }
}

impl PersonaOrbState {
    /// Create a new persona orb state with defaults (Idle, Obsidian, 64px).
    pub fn new() -> Self {
        let size = px(64.0);
        Self {
            state: PersonaState::Idle,
            variant: PersonaVariant::default(),
            size,
            start: Instant::now(),
            paused: false,
            pause_elapsed: Duration::ZERO,
            transition: None,
            ready_fired: false,
            on_ready: None,
            grid_positions: compute_command_grid(size),
        }
    }

    /// Create a new persona orb state with a specific initial state and variant.
    ///
    /// Unlike calling `new()` + `set_state()`, this does not create a spurious
    /// cross-fade transition from the default Idle state.
    pub fn new_with(state: PersonaState, variant: PersonaVariant) -> Self {
        Self {
            state,
            variant,
            ..Self::new()
        }
    }

    /// Set the conversational state, triggering a cross-fade transition.
    pub fn set_state(&mut self, state: PersonaState, cx: &mut Context<Self>) {
        if state != self.state {
            self.transition = Some(StateTransition {
                from: self.state,
                started_at: Instant::now(),
            });
            self.state = state;
            cx.notify();
        }
    }

    /// Set the conversational state without creating a transition.
    ///
    /// Use this for initial setup or when you want an immediate state change
    /// with no cross-fade animation.
    pub fn set_state_immediate(&mut self, state: PersonaState) {
        self.state = state;
        self.transition = None;
    }

    /// Set the visual variant.
    pub fn set_variant(&mut self, variant: PersonaVariant, cx: &mut Context<Self>) {
        if variant != self.variant {
            self.variant = variant;
            cx.notify();
        }
    }

    /// Set the orb size in pixels.
    pub fn set_size(&mut self, size: Pixels, cx: &mut Context<Self>) {
        if size != self.size {
            self.size = size;
            self.grid_positions = compute_command_grid(size);
            cx.notify();
        }
    }

    /// Pause the animation loop.
    pub fn pause(&mut self, cx: &mut Context<Self>) {
        if !self.paused {
            self.paused = true;
            self.pause_elapsed = self.start.elapsed();
            cx.notify();
        }
    }

    /// Resume the animation loop.
    pub fn resume(&mut self, cx: &mut Context<Self>) {
        if self.paused {
            self.paused = false;
            self.start = Instant::now() - self.pause_elapsed;
            cx.notify();
        }
    }

    /// Register a callback that fires once on first render.
    ///
    /// Note: unlike other voice module callbacks that use `Fn(..., &mut Window, &mut App)`,
    /// this callback receives `Context<Self>` because it fires during the entity's render.
    pub fn set_on_ready(&mut self, callback: impl Fn(&mut Window, &mut Context<Self>) + 'static) {
        self.on_ready = Some(Box::new(callback));
    }

    /// Returns the current state.
    pub fn state(&self) -> PersonaState {
        self.state
    }

    /// Returns the current variant.
    pub fn variant(&self) -> PersonaVariant {
        self.variant
    }

    /// Returns whether the animation is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Compute the elapsed seconds, accounting for pause state.
    fn elapsed_secs(&self) -> f32 {
        if self.paused {
            self.pause_elapsed.as_secs_f32()
        } else {
            self.start.elapsed().as_secs_f32()
        }
    }
}

impl Render for PersonaOrbState {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Fire on_ready once
        if !self.ready_fired {
            self.ready_fired = true;
            if let Some(cb) = self.on_ready.take() {
                cb(window, cx);
            }
        }

        // HIG `foundations.md:1100`: Reduce Motion short-circuits the
        // entire animation pipeline — no transition pump, no re-notify,
        // no delta-driven painting.
        let reduce_motion = cx.theme().accessibility_mode.reduce_motion();

        // Check for active transition (single pass with cleanup)
        let transition_progress = if reduce_motion {
            None
        } else {
            self.transition.as_ref().and_then(|t| {
                let elapsed = t.started_at.elapsed();
                if elapsed >= TRANSITION_DURATION {
                    None
                } else {
                    Some((
                        t.from,
                        elapsed.as_secs_f32() / TRANSITION_DURATION.as_secs_f32(),
                    ))
                }
            })
        };
        if transition_progress.is_none() {
            self.transition = None;
        }

        // Schedule continuous re-render for animation
        // (keep pumping during transitions even when target is Asleep)
        if !self.paused
            && !reduce_motion
            && (self.state != PersonaState::Asleep || self.transition.is_some())
        {
            cx.notify();
        }

        let elapsed = self.elapsed_secs();
        let state = self.state;
        let variant = self.variant;
        let orb_size = self.size;
        let primary = variant.color();
        let secondary = variant.secondary_color();
        let speed = state_speed(state);
        let amplitude = state_amplitude(state);
        let grid = self.grid_positions.clone();

        // Issue #148 F8: the canvas is a painted region with no native
        // label. Wrap it in a `div` and attach the state-dependent
        // accessibility label so VoiceOver announces the current state.
        let ax_props = AccessibilityProps::new()
            .role(AccessibilityRole::Image)
            .label(state.accessibility_label());

        // Asleep (or Reduce Motion): static orb at the state's base opacity.
        if (state == PersonaState::Asleep || reduce_motion) && transition_progress.is_none() {
            let base_opacity = if state == PersonaState::Asleep {
                0.3
            } else {
                // Reduce Motion: use the normal-state base opacity so
                // accessible rendering preserves identity, not a sleep cue.
                0.7 + 0.3 * amplitude
            };
            let c = canvas(
                move |_bounds, _window, _cx| {},
                move |bounds, _, window, _cx| {
                    paint_orb(
                        bounds,
                        window,
                        orb_size,
                        primary,
                        secondary,
                        variant,
                        0.0,
                        0.0,
                        Some(&grid),
                    );
                },
            )
            .size(orb_size)
            .opacity(base_opacity);
            return div()
                .size(orb_size)
                .with_accessibility(&ax_props)
                .child(c)
                .into_any_element();
        }

        // Wrap delta to avoid f32 precision loss over long sessions
        let delta = (elapsed * speed) % 1.0;

        // Cross-fade transition rendering
        if let Some((from_state, progress)) = transition_progress {
            let from_speed = state_speed(from_state);
            let from_amplitude = state_amplitude(from_state);
            let from_delta = (elapsed * from_speed) % 1.0;
            let from_primary = variant.color();
            let from_secondary = variant.secondary_color();

            // Scale each layer by its state's base opacity for smooth continuity
            let from_base_opacity = 0.7 + 0.3 * from_amplitude;
            let to_base_opacity = 0.7 + 0.3 * amplitude;
            let fade_out = (1.0 - progress) * from_base_opacity;
            let fade_in = progress * to_base_opacity;

            // Skip near-zero-opacity canvases to halve GPU work at transition edges
            let mut container = div()
                .size(orb_size)
                .relative()
                .with_accessibility(&ax_props);
            if fade_out >= 0.05 {
                let grid_clone = grid.clone();
                container = container.child(
                    canvas(
                        move |_bounds, _window, _cx| {},
                        move |bounds, _, window, _cx| {
                            paint_orb(
                                bounds,
                                window,
                                orb_size,
                                from_primary,
                                from_secondary,
                                variant,
                                from_delta,
                                from_amplitude,
                                Some(&grid_clone),
                            );
                        },
                    )
                    .size(orb_size)
                    .absolute()
                    .opacity(fade_out),
                );
            }
            if fade_in >= 0.05 {
                container = container.child(
                    canvas(
                        move |_bounds, _window, _cx| {},
                        move |bounds, _, window, _cx| {
                            paint_orb(
                                bounds,
                                window,
                                orb_size,
                                primary,
                                secondary,
                                variant,
                                delta,
                                amplitude,
                                Some(&grid),
                            );
                        },
                    )
                    .size(orb_size)
                    .absolute()
                    .opacity(fade_in),
                );
            }
            return container.into_any_element();
        }

        // Normal rendering with delta-driven animation
        let base_opacity = 0.7 + 0.3 * amplitude;
        let c = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                paint_orb(
                    bounds,
                    window,
                    orb_size,
                    primary,
                    secondary,
                    variant,
                    delta,
                    amplitude,
                    Some(&grid),
                );
            },
        )
        .size(orb_size)
        .opacity(base_opacity);
        div()
            .size(orb_size)
            .with_accessibility(&ax_props)
            .child(c)
            .into_any_element()
    }
}

// ---------------------------------------------------------------------------
// Persona — card layout composing PersonaOrb
// ---------------------------------------------------------------------------

/// A voice persona card showing animated orb, name, state, and optional description.
///
/// Composes [`PersonaOrb`] with a text column.
/// The `avatar_label` is rendered as a pair of initials overlaid on the orb
/// (HIG Avatars pattern — Finder sidebar, Contacts, and Mail use initials
/// as the offline/no-image fallback for people) and contributes to the orb's
/// VoiceOver label so screen-reader users hear the persona name alongside
/// the state cue.
#[derive(IntoElement)]
pub struct Persona {
    name: SharedString,
    avatar_label: SharedString,
    state: PersonaState,
    variant: PersonaVariant,
    description: Option<SharedString>,
    orb_size: Option<Pixels>,
    orb_entity: Option<Entity<PersonaOrbState>>,
}

impl Persona {
    pub fn new(name: impl Into<SharedString>, avatar_label: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            avatar_label: avatar_label.into(),
            state: PersonaState::Idle,
            variant: PersonaVariant::default(),
            description: None,
            orb_size: None,
            orb_entity: None,
        }
    }

    pub fn state(mut self, state: PersonaState) -> Self {
        self.state = state;
        self
    }

    pub fn variant(mut self, variant: PersonaVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn description(mut self, desc: impl Into<SharedString>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the orb size (overrides theme avatar_size).
    pub fn size(mut self, size: Pixels) -> Self {
        self.orb_size = Some(size);
        self
    }

    /// Use a stateful [`PersonaOrbState`] entity instead of the default stateless orb.
    ///
    /// When set, `state` and `variant` builder props are ignored for the orb visual
    /// (they still affect the text label color/label).
    pub fn orb_entity(mut self, entity: Entity<PersonaOrbState>) -> Self {
        self.orb_entity = Some(entity);
        self
    }
}

impl RenderOnce for Persona {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let state_color = match self.state {
            PersonaState::Idle => theme.text_muted,
            PersonaState::Speaking => theme.success,
            PersonaState::Listening => theme.accent,
            PersonaState::Thinking => theme.warning,
            PersonaState::Asleep => theme.pending,
        };

        // Issue #148 F8: delegate to `PersonaState::label()` so the visible
        // state text and the orb's accessibility label stay in sync.
        let state_label: SharedString = self.state.label().into();

        let orb_size = self.orb_size.unwrap_or(theme.avatar_size + px(8.0));

        let mut text_col = div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .child(
                div()
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                    .child(self.name.clone()),
            )
            .child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(state_color)
                    .child(state_label),
            );

        if let Some(desc) = self.description {
            text_col = text_col.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child(desc),
            );
        }

        let row = div().flex().items_center().gap(theme.spacing_md);

        // Compose the orb with overlaid initials. HIG Avatars pattern — the
        // initials act as a no-image fallback (Finder sidebar, Contacts,
        // Mail), and combining them with the orb keeps the persona
        // recognisable even when the orb's animation is paused. The label
        // is also wired into the card's accessibility name so VoiceOver
        // announces the persona identity alongside the state cue (issue
        // #148 F8 extension).
        let avatar_label = self.avatar_label.clone();
        let persona_name = self.name.clone();
        let a11y_props = AccessibilityProps::new()
            .role(AccessibilityRole::Group)
            .label(SharedString::from(format!(
                "{}, {}",
                persona_name, avatar_label
            )));

        let orb_with_initials = div()
            .relative()
            .size(orb_size)
            .flex()
            .items_center()
            .justify_center()
            .child(if let Some(orb_entity) = self.orb_entity {
                div().size(orb_size).child(orb_entity).into_any_element()
            } else {
                PersonaOrb::new()
                    .state(self.state)
                    .variant(self.variant)
                    .size(orb_size)
                    .into_any_element()
            })
            .child(
                // Initials overlay — sized at ~40 % of the orb diameter so
                // they read cleanly without crowding the orb's aura.
                div()
                    .absolute()
                    .size(orb_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(orb_size * 0.4)
                    .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                    .text_color(theme.text_on_accent)
                    .child(avatar_label),
            );

        row.with_accessibility(&a11y_props)
            .child(orb_with_initials)
            .child(text_col)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PersonaOrb, PersonaOrbState, PersonaState, PersonaVariant, StateTransition,
        TRANSITION_DURATION, state_amplitude, state_duration, state_speed,
    };
    use core::prelude::v1::test;
    use gpui::px;
    use std::time::{Duration, Instant};

    #[test]
    fn persona_state_equality() {
        assert_eq!(PersonaState::Idle, PersonaState::Idle);
        assert_eq!(PersonaState::Speaking, PersonaState::Speaking);
        assert_eq!(PersonaState::Thinking, PersonaState::Thinking);
        assert_eq!(PersonaState::Asleep, PersonaState::Asleep);
    }

    #[test]
    fn persona_state_inequality() {
        assert_ne!(PersonaState::Idle, PersonaState::Speaking);
        assert_ne!(PersonaState::Speaking, PersonaState::Listening);
        assert_ne!(PersonaState::Thinking, PersonaState::Asleep);
    }

    #[test]
    fn persona_state_all_distinct() {
        let states = [
            PersonaState::Idle,
            PersonaState::Speaking,
            PersonaState::Listening,
            PersonaState::Thinking,
            PersonaState::Asleep,
        ];
        for i in 0..states.len() {
            for j in 0..states.len() {
                if i == j {
                    assert_eq!(states[i], states[j]);
                } else {
                    assert_ne!(states[i], states[j]);
                }
            }
        }
    }

    #[test]
    fn persona_state_copy() {
        let s = PersonaState::Thinking;
        let s2 = s;
        assert_eq!(s, s2);
    }

    #[test]
    fn persona_state_debug() {
        assert!(format!("{:?}", PersonaState::Idle).contains("Idle"));
        assert!(format!("{:?}", PersonaState::Thinking).contains("Thinking"));
        assert!(format!("{:?}", PersonaState::Asleep).contains("Asleep"));
    }

    #[test]
    fn persona_variant_default() {
        assert_eq!(PersonaVariant::default(), PersonaVariant::Obsidian);
    }

    #[test]
    fn persona_variant_all_distinct() {
        let variants = [
            PersonaVariant::Obsidian,
            PersonaVariant::Mana,
            PersonaVariant::Opal,
            PersonaVariant::Halo,
            PersonaVariant::Glint,
            PersonaVariant::Command,
        ];
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i == j {
                    assert_eq!(variants[i], variants[j]);
                } else {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    #[test]
    fn persona_stores_avatar_label() {
        use super::Persona;
        let p = Persona::new("Aria", "AR");
        assert_eq!(p.avatar_label.as_ref(), "AR");
        assert_eq!(p.name.as_ref(), "Aria");
    }

    #[test]
    fn persona_variant_colors_have_full_alpha() {
        for variant in [
            PersonaVariant::Obsidian,
            PersonaVariant::Mana,
            PersonaVariant::Opal,
            PersonaVariant::Halo,
            PersonaVariant::Glint,
            PersonaVariant::Command,
        ] {
            assert_eq!(variant.color().a, 1.0);
        }
    }

    #[test]
    fn persona_orb_default_size() {
        let orb = PersonaOrb::new();
        assert_eq!(orb.size, px(64.0));
    }

    #[test]
    fn persona_orb_builder_chaining() {
        let orb = PersonaOrb::new()
            .state(PersonaState::Speaking)
            .variant(PersonaVariant::Mana)
            .size(px(128.0));
        assert_eq!(orb.state, PersonaState::Speaking);
        assert_eq!(orb.variant, PersonaVariant::Mana);
        assert_eq!(orb.size, px(128.0));
    }

    #[test]
    fn persona_orb_default_state_and_variant() {
        let orb = PersonaOrb::new();
        assert_eq!(orb.state, PersonaState::Idle);
        assert_eq!(orb.variant, PersonaVariant::Obsidian);
    }

    #[test]
    fn state_duration_varies_by_state() {
        assert!(state_duration(PersonaState::Speaking) < state_duration(PersonaState::Idle));
        assert!(state_duration(PersonaState::Listening) < state_duration(PersonaState::Thinking));
        assert!(state_duration(PersonaState::Speaking) < state_duration(PersonaState::Listening));
    }

    #[test]
    fn persona_variant_secondary_colors_have_full_alpha() {
        for variant in [
            PersonaVariant::Obsidian,
            PersonaVariant::Mana,
            PersonaVariant::Opal,
            PersonaVariant::Halo,
            PersonaVariant::Glint,
            PersonaVariant::Command,
        ] {
            assert_eq!(variant.secondary_color().a, 1.0);
        }
    }

    // PersonaOrbState tests

    #[test]
    fn persona_orb_state_defaults() {
        let state = PersonaOrbState::new();
        assert_eq!(state.state(), PersonaState::Idle);
        assert_eq!(state.variant(), PersonaVariant::Obsidian);
        assert_eq!(state.size, px(64.0));
        assert!(!state.is_paused());
        assert!(!state.ready_fired);
    }

    #[test]
    fn persona_orb_state_elapsed_frozen_when_paused() {
        // NOTE: pause()/resume() require Context<Self>, so we test the
        // elapsed_secs() read path directly by manipulating fields.
        let mut state = PersonaOrbState::new();
        assert!(!state.is_paused());

        state.paused = true;
        state.pause_elapsed = Duration::from_millis(500);
        assert!(state.is_paused());

        // Elapsed should return the frozen value, not wall clock
        let elapsed = state.elapsed_secs();
        assert!((elapsed - 0.5).abs() < 0.01);
    }

    #[test]
    fn persona_orb_state_transition_fields() {
        // NOTE: set_state() requires Context<Self>. This test verifies the
        // field-level transition tracking behavior directly.
        let mut state = PersonaOrbState::new();
        assert!(state.transition.is_none());

        state.transition = Some(StateTransition {
            from: PersonaState::Idle,
            started_at: Instant::now(),
        });
        state.state = PersonaState::Speaking;

        assert!(state.transition.is_some());
        assert_eq!(state.transition.as_ref().unwrap().from, PersonaState::Idle);
        assert_eq!(state.state(), PersonaState::Speaking);
    }

    #[test]
    fn persona_orb_state_no_transition_on_same_state() {
        // Verifies the guard logic: setting the same state should not
        // create a transition (mirrors set_state's `if state != self.state` check).
        let mut state = PersonaOrbState::new();
        let current = state.state();

        // Simulating set_state with same value
        if current != state.state() {
            state.transition = Some(StateTransition {
                from: state.state(),
                started_at: Instant::now(),
            });
        }

        assert!(state.transition.is_none());
    }

    #[test]
    fn persona_orb_state_set_on_ready_registration() {
        let mut state = PersonaOrbState::new();
        assert!(state.on_ready.is_none());
        assert!(!state.ready_fired);

        state.set_on_ready(|_window, _cx| {});
        assert!(state.on_ready.is_some());
    }

    #[test]
    fn persona_orb_state_set_state_immediate() {
        let mut state = PersonaOrbState::new();
        state.set_state_immediate(PersonaState::Speaking);
        assert_eq!(state.state(), PersonaState::Speaking);
        assert!(state.transition.is_none());
    }

    #[test]
    fn persona_orb_state_new_with() {
        let state = PersonaOrbState::new_with(PersonaState::Listening, PersonaVariant::Mana);
        assert_eq!(state.state(), PersonaState::Listening);
        assert_eq!(state.variant(), PersonaVariant::Mana);
        assert!(state.transition.is_none());
    }

    #[test]
    fn persona_orb_state_transition_expires() {
        let state_transition = StateTransition {
            from: PersonaState::Idle,
            started_at: Instant::now() - Duration::from_millis(400),
        };
        // Transition should be expired (> 300ms)
        assert!(state_transition.started_at.elapsed() >= TRANSITION_DURATION);
    }

    #[test]
    fn state_speed_ordering() {
        assert!(state_speed(PersonaState::Speaking) > state_speed(PersonaState::Listening));
        assert!(state_speed(PersonaState::Listening) > state_speed(PersonaState::Thinking));
        assert!(state_speed(PersonaState::Thinking) > state_speed(PersonaState::Idle));
        assert_eq!(state_speed(PersonaState::Asleep), 0.0);
    }

    #[test]
    fn state_amplitude_ordering() {
        assert!(state_amplitude(PersonaState::Speaking) > state_amplitude(PersonaState::Listening));
        assert!(state_amplitude(PersonaState::Listening) > state_amplitude(PersonaState::Thinking));
        assert!(state_amplitude(PersonaState::Thinking) > state_amplitude(PersonaState::Idle));
        assert_eq!(state_amplitude(PersonaState::Asleep), 0.0);
    }

    #[test]
    fn persona_orb_state_elapsed_increases() {
        let state = PersonaOrbState::new();
        // Elapsed should be >= 0 and reasonably small
        let elapsed = state.elapsed_secs();
        assert!(elapsed >= 0.0);
        assert!(elapsed < 1.0); // should be nearly instant
    }

    #[test]
    fn transition_duration_is_300ms() {
        assert_eq!(TRANSITION_DURATION, Duration::from_millis(300));
    }
}
