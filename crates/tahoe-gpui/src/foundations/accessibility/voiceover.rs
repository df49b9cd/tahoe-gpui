//! VoiceOver / AX-tree scaffolding.
//!
//! GPUI `0.2.2` has no AX tree API — see the crate-level doc in
//! `accessibility/mod.rs`. These types exist so that when GPUI lands one,
//! wiring it up is a single-file change.

use gpui::SharedString;

/// Heading depth constrained to the HTML / HIG range `1..=6`.
///
/// Wraps a `u8` so `AccessibilityRole::Heading` cannot carry an
/// out-of-range value that would mislead VoiceOver's heading-level
/// navigation. Construct via [`HeadingLevel::new`] (fallible) or
/// [`HeadingLevel::new_clamped`] (infallible, saturates to `1..=6`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeadingLevel(u8);

impl HeadingLevel {
    /// Returns `Some(level)` when `level` is in `1..=6`, `None` otherwise.
    pub const fn new(level: u8) -> Option<Self> {
        if level >= 1 && level <= 6 {
            Some(Self(level))
        } else {
            None
        }
    }

    /// Clamps `level` into the valid `1..=6` range. Use at the boundary
    /// from an external source (e.g. markdown parser output) where a
    /// saturating conversion is preferable to propagating an error.
    pub const fn new_clamped(level: u8) -> Self {
        let clamped = if level < 1 {
            1
        } else if level > 6 {
            6
        } else {
            level
        };
        Self(clamped)
    }

    /// The wrapped depth (always in `1..=6`).
    pub const fn get(self) -> u8 {
        self.0
    }
}

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
    /// Menu item inside a menu or pop-up. Maps to `NSAccessibilityMenuItemRole`.
    MenuItem,
    /// Top-level menu bar title (e.g. "File", "Edit"). Maps to
    /// `NSAccessibilityMenuBarItemRole`, which AppKit treats distinctly from
    /// `MenuItem` — VoiceOver's menu-bar navigation gestures expect the
    /// top-level titles to carry the MenuBarItem role rather than MenuItem.
    MenuBarItem,
    /// Tab in a tab bar.
    Tab,
    /// Checkbox (independent boolean).
    Checkbox,
    /// Radio button (exclusive choice).
    RadioButton,
    /// Simple alert announcement (WAI-ARIA `role="alert"`). Use for
    /// non-interactive notifications that VoiceOver auto-announces without
    /// requiring user response. For interactive alerts with action buttons
    /// or text inputs, use [`AlertDialog`](AccessibilityRole::AlertDialog).
    Alert,
    /// Interactive alert dialog requiring user response (WAI-ARIA
    /// `role="alertdialog"`). VoiceOver does NOT auto-announce this role —
    /// the component manages focus explicitly.
    AlertDialog,
    /// Modal dialog (WAI-ARIA `role="dialog"`).
    Dialog,
    /// Progress indicator.
    ProgressIndicator,
    /// Group of related controls with an accessibility label.
    Group,
    /// Group of mutually exclusive radio buttons. Maps to WAI-ARIA
    /// `role="radiogroup"` / NSAccessibility `AXRadioGroup`.
    RadioGroup,
    /// Toolbar — horizontal bar of related actions. Matches WAI-ARIA
    /// `role="toolbar"` and NSAccessibilityRole `.toolbar`.
    ///
    /// Keyboard semantics implemented by
    /// [`crate::components::navigation_and_search::Toolbar`]: Left / Right
    /// walk between registered items, Home / End jump to endpoints. Tab
    /// follows GPUI's native tab-stop map — today that walks every
    /// registered item in registration order before advancing past the
    /// toolbar; full WAI-ARIA APG roving tabindex (where only the active
    /// member exposes `tab_index(0)`) is not yet implemented.
    Toolbar,
    /// Image / decorative media.
    Image,
    /// Heading at the given depth. Carries the level so VoiceOver's
    /// "next heading" and "headings at level N" gestures can land on the
    /// right rung of the document outline when GPUI exposes an AX tree.
    /// Consumers that pattern-match this role should treat the payload
    /// as the HTML / HIG h-level.
    Heading(HeadingLevel),
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
    /// 1-based position within a group (WAI-ARIA `aria-posinset`).
    pub posinset: Option<usize>,
    /// Total items in the group (WAI-ARIA `aria-setsize`).
    pub setsize: Option<usize>,
    /// Declares this element as a modal container (WAI-ARIA `aria-modal`
    /// / NSAccessibility `AXModal`). Pairs with [`AccessibilityRole::Dialog`],
    /// [`AccessibilityRole::AlertDialog`], or [`AccessibilityRole::Alert`] so
    /// VoiceOver announces "modal dialog" once GPUI lands an AX tree.
    pub modal: bool,
}

impl AccessibilityProps {
    /// Builder for an accessibility-labelled element.
    pub fn new() -> Self {
        Self::default()
    }

    /// Shortcut for a menu row: `role = MenuItem`, `label = label`.
    /// Covers the ~5 menu-surface call sites that all shared the same
    /// `new().role(MenuItem).label(...)` triple.
    pub fn menu_item(label: impl Into<SharedString>) -> Self {
        Self::new()
            .role(AccessibilityRole::MenuItem)
            .label(label.into())
    }

    /// Shortcut for a menu-bar title: `role = MenuBarItem`, `label = label`.
    /// Use this for top-level titles in a [`crate::components::menus_and_actions::menu_bar::MenuBar`],
    /// not for items inside a popup — those take [`AccessibilityProps::menu_item`].
    pub fn menu_bar_item(label: impl Into<SharedString>) -> Self {
        Self::new()
            .role(AccessibilityRole::MenuBarItem)
            .label(label.into())
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

    /// Mark this element as a modal container.
    pub fn modal(mut self, modal: bool) -> Self {
        self.modal = modal;
        self
    }

    /// Set the 1-based position within a group (WAI-ARIA `aria-posinset`).
    pub fn posinset(mut self, pos: usize) -> Self {
        self.posinset = Some(pos);
        self
    }

    /// Set the total number of items in the group (WAI-ARIA `aria-setsize`).
    pub fn setsize(mut self, size: usize) -> Self {
        self.setsize = Some(size);
        self
    }

    /// Returns true when at least one field carries information.
    pub fn is_some(&self) -> bool {
        self.label.is_some()
            || self.role.is_some()
            || self.value.is_some()
            || self.modal
            || self.posinset.is_some()
            || self.setsize.is_some()
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
         so VoiceOver/AX tree see nothing (this warning fires once per \
         process).",
        loc.file(),
        loc.line(),
    );
}

#[cfg(test)]
mod tests {
    use super::{AccessibilityProps, AccessibilityRole, AccessibleExt, HeadingLevel};
    use core::prelude::v1::test;

    #[test]
    fn heading_level_new_accepts_1_through_6() {
        for level in 1u8..=6 {
            assert_eq!(HeadingLevel::new(level).map(|h| h.get()), Some(level));
        }
    }

    #[test]
    fn heading_level_new_rejects_out_of_range() {
        assert_eq!(HeadingLevel::new(0), None);
        assert_eq!(HeadingLevel::new(7), None);
        assert_eq!(HeadingLevel::new(99), None);
    }

    #[test]
    fn heading_level_new_clamped_saturates() {
        assert_eq!(HeadingLevel::new_clamped(0).get(), 1);
        assert_eq!(HeadingLevel::new_clamped(7).get(), 6);
        assert_eq!(HeadingLevel::new_clamped(3).get(), 3);
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

        let with_toolbar = AccessibilityProps::new().role(AccessibilityRole::Toolbar);
        assert!(with_toolbar.is_some());
        assert_eq!(with_toolbar.role, Some(AccessibilityRole::Toolbar));

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
    fn menu_item_constructor_sets_role_and_label() {
        let props = AccessibilityProps::menu_item("Copy");
        assert_eq!(props.role, Some(AccessibilityRole::MenuItem));
        assert_eq!(props.label.as_ref().map(|s| s.as_ref()), Some("Copy"));
        assert!(props.value.is_none());
    }

    #[test]
    fn menu_bar_item_constructor_distinguishes_from_menu_item() {
        let bar = AccessibilityProps::menu_bar_item("File");
        assert_eq!(bar.role, Some(AccessibilityRole::MenuBarItem));
        assert_eq!(bar.label.as_ref().map(|s| s.as_ref()), Some("File"));

        // Contract: MenuBarItem and MenuItem are NOT aliases — the enum
        // variants differ so the trait impl that lands with GPUI's AX API
        // can branch on them to map to AXMenuBarItemRole vs AXMenuItemRole.
        assert_ne!(bar.role, Some(AccessibilityRole::MenuItem));
    }

    #[test]
    fn accessibility_props_modal_defaults_false() {
        let props = AccessibilityProps::new();
        assert!(!props.modal);
    }

    #[test]
    fn accessibility_props_modal_builder_sets_flag() {
        let props = AccessibilityProps::new().modal(true);
        assert!(props.modal);
    }

    #[test]
    fn accessibility_props_is_some_true_when_only_modal_set() {
        let props = AccessibilityProps::new().modal(true);
        assert!(props.is_some());
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
    fn accessibility_role_alert_dialog_is_distinct() {
        assert_ne!(AccessibilityRole::AlertDialog, AccessibilityRole::Alert);
        assert_ne!(AccessibilityRole::AlertDialog, AccessibilityRole::Dialog);
    }

    #[test]
    fn accessibility_props_alert_dialog_roundtrip() {
        let props = AccessibilityProps::new()
            .role(AccessibilityRole::AlertDialog)
            .modal(true)
            .label("Delete item?");
        assert_eq!(props.role, Some(AccessibilityRole::AlertDialog));
        assert!(props.modal);
        assert_eq!(
            props.label.as_ref().map(|s| s.as_ref()),
            Some("Delete item?")
        );
        assert!(props.is_some());
    }

    #[test]
    fn radio_group_role_differs_from_group() {
        assert_ne!(AccessibilityRole::RadioGroup, AccessibilityRole::Group);
    }

    #[test]
    fn accessibility_props_posinset_setsize_builders() {
        let props = AccessibilityProps::new().posinset(2).setsize(5);
        assert_eq!(props.posinset, Some(2));
        assert_eq!(props.setsize, Some(5));
    }

    #[test]
    fn accessibility_props_is_some_true_when_only_posinset() {
        assert!(AccessibilityProps::new().posinset(1).is_some());
    }

    #[test]
    fn accessibility_props_is_some_true_when_only_setsize() {
        assert!(AccessibilityProps::new().setsize(3).is_some());
    }
}
