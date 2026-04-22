//! Motion and animation utilities aligned with HIG.
//!
//! Provides animation tokens, shimmer effects, spring animations,
//! and morph state interpolation for glass transitions.

use gpui::Animation;
use std::time::Duration;

// Note: `Shimmer`/`ShimmerEasing`/`SweepDirection` live in
// `components::status::shimmer` and are re-exported from the crate prelude.
// Foundations must not import from `components::` — doing so inverts the
// layering. Consumers should use `crate::components::status::{...}` directly
// or the crate prelude.

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Duration ramps
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Semantic duration ramps covering the HIG 250–500ms system motion range.
///
/// Maps to the same ladder as SwiftUI's `.standard` / `.emphasized` timing
/// family and the short/medium/long labels in the HIG Motion section
/// (`foundations.md:1083–1138`). Consumers should prefer a ramp over a raw
/// millisecond constant so that system-wide retuning stays in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionRamp {
    /// ~200ms — brief acknowledgements (flex, press/release).
    Short,
    /// ~300ms — lifts, in-place transitions (default).
    Medium,
    /// ~450ms — shape shifts, sheet presentation, large geometry changes.
    Long,
}

impl MotionRamp {
    /// Millisecond value for this ramp. Values target the HIG 250–500ms
    /// system duration range with a small below-range "short" step for
    /// micro-interactions.
    pub const fn duration_ms(self) -> u64 {
        match self {
            MotionRamp::Short => 200,
            MotionRamp::Medium => 300,
            MotionRamp::Long => 450,
        }
    }

    /// Convert directly to a [`Duration`].
    pub fn duration(self) -> Duration {
        Duration::from_millis(self.duration_ms())
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Spring presets (SwiftUI parity)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Named spring presets matching SwiftUI's documented response/damping/bounce
/// values. The underlying easing here is still the exponential approximation
/// in [`spring_easing`] — for tactile parity with SwiftUI's physics solver,
/// GPUI needs a velocity-preserving spring primitive (tracked as open
/// question #1 on the internal tracker).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpringPreset {
    /// SwiftUI `.spring` default — `response=0.55, damping=0.825, bounce=0.0`.
    /// This is the implicit spring used by every unqualified `.animation(.spring)`
    /// call in SwiftUI and the closest match to the "general purpose" system
    /// spring on macOS Tahoe.
    Default,
    /// SwiftUI `.smooth` — no bounce, slow settle. `response=0.5, damping=1.0`.
    Smooth,
    /// SwiftUI `.snappy` — fast, no bounce. `response=0.3, damping=0.9`.
    Snappy,
    /// SwiftUI `.bouncy` — visible overshoot. `response=0.5, damping=0.7, bounce=0.15`.
    Bouncy,
    /// `.interactive` — Liquid Glass default, responsive without overshoot.
    /// `response=0.35, damping=0.85`. Matches the values in
    /// `theme::build_glass()::MOTION`.
    Interactive,
}

impl SpringPreset {
    /// `(damping, response_seconds, bounce)` tuple for this preset.
    pub const fn params(self) -> (f32, f32, f32) {
        match self {
            SpringPreset::Default => (0.825, 0.55, 0.0),
            SpringPreset::Smooth => (1.0, 0.5, 0.0),
            SpringPreset::Snappy => (0.9, 0.3, 0.0),
            SpringPreset::Bouncy => (0.7, 0.5, 0.15),
            SpringPreset::Interactive => (0.85, 0.35, 0.0),
        }
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MotionTokens
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Motion tokens for Liquid Glass per HIG.
///
/// The three `*_duration_ms` fields correspond to the [`MotionRamp`] ladder
/// (short / medium / long) but carry Glass-specific names for historical
/// reasons. Prefer reading via [`MotionTokens::duration_for`] so theme
/// overrides propagate.
///
/// Cheap to copy: three `u64` duration fields plus three `f32` spring
/// parameters. `Copy` lets callers store a token snapshot in a closure
/// without a visible `.clone()` at every capture site.
#[derive(Debug, Clone, Copy)]
pub struct MotionTokens {
    /// Flex response press/release duration in ms (semantic: short ramp).
    ///
    /// GPUI limitation: `.hover()` style API has no transition duration,
    /// so this token is consumed only by explicit `AnimationElement`
    /// wrappers (not by `interactive_hover`). Tracked as open question #3
    /// on the internal tracker.
    pub flex_duration_ms: u64,
    /// Lift-on-hover duration in ms (semantic: short-to-medium ramp).
    pub lift_duration_ms: u64,
    /// Shape-shift transition duration in ms (semantic: long ramp).
    pub shape_shift_duration_ms: u64,
    /// Spring animation damping ratio (0.0-1.0). 0.85 = slightly underdamped.
    pub spring_damping: f32,
    /// Spring animation response time in seconds. 0.35 = standard Apple feel.
    pub spring_response: f32,
    /// Spring bounce coefficient. 0.0 = no bounce, higher = more overshoot.
    pub spring_bounce: f32,
}

impl MotionTokens {
    /// Approximate settling time for the spring in milliseconds, capped at
    /// [`MotionRamp::Long`] (450 ms) to stay within the HIG 250-500 ms system
    /// animation range.
    pub fn spring_duration_ms(&self) -> u64 {
        if self.spring_response <= 0.0 {
            return 0;
        }
        const HIG_MAX_SPRING_MS: u64 = MotionRamp::Long.duration_ms();
        let raw = ((4.0 * self.spring_response) * 1000.0) as u64;
        raw.min(HIG_MAX_SPRING_MS)
    }

    /// Resolve the token duration for a semantic [`MotionRamp`].
    ///
    /// Reads from the existing glass duration fields (`flex` / `lift` /
    /// `shape_shift`) so themed overrides in `theme::build_glass` propagate.
    pub fn duration_for(&self, ramp: MotionRamp) -> Duration {
        let ms = match ramp {
            MotionRamp::Short => self.flex_duration_ms,
            MotionRamp::Medium => self.lift_duration_ms,
            MotionRamp::Long => self.shape_shift_duration_ms,
        };
        Duration::from_millis(ms)
    }

    /// Build a `MotionTokens` with the given spring preset applied to the
    /// spring fields, keeping existing duration fields.
    pub fn with_spring_preset(mut self, preset: SpringPreset) -> Self {
        let (damping, response, bounce) = preset.params();
        self.spring_damping = damping;
        self.spring_response = response;
        self.spring_bounce = bounce;
        self
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Morph State
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Captured layout state for morphing transitions.
///
/// Snapshot the source and destination bounds, then interpolate between
/// them frame-by-frame using [`spring_animation()`].
#[derive(Debug, Clone, Copy)]
pub struct MorphState {
    /// Horizontal position.
    pub x: f32,
    /// Vertical position.
    pub y: f32,
    /// Element width.
    pub width: f32,
    /// Element height.
    pub height: f32,
    /// Corner radius.
    pub corner_radius: f32,
    /// Element opacity.
    pub opacity: f32,
}

impl MorphState {
    /// Create a new morph state from layout values (opacity defaults to 1.0).
    pub fn new(x: f32, y: f32, width: f32, height: f32, corner_radius: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            corner_radius,
            opacity: 1.0,
        }
    }

    /// Linearly interpolate between two morph states at time `t` (0.0 to 1.0).
    pub fn lerp(from: &Self, to: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            x: from.x + (to.x - from.x) * t,
            y: from.y + (to.y - from.y) * t,
            width: from.width + (to.width - from.width) * t,
            height: from.height + (to.height - from.height) * t,
            corner_radius: from.corner_radius + (to.corner_radius - from.corner_radius) * t,
            opacity: from.opacity + (to.opacity - from.opacity) * t,
        }
    }

    /// Capture the interpolated position at `t` as a new `from` state for a
    /// re-triggered morph, preserving mid-flight position.
    ///
    /// Short-term workaround for the lack of velocity-preserving spring
    /// interruption in GPUI (see open question #1 on
    /// the internal tracker). Velocity is dropped; only position carries
    /// over, so re-triggers near peak velocity will still visibly snap.
    pub fn capture_midflight(from: &Self, to: &Self, t: f32) -> Self {
        Self::lerp(from, to, t)
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Spring Animation
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Create a spring-timed animation from motion tokens.
///
/// Uses the token's response time as duration and applies a spring easing
/// curve derived from the damping and bounce parameters.
///
/// Prefer [`accessible_spring_animation`] at consumer sites — this base
/// function is blind to Reduce Motion. `accessible_spring_animation`
/// substitutes a short linear cross-fade when the user has Reduce Motion
/// enabled, matching the HIG requirement to "replace large, dramatic
/// transitions with subtle cross-fades."
///
/// # GPUI limitation (Finding 25 in the Zed cross-reference audit)
///
/// The returned animation uses [`spring_easing`] — an exponential
/// approximation, not SwiftUI's physical spring solver. GPUI
/// v0.231.1-pre ships no velocity-preserving spring primitive, so
/// mid-flight re-triggers (the classic case where SwiftUI blends the
/// old velocity into the new target) snap instead of bouncing. When
/// GPUI lands a native Spring easing type (tracked upstream) this
/// helper can swap `spring_easing` for the real solver in one place
/// and every caller inherits the upgrade.
pub fn spring_animation(tokens: &MotionTokens) -> Animation {
    let duration = Duration::from_millis(tokens.spring_duration_ms());
    Animation::new(duration).with_easing(spring_easing(
        tokens.spring_damping,
        tokens.spring_response,
        tokens.spring_bounce,
    ))
}

/// Reduce-Motion-aware spring animation.
///
/// When `reduce_motion` is true, returns a short linear 150ms animation
/// (the HIG-recommended fallback for dramatic transitions). Otherwise
/// behaves identically to [`spring_animation`].
///
/// The 150ms fallback is a deliberate non-zero value: returning
/// `Animation::new(0ms)` produces a division-by-zero in
/// GPUI's `AnimationElement` delta calculation. Consumers wanting a
/// truly instant swap should branch on `reduce_motion` earlier and skip
/// wrapping the element with an animation at all.
pub fn accessible_spring_animation(tokens: &MotionTokens, reduce_motion: bool) -> Animation {
    if reduce_motion {
        // HIG: "replace large, dramatic transitions with subtle cross-fades"
        Animation::new(REDUCE_MOTION_CROSSFADE)
    } else {
        spring_animation(tokens)
    }
}

/// Returns the animation to use for a transition, honouring both
/// `REDUCE_MOTION` and `PREFER_CROSS_FADE_TRANSITIONS`.
///
/// * `REDUCE_MOTION` → short linear cross-fade (same as
///   [`accessible_spring_animation`]). Overrides Prefer Cross-Fade.
/// * `PREFER_CROSS_FADE_TRANSITIONS` → linear cross-fade at
///   `natural_duration`. The curve swaps from spring to linear so
///   translation-heavy animations read as pure opacity changes when the
///   caller uses `el.opacity(delta)`.
/// * Otherwise → spring easing at `natural_duration`.
///
/// Callers should pass their own transition duration (e.g.
/// `lift_duration_ms` or `shape_shift_duration_ms`) as `natural_duration`.
/// The function only overrides timing for `REDUCE_MOTION`.
///
/// Prefer this over `accessible_spring_animation` in presentation surfaces
/// (sheets, modals, popovers) where the HIG Motion section treats
/// Cross-Fade as a separate accessibility preference.
pub fn accessible_transition_animation(
    tokens: &MotionTokens,
    natural_duration: Duration,
    accessibility: crate::foundations::accessibility::AccessibilityMode,
) -> Animation {
    if accessibility.reduce_motion() {
        Animation::new(REDUCE_MOTION_CROSSFADE)
    } else if accessibility.prefer_cross_fade_transitions() {
        // Preserve the caller's natural duration so the transition doesn't
        // feel abruptly curtailed — just swap the curve for linear opacity.
        Animation::new(natural_duration)
    } else {
        Animation::new(natural_duration).with_easing(spring_easing(
            tokens.spring_damping,
            tokens.spring_response,
            tokens.spring_bounce,
        ))
    }
}

/// Duration of the Reduce-Motion cross-fade fallback.
///
/// 150ms chosen to stay below the 250ms HIG short-duration threshold while
/// remaining above the ~120ms visual-continuity floor.
pub const REDUCE_MOTION_CROSSFADE: Duration = Duration::from_millis(150);

/// Read the current accessibility preference for Reduce Motion.
///
/// Reads `TahoeTheme::accessibility_mode` and returns `true` when the
/// `REDUCE_MOTION` flag is set. Components that gate animations on the
/// system preference should call this instead of threading a `bool`
/// parameter — centralising the lookup here makes it trivial to swap the
/// source (today the theme-owned flag; tomorrow a GPUI-native
/// `Window::prefers_reduced_motion()` when that API lands).
///
/// Example:
///
/// ```ignore
/// use tahoe_gpui::foundations::motion::prefers_reduced_motion;
///
/// if prefers_reduced_motion(cx) {
///     // render the static fallback
/// }
/// ```
pub fn prefers_reduced_motion(cx: &gpui::App) -> bool {
    use crate::foundations::theme::ActiveTheme;
    cx.theme().accessibility_mode.reduce_motion()
}

/// Read the current accessibility preference for Prefer Cross-Fade
/// Transitions.
///
/// Distinct from [`prefers_reduced_motion`] — callers that drive
/// sheet/modal/popover transitions should read this flag to decide
/// whether to collapse a translation animation into a pure opacity
/// cross-fade while keeping the natural duration.
pub fn prefers_cross_fade_transitions(cx: &gpui::App) -> bool {
    use crate::foundations::theme::ActiveTheme;
    cx.theme()
        .accessibility_mode
        .prefer_cross_fade_transitions()
}

/// Substitute a short cross-fade for `base` when Reduce Motion is set.
///
/// Finding 22 in the Zed cross-reference audit
///. HIG Motion: *"replace large, dramatic
/// transitions with subtle cross-fades."* Zeroing the duration (as the
/// older `effective_duration` helper does) produces a position snap rather
/// than the fade HIG prescribes. This helper preserves the caller's easing
/// function but shortens the animation to `REDUCE_MOTION_CROSSFADE` when
/// the `REDUCE_MOTION` accessibility flag is set, so callers can write:
///
/// ```ignore
/// use tahoe_gpui::foundations::motion::reduce_motion_substitute;
///
/// let anim = reduce_motion_substitute(
///     spring_animation(&theme.glass.motion),
///     theme.accessibility_mode,
/// );
/// ```
///
/// and pass `anim` to `.with_animation(...)` — the curve the caller
/// authored still runs, but compressed into a 150 ms window that reads as
/// a cross-fade when driven against `el.opacity(delta)`.
pub fn reduce_motion_substitute(
    base: Animation,
    accessibility: crate::foundations::accessibility::AccessibilityMode,
) -> Animation {
    if accessibility.reduce_motion() {
        // Preserve the caller-supplied easing so the visual character is
        // consistent with the long-form transition (e.g. a bounce-eased
        // opacity fade still looks different from a linear fade). Only
        // shorten the duration.
        Animation {
            duration: REDUCE_MOTION_CROSSFADE,
            ..base
        }
    } else {
        base
    }
}

/// Spring easing function.
///
/// Creates an easing curve that approximates an underdamped spring:
/// - damping < 1.0: underdamped (oscillates before settling)
/// - damping = 1.0: critically damped (fastest settle without overshoot)
/// - damping > 1.0: overdamped (slow exponential settle)
///
/// `response` is the spring's perceived period in seconds (SwiftUI's
/// `response` / `dampingFraction`). It controls the natural frequency
/// `ωₙ = 2π / response`, so shorter response → stiffer spring → tighter
/// oscillation cycles across the same normalized `t ∈ [0, 1]` window. With
/// the previous formula every spring of the same damping looked identical
/// regardless of response — visual feel only changed via `Animation::duration`.
///
/// The `bounce` parameter adds overshoot (0.0 = none, higher = more).
///
/// This is an analytical approximation, not SwiftUI's physical solver.
/// For interactive spring feel that matches tactile expectations, use
/// [`SpringPreset`] constants and note GPUI's missing velocity-preserving
/// interruption (open question #1 on the internal tracker).
pub fn spring_easing(damping: f32, response: f32, bounce: f32) -> impl Fn(f32) -> f32 {
    // Total animation spans ~`4 * response` seconds, but the wall-clock
    // duration is capped by `spring_duration_ms` at `MotionRamp::Long`
    // (450 ms) per HIG. Decay rate and bounce oscillation frequency both
    // scale with `periods` (= 4/response) so shorter response yields faster
    // settle and tighter wobble; at the 450 ms boundary the exponential
    // decay is already imperceptible.
    let response = response.max(1e-3);
    let periods = (4.0 / response).clamp(1.0, 12.0);
    move |t: f32| {
        if t >= 1.0 {
            return 1.0;
        }
        if t <= 0.0 {
            return 0.0;
        }
        // Exponential decay envelope — shorter response yields faster settle.
        let decay = (-damping * t * (periods + 2.0)).exp();
        if bounce > 0.0 {
            // Underdamped spring: cos oscillation lets the curve overshoot
            // past 1.0 (cos goes negative) before settling back.
            let phase = t * std::f32::consts::PI * 2.0 * (periods / 8.0 + bounce);
            1.0 - decay * phase.cos()
        } else {
            1.0 - decay
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MorphState, MotionRamp, MotionTokens, SpringPreset};
    use core::prelude::v1::test;

    fn default_tokens() -> MotionTokens {
        MotionTokens {
            flex_duration_ms: 150,
            lift_duration_ms: 200,
            shape_shift_duration_ms: 350,
            spring_damping: 0.85,
            spring_response: 0.35,
            spring_bounce: 0.0,
        }
    }

    #[test]
    fn motion_ramp_short_under_hig_range() {
        // 200ms sits just below the HIG 250–500ms range, reserved for
        // micro-interactions (press/release, flex).
        assert!(MotionRamp::Short.duration_ms() < 250);
    }

    #[test]
    fn motion_ramp_medium_in_hig_range() {
        let ms = MotionRamp::Medium.duration_ms();
        assert!((250..=500).contains(&ms), "medium {} outside 250-500", ms);
    }

    #[test]
    fn motion_ramp_long_in_hig_range() {
        let ms = MotionRamp::Long.duration_ms();
        assert!((250..=500).contains(&ms), "long {} outside 250-500", ms);
    }

    #[test]
    fn motion_ramp_monotonic() {
        assert!(MotionRamp::Short.duration_ms() < MotionRamp::Medium.duration_ms());
        assert!(MotionRamp::Medium.duration_ms() < MotionRamp::Long.duration_ms());
    }

    #[test]
    fn spring_preset_smooth_matches_swiftui() {
        let (damping, response, bounce) = SpringPreset::Smooth.params();
        assert!((damping - 1.0).abs() < f32::EPSILON);
        assert!((response - 0.5).abs() < f32::EPSILON);
        assert!(bounce.abs() < f32::EPSILON);
    }

    #[test]
    fn spring_preset_snappy_matches_swiftui() {
        let (damping, response, bounce) = SpringPreset::Snappy.params();
        assert!((damping - 0.9).abs() < f32::EPSILON);
        assert!((response - 0.3).abs() < f32::EPSILON);
        assert!(bounce.abs() < f32::EPSILON);
    }

    #[test]
    fn spring_preset_bouncy_has_bounce() {
        let (_, _, bounce) = SpringPreset::Bouncy.params();
        assert!(bounce > 0.0);
    }

    #[test]
    fn spring_easing_bouncy_produces_overshoot() {
        let (damping, response, bounce) = SpringPreset::Bouncy.params();
        let easing = super::spring_easing(damping, response, bounce);
        let mut max = 0.0_f32;
        let mut max_t = 0.0_f32;
        for i in 0..=1000 {
            let t = i as f32 / 1000.0;
            let v = easing(t);
            if v > max {
                max = v;
                max_t = t;
            }
        }
        assert!(
            max > 1.0,
            "Bouncy spring must overshoot past 1.0, but max was {max}"
        );
        assert!(
            max_t > 0.0 && max_t < 1.0,
            "Overshoot must occur at an intermediate t, but max_t was {max_t}"
        );
    }

    #[test]
    fn spring_preset_interactive_matches_liquid_glass_default() {
        // These must match the MOTION constant in theme::build_glass.
        let (damping, response, bounce) = SpringPreset::Interactive.params();
        assert!((damping - 0.85).abs() < f32::EPSILON);
        assert!((response - 0.35).abs() < f32::EPSILON);
        assert!(bounce.abs() < f32::EPSILON);
    }

    #[test]
    fn spring_preset_default_matches_swiftui_spring() {
        // SwiftUI's documented `.spring` defaults for an unqualified
        // `.animation(.spring)` call.
        let (damping, response, bounce) = SpringPreset::Default.params();
        assert!((damping - 0.825).abs() < f32::EPSILON);
        assert!((response - 0.55).abs() < f32::EPSILON);
        assert!(bounce.abs() < f32::EPSILON);
    }

    #[test]
    fn motion_tokens_duration_for_uses_ramp_fields() {
        let t = default_tokens();
        assert_eq!(t.duration_for(MotionRamp::Short).as_millis() as u64, 150);
        assert_eq!(t.duration_for(MotionRamp::Medium).as_millis() as u64, 200);
        assert_eq!(t.duration_for(MotionRamp::Long).as_millis() as u64, 350);
    }

    #[test]
    fn motion_tokens_with_spring_preset_replaces_spring_fields() {
        let t = default_tokens().with_spring_preset(SpringPreset::Bouncy);
        let (damping, response, bounce) = SpringPreset::Bouncy.params();
        assert!((t.spring_damping - damping).abs() < f32::EPSILON);
        assert!((t.spring_response - response).abs() < f32::EPSILON);
        assert!((t.spring_bounce - bounce).abs() < f32::EPSILON);
    }

    #[test]
    fn accessible_spring_animation_uses_crossfade_under_reduce_motion() {
        use super::{REDUCE_MOTION_CROSSFADE, accessible_spring_animation};
        let tokens = default_tokens();
        let anim = accessible_spring_animation(&tokens, true);
        assert_eq!(anim.duration, REDUCE_MOTION_CROSSFADE);
    }

    #[test]
    fn accessible_spring_animation_uses_spring_when_motion_allowed() {
        use super::accessible_spring_animation;
        let tokens = default_tokens();
        let anim = accessible_spring_animation(&tokens, false);
        // Duration is capped at MotionRamp::Long (450ms) per HIG.
        assert_eq!(
            anim.duration.as_millis() as u64,
            MotionRamp::Long.duration_ms()
        );
    }

    #[test]
    fn morph_state_capture_midflight_equals_lerp() {
        let from = MorphState::new(0.0, 0.0, 100.0, 100.0, 10.0);
        let to = MorphState::new(50.0, 50.0, 200.0, 200.0, 20.0);
        let captured = MorphState::capture_midflight(&from, &to, 0.5);
        let lerped = MorphState::lerp(&from, &to, 0.5);
        assert!((captured.x - lerped.x).abs() < f32::EPSILON);
        assert!((captured.width - lerped.width).abs() < f32::EPSILON);
    }

    #[test]
    fn reduce_motion_substitute_shortens_only_when_flag_set() {
        use super::{REDUCE_MOTION_CROSSFADE, reduce_motion_substitute, spring_animation};
        use crate::foundations::accessibility::AccessibilityMode;

        let tokens = default_tokens();
        let base = spring_animation(&tokens);
        let base_duration = base.duration;

        // Reduce Motion off → duration preserved.
        let kept = reduce_motion_substitute(base.clone(), AccessibilityMode::DEFAULT);
        assert_eq!(kept.duration, base_duration);

        // Reduce Motion on → duration compressed to the cross-fade window.
        let crossfaded = reduce_motion_substitute(base.clone(), AccessibilityMode::REDUCE_MOTION);
        assert_eq!(crossfaded.duration, REDUCE_MOTION_CROSSFADE);
    }

    #[test]
    fn accessible_transition_animation_covers_three_a11y_states() {
        use std::time::Duration;

        use super::{REDUCE_MOTION_CROSSFADE, accessible_transition_animation};
        use crate::foundations::accessibility::AccessibilityMode;

        let tokens = default_tokens();
        let natural_duration = Duration::from_millis(tokens.spring_duration_ms());

        // REDUCE_MOTION overrides everything: 150 ms linear cross-fade.
        let reduced = accessible_transition_animation(
            &tokens,
            natural_duration,
            AccessibilityMode::REDUCE_MOTION,
        );
        assert_eq!(reduced.duration, REDUCE_MOTION_CROSSFADE);
        assert!(
            ((reduced.easing)(0.5) - 0.5).abs() < 1e-6,
            "REDUCE_MOTION easing must be linear"
        );

        // PREFER_CROSS_FADE keeps the natural duration but swaps the curve
        // for linear so translation-heavy transitions read as pure opacity.
        let cross_fade = accessible_transition_animation(
            &tokens,
            natural_duration,
            AccessibilityMode::PREFER_CROSS_FADE_TRANSITIONS,
        );
        assert_eq!(cross_fade.duration, natural_duration);
        assert!(
            ((cross_fade.easing)(0.5) - 0.5).abs() < 1e-6,
            "PREFER_CROSS_FADE easing must be linear"
        );

        // Default path uses the spring curve (non-linear at t = 0.5).
        let spring =
            accessible_transition_animation(&tokens, natural_duration, AccessibilityMode::DEFAULT);
        assert_eq!(spring.duration, natural_duration);
        assert!(
            ((spring.easing)(0.5) - 0.5).abs() > 1e-3,
            "Default path must use a non-linear spring easing"
        );

        // Combined flags: REDUCE_MOTION takes priority.
        let combined = accessible_transition_animation(
            &tokens,
            natural_duration,
            AccessibilityMode::REDUCE_MOTION | AccessibilityMode::PREFER_CROSS_FADE_TRANSITIONS,
        );
        assert_eq!(combined.duration, REDUCE_MOTION_CROSSFADE);
    }
}
