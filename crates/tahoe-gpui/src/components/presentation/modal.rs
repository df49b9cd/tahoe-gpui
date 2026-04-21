//! Modal dialog component per HIG `#modality`.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/modality>

use crate::foundations::accessibility::{FocusGroup, FocusGroupMode};
use crate::foundations::layout::{MODAL_MAX_HEIGHT, MODAL_WIDTH};
use crate::foundations::motion::accessible_transition_animation;
use crate::foundations::theme::{ActiveTheme, GlassSize};
use gpui::prelude::*;
use gpui::{
    AnimationExt, AnyElement, AnyEntity, App, DismissEvent, ElementId, EventEmitter, FocusHandle,
    KeyDownEvent, MouseDownEvent, Pixels, Window, div, px,
};

/// Scope of the modal blocking behavior per HIG `#modality`.
///
/// HIG distinguishes **window-modal** surfaces (a sheet attached to a
/// single window, blocking interaction only with that window) from
/// **app-modal** surfaces (blocking every window in the app).
///
/// Today both levels render the same visual chrome (centered panel with
/// full-screen backdrop). The level is still surfaced so callers can
/// express intent and future revisions can render window-modals as
/// AppKit sheets attached to the parent window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ModalLevel {
    /// Blocks a single window. Pair with a [`FocusHandle`] tied to the
    /// owning window.
    #[default]
    Window,
    /// Blocks the entire app. Use sparingly per HIG: "Prefer a
    /// nonmodal alternative when possible."
    App,
}

/// Type-erased emit callback used by [`Modal::dismiss_emitter`] to
/// dispatch [`DismissEvent`] on the registered entity without the
/// caller threading the entity's type through `Modal`.
type DismissEmitFn = std::rc::Rc<dyn Fn(&AnyEntity, &mut Window, &mut App) + 'static>;

/// A centered modal dialog with backdrop.
///
/// The parent manages `is_open` state and provides an `on_dismiss` callback.
/// Uses GPUI's `overlay()` for rendering above other content.
///
/// # Event-emitter pattern (Finding 5 in the Zed cross-reference audit)
///
/// Zed's `Modal` emits [`DismissEvent`] via `EventEmitter` so parents
/// subscribe reactively with `cx.subscribe(&modal, Self::on_dismiss)`
/// instead of capturing a closure. `tahoe-gpui`'s Modal stays a
/// stateless `RenderOnce` builder so existing callers don't need an
/// `Entity<Modal>` to use it, but the event-emitter pattern is also
/// supported: pass any entity that `impl EventEmitter<DismissEvent>` via
/// [`Modal::dismiss_emitter`] and the modal will emit `DismissEvent` on
/// it on every dismissal path (backdrop click, Escape, Tab-trap escape).
/// Subscribers can then use the idiomatic Zed-style subscription:
///
/// ```ignore
/// cx.subscribe(&modal_host, Self::on_modal_dismiss).detach();
/// ```
///
/// The closure-based [`Modal::on_dismiss`] continues to work and can be
/// combined with the emitter — both fire if both are configured.
use crate::callback_types::OnMutCallback;
#[derive(IntoElement)]
pub struct Modal {
    id: ElementId,
    is_open: bool,
    content: AnyElement,
    width: Option<Pixels>,
    on_dismiss: OnMutCallback,
    focus_handle: Option<FocusHandle>,
    scroll: bool,
    /// Child focus handles (in tab order) wrapped in a Trap-mode
    /// [`FocusGroup`]. Tab / Shift+Tab cycle through the registered
    /// handles with wrap-around per the WAI-ARIA dialog pattern; the
    /// group's `handle_key_down` helper consumes Tab so focus cannot
    /// escape the modal surface.
    focus_group: FocusGroup,
    /// Focus handle to restore when the modal dismisses. HIG: "Return focus
    /// to a sensible location after modal dismissal."
    restore_focus_to: Option<FocusHandle>,
    level: ModalLevel,
    /// Type-erased handle to an `Entity<T>` where `T: EventEmitter<DismissEvent>`.
    /// Paired with `dismiss_emit_fn` so the render path can dispatch
    /// `cx.emit(DismissEvent)` through the erased entity without the
    /// surrounding struct generics. Finding 5 in the Zed cross-reference audit.
    dismiss_emitter: Option<AnyEntity>,
    dismiss_emit_fn: Option<DismissEmitFn>,
}

impl Modal {
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            is_open: false,
            content: content.into_any_element(),
            width: None,
            on_dismiss: None,
            focus_handle: None,
            scroll: true,
            focus_group: FocusGroup::trap(),
            restore_focus_to: None,
            level: ModalLevel::default(),
            dismiss_emitter: None,
            dismiss_emit_fn: None,
        }
    }

    /// Declare the modal's scope (window-modal vs app-modal) per HIG
    /// `#modality`. Defaults to [`ModalLevel::Window`].
    pub fn level(mut self, level: ModalLevel) -> Self {
        self.level = level;
        self
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Control whether the modal content scrolls. Default is `true`.
    /// Set to `false` when the content manages its own scrolling internally.
    pub fn scroll(mut self, scroll: bool) -> Self {
        self.scroll = scroll;
        self
    }

    /// Register the focus handles of interactive children in tab order.
    ///
    /// When the user presses Tab past the last handle, focus returns to the
    /// first; Shift+Tab past the first returns to the last. Without this,
    /// `stop_propagation()` on Tab would only prevent focus from escaping the
    /// modal but leave the focus chain stalled — keyboard users would be
    /// stuck on whatever element they landed on.
    ///
    /// Internally allocates a fresh Trap-mode [`FocusGroup`]. Calling this
    /// after [`Modal::focus_group`] is **last-write-wins**: the caller's
    /// shared group is discarded and replaced with the handles passed here.
    /// Callers that already manage a [`FocusGroup`] should use
    /// [`Modal::focus_group`] instead.
    pub fn focus_cycle(mut self, handles: Vec<FocusHandle>) -> Self {
        self.focus_group = FocusGroup::trap();
        for handle in &handles {
            self.focus_group.register(handle);
        }
        self
    }

    /// Attach a caller-managed [`FocusGroup`] driving the modal's Tab
    /// trap. Overrides any handles previously supplied via
    /// [`Modal::focus_cycle`]. Use when the host already maintains a
    /// FocusGroup (e.g. for arrow-key navigation inside the modal's
    /// content) and wants the same group to drive the Tab trap.
    ///
    /// The group **must** be in [`FocusGroupMode::Trap`] mode. When it
    /// is not, the modal's Tab handler still swallows Tab (to prevent
    /// focus escaping) but does not advance focus — keyboard users get
    /// stranded on whatever element they landed on. A `debug_assert!`
    /// fires in debug builds to catch this early.
    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        debug_assert_eq!(
            group.mode(),
            FocusGroupMode::Trap,
            "Modal::focus_group requires a Trap-mode FocusGroup; other \
             modes cause Tab to be swallowed without advancing focus, \
             stranding keyboard users."
        );
        self.focus_group = group;
        self
    }

    /// Record the focus handle that should regain focus when the modal
    /// dismisses. Call with the currently-focused element *before* opening
    /// the modal so the user returns to the same position on dismiss.
    pub fn restore_focus_to(mut self, handle: FocusHandle) -> Self {
        self.restore_focus_to = Some(handle);
        self
    }

    /// Register an entity that will receive a [`DismissEvent`] on every
    /// dismissal path. Matches Zed's `EventEmitter<DismissEvent>`
    /// convention so parents can wire `cx.subscribe(&emitter,
    /// Self::on_dismiss).detach()` instead of capturing a closure.
    ///
    /// The entity must implement `EventEmitter<DismissEvent>`. Any existing
    /// [`Modal::on_dismiss`] callback still fires — the emitter is
    /// additive, not a replacement. Finding 5 in the Zed cross-reference audit.
    ///
    /// ```ignore
    /// struct MySheet;
    /// impl gpui::EventEmitter<gpui::DismissEvent> for MySheet {}
    ///
    /// let emitter = cx.new(|_| MySheet);
    /// cx.subscribe(&emitter, |_this, _emitter, _ev: &gpui::DismissEvent, cx| {
    ///     // close the modal in your state
    ///     cx.notify();
    /// }).detach();
    ///
    /// Modal::new("confirm", body)
    ///     .open(true)
    ///     .dismiss_emitter(&emitter)
    /// ```
    pub fn dismiss_emitter<T>(mut self, entity: &gpui::Entity<T>) -> Self
    where
        T: EventEmitter<DismissEvent> + 'static,
    {
        let any_entity: AnyEntity = entity.clone().into();
        self.dismiss_emitter = Some(any_entity);
        // Capture the concrete `T` in the emit closure so the erased
        // entity can still be downcast at render time without the caller
        // threading generics through `Modal`.
        self.dismiss_emit_fn = Some(std::rc::Rc::new(
            move |any_entity: &AnyEntity, _window: &mut Window, cx: &mut App| {
                if let Ok(typed) = any_entity.clone().downcast::<T>() {
                    typed.update(cx, |_this, cx| cx.emit(DismissEvent));
                }
            },
        ));
        self
    }
}

impl RenderOnce for Modal {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        // Focus trap: mint a focus handle if the parent didn't provide one and
        // request focus when the modal opens so subsequent Tab/Shift+Tab events
        // reach the modal's key handler and can be contained. When the caller
        // registered focusable children (via `focus_cycle` / `focus_group`),
        // land initial focus on the first member per the WAI-ARIA dialog
        // pattern; otherwise focus the outer container so Tab still reaches
        // the modal's key handler. Once a child in the group takes focus we
        // leave it alone — otherwise the next render would steal focus back
        // to the container and break the cycle.
        let focus_handle = self
            .focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle());
        let any_modal_focus =
            focus_handle.is_focused(window) || self.focus_group.contains_focused(window);
        if !any_modal_focus {
            if self.focus_group.is_empty() {
                focus_handle.focus(window, cx);
            } else {
                self.focus_group.focus_first(window, cx);
            }
        }

        let theme = cx.theme();

        let width = self.width.unwrap_or(px(MODAL_WIDTH));

        let level_selector = match self.level {
            ModalLevel::Window => "modal-backdrop",
            ModalLevel::App => "modal-backdrop-app",
        };
        let backdrop = crate::foundations::materials::backdrop_overlay(theme)
            .id(self.id.clone())
            .debug_selector(move || level_selector.into())
            .flex()
            .items_center()
            .justify_center();

        // Wrap the caller's on_dismiss so every dismissal path (backdrop
        // click, Escape) restores focus to `restore_focus_to` AND emits
        // `DismissEvent` on the caller's subscribed entity (Finding 5)
        // before invoking the closure handler. The Rc shape lets the
        // wrapped callback be shared between the two dismissal entry
        // points.
        type DismissFn = dyn Fn(&mut Window, &mut App) + 'static;
        let restore = self.restore_focus_to.clone();
        let dismiss_emitter = self.dismiss_emitter.clone();
        let dismiss_emit_fn = self.dismiss_emit_fn.clone();
        let inner_dismiss = self.on_dismiss;

        let on_dismiss_rc: Option<std::rc::Rc<Box<DismissFn>>> =
            if inner_dismiss.is_some() || dismiss_emitter.is_some() {
                let wrapped: Box<DismissFn> = Box::new(move |window: &mut Window, cx: &mut App| {
                    if let Some(handle) = restore.as_ref() {
                        handle.focus(window, cx);
                    }
                    // Event-emitter path (Finding 5): subscribers wired
                    // via `cx.subscribe` receive DismissEvent here.
                    if let (Some(emitter), Some(emit_fn)) =
                        (dismiss_emitter.as_ref(), dismiss_emit_fn.as_ref())
                    {
                        emit_fn(emitter, window, cx);
                    }
                    // Closure path: for callers that pass a dismiss
                    // handler directly via the `on_dismiss` builder.
                    if let Some(ref inner) = inner_dismiss {
                        inner(window, cx);
                    }
                });
                Some(std::rc::Rc::new(wrapped))
            } else {
                None
            };

        // Build content container with optional focus tracking and escape key support
        let content_id =
            ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "content".into());
        let mut content_div = crate::foundations::materials::glass_surface(
            div()
                .w(width)
                .max_h(px(MODAL_MAX_HEIGHT))
                .overflow_x_hidden(),
            theme,
            GlassSize::Large,
        )
        .id(content_id)
        .debug_selector(|| "modal-content".into());

        if self.scroll {
            content_div = content_div.overflow_y_scroll();
        } else {
            content_div = content_div.overflow_y_hidden();
        }

        if let Some(ref handler) = on_dismiss_rc {
            let handler = handler.clone();
            content_div =
                content_div.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
                    handler(window, cx);
                });
        }

        content_div = content_div.track_focus(&focus_handle);

        // Key handler: Escape / Cmd-. dismiss per HIG `#modality`; Tab /
        // Shift+Tab routed through the modal's Trap-mode `FocusGroup`
        // (`handle_key_down` consumes the event and wraps through the
        // registered handles). When the group is empty we still swallow
        // Tab so focus cannot escape the modal surface — the "no-op trap"
        // required by the WAI-ARIA dialog pattern when no child is
        // focusable.
        let dismiss_for_keys = on_dismiss_rc.clone();
        let focus_group = self.focus_group.clone();
        content_div = content_div.on_key_down(move |event: &KeyDownEvent, window, cx| {
            let modifiers = &event.keystroke.modifiers;
            let is_cmd_period = modifiers.platform && event.keystroke.key.as_str() == ".";
            if crate::foundations::keyboard::is_escape_key(event) || is_cmd_period {
                if let Some(handler) = &dismiss_for_keys {
                    handler(window, cx);
                }
                return;
            }
            if event.keystroke.key.as_str() == "tab" {
                // In Trap mode with members, this wraps focus; with no
                // members it still stops propagation so the dialog pattern
                // is preserved.
                if !focus_group.handle_key_down(event, window, cx) {
                    cx.stop_propagation();
                }
            }
        });

        // Present-transition: HIG specifies a cross-fade / scale-in for
        // centered modals (`foundations.md:1096`). We cross-fade only the
        // inner body rather than the tracked `content_div` so focus,
        // escape/tab handlers, and mouse-down-out guards remain attached
        // to the element GPUI's focus chain expects.
        // `accessible_transition_animation` handles Reduce Motion (short
        // linear cross-fade) and Prefer Cross-Fade (linear opacity at the
        // natural spring duration) in one place.
        let anim_id = ElementId::NamedChild(std::sync::Arc::new(self.id.clone()), "present".into());
        let natural_duration =
            std::time::Duration::from_millis(theme.glass.motion.lift_duration_ms);
        let animated_body = div().child(self.content).with_animation(
            anim_id,
            accessible_transition_animation(
                &theme.glass.motion,
                natural_duration,
                theme.accessibility_mode,
            ),
            |el, delta| el.opacity(delta),
        );

        backdrop
            .child(content_div.child(animated_body))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::Modal;
    use core::prelude::v1::test;

    #[test]
    fn modal_default_is_closed() {
        let modal = Modal::new("test", gpui::div());
        assert!(!modal.is_open);
    }

    #[test]
    fn modal_width_default_is_none() {
        let modal = Modal::new("test", gpui::div());
        assert_eq!(modal.width, None);
    }

    #[test]
    fn modal_on_dismiss_default_is_none() {
        let modal = Modal::new("test", gpui::div());
        assert!(modal.on_dismiss.is_none());
    }

    #[test]
    fn modal_scroll_default_is_true() {
        let modal = Modal::new("test", gpui::div());
        assert!(modal.scroll);
    }

    #[test]
    fn modal_focus_cycle_defaults_to_empty() {
        let modal = Modal::new("test", gpui::div());
        assert!(modal.focus_group.is_empty());
    }

    #[test]
    fn modal_focus_group_defaults_to_trap_mode() {
        let modal = Modal::new("test", gpui::div());
        assert_eq!(modal.focus_group.mode(), super::FocusGroupMode::Trap);
    }

    #[test]
    #[should_panic(expected = "Modal::focus_group requires a Trap-mode FocusGroup")]
    fn modal_focus_group_rejects_non_trap_mode() {
        use super::FocusGroup;
        let cycle = FocusGroup::cycle();
        let _ = Modal::new("test", gpui::div()).focus_group(cycle);
    }

    #[test]
    fn modal_restore_focus_default_is_none() {
        let modal = Modal::new("test", gpui::div());
        assert!(modal.restore_focus_to.is_none());
    }

    #[test]
    fn modal_level_default_is_window() {
        let modal = Modal::new("test", gpui::div());
        assert_eq!(modal.level, super::ModalLevel::Window);
    }

    #[test]
    fn modal_level_builder() {
        let modal = Modal::new("test", gpui::div()).level(super::ModalLevel::App);
        assert_eq!(modal.level, super::ModalLevel::App);
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    use super::Modal;
    use crate::test_helpers::helpers::{
        InteractionExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const MODAL_BACKDROP: &str = "modal-backdrop";
    const MODAL_CONTENT: &str = "modal-content";

    struct ModalHarness {
        focus_handle: FocusHandle,
        is_open: bool,
        dismiss_count: usize,
    }

    impl ModalHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                focus_handle: cx.focus_handle(),
                is_open: true,
                dismiss_count: 0,
            }
        }
    }

    impl Render for ModalHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            Modal::new("modal", div().w(px(160.0)).h(px(80.0)).child("Modal body"))
                .open(self.is_open)
                .focus_handle(self.focus_handle.clone())
                .on_dismiss(move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.dismiss_count += 1;
                        this.is_open = false;
                        cx.notify();
                    });
                })
        }
    }

    fn focus_modal(host: &gpui::Entity<ModalHarness>, cx: &mut gpui::VisualTestContext) {
        host.update_in(cx, |host, window, cx| {
            host.focus_handle.focus(window, cx);
        });
    }

    #[gpui::test]
    async fn outside_click_dismisses_modal(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ModalHarness::new(cx));

        assert_element_exists(cx, MODAL_CONTENT);
        cx.click_within(MODAL_BACKDROP, 0.05, 0.05);

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.dismiss_count, 1);
            assert!(!host.is_open);
        });
        assert_element_absent(cx, MODAL_CONTENT);
    }

    #[gpui::test]
    async fn inside_click_does_not_dismiss_modal(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ModalHarness::new(cx));

        cx.click_on(MODAL_CONTENT);

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.dismiss_count, 0);
            assert!(host.is_open);
        });
        assert_element_exists(cx, MODAL_CONTENT);
    }

    // Harness for Tab-cycle behaviour: the modal is told about two focusable
    // child handles and we verify Tab cycles forward through both, then
    // wraps to the first on a third press.
    struct TabCycleHarness {
        outer_focus: FocusHandle,
        first: FocusHandle,
        second: FocusHandle,
    }

    impl TabCycleHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                outer_focus: cx.focus_handle(),
                first: cx.focus_handle(),
                second: cx.focus_handle(),
            }
        }
    }

    impl Render for TabCycleHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            // The focus cycle handles have to be attached to rendered elements
            // for key events to reach the modal's `on_key_down` handler (GPUI
            // falls back to the root dispatch node otherwise). Two stateful
            // divs that `track_focus` each handle stand in for real interactive
            // children (Button, TextField) in production usage.
            let body = div()
                .w(px(160.0))
                .h(px(80.0))
                .flex()
                .flex_col()
                .child(div().id("first").track_focus(&self.first).child("First"))
                .child(div().id("second").track_focus(&self.second).child("Second"));
            Modal::new("modal", body)
                .open(true)
                .focus_handle(self.outer_focus.clone())
                .focus_cycle(vec![self.first.clone(), self.second.clone()])
                .on_dismiss(|_, _| {})
        }
    }

    #[gpui::test]
    async fn tab_cycles_focus_forward(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| TabCycleHarness::new(cx));

        // Focus the modal's content first so its on_key_down handler is live.
        host.update_in(cx, |host, window, cx| {
            host.outer_focus.focus(window, cx);
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.first.is_focused(window), "first Tab lands on first");
        });

        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.second.is_focused(window), "second Tab lands on second");
        });

        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.first.is_focused(window),
                "third Tab wraps back to first"
            );
        });
    }

    #[gpui::test]
    async fn shift_tab_cycles_focus_backward(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| TabCycleHarness::new(cx));

        host.update_in(cx, |host, window, cx| {
            host.outer_focus.focus(window, cx);
        });

        cx.press("shift-tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.second.is_focused(window),
                "Shift+Tab with no prior cycle focus lands on last"
            );
        });

        cx.press("shift-tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.first.is_focused(window),
                "Shift+Tab moves backward to first"
            );
        });
    }

    #[gpui::test]
    async fn initial_focus_lands_on_first_focus_group_member(cx: &mut TestAppContext) {
        // WAI-ARIA dialog pattern: on open, focus the first focusable
        // child. The harness hands the modal a populated focus group but
        // never focuses the outer handle — opening the modal must land
        // focus on the first registered member on its own.
        let (host, cx) = setup_test_window(cx, |_window, cx| TabCycleHarness::new(cx));

        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.first.is_focused(window),
                "initial render should focus the first focus-group member"
            );
            assert!(
                !host.outer_focus.is_focused(window),
                "outer container must not steal focus when a member exists"
            );
        });
    }

    // Harness that wires a `restore_focus_to` handle and lets us observe
    // focus returning on dismiss. The `previous` handle mimics the
    // app-level control the user was on before the modal opened.
    struct RestoreFocusHarness {
        modal_focus: FocusHandle,
        previous: FocusHandle,
        is_open: bool,
    }

    impl RestoreFocusHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                modal_focus: cx.focus_handle(),
                previous: cx.focus_handle(),
                is_open: true,
            }
        }
    }

    impl Render for RestoreFocusHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            let body = div()
                .id("prev-anchor")
                .track_focus(&self.previous)
                .w(px(160.0))
                .h(px(80.0))
                .child("Modal body");
            Modal::new("modal", body)
                .open(self.is_open)
                .focus_handle(self.modal_focus.clone())
                .restore_focus_to(self.previous.clone())
                .on_dismiss(move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.is_open = false;
                        cx.notify();
                    });
                })
        }
    }

    #[gpui::test]
    async fn escape_restores_focus_to_previous_element(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| RestoreFocusHarness::new(cx));

        host.update_in(cx, |host, window, cx| {
            host.modal_focus.focus(window, cx);
            assert!(host.modal_focus.is_focused(window));
        });
        cx.press("escape");

        host.update_in(cx, |host, window, _cx| {
            assert!(!host.is_open);
            assert!(
                host.previous.is_focused(window),
                "focus should be restored to `previous` after dismiss"
            );
        });
    }

    #[gpui::test]
    async fn escape_dismisses_modal(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ModalHarness::new(cx));

        focus_modal(&host, cx);
        cx.press("escape");

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.dismiss_count, 1);
            assert!(!host.is_open);
        });
        assert_element_absent(cx, MODAL_CONTENT);
    }

    // Harness verifying Finding 5 — Modal emits DismissEvent on the
    // subscribed entity alongside the `on_dismiss` callback.
    struct DismissEmitterEntity {
        events: usize,
    }
    impl gpui::EventEmitter<gpui::DismissEvent> for DismissEmitterEntity {}

    struct EmitterHarness {
        focus_handle: FocusHandle,
        is_open: bool,
        closure_dismisses: usize,
        emitter: gpui::Entity<DismissEmitterEntity>,
        _subscription: gpui::Subscription,
    }

    impl EmitterHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            let emitter = cx.new(|_| DismissEmitterEntity { events: 0 });
            let sub = cx.subscribe(
                &emitter,
                |_this, emitter, _event: &gpui::DismissEvent, cx| {
                    emitter.update(cx, |e, _| e.events += 1);
                },
            );
            Self {
                focus_handle: cx.focus_handle(),
                is_open: true,
                closure_dismisses: 0,
                emitter,
                _subscription: sub,
            }
        }
    }

    impl Render for EmitterHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            Modal::new("modal", div().w(px(160.0)).h(px(80.0)).child("Modal body"))
                .open(self.is_open)
                .focus_handle(self.focus_handle.clone())
                .dismiss_emitter(&self.emitter)
                .on_dismiss(move |_, cx| {
                    entity.update(cx, |this, cx| {
                        this.closure_dismisses += 1;
                        this.is_open = false;
                        cx.notify();
                    });
                })
        }
    }

    #[gpui::test]
    async fn dismiss_emitter_fires_alongside_closure(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| EmitterHarness::new(cx));

        // Trigger dismissal via Escape.
        host.update_in(cx, |host, window, cx| {
            host.focus_handle.focus(window, cx);
        });
        cx.press("escape");

        host.update_in(cx, |host, _window, cx| {
            // Both the closure and the emitter subscriber must have run.
            assert_eq!(host.closure_dismisses, 1, "closure on_dismiss should fire");
            host.emitter.update(cx, |e, _| {
                assert_eq!(e.events, 1, "DismissEvent should be emitted exactly once");
            });
            assert!(!host.is_open);
        });
    }

    #[gpui::test]
    async fn tab_in_empty_focus_group_does_not_escape_modal(cx: &mut TestAppContext) {
        // WAI-ARIA dialog pattern: Tab must not escape a modal even when no
        // focusable children are registered. The modal's Tab handler
        // unconditionally calls `stop_propagation()` when the FocusGroup has
        // nothing to cycle through — we verify here that after a Tab press
        // the modal stays mounted and focus does not leave its outer
        // container.
        let (host, cx) = setup_test_window(cx, |_window, cx| ModalHarness::new(cx));
        focus_modal(&host, cx);
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.focus_handle.is_focused(window),
                "precondition: modal outer handle is focused"
            );
        });

        cx.press("tab");

        host.update_in(cx, |host, window, _cx| {
            assert!(host.is_open, "Tab must not trigger dismissal");
            assert!(
                host.focus_handle.is_focused(window),
                "Tab with an empty FocusGroup must leave outer focus put"
            );
        });
        assert_element_exists(cx, MODAL_CONTENT);
    }

    // Harness that hands the modal an externally-owned `FocusGroup` so tests
    // can observe that registration through the shared handle (via
    // `FocusGroupExt::focus_group`) is visible on the group the modal stores
    // — i.e. both clones share inner state. Demonstrates the pattern a host
    // would use when it already owns a FocusGroup for arrow-key nav.
    struct SharedGroupHarness {
        modal_focus: FocusHandle,
        shared_group: crate::foundations::accessibility::FocusGroup,
        first: FocusHandle,
        second: FocusHandle,
    }

    impl SharedGroupHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                modal_focus: cx.focus_handle(),
                shared_group: crate::foundations::accessibility::FocusGroup::trap(),
                first: cx.focus_handle(),
                second: cx.focus_handle(),
            }
        }
    }

    impl Render for SharedGroupHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            use crate::foundations::accessibility::FocusGroupExt;
            let body = div()
                .w(px(160.0))
                .h(px(80.0))
                .flex()
                .flex_col()
                .child(
                    div()
                        .id("shared-first")
                        .focus_group(&self.shared_group, &self.first)
                        .child("First"),
                )
                .child(
                    div()
                        .id("shared-second")
                        .focus_group(&self.shared_group, &self.second)
                        .child("Second"),
                );
            Modal::new("modal", body)
                .open(true)
                .focus_handle(self.modal_focus.clone())
                .focus_group(self.shared_group.clone())
                .on_dismiss(|_, _| {})
        }
    }

    #[gpui::test]
    async fn modal_focus_group_override_shares_inner(cx: &mut TestAppContext) {
        // Registrations through the caller's shared clone of the group must
        // be visible inside the modal's Tab trap — Rc<RefCell<_>> identity
        // is what makes caller-managed arrow-key nav and modal Tab traversal
        // walk the same ordered membership.
        let (host, cx) = setup_test_window(cx, |_window, cx| SharedGroupHarness::new(cx));
        // First render through SharedGroupHarness wired both children
        // through FocusGroupExt::focus_group, so the group should hold
        // exactly 2 members.
        host.update(cx, |host, _cx| {
            assert_eq!(
                host.shared_group.len(),
                2,
                "external handle observes registrations from the render path"
            );
        });

        // Tab inside the modal must cycle through the same members.
        host.update_in(cx, |host, window, cx| {
            host.modal_focus.focus(window, cx);
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.first.is_focused(window), "Tab lands on shared first");
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.second.is_focused(window), "Tab lands on shared second");
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.first.is_focused(window),
                "Tab wraps back to shared first"
            );
        });
    }

    #[gpui::test]
    async fn modal_focus_cycle_after_focus_group_resets(cx: &mut TestAppContext) {
        // Last-write-wins contract: calling `focus_cycle` after
        // `focus_group` replaces the external group with a fresh Trap-mode
        // group populated only with the passed handles. We exercise the
        // builder directly (no render) to keep this test independent of
        // GPUI's focus machinery — the render path is covered by
        // `modal_focus_group_override_shares_inner`.
        use crate::foundations::accessibility::FocusGroup;
        cx.update(|cx| {
            let handle_a = cx.focus_handle();
            let handle_b = cx.focus_handle();
            let external = FocusGroup::trap();
            external.register(&handle_a);

            let modal = Modal::new("test", div())
                .focus_group(external.clone())
                .focus_cycle(vec![handle_b.clone()]);

            // External still holds the original handle; modal's group was
            // replaced by a fresh one that contains only `handle_b`.
            assert_eq!(external.len(), 1, "external group is untouched");
            assert_eq!(modal.focus_group.len(), 1, "modal has a fresh group");
            // Mutating the external group must not affect the modal's.
            external.clear();
            assert_eq!(external.len(), 0);
            assert_eq!(
                modal.focus_group.len(),
                1,
                "modal's group is a different Rc — unaffected by external.clear()"
            );
        });
    }
}
