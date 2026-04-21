//! Sheet component (HIG `#sheets`).
//!
//! Distinct from [`Modal`](super::modal::Modal). The rendered chrome
//! varies by platform:
//!
//! - **iOS / iPadOS / watchOS** — bottom-anchored drawer that slides up
//!   from the bottom edge with a drag indicator pill and detent
//!   heights (medium / large / custom fraction).
//! - **macOS / visionOS** — cardlike centered panel that floats on
//!   top of the parent window. HIG `#sheets` macOS: "a sheet is a
//!   cardlike view with rounded corners that floats on top of its
//!   parent window." No drag indicator and no bottom anchoring; the
//!   parent window dims behind the sheet.
//!
//! The variant is selected automatically from [`TahoeTheme::platform`]
//! or explicitly via [`Sheet::presentation`].
//!
//! # macOS implementation note
//!
//! The current macOS [`SheetPresentation::Cardlike`] variant renders a
//! centered panel with a dimming backdrop inside the parent GPUI
//! window. It is **not** a true NSWindow title-bar-attached sheet (the
//! native AppKit behavior where the sheet slides out from the window
//! chrome and shares the parent window's title bar). Wiring that up
//! requires AppKit sheet APIs (`beginSheet:completionHandler:` etc.)
//! which GPUI does not currently expose. Once GPUI surfaces the
//! underlying NSWindow so callers can attach sheets natively, this
//! variant should be upgraded to match AppKit behavior.
//!
//! # Hosting
//!
//! `Sheet` is a stateless builder — it does not own its `is_open`
//! state. Hosts should put a `bool is_open` (and, on macOS, an
//! `Option<ActiveModal>` to enforce HIG "don't nest modals") on the
//! parent [`Entity`](gpui::Entity) and drive the sheet from there.
//! Acquire the modality slot via
//! [`ModalGuard::global().present()`](crate::patterns::modality::ModalGuard::present)
//! at the same entity that owns `is_open`, then drop the returned
//! [`ActiveModal`](crate::patterns::modality::ActiveModal) when the
//! sheet closes. See `patterns::modality` for the host-integration
//! pattern.
//!
//! Cmd+. and Esc dismiss semantics are unchanged: the panel's focus
//! handler invokes `on_dismiss` on Escape; outside-click also dismisses
//! the panel.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/sheets>

use gpui::prelude::*;
use gpui::{
    AnimationExt, AnyElement, App, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, Pixels,
    Window, div, px,
};

use crate::callback_types::OnMutCallback;
use crate::foundations::accessibility::{FocusGroup, FocusGroupMode};
use crate::foundations::layout::Platform;
use crate::foundations::materials::{backdrop_overlay, glass_surface};
use crate::foundations::motion::accessible_transition_animation;
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme};

// ── Constants ───────────────────────────────────────────────────────────────

/// Drag indicator pill width (pt).
const DRAG_INDICATOR_WIDTH: f32 = 36.0;
/// Drag indicator pill height (pt).
const DRAG_INDICATOR_HEIGHT: f32 = 5.0;
/// Vertical padding above the drag indicator.
const DRAG_INDICATOR_TOP_PADDING: f32 = 8.0;
/// Vertical padding below the drag indicator.
const DRAG_INDICATOR_BOTTOM_PADDING: f32 = 4.0;

/// macOS cardlike sheet default width (pt).
const MACOS_SHEET_WIDTH: f32 = 480.0;
/// macOS cardlike sheet maximum height as fraction of viewport.
const MACOS_SHEET_MAX_HEIGHT_FRACTION: f32 = 0.82;

/// Shared dismiss callback wrapped in `Rc<Box<..>>` so a single boxed
/// closure can be cloned into several handler sites (mouse-down-out,
/// Escape) without reboxing.
type DismissRc = std::rc::Rc<Box<dyn Fn(&mut Window, &mut App) + 'static>>;

/// Sheet detent — controls how much of the screen the sheet covers on
/// iOS/iPadOS. macOS sheets always use `MACOS_SHEET_WIDTH` and
/// auto-sized height; detents are only consulted in
/// [`SheetPresentation::BottomDrawer`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SheetDetent {
    /// Half the available height.
    Medium,
    /// Near-full height (leaving a small top margin).
    #[default]
    Large,
    /// Explicit height as a fraction of the viewport. Values are
    /// clamped to `(0.0, 1.0]`. HIG iOS 16+: sheets support custom
    /// fractional heights for form-size presentations.
    Custom(f32),
}

impl SheetDetent {
    fn height_fraction(self) -> f32 {
        match self {
            Self::Medium => 0.50,
            Self::Large => 0.92,
            Self::Custom(f) => f.clamp(0.05, 1.0),
        }
    }
}

/// Where the sheet renders on screen. Auto-selected from
/// [`TahoeTheme::platform`] unless [`Sheet::presentation`] is called.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SheetPresentation {
    /// iOS bottom-anchored drawer with drag indicator and detent.
    #[default]
    BottomDrawer,
    /// macOS cardlike centered panel — no drag indicator, no bottom
    /// anchoring. HIG `#sheets` macOS wording.
    Cardlike,
}

impl SheetPresentation {
    /// Default presentation for a given platform per HIG.
    pub fn for_platform(platform: Platform) -> Self {
        match platform {
            Platform::MacOS | Platform::VisionOS | Platform::TvOS => Self::Cardlike,
            Platform::IOS | Platform::WatchOS => Self::BottomDrawer,
        }
    }
}

/// A sheet overlay following Human Interface Guidelines.
///
/// The sheet renders a semi-transparent backdrop and applies Liquid
/// Glass styling. On iOS it anchors to the bottom and exposes a drag
/// indicator; on macOS it renders as a cardlike centered panel.
///
/// # Example
///
/// ```ignore
/// Sheet::new("my-sheet", div().child("Hello"))
///     .open(true)
///     .detent(SheetDetent::Medium) // iOS only
///     .on_dismiss(|_window, _cx| { /* close state */ })
/// ```
#[derive(IntoElement)]
pub struct Sheet {
    id: ElementId,
    is_open: bool,
    detent: SheetDetent,
    content: AnyElement,
    on_dismiss: OnMutCallback,
    focus_handle: Option<FocusHandle>,
    /// Child focus handles wrapped in a Trap-mode [`FocusGroup`]. Tab /
    /// Shift+Tab cycle through the registered handles with wrap-around per
    /// the WAI-ARIA dialog pattern; the group's `handle_key_down` helper
    /// consumes Tab so focus cannot escape the sheet surface.
    focus_group: FocusGroup,
    /// Focus handle to restore when the sheet dismisses. HIG: "Return
    /// focus to a sensible location after modal dismissal."
    restore_focus_to: Option<FocusHandle>,
    presentation: Option<SheetPresentation>,
    width: Option<Pixels>,
}

impl Sheet {
    /// Create a new sheet with the given id and content element.
    pub fn new(id: impl Into<ElementId>, content: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            is_open: false,
            detent: SheetDetent::default(),
            content: content.into_any_element(),
            on_dismiss: None,
            focus_handle: None,
            focus_group: FocusGroup::trap(),
            restore_focus_to: None,
            presentation: None,
            width: None,
        }
    }

    /// Control visibility. When `false` the sheet renders nothing.
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set the height detent (iOS / iPadOS / watchOS only).
    pub fn detent(mut self, detent: SheetDetent) -> Self {
        self.detent = detent;
        self
    }

    /// Called when the user taps the backdrop or presses Escape.
    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    /// Override the focus handle tracked by the sheet panel.
    pub fn focus_handle(mut self, handle: FocusHandle) -> Self {
        self.focus_handle = Some(handle);
        self
    }

    /// Force a specific presentation chrome regardless of platform.
    pub fn presentation(mut self, presentation: SheetPresentation) -> Self {
        self.presentation = Some(presentation);
        self
    }

    /// Override the cardlike sheet width (macOS). Ignored on iOS which
    /// always spans the full viewport width.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Convenience: pass a child element (equivalent to providing content in `new`).
    pub fn child(self, child: impl IntoElement) -> Self {
        Self {
            content: child.into_any_element(),
            ..self
        }
    }

    /// Register the focus handles of interactive children in tab order.
    ///
    /// When the user presses Tab past the last handle, focus returns to the
    /// first; Shift+Tab past the first returns to the last. The sheet's
    /// `on_key_down` handler consumes Tab so focus cannot escape the sheet
    /// surface even when no children are registered.
    ///
    /// # Constraint
    ///
    /// Handles MUST be tracked (via `track_focus`) on elements that are
    /// descendants of the sheet's content subtree. A handle tracked on a
    /// sibling element escapes the trap — on-open initial focus lands on
    /// the external handle and subsequent Tab presses route there.
    ///
    /// Internally allocates a fresh Trap-mode [`FocusGroup`]. Calling this
    /// after [`Sheet::focus_group`] is last-write-wins: the caller's shared
    /// group is discarded and replaced with the handles passed here.
    /// Callers that already manage a [`FocusGroup`] should use
    /// [`Sheet::focus_group`] instead.
    pub fn focus_cycle(mut self, handles: Vec<FocusHandle>) -> Self {
        self.focus_group = FocusGroup::trap();
        for handle in &handles {
            self.focus_group.register(handle);
        }
        self
    }

    /// Attach a caller-managed [`FocusGroup`] driving the sheet's Tab trap.
    /// Overrides any handles previously supplied via [`Sheet::focus_cycle`].
    ///
    /// The group **must** be in [`FocusGroupMode::Trap`] mode. Open / Cycle
    /// groups make the sheet's Tab handler swallow Tab without advancing
    /// focus, stranding keyboard users — so this is a runtime `assert!`
    /// rather than a `debug_assert!`, firing in release as well.
    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        assert_eq!(
            group.mode(),
            FocusGroupMode::Trap,
            "Sheet::focus_group requires a Trap-mode FocusGroup; other \
             modes cause Tab to be swallowed without advancing focus, \
             stranding keyboard users."
        );
        self.focus_group = group;
        self
    }

    /// Record the focus handle that should regain focus when the sheet
    /// dismisses. Call with the currently-focused element *before* opening
    /// the sheet so the user returns to the same position on dismiss.
    pub fn restore_focus_to(mut self, handle: FocusHandle) -> Self {
        self.restore_focus_to = Some(handle);
        self
    }
}

impl RenderOnce for Sheet {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        // Resolve presentation first (pure read of TahoeTheme) so the
        // subsequent `cx.focus_handle()` mutable-borrow doesn't clash.
        let presentation = {
            let theme = cx.theme();
            self.presentation
                .unwrap_or_else(|| SheetPresentation::for_platform(theme.platform))
        };

        // Initial-focus bootstrap per WAI-ARIA dialog pattern: land on the
        // first focus-group member when one is registered, otherwise on
        // the outer handle so Escape / Tab reach the key handler. Mirrors
        // Modal's two-check logic so a pre-focused member (e.g. an
        // auto-focused `TextField`) is seen on the first render.
        let focus_handle = self
            .focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle());
        let any_sheet_focus =
            self.focus_group.contains_focused(window) || focus_handle.contains_focused(window, cx);
        if !any_sheet_focus {
            if self.focus_group.is_empty() {
                focus_handle.focus(window, cx);
            } else {
                self.focus_group.focus_first(window, cx);
            }
        }

        let theme = cx.theme();

        // Wrap the caller's on_dismiss so every dismissal path (backdrop
        // click, Escape / Cmd-.) restores focus to `restore_focus_to`
        // before invoking the inner callback. The Rc shape lets the
        // wrapped callback be shared between the mouse-down-out and key
        // handlers.
        let restore = self.restore_focus_to.clone();
        let on_dismiss_rc: Option<DismissRc> = self.on_dismiss.map(move |inner| {
            let wrapped: Box<dyn Fn(&mut Window, &mut App) + 'static> =
                Box::new(move |window: &mut Window, cx: &mut App| {
                    if let Some(handle) = restore.as_ref() {
                        handle.focus(window, cx);
                    }
                    inner(window, cx);
                });
            std::rc::Rc::new(wrapped)
        });

        match presentation {
            SheetPresentation::BottomDrawer => render_bottom_drawer(
                self.id,
                self.detent,
                self.content,
                theme,
                focus_handle,
                on_dismiss_rc,
                self.focus_group,
            ),
            SheetPresentation::Cardlike => render_cardlike(
                self.id,
                self.content,
                self.width.unwrap_or(px(MACOS_SHEET_WIDTH)),
                theme,
                focus_handle,
                on_dismiss_rc,
                self.focus_group,
            ),
        }
    }
}

/// iOS/iPadOS bottom-drawer rendering.
fn render_bottom_drawer(
    id: ElementId,
    detent: SheetDetent,
    content: AnyElement,
    theme: &TahoeTheme,
    focus_handle: FocusHandle,
    on_dismiss_rc: Option<DismissRc>,
    focus_group: FocusGroup,
) -> gpui::AnyElement {
    // ── Backdrop ────────────────────────────────────────────────────────
    let backdrop = backdrop_overlay(theme).flex().flex_col().justify_end();

    // ── Sheet height based on detent ────────────────────────────────────
    let height_frac = detent.height_fraction();

    // ── Drag indicator pill (iOS only) ──────────────────────────────────
    let indicator_color = theme.text_quaternary();
    let drag_indicator = div()
        .flex()
        .justify_center()
        .pt(px(DRAG_INDICATOR_TOP_PADDING))
        .pb(px(DRAG_INDICATOR_BOTTOM_PADDING))
        .child(
            div()
                .w(px(DRAG_INDICATOR_WIDTH))
                .h(px(DRAG_INDICATOR_HEIGHT))
                .rounded(px(DRAG_INDICATOR_HEIGHT / 2.0))
                .bg(indicator_color),
        );

    // ── Scrollable content area (animated) ──────────────────────────────
    // Under Reduce Motion or Prefer Cross-Fade, the 32 pt slide-up
    // collapses to a pure opacity fade.
    let accessibility = theme.accessibility_mode;
    let slide_offset_pt =
        if accessibility.reduce_motion() || accessibility.prefer_cross_fade_transitions() {
            0.0
        } else {
            32.0
        };
    let anim_id = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "present".into());
    let scroll_id = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "scroll".into());
    let natural_duration =
        std::time::Duration::from_millis(theme.glass.motion.shape_shift_duration_ms);
    let scroll_body = div()
        .id(scroll_id)
        .flex_1()
        .overflow_y_scroll()
        .px(theme.spacing_lg)
        .pb(theme.spacing_lg)
        .child(content)
        .with_animation(
            anim_id,
            accessible_transition_animation(&theme.glass.motion, natural_duration, accessibility),
            move |el, delta| {
                let offset = slide_offset_pt * (1.0 - delta);
                el.opacity(delta).mt(px(offset))
            },
        );

    // ── Sheet panel (glass surface) ─────────────────────────────────────
    let top_radius = theme.glass.radius(GlassSize::Large);
    let panel_id = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "panel".into());
    let mut panel = glass_surface(div().w_full().overflow_hidden(), theme, GlassSize::Large)
        .rounded_t(top_radius)
        .rounded_b(px(0.0))
        .id(panel_id)
        .track_focus(&focus_handle)
        .flex()
        .flex_col()
        .w_full()
        .h(gpui::relative(height_frac))
        .child(drag_indicator)
        .child(scroll_body);

    if let Some(ref handler) = on_dismiss_rc {
        let h = handler.clone();
        panel = panel.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
            h(window, cx);
        });
    }

    // Key handler: Escape / Cmd-. dismiss per HIG `#sheets`; Tab / Shift+Tab
    // routed through the sheet's Trap-mode `FocusGroup` (empty-group Tab
    // still swallowed so focus cannot escape the sheet surface).
    let dismiss_for_keys = on_dismiss_rc;
    panel = panel.on_key_down(move |event: &KeyDownEvent, window, cx| {
        let modifiers = &event.keystroke.modifiers;
        let is_cmd_period = modifiers.platform && event.keystroke.key.as_str() == ".";
        if crate::foundations::keyboard::is_escape_key(event) || is_cmd_period {
            if let Some(handler) = &dismiss_for_keys {
                handler(window, cx);
            }
            return;
        }
        match event.keystroke.key.as_str() {
            "tab" if !focus_group.handle_key_down(event, window, cx) => {
                cx.stop_propagation();
            }
            "home" if focus_group.contains_focused(window) => {
                focus_group.focus_first(window, cx);
                cx.stop_propagation();
            }
            "end" if focus_group.contains_focused(window) => {
                focus_group.focus_last(window, cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    });

    backdrop.child(panel).into_any_element()
}

/// macOS cardlike (centered) rendering.
fn render_cardlike(
    id: ElementId,
    content: AnyElement,
    width: Pixels,
    theme: &TahoeTheme,
    focus_handle: FocusHandle,
    on_dismiss_rc: Option<DismissRc>,
    focus_group: FocusGroup,
) -> gpui::AnyElement {
    let backdrop = backdrop_overlay(theme)
        .flex()
        .items_center()
        .justify_center();

    let anim_id = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "present".into());
    let scroll_id = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "scroll".into());
    let natural_duration = std::time::Duration::from_millis(theme.glass.motion.lift_duration_ms);

    let animated_body = div()
        .id(scroll_id)
        .overflow_y_scroll()
        .p(theme.spacing_lg)
        .child(content)
        .with_animation(
            anim_id,
            accessible_transition_animation(
                &theme.glass.motion,
                natural_duration,
                theme.accessibility_mode,
            ),
            |el, delta| el.opacity(delta),
        );

    let panel_id = ElementId::NamedChild(std::sync::Arc::new(id.clone()), "panel".into());
    let mut panel = glass_surface(div().w(width).overflow_hidden(), theme, GlassSize::Large)
        .id(panel_id)
        .track_focus(&focus_handle)
        .flex()
        .flex_col()
        .max_h(gpui::relative(MACOS_SHEET_MAX_HEIGHT_FRACTION))
        .child(animated_body);

    if let Some(ref handler) = on_dismiss_rc {
        let h = handler.clone();
        panel = panel.on_mouse_down_out(move |_event: &MouseDownEvent, window, cx| {
            h(window, cx);
        });
    }

    // Key handler: Escape / Cmd-. dismiss per HIG `#sheets`; Tab / Shift+Tab
    // routed through the sheet's Trap-mode `FocusGroup` (empty-group Tab
    // still swallowed so focus cannot escape the sheet surface).
    let dismiss_for_keys = on_dismiss_rc;
    panel = panel.on_key_down(move |event: &KeyDownEvent, window, cx| {
        let modifiers = &event.keystroke.modifiers;
        let is_cmd_period = modifiers.platform && event.keystroke.key.as_str() == ".";
        if crate::foundations::keyboard::is_escape_key(event) || is_cmd_period {
            if let Some(handler) = &dismiss_for_keys {
                handler(window, cx);
            }
            return;
        }
        match event.keystroke.key.as_str() {
            "tab" if !focus_group.handle_key_down(event, window, cx) => {
                cx.stop_propagation();
            }
            "home" if focus_group.contains_focused(window) => {
                focus_group.focus_first(window, cx);
                cx.stop_propagation();
            }
            "end" if focus_group.contains_focused(window) => {
                focus_group.focus_last(window, cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    });

    backdrop.child(panel).into_any_element()
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::prelude::*;

    use crate::components::presentation::sheet::{
        DRAG_INDICATOR_HEIGHT, DRAG_INDICATOR_WIDTH, Sheet, SheetDetent, SheetPresentation,
    };
    use crate::foundations::accessibility::{FocusGroup, FocusGroupMode};
    use crate::foundations::layout::Platform;

    #[test]
    fn sheet_defaults() {
        let sheet = Sheet::new("test-sheet", gpui::div().child("hello"));
        assert!(!sheet.is_open);
        assert_eq!(sheet.detent, SheetDetent::Large);
        assert!(sheet.on_dismiss.is_none());
        assert!(sheet.focus_handle.is_none());
        assert!(sheet.presentation.is_none());
        assert!(sheet.width.is_none());
    }

    #[test]
    fn sheet_open_builder() {
        let sheet = Sheet::new("s", gpui::div()).open(true);
        assert!(sheet.is_open);
    }

    #[test]
    fn sheet_detent_medium() {
        let sheet = Sheet::new("s", gpui::div()).detent(SheetDetent::Medium);
        assert_eq!(sheet.detent, SheetDetent::Medium);
    }

    #[test]
    fn sheet_detent_large() {
        let sheet = Sheet::new("s", gpui::div()).detent(SheetDetent::Large);
        assert_eq!(sheet.detent, SheetDetent::Large);
    }

    #[test]
    fn sheet_detent_custom_clamps() {
        assert!((SheetDetent::Custom(0.65).height_fraction() - 0.65).abs() < f32::EPSILON);
        // Out-of-range values are clamped into (0.05, 1.0].
        assert!((SheetDetent::Custom(2.0).height_fraction() - 1.0).abs() < f32::EPSILON);
        assert!((SheetDetent::Custom(0.0).height_fraction() - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn sheet_on_dismiss_sets_callback() {
        let sheet = Sheet::new("s", gpui::div()).on_dismiss(|_w, _cx| {});
        assert!(sheet.on_dismiss.is_some());
    }

    #[test]
    fn sheet_child_builder() {
        let _sheet = Sheet::new("s", gpui::div()).child(gpui::div().child("content"));
    }

    #[test]
    fn sheet_full_builder_chain() {
        let _sheet = Sheet::new("s", gpui::div())
            .open(true)
            .detent(SheetDetent::Medium)
            .on_dismiss(|_w, _cx| {})
            .child(gpui::div().child("body"));
    }

    #[test]
    fn sheet_presentation_builder() {
        let sheet = Sheet::new("s", gpui::div()).presentation(SheetPresentation::Cardlike);
        assert_eq!(sheet.presentation, Some(SheetPresentation::Cardlike));
    }

    #[test]
    fn sheet_presentation_for_platform_matches_hig() {
        assert_eq!(
            SheetPresentation::for_platform(Platform::MacOS),
            SheetPresentation::Cardlike
        );
        assert_eq!(
            SheetPresentation::for_platform(Platform::IOS),
            SheetPresentation::BottomDrawer
        );
        assert_eq!(
            SheetPresentation::for_platform(Platform::WatchOS),
            SheetPresentation::BottomDrawer
        );
    }

    #[test]
    fn default_detent_is_large() {
        assert_eq!(SheetDetent::default(), SheetDetent::Large);
    }

    #[test]
    fn detent_variants_are_distinct() {
        assert_ne!(SheetDetent::Medium, SheetDetent::Large);
        assert_ne!(SheetDetent::Medium, SheetDetent::Custom(0.5));
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn drag_indicator_dimensions_are_positive() {
        assert!(
            DRAG_INDICATOR_WIDTH > 0.0,
            "drag indicator width must be positive"
        );
        assert!(
            DRAG_INDICATOR_HEIGHT > 0.0,
            "drag indicator height must be positive"
        );
    }

    #[test]
    fn sheet_focus_cycle_defaults_to_empty() {
        let sheet = Sheet::new("s", gpui::div());
        assert!(sheet.focus_group.is_empty());
    }

    #[test]
    fn sheet_focus_group_defaults_to_trap_mode() {
        let sheet = Sheet::new("s", gpui::div());
        assert_eq!(sheet.focus_group.mode(), FocusGroupMode::Trap);
    }

    #[test]
    #[should_panic(expected = "Sheet::focus_group requires a Trap-mode FocusGroup")]
    fn sheet_focus_group_rejects_non_trap_mode() {
        let _ = Sheet::new("s", gpui::div()).focus_group(FocusGroup::cycle());
    }

    #[test]
    fn sheet_restore_focus_default_is_none() {
        let sheet = Sheet::new("s", gpui::div());
        assert!(sheet.restore_focus_to.is_none());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div, px};

    use super::{Sheet, SheetPresentation};
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    // Harness for Tab-cycle behaviour: the sheet registers two focusable
    // child handles and we verify Tab cycles forward through both, then
    // wraps back to the first on a third press.
    struct TabCycleHarness {
        outer_focus: FocusHandle,
        first: FocusHandle,
        second: FocusHandle,
        presentation: SheetPresentation,
    }

    impl TabCycleHarness {
        fn new(cx: &mut Context<Self>, presentation: SheetPresentation) -> Self {
            Self {
                outer_focus: cx.focus_handle(),
                first: cx.focus_handle(),
                second: cx.focus_handle(),
                presentation,
            }
        }
    }

    impl Render for TabCycleHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let body = div()
                .w(px(160.0))
                .h(px(80.0))
                .flex()
                .flex_col()
                .child(div().id("first").track_focus(&self.first).child("First"))
                .child(div().id("second").track_focus(&self.second).child("Second"));
            Sheet::new("sheet", body)
                .open(true)
                .presentation(self.presentation)
                .focus_handle(self.outer_focus.clone())
                .focus_cycle(vec![self.first.clone(), self.second.clone()])
                .on_dismiss(|_, _| {})
        }
    }

    #[gpui::test]
    async fn tab_cycles_focus_forward_bottom_drawer(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabCycleHarness::new(cx, SheetPresentation::BottomDrawer)
        });

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
    async fn tab_cycles_focus_forward_cardlike(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabCycleHarness::new(cx, SheetPresentation::Cardlike)
        });

        host.update_in(cx, |host, window, cx| {
            host.outer_focus.focus(window, cx);
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.first.is_focused(window), "first Tab lands on first");
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.second.is_focused(window));
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.first.is_focused(window),
                "Tab wraps in Cardlike presentation too"
            );
        });
    }

    #[gpui::test]
    async fn shift_tab_cycles_focus_backward(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabCycleHarness::new(cx, SheetPresentation::BottomDrawer)
        });

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

    // Harness with no focus_cycle — verifies the empty-trap contract:
    // Tab must be swallowed even when no child handles are registered,
    // so focus cannot escape the sheet surface.
    struct EmptyTrapHarness {
        outer_focus: FocusHandle,
        outside: FocusHandle,
        presentation: SheetPresentation,
    }

    impl EmptyTrapHarness {
        fn new(cx: &mut Context<Self>, presentation: SheetPresentation) -> Self {
            Self {
                outer_focus: cx.focus_handle(),
                outside: cx.focus_handle(),
                presentation,
            }
        }
    }

    impl Render for EmptyTrapHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div()
                .child(
                    div()
                        .id("outside")
                        .track_focus(&self.outside)
                        .child("Outside"),
                )
                .child(
                    Sheet::new("sheet", div().w(px(120.0)).h(px(40.0)).child("Body"))
                        .open(true)
                        .presentation(self.presentation)
                        .focus_handle(self.outer_focus.clone())
                        .on_dismiss(|_, _| {}),
                )
        }
    }

    #[gpui::test]
    async fn tab_in_empty_focus_group_does_not_escape_sheet_bottom_drawer(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            EmptyTrapHarness::new(cx, SheetPresentation::BottomDrawer)
        });
        host.update_in(cx, |host, window, cx| {
            host.outer_focus.focus(window, cx);
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.outer_focus.is_focused(window),
                "Tab with an empty FocusGroup must leave outer focus put"
            );
            assert!(
                !host.outside.is_focused(window),
                "Tab must not reach elements outside the sheet"
            );
        });
    }

    #[gpui::test]
    async fn tab_in_empty_focus_group_does_not_escape_sheet_cardlike(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            EmptyTrapHarness::new(cx, SheetPresentation::Cardlike)
        });
        host.update_in(cx, |host, window, cx| {
            host.outer_focus.focus(window, cx);
        });
        cx.press("tab");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.outer_focus.is_focused(window));
            assert!(!host.outside.is_focused(window));
        });
    }

    // Harness that wires `restore_focus_to` and lets us observe focus
    // returning on dismiss. The `previous` handle stands in for the
    // app-level control that was focused before the sheet opened.
    struct RestoreFocusHarness {
        sheet_focus: FocusHandle,
        previous: FocusHandle,
        is_open: bool,
    }

    impl RestoreFocusHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                sheet_focus: cx.focus_handle(),
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
                .child("Sheet body");
            Sheet::new("sheet", body)
                .open(self.is_open)
                .focus_handle(self.sheet_focus.clone())
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
            host.sheet_focus.focus(window, cx);
            assert!(host.sheet_focus.is_focused(window));
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
}
