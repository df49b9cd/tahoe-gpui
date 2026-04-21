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

use gpui::{App, FocusHandle, Hsla, InteractiveElement, KeyDownEvent, SharedString, Window};

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
    /// Heading at the given level (1–6). Carries the level so
    /// VoiceOver's "next heading" and "headings at level N" gestures can
    /// land on the right rung of the document outline when GPUI exposes
    /// an AX tree. Consumers that pattern-match this role should treat
    /// the payload as the HTML / HIG h-level.
    ///
    /// **Invariant**: the inner value must be in `1..=6`. Values outside
    /// this range will mislead VoiceOver's heading-level navigation.
    /// Construct via `AccessibilityRole::Heading(level)` only with
    /// levels produced by the markdown parser (which guarantees 1–6).
    Heading(u8),
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
// FocusGroup
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Traversal behavior at the edges of a [`FocusGroup`].
///
/// - [`Open`](Self::Open): the default — no Tab interception; focus falls
///   through to the enclosing order at edges. Use for logical clusters where
///   Tab should still exit naturally (toolbar slot clusters, form rows).
/// - [`Cycle`](Self::Cycle): wrap around inside the group's programmatic
///   navigation (`focus_next` / `focus_previous`). Intended for arrow-key
///   navigation inside radio groups, segmented controls, or tab bars. Tab is
///   left to GPUI's native [`TabStopMap`](https://docs.rs/gpui) so the
///   surrounding document order stays walkable.
/// - [`Trap`](Self::Trap): Tab and Shift+Tab are intercepted, wrapped, and
///   consumed so focus cannot escape the group. Use for modal dialogs and
///   action sheets following the WAI-ARIA dialog pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusGroupMode {
    #[default]
    Open,
    Cycle,
    Trap,
}

struct FocusGroupInner {
    handles: Vec<FocusHandle>,
    mode: FocusGroupMode,
}

/// Ordered collection of [`FocusHandle`]s forming a logical focus cluster.
///
/// Cheap to clone — the backing storage is an [`Rc`](std::rc::Rc), shared by
/// all clones. Intended to live on the parent entity (or as a stateless
/// value passed by reference) and be populated by children during render.
///
/// # Why this exists
///
/// GPUI exposes per-element `tab_index` / `tab_stop` / `tab_group` on
/// [`InteractiveElement`] plus `Window::focus_next` / `focus_prev` for Tab
/// traversal, but has no grouping primitive that bundles those with programmatic
/// navigation (`focus_next`/`focus_previous`/`focus_first`/`focus_last`) and
/// trap/cycle semantics. Without this layer, every component that needs a
/// radio-group, tab-bar, or modal focus trap re-implements the wrap math and
/// Tab-swallow. See [`Modal`](crate::components::presentation::Modal) for a
/// concrete Trap-mode user.
///
/// # Typical usage
///
/// ```ignore
/// // On the parent entity:
/// struct MyRadioGroup {
///     options: Vec<FocusHandle>,
///     group: FocusGroup,
/// }
///
/// impl MyRadioGroup {
///     fn new(cx: &mut Context<Self>) -> Self {
///         Self {
///             options: (0..3).map(|_| cx.focus_handle()).collect(),
///             group: FocusGroup::cycle(),
///         }
///     }
/// }
///
/// // In render, register each option and attach keyboard nav:
/// div().on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
///     match ev.keystroke.key.as_str() {
///         "right" | "down" => this.group.focus_next(window, cx),
///         "left"  | "up"   => this.group.focus_previous(window, cx),
///         _ => {}
///     }
/// }))
/// ```
#[derive(Clone)]
pub struct FocusGroup {
    inner: std::rc::Rc<std::cell::RefCell<FocusGroupInner>>,
}

impl Default for FocusGroup {
    fn default() -> Self {
        Self::new(FocusGroupMode::default())
    }
}

impl std::fmt::Debug for FocusGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("FocusGroup")
            .field("mode", &inner.mode)
            .field("len", &inner.handles.len())
            .finish()
    }
}

impl FocusGroup {
    pub fn new(mode: FocusGroupMode) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(FocusGroupInner {
                handles: Vec::new(),
                mode,
            })),
        }
    }

    pub fn open() -> Self {
        Self::new(FocusGroupMode::Open)
    }

    pub fn cycle() -> Self {
        Self::new(FocusGroupMode::Cycle)
    }

    pub fn trap() -> Self {
        Self::new(FocusGroupMode::Trap)
    }

    pub fn mode(&self) -> FocusGroupMode {
        self.inner.borrow().mode
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().handles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().handles.is_empty()
    }

    /// Register `handle` as a member, appending to tab order.
    ///
    /// Idempotent: if a handle comparing equal (`FocusHandle: PartialEq` by
    /// FocusId) is already registered, the collection is left unchanged and the
    /// existing index is returned. Safe to call every render.
    pub fn register(&self, handle: &FocusHandle) -> usize {
        let mut inner = self.inner.borrow_mut();
        if let Some(idx) = inner.handles.iter().position(|h| h == handle) {
            return idx;
        }
        inner.handles.push(handle.clone());
        inner.handles.len() - 1
    }

    /// Remove all registered handles. Call from the parent's render when
    /// member identity changes frame-to-frame (e.g. the number of options in
    /// a radio group depends on runtime state).
    pub fn clear(&self) {
        self.inner.borrow_mut().handles.clear();
    }

    pub fn focus_first(&self, window: &mut Window, cx: &mut App) {
        if let Some(handle) = self.inner.borrow().handles.first().cloned() {
            handle.focus(window, cx);
        }
    }

    pub fn focus_last(&self, window: &mut Window, cx: &mut App) {
        if let Some(handle) = self.inner.borrow().handles.last().cloned() {
            handle.focus(window, cx);
        }
    }

    /// Advance focus to the next member. In [`FocusGroupMode::Open`] stops at
    /// the last member (no wrap); in `Cycle` / `Trap` wraps to the first.
    /// When no member currently holds focus, lands on the first.
    pub fn focus_next(&self, window: &mut Window, cx: &mut App) {
        self.advance(window, cx, /* forward */ true);
    }

    /// Retreat focus to the previous member. In [`FocusGroupMode::Open`]
    /// stops at the first member (no wrap); in `Cycle` / `Trap` wraps to the
    /// last. When no member currently holds focus, lands on the last.
    pub fn focus_previous(&self, window: &mut Window, cx: &mut App) {
        self.advance(window, cx, /* forward */ false);
    }

    fn advance(&self, window: &mut Window, cx: &mut App, forward: bool) {
        let (next_handle, _) = {
            let inner = self.inner.borrow();
            let len = inner.handles.len();
            if len == 0 {
                return;
            }
            let current = inner.handles.iter().position(|h| h.is_focused(window));
            let wrap = matches!(inner.mode, FocusGroupMode::Cycle | FocusGroupMode::Trap);
            let next = match current {
                Some(idx) if forward => {
                    if idx + 1 < len {
                        Some(idx + 1)
                    } else if wrap {
                        Some(0)
                    } else {
                        None
                    }
                }
                Some(idx) => {
                    if idx > 0 {
                        Some(idx - 1)
                    } else if wrap {
                        Some(len - 1)
                    } else {
                        None
                    }
                }
                None if forward => Some(0),
                None => Some(len - 1),
            };
            (next.map(|i| inner.handles[i].clone()), len)
        };
        if let Some(handle) = next_handle {
            handle.focus(window, cx);
        }
    }

    /// True when any registered member currently holds focus.
    pub fn contains_focused(&self, window: &Window) -> bool {
        self.inner
            .borrow()
            .handles
            .iter()
            .any(|h| h.is_focused(window))
    }

    /// Hook to call from the group host's `on_key_down`.
    ///
    /// In [`FocusGroupMode::Trap`], intercepts Tab / Shift+Tab, calls
    /// `cx.stop_propagation()`, and advances focus through the group with
    /// wrap-around. Returns `true` when the event was consumed. In `Open` /
    /// `Cycle` modes this is a no-op (`false`) so GPUI's native TabStopMap
    /// continues to drive Tab traversal.
    pub fn handle_key_down(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) -> bool {
        if event.keystroke.key.as_str() != "tab" {
            return false;
        }
        if self.mode() != FocusGroupMode::Trap {
            return false;
        }
        cx.stop_propagation();
        if event.keystroke.modifiers.shift {
            self.focus_previous(window, cx);
        } else {
            self.focus_next(window, cx);
        }
        true
    }
}

/// Extension trait that opts a GPUI [`InteractiveElement`] into a
/// [`FocusGroup`].
///
/// Equivalent to calling `.track_focus(handle).tab_index(group_index)` with
/// the group managing the index. Safe to call every render — [`FocusGroup::register`]
/// is idempotent by FocusId.
pub trait FocusGroupExt: InteractiveElement + Sized {
    fn focus_group(self, group: &FocusGroup, handle: &FocusHandle) -> Self {
        let index = group.register(handle) as isize;
        self.track_focus(handle).tab_index(index)
    }
}

impl<E: InteractiveElement + Sized> FocusGroupExt for E {}

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
        // until GPUI lands an AX API. Starting from a mutated refinement
        // (visibility = Hidden via `invisible()`) catches both in-place
        // mutation and "returns a fresh default" failure modes. If this
        // test starts failing, it is the cue to thread props into the
        // real AX path.
        use gpui::Styled;

        let props = AccessibilityProps::new()
            .role(AccessibilityRole::Button)
            .label("Send message");

        let before = gpui::StyleRefinement::default().invisible();
        let after = gpui::StyleRefinement::default()
            .invisible()
            .with_accessibility(&props);
        assert_eq!(after, before);
    }

    #[test]
    fn focus_group_mode_default_is_open() {
        use super::FocusGroupMode;
        assert_eq!(FocusGroupMode::default(), FocusGroupMode::Open);
    }

    #[test]
    fn focus_group_constructors_set_mode() {
        use super::{FocusGroup, FocusGroupMode};
        assert_eq!(FocusGroup::open().mode(), FocusGroupMode::Open);
        assert_eq!(FocusGroup::cycle().mode(), FocusGroupMode::Cycle);
        assert_eq!(FocusGroup::trap().mode(), FocusGroupMode::Trap);
        assert_eq!(FocusGroup::default().mode(), FocusGroupMode::Open);
    }

    #[test]
    fn focus_group_starts_empty() {
        use super::FocusGroup;
        let group = FocusGroup::trap();
        assert!(group.is_empty());
        assert_eq!(group.len(), 0);
    }

    #[test]
    fn focus_group_clones_share_inner() {
        use super::FocusGroup;
        let group = FocusGroup::cycle();
        let clone = group.clone();
        // Shared Rc<RefCell<_>>: clearing via the clone is observable
        // through the original handle. Writing to one must not create a
        // snapshot.
        clone.clear();
        assert_eq!(group.len(), 0);
        assert_eq!(clone.len(), 0);
    }

    #[test]
    fn focus_group_debug_contains_mode_and_len() {
        use super::FocusGroup;
        let group = FocusGroup::trap();
        let repr = format!("{:?}", group);
        assert!(
            repr.contains("Trap"),
            "Debug output should mention mode: {repr}"
        );
        assert!(
            repr.contains("len"),
            "Debug output should include member count: {repr}"
        );
    }
}

#[cfg(test)]
mod focus_group_interaction_tests {
    use super::{FocusGroup, FocusGroupMode};
    use crate::test_helpers::helpers::setup_test_window;
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    /// Harness that mints three focus handles and tracks them inside a
    /// `FocusGroup`. Tests drive the group API directly via `update_in` on
    /// this entity — the render path exists so GPUI's window has live focus
    /// wiring (focus requests and `is_focused` queries need a rendered frame).
    struct GroupHarness {
        handles: [FocusHandle; 3],
        group: FocusGroup,
    }

    impl GroupHarness {
        fn new(mode: FocusGroupMode, cx: &mut Context<Self>) -> Self {
            let handles = [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()];
            let group = FocusGroup::new(mode);
            for handle in &handles {
                group.register(handle);
            }
            Self { handles, group }
        }
    }

    impl Render for GroupHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div()
                .w(px(200.0))
                .h(px(80.0))
                .flex()
                .flex_col()
                .child(
                    div()
                        .id("member-0")
                        .track_focus(&self.handles[0])
                        .child("0"),
                )
                .child(
                    div()
                        .id("member-1")
                        .track_focus(&self.handles[1])
                        .child("1"),
                )
                .child(
                    div()
                        .id("member-2")
                        .track_focus(&self.handles[2])
                        .child("2"),
                )
        }
    }

    #[gpui::test]
    async fn register_is_idempotent(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update(cx, |host, _cx| {
            assert_eq!(host.group.len(), 3);
            // Re-registering the same handles must be a no-op.
            let idx = host.group.register(&host.handles[1]);
            assert_eq!(idx, 1, "re-register returns the existing index");
            assert_eq!(host.group.len(), 3, "len unchanged after re-register");
        });
    }

    #[gpui::test]
    async fn focus_first_on_empty_is_noop(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.clear();
            // Must not panic — empty group is a valid state.
            host.group.focus_first(window, cx);
            host.group.focus_last(window, cx);
            host.group.focus_next(window, cx);
            host.group.focus_previous(window, cx);
        });
    }

    #[gpui::test]
    async fn focus_first_lands_on_first_member(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_first(window, cx);
            assert!(host.handles[0].is_focused(window));
        });
    }

    #[gpui::test]
    async fn focus_last_lands_on_last_member(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_last(window, cx);
            assert!(host.handles[2].is_focused(window));
        });
    }

    #[gpui::test]
    async fn focus_next_wraps_in_cycle_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[2].focus(window, cx);
            assert!(host.handles[2].is_focused(window));
            host.group.focus_next(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "Cycle: focus_next past last wraps to first"
            );
        });
    }

    #[gpui::test]
    async fn focus_previous_wraps_in_trap_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_previous(window, cx);
            assert!(
                host.handles[2].is_focused(window),
                "Trap: focus_previous past first wraps to last"
            );
        });
    }

    #[gpui::test]
    async fn focus_next_stops_at_edge_in_open_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[2].focus(window, cx);
            host.group.focus_next(window, cx);
            assert!(
                host.handles[2].is_focused(window),
                "Open: focus_next past last stays on last"
            );
        });
    }

    #[gpui::test]
    async fn focus_previous_stops_at_edge_in_open_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
            host.group.focus_previous(window, cx);
            assert!(
                host.handles[0].is_focused(window),
                "Open: focus_previous past first stays on first"
            );
        });
    }

    #[gpui::test]
    async fn focus_next_with_no_current_focus_lands_on_first(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_next(window, cx);
            assert!(host.handles[0].is_focused(window));
        });
    }

    #[gpui::test]
    async fn focus_previous_with_no_current_focus_lands_on_last(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.group.focus_previous(window, cx);
            assert!(host.handles[2].is_focused(window));
        });
    }

    #[gpui::test]
    async fn contains_focused_tracks_membership(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Open, cx)
        });
        host.update_in(cx, |host, window, _cx| {
            assert!(!host.group.contains_focused(window));
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        host.update_in(cx, |host, window, _cx| {
            assert!(host.group.contains_focused(window));
        });
    }

    #[gpui::test]
    async fn handle_key_down_ignores_non_tab_keys(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers: Modifiers::default(),
                    key: "enter".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(!host.group.handle_key_down(&event, window, cx));
        });
    }

    #[gpui::test]
    async fn handle_key_down_swallows_tab_in_trap_mode(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        host.update_in(cx, |host, window, cx| {
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers: Modifiers::default(),
                    key: "tab".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(
                host.group.handle_key_down(&event, window, cx),
                "Trap mode consumes Tab"
            );
            assert!(
                host.handles[1].is_focused(window),
                "Trap Tab advances to next member"
            );
        });
    }

    #[gpui::test]
    async fn handle_key_down_passes_through_tab_in_cycle_mode(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Cycle, cx)
        });
        host.update_in(cx, |host, window, cx| {
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers: Modifiers::default(),
                    key: "tab".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(
                !host.group.handle_key_down(&event, window, cx),
                "Cycle mode leaves Tab to GPUI's native tab-stop map"
            );
        });
    }

    #[gpui::test]
    async fn handle_key_down_shift_tab_trap_retreats(cx: &mut TestAppContext) {
        use gpui::{KeyDownEvent, Keystroke, Modifiers};

        let (host, cx) = setup_test_window(cx, |_window, cx| {
            GroupHarness::new(FocusGroupMode::Trap, cx)
        });
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        host.update_in(cx, |host, window, cx| {
            let modifiers = Modifiers {
                shift: true,
                ..Default::default()
            };
            let event = KeyDownEvent {
                keystroke: Keystroke {
                    modifiers,
                    key: "tab".into(),
                    key_char: None,
                },
                is_held: false,
                prefer_character_input: false,
            };
            assert!(host.group.handle_key_down(&event, window, cx));
            assert!(
                host.handles[2].is_focused(window),
                "Trap Shift+Tab wraps to last from first"
            );
        });
    }
}
