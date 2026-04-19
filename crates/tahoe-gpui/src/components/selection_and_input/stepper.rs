//! HIG Stepper -- compact +/- button pair.

use gpui::prelude::*;
use gpui::{App, ElementId, FocusHandle, KeyDownEvent, Window, div, px};

use crate::callback_types::{OnF64Change, rc_wrap};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_standard_control_styling;
use crate::foundations::theme::{ActiveTheme, GlassSize};

/// A compact +/- stepper control per Human Interface Guidelines.
///
/// Stateless `RenderOnce` -- the parent owns the `value` and provides
/// an `on_change` callback to receive incremented/decremented values.
///
/// # Example
///
/// ```ignore
/// Stepper::new("qty")
///     .value(5.0)
///     .min(0.0)
///     .max(10.0)
///     .step(1.0)
///     .on_change(|new_val, _window, cx| { /* update model */ })
/// ```
#[derive(IntoElement)]
pub struct Stepper {
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    focused: bool,
    /// Optional focus handle supplied by the host — see Finding 18 in
    /// the Zed cross-reference audit. When set, the stepper derives its
    /// focus-ring visibility from `handle.is_focused(window)` instead
    /// of the explicit [`focused`](Self::focused) bool and threads the
    /// handle through `track_focus` so host keyboard navigation stays
    /// coherent with the shared focus graph.
    focus_handle: Option<FocusHandle>,
    /// When true, incrementing past `max` wraps around to `min` (and
    /// vice versa) instead of clamping. Matches HIG NSStepper's `wraps`
    /// property.
    wraps: bool,
    on_change: OnF64Change,
}

impl Stepper {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            disabled: false,
            focused: false,
            focus_handle: None,
            wraps: false,
            on_change: None,
        }
    }

    /// Enable wrap-around behavior. When true, `+` past `max` returns to
    /// `min` and `-` below `min` returns to `max`. Matches the
    /// NSStepper `wraps` property.
    pub fn wraps(mut self, wraps: bool) -> Self {
        self.wraps = wraps;
        self
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = min;
        self.value = self.value.max(self.min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = max;
        self.value = self.value.min(self.max);
        self
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step.abs().max(f64::EPSILON);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the stepper participates in the
    /// host's focus graph. When set, the focus-ring state is derived
    /// from `handle.is_focused(window)` and the root element threads
    /// `track_focus(&handle)` so Tab-cycling and keyboard shortcuts
    /// scoped to the handle fire correctly. Finding 18 in
    /// the Zed cross-reference audit.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    pub fn on_change(mut self, handler: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Clamp `v` to the [min, max] range.
    fn clamp(&self, v: f64) -> f64 {
        v.clamp(self.min, self.max)
    }

    /// Apply `delta` to `value`, wrapping around the range if `self.wraps`
    /// is set and clamping otherwise. The shared step path avoids drift
    /// from repeated float math at the edges.
    fn apply_delta(&self, delta: f64) -> f64 {
        let raw = self.value + delta;
        if self.wraps {
            let range = self.max - self.min;
            if range <= 0.0 {
                return self.clamp(raw);
            }
            // Wrap around using modulo. Inclusive of both endpoints: going
            // past `max` lands on `min` (not on `min + step`).
            if raw > self.max {
                self.min + ((raw - self.max - f64::EPSILON).rem_euclid(range))
            } else if raw < self.min {
                self.max - ((self.min - raw - f64::EPSILON).rem_euclid(range))
            } else {
                raw
            }
        } else {
            self.clamp(raw)
        }
    }
}

impl RenderOnce for Stepper {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        // When the host supplies a focus handle, derive the focus-ring from it;
        // otherwise use the explicit bool.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        // In wrap mode the edges remain active — HIG NSStepper with
        // `wraps=true` lets `+` from `max` hop to `min`.
        let at_min = !self.wraps && self.value <= self.min;
        let at_max = !self.wraps && self.value >= self.max;

        let decremented = self.apply_delta(-self.step);
        let incremented = self.apply_delta(self.step);
        // HIG macOS: "consider supporting Shift-click to change the value
        // quickly" — apply 10× the step for Shift-modified clicks.
        let shift_decremented = self.apply_delta(-self.step * 10.0);
        let shift_incremented = self.apply_delta(self.step * 10.0);
        let selector_id = self.id.to_string();
        let minus_selector = format!("stepper-{selector_id}-minus");
        let plus_selector = format!("stepper-{selector_id}-plus");

        let handler_rc = if !self.disabled {
            rc_wrap(self.on_change)
        } else {
            None
        };

        let btn_size = theme.target_size();
        let icon_size = theme.icon_size_inline;
        // Separator height ≈ the central half of the button — derived from
        // `separator_thickness * 12` so it scales with the hairline token.
        let separator_h = theme.separator_thickness * 12.0;

        // ── Minus button ────────────────────────────────────────────────
        let mut minus_btn = div()
            .id(ElementId::from((self.id.clone(), "minus")))
            .debug_selector(move || minus_selector.clone())
            .min_w(px(btn_size))
            .h(px(btn_size))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .child(
                Icon::new(IconName::Minus)
                    .size(icon_size)
                    .color(theme.text_muted),
            );

        if self.disabled || at_min {
            minus_btn = minus_btn.opacity(0.4);
        } else if let Some(ref handler) = handler_rc {
            let h = handler.clone();
            minus_btn =
                minus_btn
                    .cursor_pointer()
                    .on_click(move |event: &gpui::ClickEvent, window, cx| {
                        let next = if event.modifiers().shift {
                            shift_decremented
                        } else {
                            decremented
                        };
                        h(next, window, cx);
                    });
        }

        // ── Plus button ─────────────────────────────────────────────────
        let mut plus_btn = div()
            .id(ElementId::from((self.id.clone(), "plus")))
            .debug_selector(move || plus_selector.clone())
            .min_w(px(btn_size))
            .h(px(btn_size))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .child(
                Icon::new(IconName::Plus)
                    .size(icon_size)
                    .color(theme.text_muted),
            );

        if self.disabled || at_max {
            plus_btn = plus_btn.opacity(0.4);
        } else if let Some(ref handler) = handler_rc {
            let h = handler.clone();
            plus_btn =
                plus_btn
                    .cursor_pointer()
                    .on_click(move |event: &gpui::ClickEvent, window, cx| {
                        let next = if event.modifiers().shift {
                            shift_incremented
                        } else {
                            incremented
                        };
                        h(next, window, cx);
                    });
        }

        // ── Vertical divider (centered, partial height per HIG) ──
        let divider = div()
            .w(theme.separator_thickness)
            .h(separator_h)
            .bg(theme.border)
            .flex_shrink_0();

        // ── Row container (capsule shape) ───────────────────────────────
        let capsule_radius = px(btn_size / 2.0);
        let row = div()
            .flex()
            .flex_row()
            .items_center()
            .child(minus_btn)
            .child(divider)
            .child(plus_btn);

        let mut container = apply_standard_control_styling(row, theme, GlassSize::Small, focused)
            .rounded(capsule_radius)
            .id(self.id.clone())
            .focusable();

        if let Some(handle) = self.focus_handle.as_ref() {
            container = container.track_focus(handle);
        }

        if self.disabled {
            container = container.opacity(0.5);
        } else if let Some(handler) = handler_rc {
            container =
                container.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    match event.keystroke.key.as_str() {
                        "up" | "right" => handler(incremented, window, cx),
                        "down" | "left" => handler(decremented, window, cx),
                        _ => {}
                    }
                });
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::Stepper;
    use core::prelude::v1::test;

    #[test]
    fn stepper_defaults() {
        let s = Stepper::new("test");
        assert!((s.value - 0.0).abs() < f64::EPSILON);
        assert!((s.min - 0.0).abs() < f64::EPSILON);
        assert!((s.max - 100.0).abs() < f64::EPSILON);
        assert!((s.step - 1.0).abs() < f64::EPSILON);
        assert!(!s.disabled);
        assert!(!s.focused);
        assert!(s.on_change.is_none());
    }

    #[test]
    fn stepper_value_builder() {
        let s = Stepper::new("test").value(42.0);
        assert!((s.value - 42.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stepper_min_max_builder() {
        let s = Stepper::new("test").min(5.0).max(50.0);
        assert!((s.min - 5.0).abs() < f64::EPSILON);
        assert!((s.max - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stepper_step_builder() {
        let s = Stepper::new("test").step(0.5);
        assert!((s.step - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn stepper_disabled_builder() {
        let s = Stepper::new("test").disabled(true);
        assert!(s.disabled);
    }

    #[test]
    fn stepper_on_change_is_some() {
        let s = Stepper::new("test").on_change(|_, _, _| {});
        assert!(s.on_change.is_some());
    }

    #[test]
    fn stepper_clamp_within_range() {
        let s = Stepper::new("test").min(0.0).max(10.0);
        assert!((s.clamp(5.0) - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stepper_clamp_below_min() {
        let s = Stepper::new("test").min(0.0).max(10.0);
        assert!((s.clamp(-5.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn stepper_clamp_above_max() {
        let s = Stepper::new("test").min(0.0).max(10.0);
        assert!((s.clamp(15.0) - 10.0).abs() < f64::EPSILON);
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::{Context, IntoElement, Render, TestAppContext};

    use super::Stepper;
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    const STEPPER_MINUS: &str = "stepper-stepper-minus";
    const STEPPER_PLUS: &str = "stepper-stepper-plus";

    struct StepperHarness {
        value: f64,
        changes: Vec<f64>,
    }

    impl StepperHarness {
        fn new(_cx: &mut Context<Self>, value: f64) -> Self {
            Self {
                value,
                changes: Vec::new(),
            }
        }
    }

    impl Render for StepperHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            Stepper::new("stepper")
                .value(self.value)
                .min(0.0)
                .max(10.0)
                .step(1.0)
                .on_change(move |value, _window, cx| {
                    entity.update(cx, |this, cx| {
                        this.value = value;
                        this.changes.push(value);
                        cx.notify();
                    });
                })
        }
    }

    #[gpui::test]
    async fn clicking_plus_and_minus_updates_value(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| StepperHarness::new(cx, 5.0));

        cx.click_on(STEPPER_PLUS);
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 6.0).abs() < f64::EPSILON);
            assert_eq!(host.changes.last().copied(), Some(6.0));
        });

        cx.click_on(STEPPER_MINUS);
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 5.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![6.0, 5.0]);
        });
    }

    #[gpui::test]
    async fn arrow_keys_increment_and_decrement_after_focus(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| StepperHarness::new(cx, 5.0));

        cx.click_on(STEPPER_PLUS);
        cx.press("up");
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 7.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![6.0, 7.0]);
        });

        cx.click_on(STEPPER_MINUS);
        cx.press("down");
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 5.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![6.0, 7.0, 6.0, 5.0]);
        });
    }
}
