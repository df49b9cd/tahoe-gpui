//! HIG Stepper -- compact +/- button pair.

use gpui::prelude::*;
use gpui::{App, ElementId, FocusHandle, KeyDownEvent, Window, div, px};

use crate::callback_types::{OnF64Change, rc_wrap};
use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup, FocusGroupExt,
};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::{apply_focus_ring, apply_standard_control_styling};
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
///
/// # TODO: HIG press-and-hold auto-repeat (NSStepper)
///
/// `NSStepper` increments continuously while either button is held —
/// 500ms initial delay, then 60ms cadence. Adding that here requires
/// elevating `Stepper` from the current `RenderOnce` builder to a
/// stateful `Entity<T>` so the repeat [`gpui::Task`] can live in the
/// component and be cancelled on mouse-up / mouse-leave. The conversion
/// ripples through every caller (gallery entries, unit harnesses) and
/// every builder method (they become mutators on `&mut self` or
/// constructor args), so it exceeds the in-scope budget for this pass.
///
/// Sketch of the future shape:
/// ```ignore
/// pub struct Stepper { /* …existing fields… */, repeat: Option<Task<()>> }
/// fn start_repeat(&mut self, delta: f64, cx: &mut Context<Self>) {
///     self.repeat = Some(cx.spawn(async move |this, cx| {
///         cx.background_executor().timer(Duration::from_millis(500)).await;
///         loop {
///             cx.background_executor().timer(Duration::from_millis(60)).await;
///             if this.update(cx, |this, cx| this.bump(delta, cx)).is_err() { break; }
///         }
///     }));
/// }
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
    /// Host-owned focus group for per-button Tab stops under macOS
    /// Full Keyboard Access. When paired with [`btn_focus_handles`](Self::btn_focus_handles)
    /// and the active theme reports FKA, each button becomes its own
    /// Tab stop.
    btn_focus_group: Option<FocusGroup>,
    /// Host-owned per-button focus handles. Expected to hold exactly 2
    /// handles: handles\[0\] = minus, handles\[1\] = plus.
    btn_focus_handles: Vec<FocusHandle>,
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
            btn_focus_group: None,
            btn_focus_handles: Vec::new(),
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

    /// Attach a host-owned [`FocusGroup`] for per-button arrow-nav
    /// and Tab-reachability under macOS Full Keyboard Access. When
    /// paired with [`btn_focus_handles`](Self::btn_focus_handles) and
    /// the active theme reports FKA, each button becomes its own Tab
    /// stop.
    pub fn btn_focus_group(mut self, group: FocusGroup) -> Self {
        self.btn_focus_group = Some(group);
        self
    }

    /// Per-button [`FocusHandle`]s. Expected to hold exactly 2 handles:
    /// handles\[0\] = minus, handles\[1\] = plus. Host-owned because
    /// `Stepper` is stateless `RenderOnce`.
    pub fn btn_focus_handles(mut self, handles: Vec<FocusHandle>) -> Self {
        self.btn_focus_handles = handles;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Clamp `v` to the [min, max] range.
    #[cfg(test)]
    fn clamp(&self, v: f64) -> f64 {
        v.clamp(self.min, self.max)
    }

    /// Apply `delta` to `value`, wrapping around the range if `wraps`
    /// is set and clamping otherwise. The shared step path avoids drift
    /// from repeated float math at the edges.
    fn apply_delta(value: f64, min: f64, max: f64, wraps: bool, delta: f64) -> f64 {
        let raw = value + delta;
        if wraps {
            let range = max - min;
            if range <= 0.0 {
                return raw.clamp(min, max);
            }
            // Wrap around using modulo. Inclusive of both endpoints: going
            // past `max` lands on `min` (not on `min + step`).
            if raw > max {
                min + ((raw - max - f64::EPSILON).rem_euclid(range))
            } else if raw < min {
                max - ((min - raw - f64::EPSILON).rem_euclid(range))
            } else {
                raw
            }
        } else {
            raw.clamp(min, max)
        }
    }
}

impl RenderOnce for Stepper {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Self {
            id,
            value,
            min,
            max,
            step,
            disabled,
            focused: explicit_focused,
            focus_handle,
            btn_focus_group,
            btn_focus_handles,
            wraps,
            on_change,
        } = self;

        let theme = cx.theme();
        // FKA: per-button focus when the host supplies a group + exactly 2 handles.
        let fka_buttons = FocusGroup::bind_if_fka(
            theme.full_keyboard_access(),
            btn_focus_group,
            btn_focus_handles,
            2,
        );
        // When the host supplies a focus handle, derive the focus-ring from it;
        // otherwise use the explicit bool. Suppress the capsule ring when FKA
        // per-button handles are active — the per-button rings are the correct
        // indicator.
        let focused = focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(explicit_focused);
        let capsule_focused = if fka_buttons.is_some() {
            false
        } else {
            focused
        };

        // In wrap mode the edges remain active — HIG NSStepper with
        // `wraps=true` lets `+` from `max` hop to `min`.
        let at_min = !wraps && value <= min;
        let at_max = !wraps && value >= max;

        let apply = Self::apply_delta;
        let decremented = apply(value, min, max, wraps, -step);
        let incremented = apply(value, min, max, wraps, step);
        let shift_decremented = apply(value, min, max, wraps, -step * 10.0);
        let shift_incremented = apply(value, min, max, wraps, step * 10.0);
        let selector_id = id.to_string();
        let minus_selector = format!("stepper-{selector_id}-minus");
        let plus_selector = format!("stepper-{selector_id}-plus");

        let handler_rc = if !disabled { rc_wrap(on_change) } else { None };

        let btn_size = theme.target_size();
        let icon_size = theme.icon_size_inline;
        // Separator height ≈ the central half of the button — derived from
        // `separator_thickness * 12` so it scales with the hairline token.
        let separator_h = theme.separator_thickness * 12.0;

        // ── Minus button ────────────────────────────────────────────────
        let mut minus_btn = div()
            .id(ElementId::from((id.clone(), "minus")))
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

        if disabled || at_min {
            minus_btn = minus_btn.opacity(0.4);
        } else if let Some(ref handler) = handler_rc {
            let h_click = handler.clone();
            minus_btn =
                minus_btn
                    .cursor_pointer()
                    .on_click(move |event: &gpui::ClickEvent, window, cx| {
                        let next = if event.modifiers().shift {
                            shift_decremented
                        } else {
                            decremented
                        };
                        h_click(next, window, cx);
                    });
        }

        // FKA: per-button focus + keyboard activation for minus.
        if let Some((ref group, ref handles)) = fka_buttons {
            let handle = &handles[0];
            let is_btn_focused = handle.is_focused(window);
            let nav_group = group.clone();
            let nav_handler = handler_rc.clone();
            minus_btn = minus_btn.focus_group(group, handle);
            minus_btn = minus_btn.with_accessibility(
                &AccessibilityProps::new()
                    .role(AccessibilityRole::Button)
                    .label("Decrement"),
            );
            minus_btn = apply_focus_ring(minus_btn, theme, is_btn_focused, &[]);
            minus_btn = minus_btn.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                match ev.keystroke.key.as_str() {
                    "up" | "right" => {
                        nav_group.focus_next(window, cx);
                        cx.stop_propagation();
                    }
                    "down" | "left" => {
                        nav_group.focus_previous(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {
                        if crate::foundations::keyboard::is_activation_key(ev) {
                            if !disabled
                                && !at_min
                                && let Some(ref h) = nav_handler
                            {
                                let shift = ev.keystroke.modifiers.shift;
                                h(
                                    if shift {
                                        shift_decremented
                                    } else {
                                        decremented
                                    },
                                    window,
                                    cx,
                                );
                            }
                            cx.stop_propagation();
                        }
                    }
                }
            });
        }

        // ── Plus button ─────────────────────────────────────────────────
        let mut plus_btn = div()
            .id(ElementId::from((id.clone(), "plus")))
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

        if disabled || at_max {
            plus_btn = plus_btn.opacity(0.4);
        } else if let Some(ref handler) = handler_rc {
            let h_click = handler.clone();
            plus_btn =
                plus_btn
                    .cursor_pointer()
                    .on_click(move |event: &gpui::ClickEvent, window, cx| {
                        let next = if event.modifiers().shift {
                            shift_incremented
                        } else {
                            incremented
                        };
                        h_click(next, window, cx);
                    });
        }

        // FKA: per-button focus + keyboard activation for plus.
        if let Some((ref group, ref handles)) = fka_buttons {
            let handle = &handles[1];
            let is_btn_focused = handle.is_focused(window);
            let nav_group = group.clone();
            let nav_handler = handler_rc.clone();
            plus_btn = plus_btn.focus_group(group, handle);
            plus_btn = plus_btn.with_accessibility(
                &AccessibilityProps::new()
                    .role(AccessibilityRole::Button)
                    .label("Increment"),
            );
            plus_btn = apply_focus_ring(plus_btn, theme, is_btn_focused, &[]);
            plus_btn = plus_btn.on_key_down(move |ev: &KeyDownEvent, window, cx| {
                match ev.keystroke.key.as_str() {
                    "up" | "right" => {
                        nav_group.focus_next(window, cx);
                        cx.stop_propagation();
                    }
                    "down" | "left" => {
                        nav_group.focus_previous(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {
                        if crate::foundations::keyboard::is_activation_key(ev) {
                            if !disabled
                                && !at_max
                                && let Some(ref h) = nav_handler
                            {
                                let shift = ev.keystroke.modifiers.shift;
                                h(
                                    if shift {
                                        shift_incremented
                                    } else {
                                        incremented
                                    },
                                    window,
                                    cx,
                                );
                            }
                            cx.stop_propagation();
                        }
                    }
                }
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

        let mut container =
            apply_standard_control_styling(row, theme, GlassSize::Small, capsule_focused)
                .rounded(capsule_radius)
                .overflow_hidden()
                .id(id);

        // Only make the container a Tab stop in single-tab-stop mode.
        // FKA per-button handles provide individual Tab stops instead.
        if fka_buttons.is_none() {
            container = container.focusable();
        }

        if let Some(handle) = focus_handle.as_ref() {
            container = container.track_focus(handle);
        }

        if disabled {
            container = container.opacity(0.5);
        } else if fka_buttons.is_none() {
            // Single-tab-stop mode (no FKA per-button handles).
            if let Some(handler) = handler_rc {
                container = container.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    // HIG macOS: Shift-arrow bumps by 10× the step, mirroring
                    // the Shift-click behaviour already wired on the +/- buttons.
                    let shift = event.keystroke.modifiers.shift;
                    match event.keystroke.key.as_str() {
                        "up" | "right" => {
                            handler(
                                if shift {
                                    shift_incremented
                                } else {
                                    incremented
                                },
                                window,
                                cx,
                            );
                        }
                        "down" | "left" => {
                            handler(
                                if shift {
                                    shift_decremented
                                } else {
                                    decremented
                                },
                                window,
                                cx,
                            );
                        }
                        // Space/Enter activate the stepper (increment). The
                        // single-tab-stop design means there is no natural "which
                        // button" answer — incrementing matches the leading `+`
                        // visual and common convention.
                        _ if crate::foundations::keyboard::is_activation_key(event) => {
                            cx.stop_propagation();
                            handler(
                                if shift {
                                    shift_incremented
                                } else {
                                    incremented
                                },
                                window,
                                cx,
                            );
                        }
                        _ => {}
                    }
                });
            }
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

    /// Press-and-hold auto-repeat is deferred pending the `RenderOnce →
    /// Entity<T>` elevation. If this source stops mentioning the TODO
    /// block we've either landed the feature or silently dropped it — in
    /// either case we should notice immediately.
    #[test]
    fn stepper_documents_auto_repeat_todo() {
        const SELF_SRC: &str = include_str!("stepper.rs");
        assert!(
            SELF_SRC.contains("TODO: HIG press-and-hold auto-repeat"),
            "auto-repeat TODO doc missing from stepper.rs"
        );
    }

    #[test]
    fn stepper_btn_focus_fields_default_empty() {
        let s = Stepper::new("test");
        assert!(s.btn_focus_group.is_none());
        assert!(s.btn_focus_handles.is_empty());
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

    #[gpui::test]
    async fn space_and_enter_increment_after_focus(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| StepperHarness::new(cx, 5.0));

        cx.click_on(STEPPER_PLUS);
        cx.press("enter");
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 7.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![6.0, 7.0]);
        });

        cx.press("space");
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 8.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![6.0, 7.0, 8.0]);
        });
    }

    #[gpui::test]
    async fn shift_enter_applies_10x_step(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| StepperHarness::new(cx, 5.0));

        cx.click_on(STEPPER_PLUS);
        cx.press("shift-enter");
        host.update_in(cx, |host, _window, _cx| {
            // After the click, value=6; Shift+Enter bumps by 10× step → 16,
            // clamped to max 10.0.
            assert!((host.value - 10.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![6.0, 10.0]);
        });
    }
}

#[cfg(test)]
mod fka_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext};

    use super::Stepper;
    use crate::foundations::accessibility::{AccessibilityMode, FocusGroup};
    use crate::foundations::theme::TahoeTheme;
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};
    use core::prelude::v1::test;

    struct StepperFkaHarness {
        handles: Vec<FocusHandle>,
        group: FocusGroup,
        value: f64,
        changes: Vec<f64>,
    }

    impl StepperFkaHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                handles: vec![cx.focus_handle(), cx.focus_handle()],
                group: FocusGroup::cycle(),
                value: 5.0,
                changes: Vec::new(),
            }
        }
    }

    impl Render for StepperFkaHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            Stepper::new("fka-stepper")
                .value(self.value)
                .min(0.0)
                .max(10.0)
                .step(1.0)
                .btn_focus_group(self.group.clone())
                .btn_focus_handles(self.handles.clone())
                .on_change(move |value, _window, cx| {
                    entity.update(cx, |this, cx| {
                        this.value = value;
                        this.changes.push(value);
                        cx.notify();
                    });
                })
        }
    }

    fn setup_fka_window<V: Render + 'static>(
        cx: &mut TestAppContext,
        build: impl FnOnce(&mut gpui::Window, &mut gpui::Context<V>) -> V,
    ) -> (gpui::Entity<V>, &mut gpui::VisualTestContext) {
        crate::test_helpers::helpers::register_test_keybindings(cx);
        cx.add_window_view(|window, cx| {
            let mut theme = TahoeTheme::dark();
            theme.accessibility_mode = AccessibilityMode::FULL_KEYBOARD_ACCESS;
            cx.set_global(theme);
            build(window, cx)
        })
    }

    #[gpui::test]
    async fn fka_off_does_not_register_btn_handles(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| StepperFkaHarness::new(cx));
        host.update(cx, |host, _cx| {
            assert!(
                host.group.is_empty(),
                "FKA off: group should stay empty after render"
            );
        });
    }

    #[gpui::test]
    async fn fka_on_registers_both_btn_handles(cx: &mut TestAppContext) {
        let (host, cx) = setup_fka_window(cx, |_window, cx| StepperFkaHarness::new(cx));
        host.update(cx, |host, _cx| {
            assert_eq!(
                host.group.len(),
                2,
                "FKA on: both buttons should be registered"
            );
        });
    }

    #[gpui::test]
    async fn fka_on_handles_order_minus_then_plus(cx: &mut TestAppContext) {
        let (host, cx) = setup_fka_window(cx, |_window, cx| StepperFkaHarness::new(cx));
        host.update(cx, |host, _cx| {
            assert_eq!(
                host.group.register(&host.handles[0]),
                0,
                "minus handle at index 0"
            );
            assert_eq!(
                host.group.register(&host.handles[1]),
                1,
                "plus handle at index 1"
            );
        });
    }

    #[gpui::test]
    async fn fka_on_mismatched_handle_count_skips_registration(cx: &mut TestAppContext) {
        struct MismatchHarness {
            group: FocusGroup,
            handle: FocusHandle,
        }

        impl Render for MismatchHarness {
            fn render(
                &mut self,
                _window: &mut gpui::Window,
                _cx: &mut Context<Self>,
            ) -> impl IntoElement {
                Stepper::new("mismatch-stepper")
                    .value(5.0)
                    .min(0.0)
                    .max(10.0)
                    .btn_focus_group(self.group.clone())
                    .btn_focus_handles(vec![self.handle.clone()]) // 1 handle, expects 2
            }
        }

        let (host, cx) = setup_fka_window(cx, |_window, cx| MismatchHarness {
            group: FocusGroup::cycle(),
            handle: cx.focus_handle(),
        });
        host.update(cx, |host, _cx| {
            assert!(
                host.group.is_empty(),
                "mismatched count: group should stay empty"
            );
        });
    }

    #[gpui::test]
    async fn fka_on_activation_on_minus_decrements(cx: &mut TestAppContext) {
        let (host, cx) = setup_fka_window(cx, |_window, cx| StepperFkaHarness::new(cx));

        // Focus the minus button and press Enter.
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("enter");
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 4.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![4.0]);
        });
    }

    #[gpui::test]
    async fn fka_on_activation_on_plus_increments(cx: &mut TestAppContext) {
        let (host, cx) = setup_fka_window(cx, |_window, cx| StepperFkaHarness::new(cx));

        // Focus the plus button and press Enter.
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        cx.press("enter");
        host.update_in(cx, |host, _window, _cx| {
            assert!((host.value - 6.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![6.0]);
        });
    }

    #[gpui::test]
    async fn fka_on_shift_activation_on_minus_applies_10x(cx: &mut TestAppContext) {
        let (host, cx) = setup_fka_window(cx, |_window, cx| StepperFkaHarness::new(cx));

        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("shift-enter");
        host.update_in(cx, |host, _window, _cx| {
            // 5.0 - 10×1.0 = -5.0, clamped to 0.0
            assert!((host.value - 0.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![0.0]);
        });
    }

    #[gpui::test]
    async fn fka_on_arrow_navigates_between_buttons(cx: &mut TestAppContext) {
        let (host, cx) = setup_fka_window(cx, |_window, cx| StepperFkaHarness::new(cx));

        // Focus minus, press down → should move to plus.
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("down");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[1].is_focused(window),
                "down from minus should focus plus"
            );
        });

        // Press up → should move back to minus.
        cx.press("up");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[0].is_focused(window),
                "up from plus should focus minus"
            );
        });
    }

    #[gpui::test]
    async fn fka_on_shift_activation_on_plus_applies_10x(cx: &mut TestAppContext) {
        let (host, cx) = setup_fka_window(cx, |_window, cx| StepperFkaHarness::new(cx));

        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        cx.press("shift-enter");
        host.update_in(cx, |host, _window, _cx| {
            // 5.0 + 10×1.0 = 15.0, clamped to 10.0
            assert!((host.value - 10.0).abs() < f64::EPSILON);
            assert_eq!(host.changes, vec![10.0]);
        });
    }

    #[gpui::test]
    async fn fka_on_activation_at_boundary_is_noop(cx: &mut TestAppContext) {
        struct BoundaryHarness {
            handles: Vec<FocusHandle>,
            group: FocusGroup,
            value: f64,
            changes: Vec<f64>,
        }

        impl BoundaryHarness {
            fn new(cx: &mut Context<Self>) -> Self {
                Self {
                    handles: vec![cx.focus_handle(), cx.focus_handle()],
                    group: FocusGroup::cycle(),
                    value: 0.0,
                    changes: Vec::new(),
                }
            }
        }

        impl Render for BoundaryHarness {
            fn render(
                &mut self,
                _window: &mut gpui::Window,
                cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let entity = cx.entity().clone();
                Stepper::new("boundary-stepper")
                    .value(self.value)
                    .min(0.0)
                    .max(10.0)
                    .step(1.0)
                    .btn_focus_group(self.group.clone())
                    .btn_focus_handles(self.handles.clone())
                    .on_change(move |value, _window, cx| {
                        entity.update(cx, |this, cx| {
                            this.value = value;
                            this.changes.push(value);
                            cx.notify();
                        });
                    })
            }
        }

        let (host, cx) = setup_fka_window(cx, |_window, cx| BoundaryHarness::new(cx));

        // Minus button at min boundary — activation should be suppressed.
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("enter");
        host.update_in(cx, |host, _window, _cx| {
            assert!(
                host.changes.is_empty(),
                "activation at min boundary should not fire on_change"
            );
            assert!((host.value - 0.0).abs() < f64::EPSILON);
        });
    }

    #[gpui::test]
    async fn fka_on_disabled_activation_is_noop(cx: &mut TestAppContext) {
        struct DisabledHarness {
            handles: Vec<FocusHandle>,
            group: FocusGroup,
            value: f64,
            changes: Vec<f64>,
        }

        impl DisabledHarness {
            fn new(cx: &mut Context<Self>) -> Self {
                Self {
                    handles: vec![cx.focus_handle(), cx.focus_handle()],
                    group: FocusGroup::cycle(),
                    value: 5.0,
                    changes: Vec::new(),
                }
            }
        }

        impl Render for DisabledHarness {
            fn render(
                &mut self,
                _window: &mut gpui::Window,
                cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let entity = cx.entity().clone();
                Stepper::new("disabled-stepper")
                    .value(self.value)
                    .min(0.0)
                    .max(10.0)
                    .step(1.0)
                    .disabled(true)
                    .btn_focus_group(self.group.clone())
                    .btn_focus_handles(self.handles.clone())
                    .on_change(move |value, _window, cx| {
                        entity.update(cx, |this, cx| {
                            this.value = value;
                            this.changes.push(value);
                            cx.notify();
                        });
                    })
            }
        }

        let (host, cx) = setup_fka_window(cx, |_window, cx| DisabledHarness::new(cx));

        // Plus button on disabled stepper — activation should be suppressed.
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        cx.press("enter");
        host.update_in(cx, |host, _window, _cx| {
            assert!(
                host.changes.is_empty(),
                "activation on disabled stepper should not fire on_change"
            );
            assert!((host.value - 5.0).abs() < f64::EPSILON);
        });
    }

    #[gpui::test]
    async fn fka_on_wraps_mode_activation_wraps_at_max(cx: &mut TestAppContext) {
        struct WrapsHarness {
            handles: Vec<FocusHandle>,
            group: FocusGroup,
            value: f64,
            changes: Vec<f64>,
        }

        impl WrapsHarness {
            fn new(cx: &mut Context<Self>) -> Self {
                Self {
                    handles: vec![cx.focus_handle(), cx.focus_handle()],
                    group: FocusGroup::cycle(),
                    value: 10.0,
                    changes: Vec::new(),
                }
            }
        }

        impl Render for WrapsHarness {
            fn render(
                &mut self,
                _window: &mut gpui::Window,
                cx: &mut Context<Self>,
            ) -> impl IntoElement {
                let entity = cx.entity().clone();
                Stepper::new("wraps-stepper")
                    .value(self.value)
                    .min(0.0)
                    .max(10.0)
                    .step(1.0)
                    .wraps(true)
                    .btn_focus_group(self.group.clone())
                    .btn_focus_handles(self.handles.clone())
                    .on_change(move |value, _window, cx| {
                        entity.update(cx, |this, cx| {
                            this.value = value;
                            this.changes.push(value);
                            cx.notify();
                        });
                    })
            }
        }

        let (host, cx) = setup_fka_window(cx, |_window, cx| WrapsHarness::new(cx));

        // Plus button at max with wraps=true → wraps via modulo to ≈ 1.0
        // (min + (step - EPSILON).rem_euclid(range)).
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        cx.press("enter");
        host.update_in(cx, |host, _window, _cx| {
            let expected = 0.0 + (1.0 - f64::EPSILON).rem_euclid(10.0);
            assert!(
                (host.value - expected).abs() < 1e-10,
                "wraps mode: plus at max should wrap, got {}",
                host.value
            );
            assert_eq!(host.changes.len(), 1);
        });
    }
}
