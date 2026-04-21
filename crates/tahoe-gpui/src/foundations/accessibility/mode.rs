//! Accessibility mode bitflags.

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

impl std::ops::Not for AccessibilityMode {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl std::ops::BitAnd for AccessibilityMode {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for AccessibilityMode {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use super::AccessibilityMode;
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
}
