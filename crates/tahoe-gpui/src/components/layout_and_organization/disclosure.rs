//! HIG Disclosure Control -- standalone expand/collapse arrow.

use gpui::prelude::*;
use gpui::{App, ElementId, FocusHandle, KeyDownEvent, Window, div, px};

use crate::callback_types::OnToggle;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::{apply_focus_ring, glass_interactive_hover};
use crate::foundations::theme::ActiveTheme;

/// A standalone disclosure indicator (expand/collapse triangle) per HIG.
///
/// Stateless `RenderOnce` -- the parent owns `is_expanded` and provides
/// an `on_toggle` callback to receive the toggled state.
///
/// # Glyph
///
/// HIG Disclosure Controls: "A disclosure triangle points inward from the
/// leading edge when its content is hidden and down when its content is
/// visible." The component renders [`IconName::ArrowTriangleRight`] /
/// [`IconName::ArrowTriangleDown`] — the filled SF Symbol disclosure
/// triangles. Chevrons (`ChevronRight`) are a navigation affordance and
/// intentionally *not* used here.
///
/// # Keyboard
///
/// Space / Return toggle. Right arrow expands a collapsed triangle; Left
/// arrow collapses an expanded triangle — matches HIG Accessibility >
/// Keyboard and `NSOutlineView` behaviour.
///
/// # Example
///
/// ```ignore
/// Disclosure::new("section-1")
///     .expanded(true)
///     .on_toggle(|expanded, _window, cx| { /* update model */ })
/// ```
#[derive(IntoElement)]
pub struct Disclosure {
    id: ElementId,
    is_expanded: bool,
    focused: bool,
    /// Optional host-supplied focus handle. Finding 18 in the Zed
    /// cross-reference audit — when set, the focus-ring visibility
    /// comes from `handle.is_focused(window)` and the element threads
    /// `track_focus(&handle)`; otherwise uses the explicit
    /// [`focused`](Self::focused) bool.
    focus_handle: Option<FocusHandle>,
    on_toggle: OnToggle,
}

impl Disclosure {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_expanded: false,
            focused: false,
            focus_handle: None,
            on_toggle: None,
        }
    }

    pub fn expanded(mut self, is_expanded: bool) -> Self {
        self.is_expanded = is_expanded;
        self
    }

    /// Marks the triangle as keyboard-focused so a visible focus ring is
    /// rendered. Ignored when a [`focus_handle`](Self::focus_handle) is
    /// attached — the handle's live state wins.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the disclosure participates in the
    /// host's focus graph. When set, the focus-ring visibility comes
    /// from `handle.is_focused(window)` and the element threads
    /// `track_focus(&handle)` so Tab-cycling and keyboard shortcuts
    /// scoped to the handle fire correctly. Finding 18 in the Zed
    /// cross-reference audit.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

/// Returns whether a keyboard arrow should toggle disclosure state.
///
/// In LTR: Right expands a collapsed triangle; Left collapses an expanded
/// one (matches `NSOutlineView` and HIG Accessibility > Keyboard).
/// Mirrored in RTL so the "expand" direction follows reading order and the
/// arrow still points toward the disclosed content.
fn arrow_toggles_disclosure(key: &str, is_expanded: bool, is_rtl: bool) -> bool {
    let (expand_key, collapse_key) = if is_rtl {
        ("left", "right")
    } else {
        ("right", "left")
    };
    (key == expand_key && !is_expanded) || (key == collapse_key && is_expanded)
}

/// Combined key-event gate used by the `on_key_down` handler: fires on
/// Space / Return / Enter (activation keys) OR on a layout-appropriate
/// arrow toggle. Extracted so the two branches can be composed in tests
/// exactly as the production closure composes them at render time.
fn should_fire_toggle(event: &KeyDownEvent, is_expanded: bool, is_rtl: bool) -> bool {
    crate::foundations::keyboard::is_activation_key(event)
        || arrow_toggles_disclosure(event.keystroke.key.as_str(), is_expanded, is_rtl)
}

impl RenderOnce for Disclosure {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let is_expanded = self.is_expanded;
        let new_state = !is_expanded;
        // Finding 18 in the Zed cross-reference audit.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        // HIG disclosure glyph: filled triangle. Down = expanded, right = collapsed.
        let icon_name = if is_expanded {
            IconName::ArrowTriangleDown
        } else {
            IconName::ArrowTriangleRight
        };

        // Apply glass interactive hover before adding id (which makes it Stateful<Div>)
        let base = div()
            .min_w(px(theme.target_size()))
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0();

        let base = glass_interactive_hover(base, theme);

        // HIG disclosure triangle glyph uses the inline-icon size token so
        // both `Disclosure` and `DisclosureGroup` render at the same scale.
        let mut el = base
            .id(self.id)
            .focusable()
            .child(Icon::new(icon_name).size(theme.icon_size_inline));

        if let Some(handle) = self.focus_handle.as_ref() {
            el = el.track_focus(handle);
        }

        el = apply_focus_ring(el, theme, focused, &[]);

        if let Some(handler) = self.on_toggle {
            let handler = std::rc::Rc::new(handler);
            let click_handler = handler.clone();
            let is_rtl = theme.is_rtl();
            el = el
                .cursor_pointer()
                .on_click(move |_event, window, cx| {
                    click_handler(new_state, window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if should_fire_toggle(event, is_expanded, is_rtl) {
                        cx.stop_propagation();
                        handler(new_state, window, cx);
                    }
                });
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::{Disclosure, arrow_toggles_disclosure, should_fire_toggle};
    use core::prelude::v1::test;
    use gpui::{KeyDownEvent, Keystroke};

    fn make_event(key: &str) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke::parse(key).unwrap(),
            is_held: false,
            prefer_character_input: false,
        }
    }

    #[test]
    fn disclosure_defaults() {
        let d = Disclosure::new("test");
        assert!(!d.is_expanded);
        assert!(d.on_toggle.is_none());
    }

    #[test]
    fn disclosure_expanded_builder() {
        let d = Disclosure::new("test").expanded(true);
        assert!(d.is_expanded);
    }

    #[test]
    fn disclosure_collapsed_builder() {
        let d = Disclosure::new("test").expanded(false);
        assert!(!d.is_expanded);
    }

    #[test]
    fn disclosure_on_toggle_is_some() {
        let d = Disclosure::new("test").on_toggle(|_, _, _| {});
        assert!(d.on_toggle.is_some());
    }

    #[test]
    fn disclosure_chained_builders() {
        let d = Disclosure::new("test")
            .expanded(true)
            .on_toggle(|_, _, _| {});
        assert!(d.is_expanded);
        assert!(d.on_toggle.is_some());
    }

    #[test]
    fn disclosure_focused_default_is_false() {
        let d = Disclosure::new("test");
        assert!(!d.focused);
    }

    #[test]
    fn disclosure_focused_builder() {
        let d = Disclosure::new("test").focused(true);
        assert!(d.focused);
    }

    #[test]
    fn disclosure_focus_handle_default_is_none() {
        let d = Disclosure::new("test");
        assert!(d.focus_handle.is_none());
    }

    #[test]
    fn arrow_keys_ltr_expand_right_collapse_left() {
        // LTR: right expands a collapsed triangle, left collapses an expanded one.
        assert!(arrow_toggles_disclosure("right", false, false));
        assert!(arrow_toggles_disclosure("left", true, false));
        // No-ops when direction doesn't match the current state.
        assert!(!arrow_toggles_disclosure("right", true, false));
        assert!(!arrow_toggles_disclosure("left", false, false));
        // Unrelated keys ignored.
        assert!(!arrow_toggles_disclosure("up", false, false));
        assert!(!arrow_toggles_disclosure("down", true, false));
    }

    #[test]
    fn arrow_keys_rtl_mirror_expand_and_collapse() {
        // RTL: left expands, right collapses — mirrors LTR around the
        // reading axis so the expanding arrow still points toward
        // the disclosed content.
        assert!(arrow_toggles_disclosure("left", false, true));
        assert!(arrow_toggles_disclosure("right", true, true));
        assert!(!arrow_toggles_disclosure("left", true, true));
        assert!(!arrow_toggles_disclosure("right", false, true));
    }

    /// Pins the invariant that Space / Return / Enter go exclusively
    /// through `is_activation_key` and never through the arrow-key
    /// branch. If a future key-mapping change widens the arrow arm to
    /// accept "enter" (unlikely but possible), the production gate
    /// `is_activation_key || arrow_toggles_disclosure` would still fire
    /// once — but the arrow branch would start claiming a key that
    /// belongs to the activation branch, breaking the invariant the
    /// two tests above rely on.
    #[test]
    fn activation_key_names_never_route_through_arrow_branch() {
        for key in ["enter", "space"] {
            for expanded in [false, true] {
                for is_rtl in [false, true] {
                    assert!(
                        !arrow_toggles_disclosure(key, expanded, is_rtl),
                        "{key} must not toggle via the arrow branch \
                         (expanded={expanded}, is_rtl={is_rtl})"
                    );
                }
            }
        }
    }

    /// Composition test: the production `on_key_down` gate fires iff
    /// either branch would. Catches a regression that flips `||` to `&&`
    /// or drops one of the two checks (either would silently pass every
    /// `arrow_toggles_disclosure` or `is_activation_key` test above).
    #[test]
    fn should_fire_toggle_fires_on_activation_keys_regardless_of_direction() {
        for key in ["enter", "space"] {
            for expanded in [false, true] {
                for is_rtl in [false, true] {
                    assert!(
                        should_fire_toggle(&make_event(key), expanded, is_rtl),
                        "{key} must always toggle (expanded={expanded}, is_rtl={is_rtl})"
                    );
                }
            }
        }
    }

    #[test]
    fn should_fire_toggle_mirrors_arrow_branch_under_rtl() {
        // Collapsed in LTR: right expands, left is a no-op.
        assert!(should_fire_toggle(&make_event("right"), false, false));
        assert!(!should_fire_toggle(&make_event("left"), false, false));
        // Collapsed in RTL: left expands, right is a no-op.
        assert!(should_fire_toggle(&make_event("left"), false, true));
        assert!(!should_fire_toggle(&make_event("right"), false, true));
    }

    #[test]
    fn should_fire_toggle_ignores_unrelated_keys() {
        for key in ["tab", "escape", "up", "down", "home", "end"] {
            for expanded in [false, true] {
                for is_rtl in [false, true] {
                    assert!(
                        !should_fire_toggle(&make_event(key), expanded, is_rtl),
                        "{key} must not toggle (expanded={expanded}, is_rtl={is_rtl})"
                    );
                }
            }
        }
    }
}
