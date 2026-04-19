//! Master-detail resizable split layout per HIG Split Views.
//!
//! Tracks divider drag position to resize primary and secondary panes.
//! The primary pane has a configurable width with min/max constraints.
//! The secondary pane fills remaining space via `flex_1()`.
//!
//! # Orientation
//!
//! Configurable via [`SplitOrientation`]. Horizontal splits arrange panes
//! left-to-right (default); vertical splits arrange top-to-bottom. HIG:
//! "you can arrange the panes of a split view vertically, horizontally,
//! or both."
//!
//! # Divider
//!
//! The visible divider is a 1 pt hairline (HIG: "Prefer the thin divider
//! style. The thin divider measures one point in width.") rendered inside
//! a transparent 4 pt container that provides a comfortable hover target.
//! A further 20 pt activation region extends symmetrically on either side
//! via a child overlay.
//!
//! # Collapse / reveal
//!
//! Call [`SplitView::toggle_primary`] / [`SplitView::collapse_primary`] /
//! [`SplitView::expand_primary`] from a toolbar button or menu command.
//! Double-clicking the divider toggles the primary pane. HIG: "Provide
//! multiple ways to reveal hidden panes… a toolbar button or a menu
//! command — including a keyboard shortcut."
//!
//! # Persistence
//!
//! [`SplitView::primary_width`] is suitable for serialization so clients
//! can restore the width across sessions, per HIG Launching / State
//! Restoration. The returned width is always the *user-set* width, even
//! when collapsed, so a user's preferred width isn't lost when they
//! toggle the pane off and on.

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, CursorStyle, ElementId, FocusHandle, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Window, div, px,
};

use crate::foundations::layout::{SIDEBAR_MIN_WIDTH, SPLIT_DIVIDER_HIT_AREA, SPLIT_DIVIDER_WIDTH};
use crate::foundations::theme::{ActiveTheme, TahoeTheme};
use crate::ids::next_element_id;

/// Default keyboard resize step in pixels when the user presses an arrow key.
pub const DEFAULT_KEYBOARD_RESIZE_STEP: f32 = 10.0;

/// Default coarse keyboard resize multiplier when `Shift` is held with an arrow key.
pub const DEFAULT_COARSE_RESIZE_MULTIPLIER: f32 = 5.0;

/// Split orientation per HIG Split Views.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SplitOrientation {
    /// Primary pane on the leading edge, secondary fills trailing.
    #[default]
    Horizontal,
    /// Primary pane on top, secondary fills the bottom.
    Vertical,
}

/// Boxed factory producing either pane's content.
type PaneContent = Box<dyn Fn(&mut Window, &mut App) -> AnyElement>;

pub struct SplitView {
    element_id: ElementId,
    focus_handle: FocusHandle,
    orientation: SplitOrientation,
    primary_width: f32,
    min_primary: f32,
    max_primary: f32,
    is_dragging: bool,
    drag_start_x: Option<f32>,
    drag_start_width: Option<f32>,
    primary_content: Option<PaneContent>,
    secondary_content: Option<PaneContent>,
    resize_step: f32,
    coarse_resize_multiplier: f32,
    /// `false` hides the primary pane without discarding its width. Flipping
    /// back to `true` restores the previous width exactly.
    primary_visible: bool,
    /// Cached focus flag kept in sync via
    /// [`SplitView::install_focus_subscriptions`]. Set lazily — the
    /// render path falls back to a direct `focus_handle.is_focused`
    /// check until the caller installs the subscriptions (which needs
    /// a `&mut Window`, unavailable from `new`).
    ///
    /// Finding 13 in df49b9cd/ai-sdk-rust#132.
    divider_focused: bool,
    /// Subscriptions kept alive for the lifetime of the view once
    /// [`install_focus_subscriptions`](Self::install_focus_subscriptions)
    /// has been called. `None` until then.
    _focus_subscriptions: Option<[gpui::Subscription; 2]>,
}

impl SplitView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let default_width = cx
            .try_global::<TahoeTheme>()
            .map_or(260.0, |theme| f32::from(theme.sidebar_width_default));
        Self {
            element_id: next_element_id("split-view"),
            focus_handle: cx.focus_handle(),
            orientation: SplitOrientation::default(),
            primary_width: default_width,
            min_primary: SIDEBAR_MIN_WIDTH,
            max_primary: 400.0,
            is_dragging: false,
            drag_start_x: None,
            drag_start_width: None,
            primary_content: None,
            secondary_content: None,
            resize_step: DEFAULT_KEYBOARD_RESIZE_STEP,
            coarse_resize_multiplier: DEFAULT_COARSE_RESIZE_MULTIPLIER,
            primary_visible: true,
            divider_focused: false,
            _focus_subscriptions: None,
        }
    }

    /// Install reactive focus subscriptions so the divider focus flag
    /// updates without relying on per-frame polling inside `render`.
    ///
    /// Context: GPUI's `on_focus_in` / `on_focus_out` live on `Context`
    /// but require a `&mut Window` argument, which is not available
    /// inside `SplitView::new(cx)`. Callers that want the reactive
    /// pattern should invoke this immediately after constructing the
    /// view, once they are inside a `Render::render` or similar scope
    /// where a `Window` is available:
    ///
    /// ```ignore
    /// let sv = cx.new(|cx| SplitView::new(cx));
    /// sv.update(cx, |view, cx| view.install_focus_subscriptions(window, cx));
    /// ```
    ///
    /// Hosts that skip this step get the same behaviour as before —
    /// the render path falls back to `focus_handle.is_focused(window)`.
    /// Finding 13 in df49b9cd/ai-sdk-rust#132.
    pub fn install_focus_subscriptions(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let focus_in = cx.on_focus_in(&self.focus_handle, window, |this, _window, cx| {
            this.divider_focused = true;
            cx.notify();
        });
        let focus_out = cx.on_focus_out(
            &self.focus_handle,
            window,
            |this, _event, _window, cx| {
                this.divider_focused = false;
                cx.notify();
            },
        );
        self._focus_subscriptions = Some([focus_in, focus_out]);
    }

    /// Configure the split orientation. Defaults to [`SplitOrientation::Horizontal`].
    pub fn set_orientation(&mut self, orientation: SplitOrientation, cx: &mut Context<Self>) {
        self.orientation = orientation;
        cx.notify();
    }

    pub fn orientation(&self) -> SplitOrientation {
        self.orientation
    }

    /// Configure the fine keyboard resize step (default 10pt).
    pub fn set_resize_step(&mut self, step: f32) {
        self.resize_step = step;
    }

    /// Configure the multiplier applied when `Shift` is held with an arrow
    /// key (default 5×, i.e. 50pt coarse step).
    pub fn set_coarse_resize_multiplier(&mut self, multiplier: f32) {
        self.coarse_resize_multiplier = multiplier;
    }

    /// Set the primary pane width in logical pixels.
    pub fn set_primary_width(&mut self, width: f32, cx: &mut Context<Self>) {
        self.primary_width = width.clamp(self.min_primary, self.max_primary);
        cx.notify();
    }

    /// Set the minimum width of the primary pane.
    /// Also adjusts max_primary upward if needed and re-clamps primary_width.
    pub fn set_min_primary(&mut self, min: f32, cx: &mut Context<Self>) {
        self.min_primary = min;
        self.max_primary = self.max_primary.max(min);
        self.primary_width = self.primary_width.clamp(self.min_primary, self.max_primary);
        cx.notify();
    }

    /// Set the maximum width of the primary pane.
    /// Also adjusts min_primary downward if needed and re-clamps primary_width.
    pub fn set_max_primary(&mut self, max: f32, cx: &mut Context<Self>) {
        self.max_primary = max;
        self.min_primary = self.min_primary.min(max);
        self.primary_width = self.primary_width.clamp(self.min_primary, self.max_primary);
        cx.notify();
    }

    /// Set a factory for the primary (leading) pane content.
    /// Called on each render to produce a fresh element.
    pub fn set_primary(
        &mut self,
        factory: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) {
        self.primary_content = Some(Box::new(factory));
        cx.notify();
    }

    /// Set a factory for the secondary (trailing) pane content.
    /// Called on each render to produce a fresh element.
    pub fn set_secondary(
        &mut self,
        factory: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) {
        self.secondary_content = Some(Box::new(factory));
        cx.notify();
    }

    /// Returns the current primary pane width. Always returns the
    /// user-set width, even when the primary pane is currently hidden —
    /// suitable for session persistence / state restoration.
    pub fn primary_width(&self) -> f32 {
        self.primary_width
    }

    /// Returns whether the divider is currently being dragged.
    pub fn is_dragging(&self) -> bool {
        self.is_dragging
    }

    /// Returns whether the primary pane is currently visible.
    pub fn is_primary_visible(&self) -> bool {
        self.primary_visible
    }

    /// Hide the primary pane without discarding its width.
    pub fn collapse_primary(&mut self, cx: &mut Context<Self>) {
        self.primary_visible = false;
        cx.notify();
    }

    /// Show the primary pane and restore its previous width.
    pub fn expand_primary(&mut self, cx: &mut Context<Self>) {
        self.primary_visible = true;
        cx.notify();
    }

    /// Toggle the primary pane's visibility. Wire this to a toolbar
    /// button or menu item per HIG Split Views.
    pub fn toggle_primary(&mut self, cx: &mut Context<Self>) {
        self.primary_visible = !self.primary_visible;
        cx.notify();
    }

    fn handle_divider_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_dragging = true;
        let start = match self.orientation {
            SplitOrientation::Horizontal => f32::from(event.position.x),
            SplitOrientation::Vertical => f32::from(event.position.y),
        };
        self.drag_start_x = Some(start);
        self.drag_start_width = Some(self.primary_width);
        cx.notify();
    }

    fn handle_divider_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.is_dragging {
            return;
        }
        if let (Some(start), Some(start_width)) = (self.drag_start_x, self.drag_start_width) {
            let current = match self.orientation {
                SplitOrientation::Horizontal => f32::from(event.position.x),
                SplitOrientation::Vertical => f32::from(event.position.y),
            };
            let delta = current - start;
            let new_width = (start_width + delta).clamp(self.min_primary, self.max_primary);
            if (new_width - self.primary_width).abs() < 0.5 {
                return;
            }
            self.primary_width = new_width;
            cx.notify();
        }
    }

    fn handle_divider_up(
        &mut self,
        _event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_dragging = false;
        self.drag_start_x = None;
        self.drag_start_width = None;
        cx.notify();
    }

    fn handle_divider_click(
        &mut self,
        event: &ClickEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // HIG macOS: "double-clicking the divider hides or shows a pane."
        if event.click_count() >= 2 {
            self.toggle_primary(cx);
        }
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let step = if event.keystroke.modifiers.shift {
            self.resize_step * self.coarse_resize_multiplier
        } else {
            self.resize_step
        };
        let new_width = match event.keystroke.key.as_str() {
            "right" | "down" => Some(self.primary_width + step),
            "left" | "up" => Some(self.primary_width - step),
            "home" => Some(self.min_primary),
            "end" => Some(self.max_primary),
            _ => None,
        };
        if let Some(w) = new_width {
            self.primary_width = w.clamp(self.min_primary, self.max_primary);
            cx.notify();
        }
    }
}

impl Render for SplitView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Build content elements before borrowing theme (factory closures need &mut App)
        let primary_el = self.primary_content.as_ref().map(|f| f(window, cx));
        let secondary_el = self.secondary_content.as_ref().map(|f| f(window, cx));

        let theme = cx.theme();
        // Prefer the reactively-maintained flag set by our focus_in /
        // focus_out subscriptions; fall back to a direct check for the
        // very first render before any event has fired. Finding 13 in
        // df49b9cd/ai-sdk-rust#132.
        let divider_focused = self.divider_focused || self.focus_handle.is_focused(window);
        let orientation = self.orientation;
        let primary_visible = self.primary_visible;

        // ── Primary pane ────────────────────────────────────────────────────
        let primary_opt = if primary_visible {
            let mut pane = div().overflow_hidden().bg(theme.surface);
            pane = match orientation {
                SplitOrientation::Horizontal => pane
                    .w(px(self.primary_width))
                    .h_full()
                    .border_r_1()
                    .border_color(theme.separator_color()),
                SplitOrientation::Vertical => pane
                    .h(px(self.primary_width))
                    .w_full()
                    .border_b_1()
                    .border_color(theme.separator_color()),
            };

            if let Some(content) = primary_el {
                pane = pane.child(content);
            }

            Some(pane)
        } else {
            None
        };

        // ── Divider ─────────────────────────────────────────────────────────
        // HIG: 1 pt hairline centered inside a 4 pt transparent container.
        // The 4 pt surface acts as a comfortable mouse target; the 20 pt
        // hit overlay (SPLIT_DIVIDER_HIT_AREA) extends the activation
        // region further via a child without affecting layout.
        let hairline_color = if self.is_dragging {
            theme.accent
        } else if divider_focused {
            crate::foundations::color::with_alpha(theme.accent, 0.5)
        } else {
            theme.separator_color()
        };

        // Transparent container; inner hairline is the visible 1 pt line.
        let container_size = px(SPLIT_DIVIDER_WIDTH);
        let hairline_thickness = theme.separator_thickness;

        let hairline = match orientation {
            SplitOrientation::Horizontal => div()
                .w(hairline_thickness)
                .h_full()
                .bg(hairline_color),
            SplitOrientation::Vertical => div()
                .h(hairline_thickness)
                .w_full()
                .bg(hairline_color),
        };

        let hit_area_overhang = (SPLIT_DIVIDER_HIT_AREA - SPLIT_DIVIDER_WIDTH) / 2.0;
        let mut hit_overlay = div()
            .id(self.element_id.clone())
            .debug_selector(|| "split-view-divider".into())
            .track_focus(&self.focus_handle)
            .absolute()
            .cursor(match orientation {
                SplitOrientation::Horizontal => CursorStyle::ResizeLeftRight,
                SplitOrientation::Vertical => CursorStyle::ResizeUpDown,
            })
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_divider_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_divider_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_divider_up))
            .on_mouse_move(cx.listener(Self::handle_divider_move))
            .on_click(cx.listener(Self::handle_divider_click))
            .on_key_down(cx.listener(Self::handle_key_down));

        hit_overlay = match orientation {
            SplitOrientation::Horizontal => hit_overlay
                .top_0()
                .bottom_0()
                .left(px(-hit_area_overhang))
                .w(px(SPLIT_DIVIDER_HIT_AREA)),
            SplitOrientation::Vertical => hit_overlay
                .left_0()
                .right_0()
                .top(px(-hit_area_overhang))
                .h(px(SPLIT_DIVIDER_HIT_AREA)),
        };

        let mut divider = div()
            .relative()
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_center();
        divider = match orientation {
            SplitOrientation::Horizontal => divider.w(container_size).h_full(),
            SplitOrientation::Vertical => divider.h(container_size).w_full(),
        };
        divider = divider.child(hairline).child(hit_overlay);

        // ── Secondary pane ──────────────────────────────────────────────────
        let mut secondary = div().flex_1().overflow_hidden();
        secondary = match orientation {
            SplitOrientation::Horizontal => secondary.h_full(),
            SplitOrientation::Vertical => secondary.w_full(),
        };

        if let Some(content) = secondary_el {
            secondary = secondary.child(content);
        }

        // ── Layout ──────────────────────────────────────────────────────────
        let mut root = div()
            .flex()
            .w_full()
            .h_full()
            .on_mouse_move(cx.listener(Self::handle_divider_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_divider_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::handle_divider_up));
        root = match orientation {
            SplitOrientation::Horizontal => root.flex_row(),
            SplitOrientation::Vertical => root.flex_col(),
        };

        if let Some(primary) = primary_opt {
            root = root.child(primary).child(divider);
        } else {
            // Keep the divider on screen so callers can still drag it
            // back out if they choose — HIG allows revealing a hidden
            // pane by dragging the divider.
            root = root.child(divider);
        }
        root.child(secondary)
    }
}

#[cfg(test)]
mod tests {
    use super::{SIDEBAR_MIN_WIDTH, SplitOrientation};
    use core::prelude::v1::test;

    /// Helper struct for pure logic tests that don't require GPUI context.
    /// NOTE: Default values here must match `SplitView::new()`. If the real
    /// defaults change, update this struct too.
    struct TestSplitView {
        primary_width: f32,
        min_primary: f32,
        max_primary: f32,
        is_dragging: bool,
        primary_visible: bool,
    }

    impl TestSplitView {
        fn new() -> Self {
            Self {
                primary_width: 260.0,
                min_primary: SIDEBAR_MIN_WIDTH,
                max_primary: 400.0,
                is_dragging: false,
                primary_visible: true,
            }
        }

        fn primary_width(&self) -> f32 {
            self.primary_width
        }

        fn is_dragging(&self) -> bool {
            self.is_dragging
        }
    }

    #[test]
    fn default_primary_width() {
        let sv = TestSplitView::new();
        assert!((sv.primary_width() - 260.0).abs() < f32::EPSILON);
    }

    #[test]
    fn default_not_dragging() {
        let sv = TestSplitView::new();
        assert!(!sv.is_dragging());
    }

    #[test]
    fn min_max_defaults() {
        let sv = TestSplitView::new();
        assert!((sv.min_primary - SIDEBAR_MIN_WIDTH).abs() < f32::EPSILON);
        assert!((sv.max_primary - 400.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_below_min() {
        let mut sv = TestSplitView::new();
        // Simulate clamping manually (set_primary_width needs Context)
        let new_width = 100.0_f32.clamp(sv.min_primary, sv.max_primary);
        sv.primary_width = new_width;
        assert!((sv.primary_width() - SIDEBAR_MIN_WIDTH).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_above_max() {
        let mut sv = TestSplitView::new();
        let new_width = 999.0_f32.clamp(sv.min_primary, sv.max_primary);
        sv.primary_width = new_width;
        assert!((sv.primary_width() - 400.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clamp_within_range() {
        let mut sv = TestSplitView::new();
        let new_width = 300.0_f32.clamp(sv.min_primary, sv.max_primary);
        sv.primary_width = new_width;
        assert!((sv.primary_width() - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn drag_state_tracking() {
        let mut sv = TestSplitView::new();
        assert!(!sv.is_dragging());
        sv.is_dragging = true;
        assert!(sv.is_dragging());
        sv.is_dragging = false;
        assert!(!sv.is_dragging());
    }

    #[test]
    fn keyboard_resize_step_is_10px() {
        use super::DEFAULT_KEYBOARD_RESIZE_STEP;
        assert!((DEFAULT_KEYBOARD_RESIZE_STEP - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn keyboard_resize_clamps() {
        let mut sv = TestSplitView::new();
        // Simulate right arrow
        sv.primary_width = (sv.primary_width + 10.0).clamp(sv.min_primary, sv.max_primary);
        assert!((sv.primary_width() - 270.0).abs() < f32::EPSILON);
        // Simulate left arrow past min
        sv.primary_width = SIDEBAR_MIN_WIDTH;
        sv.primary_width = (sv.primary_width - 10.0).clamp(sv.min_primary, sv.max_primary);
        assert!((sv.primary_width() - SIDEBAR_MIN_WIDTH).abs() < f32::EPSILON);
    }

    // ── set_min/max cross-adjustment ────────────────────────────────────
    // Uses the real SplitView methods which require GPUI context,
    // so we replicate the logic here to test the algorithm.

    #[test]
    fn set_min_above_width_clamps_width_up() {
        // Simulates set_min_primary(300) when max=400, width=260
        let mut sv = TestSplitView::new();
        sv.min_primary = 300.0;
        sv.max_primary = sv.max_primary.max(sv.min_primary);
        sv.primary_width = sv.primary_width.clamp(sv.min_primary, sv.max_primary);
        assert!((sv.primary_width - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn set_max_below_width_clamps_width_down() {
        // Simulates set_max_primary(220) when min=SIDEBAR_MIN_WIDTH, width=260
        let mut sv = TestSplitView::new();
        sv.max_primary = 220.0;
        sv.min_primary = sv.min_primary.min(sv.max_primary);
        sv.primary_width = sv.primary_width.clamp(sv.min_primary, sv.max_primary);
        assert!((sv.primary_width - 220.0).abs() < f32::EPSILON);
        assert!((sv.min_primary - SIDEBAR_MIN_WIDTH).abs() < f32::EPSILON);
    }

    #[test]
    fn default_orientation_is_horizontal() {
        assert_eq!(SplitOrientation::default(), SplitOrientation::Horizontal);
    }

    #[test]
    fn collapse_preserves_width() {
        // Simulates collapse_primary / expand_primary — the width is
        // preserved across visibility toggles.
        let mut sv = TestSplitView::new();
        sv.primary_width = 320.0;
        sv.primary_visible = false;
        assert!(!sv.primary_visible);
        assert!((sv.primary_width - 320.0).abs() < f32::EPSILON);
        sv.primary_visible = true;
        assert!((sv.primary_width - 320.0).abs() < f32::EPSILON);
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::{TestAppContext, point, px};

    use super::{SIDEBAR_MIN_WIDTH, SplitView};
    use crate::test_helpers::helpers::{InteractionExt, LocatorExt, setup_test_window};

    const DIVIDER: &str = "split-view-divider";

    fn focus_divider(view: &gpui::Entity<SplitView>, cx: &mut gpui::VisualTestContext) {
        view.update_in(cx, |view, window, cx| {
            view.focus_handle.focus(window, cx);
        });
    }

    #[gpui::test]
    async fn dragging_divider_resizes_and_clears_drag_state(cx: &mut TestAppContext) {
        let (view, cx) = setup_test_window(cx, |_window, cx| SplitView::new(cx));

        let divider = cx.get_element(DIVIDER);
        let start = divider.center();
        let end = point(start.x + px(80.0), start.y);
        cx.drag_between_points(start, end);

        view.update_in(cx, |view, _window, _cx| {
            assert!(
                (view.primary_width() - 340.0).abs() < 2.0,
                "width was {}",
                view.primary_width()
            );
            assert!(!view.is_dragging());
        });
    }

    #[gpui::test]
    async fn keyboard_resize_clamps_between_min_and_max(cx: &mut TestAppContext) {
        let (view, cx) = setup_test_window(cx, |_window, cx| SplitView::new(cx));

        focus_divider(&view, cx);
        cx.press("right");
        view.update_in(cx, |view, _window, _cx| {
            assert!((view.primary_width() - 270.0).abs() < f32::EPSILON);
        });

        cx.press("end");
        view.update_in(cx, |view, _window, _cx| {
            assert!((view.primary_width() - 400.0).abs() < f32::EPSILON);
        });

        cx.press("home");
        view.update_in(cx, |view, _window, _cx| {
            assert!((view.primary_width() - SIDEBAR_MIN_WIDTH).abs() < f32::EPSILON);
        });
    }

    #[gpui::test]
    async fn toggle_primary_roundtrips_visibility(cx: &mut TestAppContext) {
        let (view, cx) = setup_test_window(cx, |_window, cx| SplitView::new(cx));

        view.update_in(cx, |view, _window, cx| {
            assert!(view.is_primary_visible());
            view.collapse_primary(cx);
            assert!(!view.is_primary_visible());
            view.expand_primary(cx);
            assert!(view.is_primary_visible());
            view.toggle_primary(cx);
            assert!(!view.is_primary_visible());
            view.toggle_primary(cx);
            assert!(view.is_primary_visible());
        });
    }
}
