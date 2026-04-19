//! Toggle switch primitive.

use crate::foundations::color::text_on_background;
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::materials::apply_high_contrast_border;
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{
    AnimationExt, App, ElementId, FocusHandle, Hsla, KeyDownEvent, SharedString, Window, div, px,
};

/// Size variant for [`Toggle`] per the macOS HIG `ControlSize` ladder.
///
/// Values match the NSControl.ControlSize ladder used across AppKit
/// (`mini` / `small` / `regular` / `large`). macOS 26 Tahoe keeps the
/// same four-tier ladder — Regular remains the app default; Mini/Small
/// apply inside dense forms where the row height is shared with other
/// compact controls.
///
/// Pixel dimensions:
/// * `Mini` — 26×15 pt (track × height).
/// * `Small` — 30×17 pt.
/// * `Regular` — 36×20 pt. Default.
/// * `Large` — 46×26 pt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToggleSize {
    Mini,
    Small,
    #[default]
    Regular,
    Large,
}

/// A toggle switch component.
///
/// Stateless `RenderOnce` — the parent owns the checked state and provides
/// an `on_change` callback, same pattern as `DisclosureGroup`.
use crate::callback_types::OnToggle;
#[derive(IntoElement)]
pub struct Toggle {
    id: ElementId,
    checked: bool,
    disabled: bool,
    focused: bool,
    /// Optional focus handle supplied by the host. Mirrors Zed's
    /// `ButtonLike::focus_handle` pattern: when present, the toggle
    /// derives its focus-ring visibility from the handle
    /// (`handle.is_focused(window)`) instead of the explicit
    /// [`Toggle::focused`] bool, and threads the handle through
    /// `track_focus` so host keyboard navigation stays coherent.
    /// Finding 19 in the Zed cross-reference audit.
    focus_handle: Option<FocusHandle>,
    tint: Option<Hsla>,
    size: ToggleSize,
    on_change: OnToggle,
    /// Accessibility label for screen readers.
    accessibility_label: Option<SharedString>,
}

impl Toggle {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            checked: false,
            disabled: false,
            focused: false,
            focus_handle: None,
            tint: None,
            size: ToggleSize::Regular,
            on_change: None,
            accessibility_label: None,
        }
    }

    /// Select the control size. Use [`ToggleSize::Mini`] inside grouped
    /// forms where the row height must match an adjacent stepper or
    /// button.
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks this switch as keyboard-focused, showing a visible focus ring.
    ///
    /// Deprecated in favour of [`Toggle::focus_handle`] — passing the
    /// caller's `FocusHandle` lets the toggle derive its focus state
    /// reactively rather than forcing the parent to track a bool.
    /// Finding 19 in the Zed cross-reference audit.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the toggle participates in the host
    /// app's focus graph. When set, the focus-ring is driven by
    /// `handle.is_focused(window)` at render time *and* the underlying
    /// element wraps in `track_focus(&handle)` so Tab-cycling and
    /// keyboard shortcuts scoped to the handle fire correctly.
    ///
    /// Pass the same handle the parent uses for its own focus logic:
    ///
    /// ```ignore
    /// let handle = cx.focus_handle();
    /// Toggle::new("wifi").focus_handle(&handle).checked(wifi_on)
    /// ```
    ///
    /// When no handle is supplied the toggle reads the explicit
    /// [`focused`](Self::focused) bool instead.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Sets an accessibility label for screen readers.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Sets a custom tint color for the on-state track.
    /// When `None`, uses the theme's accent color per HIG: "you might want
    /// to use your app's accent color instead" of the default green.
    pub fn tint(mut self, color: Hsla) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Toggle {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let new_state = !self.checked;
        // When a focus handle is supplied (Finding 19), derive the
        // focus-ring state from the handle — otherwise use the explicit
        // `focused` bool.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        // HIG macOS `ControlSize` ladder. Regular pulls from the theme
        // token so app-wide overrides (e.g. a shrunk dense-form theme)
        // propagate; the other three tiers are fixed at HIG metrics
        // since they're opt-in.
        let (track_width, track_height) = match self.size {
            ToggleSize::Mini => (px(26.0), px(15.0)),
            ToggleSize::Small => (px(30.0), px(17.0)),
            ToggleSize::Regular => (theme.toggle_track_width, theme.toggle_track_height),
            ToggleSize::Large => (px(46.0), px(26.0)),
        };
        let thumb_offset = theme.separator_thickness + theme.separator_thickness;
        let thumb_size = track_height - thumb_offset * 2.0;

        // HIG Toggle: "The default green color tends to work well in
        // most cases, but you might want to use your app's accent color
        // instead." Default to accent so custom app accents (orange, blue,
        // etc.) read correctly without every call site passing `.tint()`.
        let on_tint = self.tint.unwrap_or(theme.accent);
        // Off-state track: flat translucent gray (HIG `systemTertiaryFill`
        // ≈ rgba(120,120,128,0.20)) — not a glass layer. HIG reserves glass
        // surfaces for the active/selected state or outer containers, so
        // the inert off-state must read as a plain system fill.
        let off_track = theme.semantic.tertiary_system_fill;
        let track_bg = if self.checked { on_tint } else { off_track };

        // `leading_offset` is the thumb's distance from the track's *leading*
        // edge. In LTR that is the left edge; in RTL the right edge. The
        // physical `ml` value is mirrored below so the on/off state visually
        // matches the reading direction per HIG Right-to-Left (Controls):
        // the "on" state appears on the trailing edge in either direction.
        let leading_offset = if self.checked {
            track_width - thumb_size - thumb_offset
        } else {
            thumb_offset
        };
        let thumb_ml = if theme.is_rtl() {
            track_width - thumb_size - leading_offset
        } else {
            leading_offset
        };

        // Thumb — circle with drop shadow per HIG. Colour adapts to
        // the track so it stays ≥3:1 contrast on light glass tracks (where a
        // pure-white thumb on a light fill would be effectively invisible).
        let thumb_color = text_on_background(track_bg);
        let thumb_base = div()
            .size(thumb_size)
            .rounded(thumb_size)
            .bg(thumb_color)
            .shadow_sm()
            .mt(thumb_offset);

        // HIG: thumb springs between ends on state change. Animate from
        // the *opposite* resting position up to `thumb_ml` so every toggle
        // produces a visible travel. Under Reduce Motion the spring
        // collapses to a 150 ms cross-fade; we still run the tween so the
        // element id stays stable across renders.
        let reduce_motion = theme.accessibility_mode.reduce_motion();
        let thumb_animation = crate::foundations::motion::accessible_spring_animation(
            &theme.glass.motion,
            reduce_motion,
        );
        let travel = track_width - thumb_size - thumb_offset * 2.0;
        let origin_ml = if self.checked {
            thumb_offset
        } else {
            thumb_offset + travel
        };
        let thumb_id = ElementId::from((self.id.clone(), "thumb"));
        let thumb = thumb_base.ml(origin_ml).with_animation(
            thumb_id,
            thumb_animation,
            move |el, delta| {
                let from = f32::from(origin_ml);
                let to = f32::from(thumb_ml);
                let interp = from + (to - from) * delta;
                el.ml(px(interp))
            },
        );

        // Track — the colored capsule. Shadows and border go on the track,
        // not on the outer touch target.
        let mut track_visual = div()
            .w(track_width)
            .h(track_height)
            .rounded(track_height)
            .bg(track_bg)
            .flex_shrink_0()
            .shadow(theme.glass.shadows(GlassSize::Small).to_vec())
            .child(thumb);

        // Border: off-state uses a visible hairline; on-state uses a
        // transparent border of the same thickness so the thumb doesn't
        // shift by 1pt between states.
        track_visual = track_visual.border_1();
        if self.checked {
            track_visual = track_visual.border_color(gpui::transparent_black());
        } else {
            track_visual = track_visual.border_color(theme.border);
        }

        track_visual = apply_high_contrast_border(track_visual, theme);

        // HIG: minimum touch target — wrap track in a hit area
        let id = self.id;
        let mut track = div()
            .id(id.clone())
            .debug_selector(|| format!("switch-{}", id))
            .focusable()
            .min_h(px(theme.target_size()))
            .min_w(px(theme.target_size()))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .child(track_visual);

        // When a focus handle is supplied, thread it through
        // `track_focus` so host keyboard navigation dispatches actions
        // against the toggle's handle rather than an ephemeral one.
        if let Some(handle) = self.focus_handle.as_ref() {
            track = track.track_focus(handle);
        }

        track = apply_focus_ring(track, theme, focused, &[]);

        if self.disabled {
            track = track.opacity(0.5);
        } else if let Some(handler) = self.on_change {
            let handler = std::rc::Rc::new(handler);
            let click_handler = handler.clone();
            track = track
                .cursor_pointer()
                .on_click(move |_event, window, cx| {
                    click_handler(new_state, window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if crate::foundations::keyboard::is_activation_key(event) {
                        cx.stop_propagation();
                        handler(new_state, window, cx);
                    }
                });
        }

        track
    }
}

#[cfg(test)]
mod tests {
    use super::Toggle;
    use core::prelude::v1::test;

    #[test]
    fn switch_defaults() {
        let s = Toggle::new("test");
        assert!(!s.checked);
        assert!(!s.disabled);
        assert!(s.on_change.is_none());
    }

    #[test]
    fn switch_checked_builder() {
        let s = Toggle::new("test").checked(true);
        assert!(s.checked);
    }

    #[test]
    fn switch_disabled_builder() {
        let s = Toggle::new("test").disabled(true);
        assert!(s.disabled);
    }

    #[test]
    fn switch_on_change_is_some() {
        let s = Toggle::new("test").on_change(|_, _, _| {});
        assert!(s.on_change.is_some());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::{Context, IntoElement, Render, TestAppContext};

    use super::Toggle;
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    const TOGGLE_SWITCH: &str = "switch-toggle";

    struct ToggleHarness {
        checked: bool,
        changes: Vec<bool>,
    }

    impl ToggleHarness {
        fn new(_cx: &mut Context<Self>) -> Self {
            Self {
                checked: false,
                changes: Vec::new(),
            }
        }
    }

    impl Render for ToggleHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            Toggle::new("toggle")
                .checked(self.checked)
                .on_change(move |checked, _window, cx| {
                    entity.update(cx, |this, cx| {
                        this.checked = checked;
                        this.changes.push(checked);
                        cx.notify();
                    });
                })
        }
    }

    #[gpui::test]
    async fn click_and_activation_key_toggle_state(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ToggleHarness::new(cx));

        cx.click_on(TOGGLE_SWITCH);

        host.update_in(cx, |host, _window, _cx| {
            assert!(host.checked);
            assert_eq!(host.changes, vec![true]);
        });

        cx.press("enter");

        host.update_in(cx, |host, _window, _cx| {
            assert!(!host.checked);
            assert_eq!(host.changes, vec![true, false]);
        });
    }
}
