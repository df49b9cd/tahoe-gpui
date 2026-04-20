//! Accessibility configuration aligned with HIG.
//!
//! Provides accessibility mode bitflags, accessibility tokens for
//! Liquid Glass surfaces, and the [`AccessibilityProps`] / [`AccessibleExt`]
//! scaffolding used by components to declare VoiceOver labels, roles, and
//! values.
//!
//! # VoiceOver status (GPUI upstream gap)
//!
//! GPUI `0.2.2` (tag `v0.231.1-pre`) does not yet expose
//! `accessibility_label` / `accessibility_role` APIs on `Div` /
//! `Stateful<Div>`. Verified 2026-04-18 by grepping the upstream source
//! at that tag for `accessibility`, `AXRole`, `NSAccessibility`, and
//! `VoiceOver` — no matches outside settings strings. Components store
//! their labels via [`AccessibilityProps`] and attach them through
//! [`AccessibleExt`] so that when GPUI lands the upstream API the single
//! `apply_accessibility` entry point below can wire labels to the AX
//! tree without any per-component changes.
//!
//! Tracked in <https://github.com/df49b9cd/tahoe-gpui/issues/47>; file a GPUI
//! upstream issue in zed-industries/zed if one does not yet exist.
//!
//! For keyboard graph navigation that does work today (per-component
//! focus rings, Tab-order cycling), see
//! [`crate::workflow::WorkflowCanvas::cycle_node_focus`] — the keyboard
//! half of the HIG accessibility story that doesn't depend on the missing AX API.

use gpui::{Hsla, SharedString};

use crate::foundations::theme::TahoeTheme;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AccessibilityMode
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Accessibility mode flags per HIG.
///
/// Multiple modes can be active simultaneously (e.g., BoldText + IncreaseContrast).
/// Use bitwise OR to combine: `AccessibilityMode::BOLD_TEXT | AccessibilityMode::INCREASE_CONTRAST`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AccessibilityMode(u8);

impl AccessibilityMode {
    /// No accessibility overrides.
    pub const DEFAULT: Self = Self(0);
    /// Replace translucent glass with opaque frosted fills.
    pub const REDUCE_TRANSPARENCY: Self = Self(1 << 0);
    /// Add visible borders around glass surfaces.
    pub const INCREASE_CONTRAST: Self = Self(1 << 1);
    /// Suppress all animations (duration becomes 0).
    pub const REDUCE_MOTION: Self = Self(1 << 2);
    /// Increase font weight by one step across all text.
    pub const BOLD_TEXT: Self = Self(1 << 3);
    /// macOS ctrl-F7 Full Keyboard Access — expands Tab focus to every
    /// control, not just text boxes and lists. Read from
    /// `NSApplication.shared.isFullKeyboardAccessEnabled` on macOS; hosts on
    /// other platforms leave the flag clear.
    pub const FULL_KEYBOARD_ACCESS: Self = Self(1 << 4);
    /// Differentiate Without Color (macOS System Settings → Accessibility →
    /// Display). HIG Accessibility (Color): don't rely on color alone.
    /// Components that signal state purely through colour (e.g. an error
    /// border) must add a non-color cue — icon, dashed pattern, or label —
    /// when this flag is set.
    pub const DIFFERENTIATE_WITHOUT_COLOR: Self = Self(1 << 5);
    /// Prefer Cross-Fade Transitions (macOS System Settings →
    /// Accessibility → Display). Substitute cross-fades for movement-based
    /// transitions (push/slide/zoom). Distinct from `REDUCE_MOTION`: the
    /// user tolerates transitions but wants them expressed as opacity,
    /// not translation.
    pub const PREFER_CROSS_FADE_TRANSITIONS: Self = Self(1 << 6);

    /// Returns true if no accessibility flags are set.
    pub fn is_default(self) -> bool {
        self.0 == 0
    }

    /// Returns true if the reduce transparency flag is set.
    pub fn reduce_transparency(self) -> bool {
        self.0 & Self::REDUCE_TRANSPARENCY.0 != 0
    }

    /// Returns true if the increase contrast flag is set.
    pub fn increase_contrast(self) -> bool {
        self.0 & Self::INCREASE_CONTRAST.0 != 0
    }

    /// Returns true if the reduce motion flag is set.
    pub fn reduce_motion(self) -> bool {
        self.0 & Self::REDUCE_MOTION.0 != 0
    }

    /// Returns true if the bold text flag is set.
    pub fn bold_text(self) -> bool {
        self.0 & Self::BOLD_TEXT.0 != 0
    }

    /// Returns true if Full Keyboard Access is enabled (macOS ctrl-F7).
    pub fn full_keyboard_access(self) -> bool {
        self.0 & Self::FULL_KEYBOARD_ACCESS.0 != 0
    }

    /// Returns true if the user prefers non-color cues for differentiated state.
    pub fn differentiate_without_color(self) -> bool {
        self.0 & Self::DIFFERENTIATE_WITHOUT_COLOR.0 != 0
    }

    /// Returns true if the user prefers cross-fade transitions over movement.
    pub fn prefer_cross_fade_transitions(self) -> bool {
        self.0 & Self::PREFER_CROSS_FADE_TRANSITIONS.0 != 0
    }
}

impl AccessibilityMode {
    /// Returns `self` with the bits in `flag` flipped (XOR). Used by hosts
    /// that surface user-facing toggles for individual accessibility modes.
    pub fn toggled(self, flag: Self) -> Self {
        Self(self.0 ^ flag.0)
    }

    /// Returns `true` when every bit in `flag` is also set in `self`.
    /// Convenience for selection-state queries on toggle controls.
    pub fn contains(self, flag: Self) -> bool {
        flag.0 != 0 && (self.0 & flag.0) == flag.0
    }
}

impl std::ops::BitOr for AccessibilityMode {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for AccessibilityMode {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitXor for AccessibilityMode {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }
}

impl std::ops::BitXorAssign for AccessibilityMode {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AccessibilityTokens
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Accessibility tokens for Liquid Glass per HIG.
#[derive(Debug, Clone)]
pub struct AccessibilityTokens {
    /// Reduced transparency mode: higher opacity glass fill (e.g. 0.85).
    pub reduced_transparency_bg: Hsla,
    /// Increased contrast mode: visible border color.
    pub high_contrast_border: Hsla,
    /// Reduced motion: duration multiplier (0.0 = no motion, 1.0 = full).
    pub reduced_motion_scale: f32,
}

/// In IncreaseContrast mode, adds a visible border per HIG.
/// No-op for other accessibility modes.
///
/// Generic over any GPUI element implementing `Styled` (works with both
/// `Div` and `Stateful<Div>`).
pub fn apply_high_contrast_border<E: gpui::Styled>(mut el: E, theme: &TahoeTheme) -> E {
    if theme.accessibility_mode.increase_contrast() {
        el = el
            .border_1()
            .border_color(theme.glass.accessibility.high_contrast_border);
    }
    el
}

/// Returns the effective animation duration respecting the Reduced Motion accessibility setting.
///
/// When `AccessibilityMode::REDUCE_MOTION` is active, returns 0 to suppress animations.
/// Applies to all themes (glass and non-glass).
///
/// **Note:** Returning 0 produces a zero-duration snap rather than the
/// cross-fade HIG actually prescribes ("replace large, dramatic transitions
/// with subtle cross-fades"). For transition sites that can honour a short
/// cross-fade instead, prefer [`reduce_motion_substitute_ms`] or route the
/// call through `super::motion::accessible_transition_animation`, which
/// returns the 150 ms `REDUCE_MOTION_CROSSFADE` duration when Reduce Motion
/// is on.
pub fn effective_duration(theme: &TahoeTheme, base_ms: u64) -> u64 {
    if theme.accessibility_mode.reduce_motion() {
        0
    } else {
        base_ms
    }
}

/// Returns an animation duration in milliseconds that substitutes a short
/// cross-fade for the caller's original animation when Reduce Motion is on.
///
/// Finding 22 in the Zed cross-reference audit:
/// HIG Motion says "replace large, dramatic transitions with subtle
/// cross-fades." Zeroing the duration — which is what
/// [`effective_duration`] does — produces an instant snap instead of the
/// cross-fade. Routing through this helper preserves a short (150 ms by
/// default — matches `super::motion::REDUCE_MOTION_CROSSFADE`) visual
/// continuity while still honouring the user's preference for minimal
/// movement.
///
/// Use this at transition sites (sheet / modal / popover presentation,
/// segmented-control glass morph) where an abrupt position snap would feel
/// worse than a short opacity-only fade. For movement sites where the
/// motion itself is the distraction (e.g. a spinning loader), keep using
/// [`effective_duration`] so the animation actually stops.
pub fn reduce_motion_substitute_ms(theme: &TahoeTheme, base_ms: u64) -> u64 {
    if theme.accessibility_mode.reduce_motion() {
        // Matches the `REDUCE_MOTION_CROSSFADE` constant in
        // `super::motion`. Kept inline here to avoid a dependency
        // cycle between `accessibility` and `motion`.
        150
    } else {
        base_ms
    }
}

pub use super::materials::apply_focus_ring;

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// AccessibilityProps + AccessibleExt (VoiceOver scaffolding)
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Semantic role of an accessibility-labelled element — mirrors the subset of
/// `NSAccessibilityRole` / UIAccessibilityTraits that the crate's components
/// expose. Used by [`AccessibilityProps::role`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AccessibilityRole {
    /// Static text / label content. Default.
    #[default]
    StaticText,
    /// Activatable button (including icon buttons).
    Button,
    /// Text input field.
    TextField,
    /// Two-state toggle / switch.
    Toggle,
    /// Linear range control (slider).
    Slider,
    /// Circular range control (knob).
    Dial,
    /// Menu item inside a menu or pop-up.
    MenuItem,
    /// Tab in a tab bar.
    Tab,
    /// Checkbox (independent boolean).
    Checkbox,
    /// Radio button (exclusive choice).
    RadioButton,
    /// Alert dialog.
    Alert,
    /// Modal dialog.
    Dialog,
    /// Progress indicator.
    ProgressIndicator,
    /// Group of related controls with an accessibility label.
    Group,
    /// Image / decorative media.
    Image,
}

/// Accessibility metadata for a single element.
///
/// `label` is the primary string VoiceOver reads. `role` classifies the
/// element so VoiceOver announces "button" / "slider" / etc. after the
/// label. `value` carries a current-state description for stateful controls
/// (e.g. "75 percent" for a slider, "On" / "Off" for a toggle).
///
/// The struct is carried with the component until paint; currently GPUI does
/// not ship an AX tree API, so [`AccessibleExt::with_accessibility`] is a
/// structural no-op that emits a one-shot debug-build warning to stderr
/// when non-empty props are discarded. When GPUI lands the AX API, the trait
/// wires into it in one place rather than across ~30 components.
#[derive(Debug, Clone, Default)]
pub struct AccessibilityProps {
    /// VoiceOver label (what VoiceOver reads for this element).
    pub label: Option<SharedString>,
    /// VoiceOver role (announced after the label).
    pub role: Option<AccessibilityRole>,
    /// Stateful-control value description (e.g. "75%" for sliders).
    pub value: Option<SharedString>,
}

impl AccessibilityProps {
    /// Builder for an accessibility-labelled element.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the primary label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the role.
    pub fn role(mut self, role: AccessibilityRole) -> Self {
        self.role = Some(role);
        self
    }

    /// Set the value description.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Returns true when at least one field carries information.
    pub fn is_some(&self) -> bool {
        self.label.is_some() || self.role.is_some() || self.value.is_some()
    }
}

/// Extension trait that attaches [`AccessibilityProps`] to a GPUI element.
///
/// # Important: pending GPUI support
///
/// GPUI v0.231.1-pre exposes no AX tree API. Props passed in here are
/// dropped — VoiceOver, the AX inspector, and every assistive-tech
/// consumer see nothing from this call. The trait is a forward-compat
/// shim so that when GPUI lands `accessibility_label` /
/// `accessibility_role`, rewiring the one impl below upgrades every
/// existing call site to real AX coverage.
///
/// Consumers should still call `with_accessibility(...)` everywhere
/// they would under a real AX API — it is the lift in the "file the
/// upstream issue → land the impl → reap AX for free" plan. Callers
/// relying on AX *today* must integrate with the host's native platform
/// AX path (e.g. NSAccessibility on macOS) outside this trait.
///
/// Tracked in <https://github.com/df49b9cd/tahoe-gpui/issues/47>.
pub trait AccessibleExt: gpui::Styled + Sized {
    /// Attach the given accessibility props to `self`.
    ///
    /// No-op at runtime today (see type-level docs). On first call with
    /// non-empty props in a debug build, emits a one-shot stderr warning
    /// pointing at the caller so the gap does not go unnoticed.
    #[track_caller]
    fn with_accessibility(self, props: &AccessibilityProps) -> Self {
        if cfg!(debug_assertions) && !cfg!(test) && props.is_some() {
            warn_once_a11y_dropped(std::panic::Location::caller());
        }
        self
    }
}

impl<E: gpui::Styled + Sized> AccessibleExt for E {}

/// Emits at most one stderr warning per process when an [`AccessibilityProps`]
/// value is dropped by [`AccessibleExt::with_accessibility`]. Gated by an
/// [`AtomicBool`](std::sync::atomic::AtomicBool) so a gallery with dozens of
/// a11y-annotated components does not flood stderr.
fn warn_once_a11y_dropped(loc: &'static std::panic::Location<'static>) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static WARNED: AtomicBool = AtomicBool::new(false);
    if WARNED.swap(true, Ordering::Relaxed) {
        return;
    }
    eprintln!(
        "[tahoe-gpui] AccessibleExt::with_accessibility dropped \
         AccessibilityProps at {}:{} — GPUI v0.231.1-pre has no AX API, \
         so VoiceOver/AX tree see nothing. Tracked in \
         https://github.com/df49b9cd/tahoe-gpui/issues/47 (this warning \
         fires once per process).",
        loc.file(),
        loc.line(),
    );
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// Tests
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

#[cfg(test)]
mod tests {
    use super::{AccessibilityMode, AccessibilityProps, AccessibilityRole, AccessibleExt};
    use core::prelude::v1::test;

    #[test]
    fn default_has_no_flags() {
        let mode = AccessibilityMode::DEFAULT;
        assert!(mode.is_default());
        assert!(!mode.reduce_transparency());
        assert!(!mode.increase_contrast());
        assert!(!mode.reduce_motion());
        assert!(!mode.bold_text());
        assert!(!mode.full_keyboard_access());
        assert!(!mode.differentiate_without_color());
        assert!(!mode.prefer_cross_fade_transitions());
    }

    #[test]
    fn individual_flags() {
        assert!(AccessibilityMode::REDUCE_TRANSPARENCY.reduce_transparency());
        assert!(!AccessibilityMode::REDUCE_TRANSPARENCY.bold_text());

        assert!(AccessibilityMode::INCREASE_CONTRAST.increase_contrast());
        assert!(AccessibilityMode::REDUCE_MOTION.reduce_motion());
        assert!(AccessibilityMode::BOLD_TEXT.bold_text());
        assert!(AccessibilityMode::FULL_KEYBOARD_ACCESS.full_keyboard_access());
        assert!(AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR.differentiate_without_color());
        assert!(AccessibilityMode::PREFER_CROSS_FADE_TRANSITIONS.prefer_cross_fade_transitions());
    }

    #[test]
    fn combined_flags() {
        let mode = AccessibilityMode::BOLD_TEXT | AccessibilityMode::INCREASE_CONTRAST;
        assert!(!mode.is_default());
        assert!(mode.bold_text());
        assert!(mode.increase_contrast());
        assert!(!mode.reduce_transparency());
        assert!(!mode.reduce_motion());
    }

    #[test]
    fn new_flags_combine_with_old() {
        let mode = AccessibilityMode::FULL_KEYBOARD_ACCESS
            | AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR
            | AccessibilityMode::REDUCE_MOTION;
        assert!(mode.full_keyboard_access());
        assert!(mode.differentiate_without_color());
        assert!(mode.reduce_motion());
        assert!(!mode.bold_text());
        assert!(!mode.prefer_cross_fade_transitions());
    }

    #[test]
    fn bitor_assign() {
        let mut mode = AccessibilityMode::DEFAULT;
        assert!(mode.is_default());
        mode |= AccessibilityMode::REDUCE_MOTION;
        assert!(!mode.is_default());
        assert!(mode.reduce_motion());
    }

    #[test]
    fn toggled_flips_single_flag() {
        let mut mode = AccessibilityMode::DEFAULT;
        mode = mode.toggled(AccessibilityMode::REDUCE_MOTION);
        assert!(mode.reduce_motion());
        mode = mode.toggled(AccessibilityMode::REDUCE_MOTION);
        assert!(!mode.reduce_motion());
        assert!(mode.is_default());
    }

    #[test]
    fn toggled_preserves_other_flags() {
        let starting = AccessibilityMode::BOLD_TEXT | AccessibilityMode::INCREASE_CONTRAST;
        let toggled = starting.toggled(AccessibilityMode::REDUCE_MOTION);
        assert!(toggled.reduce_motion());
        assert!(toggled.bold_text());
        assert!(toggled.increase_contrast());
    }

    #[test]
    fn contains_matches_set_flags() {
        let mode = AccessibilityMode::BOLD_TEXT | AccessibilityMode::INCREASE_CONTRAST;
        assert!(mode.contains(AccessibilityMode::BOLD_TEXT));
        assert!(mode.contains(AccessibilityMode::INCREASE_CONTRAST));
        assert!(!mode.contains(AccessibilityMode::REDUCE_MOTION));
    }

    #[test]
    fn contains_default_returns_false() {
        // `contains(DEFAULT)` is meaningless — DEFAULT has no bits set, so
        // there is nothing to test for. Returning `false` matches the
        // intent of "is this specific flag set?" semantics for callers
        // that iterate over a flag list.
        let mode = AccessibilityMode::BOLD_TEXT;
        assert!(!mode.contains(AccessibilityMode::DEFAULT));
    }

    #[test]
    fn bitxor_op() {
        let mode = AccessibilityMode::BOLD_TEXT ^ AccessibilityMode::REDUCE_MOTION;
        assert!(mode.bold_text());
        assert!(mode.reduce_motion());
        assert!(!mode.increase_contrast());
        // Same operation again clears both bits.
        let cleared = mode ^ (AccessibilityMode::BOLD_TEXT | AccessibilityMode::REDUCE_MOTION);
        assert!(cleared.is_default());
    }

    #[test]
    fn bitxor_assign_op() {
        let mut mode = AccessibilityMode::REDUCE_MOTION;
        mode ^= AccessibilityMode::REDUCE_MOTION;
        assert!(mode.is_default());
    }

    #[test]
    fn derive_default_matches_default_const() {
        assert_eq!(AccessibilityMode::default(), AccessibilityMode::DEFAULT);
    }

    #[test]
    fn all_flags_distinct_bits() {
        // Catches any future flag collision by OR-ing everything and
        // comparing with the expected bit union.
        let all = AccessibilityMode::REDUCE_TRANSPARENCY
            | AccessibilityMode::INCREASE_CONTRAST
            | AccessibilityMode::REDUCE_MOTION
            | AccessibilityMode::BOLD_TEXT
            | AccessibilityMode::FULL_KEYBOARD_ACCESS
            | AccessibilityMode::DIFFERENTIATE_WITHOUT_COLOR
            | AccessibilityMode::PREFER_CROSS_FADE_TRANSITIONS;
        assert!(all.reduce_transparency());
        assert!(all.increase_contrast());
        assert!(all.reduce_motion());
        assert!(all.bold_text());
        assert!(all.full_keyboard_access());
        assert!(all.differentiate_without_color());
        assert!(all.prefer_cross_fade_transitions());
    }

    #[test]
    fn accessibility_props_is_some_tracks_fields() {
        let empty = AccessibilityProps::new();
        assert!(!empty.is_some());

        let with_label = AccessibilityProps::new().label("Save");
        assert!(with_label.is_some());
        assert_eq!(with_label.label.as_ref().map(|s| s.as_ref()), Some("Save"));

        let with_role = AccessibilityProps::new().role(AccessibilityRole::Button);
        assert!(with_role.is_some());
        assert_eq!(with_role.role, Some(AccessibilityRole::Button));

        let with_value = AccessibilityProps::new().value("50 percent");
        assert!(with_value.is_some());
        assert_eq!(
            with_value.value.as_ref().map(|s| s.as_ref()),
            Some("50 percent")
        );
    }

    #[test]
    fn accessibility_role_default_is_static_text() {
        assert_eq!(AccessibilityRole::default(), AccessibilityRole::StaticText);
    }

    #[test]
    fn reduce_motion_substitute_ms_keeps_base_when_flag_off() {
        use super::reduce_motion_substitute_ms;
        use crate::foundations::theme::TahoeTheme;
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::DEFAULT;
        assert_eq!(reduce_motion_substitute_ms(&theme, 350), 350);
    }

    #[test]
    fn reduce_motion_substitute_ms_returns_crossfade_duration() {
        use super::reduce_motion_substitute_ms;
        use crate::foundations::theme::TahoeTheme;
        let mut theme = TahoeTheme::dark();
        theme.accessibility_mode = AccessibilityMode::REDUCE_MOTION;
        // Matches REDUCE_MOTION_CROSSFADE in super::motion (150 ms).
        assert_eq!(reduce_motion_substitute_ms(&theme, 350), 150);
    }

    #[test]
    fn with_accessibility_is_passthrough() {
        // Contract: `with_accessibility` must return its receiver unchanged
        // until GPUI lands an AX API. If this test starts failing, it is the
        // cue to thread props into the real AX path.
        let props = AccessibilityProps::new()
            .role(AccessibilityRole::Button)
            .label("Send message");

        let base = gpui::StyleRefinement::default();
        let after = gpui::StyleRefinement::default().with_accessibility(&props);
        assert_eq!(after, base);
    }
}
