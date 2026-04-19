//! Workflow zoom/pan controls overlay.
//!
//! A configurable control bar with zoom in, zoom out, fit-to-view, and
//! interactive lock/unlock buttons. Supports vertical/horizontal orientation
//! and absolute positioning within a parent container.
//!
//! The parent element must have `relative()` for position-based placement.

use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, GlassSize};
use gpui::prelude::*;
use gpui::{AnyElement, App, ClickEvent, SharedString, Window, div, px};

/// Options for the fit-to-view action, controlling padding and zoom constraints.
use crate::callback_types::OnClick;
#[derive(Debug, Clone)]
pub struct FitViewOptions {
    /// Padding in pixels around the fitted content.
    pub padding: f32,
    /// Minimum zoom level when fitting.
    pub min_zoom: f32,
    /// Maximum zoom level when fitting.
    pub max_zoom: f32,
}

impl Default for FitViewOptions {
    fn default() -> Self {
        Self {
            padding: 50.0,
            min_zoom: 0.1,
            max_zoom: 2.0,
        }
    }
}

/// Position of the controls overlay within its parent.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ControlsPosition {
    TopLeft,
    TopRight,
    #[default]
    BottomLeft,
    BottomRight,
}

/// Layout direction for the control buttons.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ControlsOrientation {
    Horizontal,
    #[default]
    Vertical,
}

/// A control bar with zoom, fit, and interactive toggle actions for the workflow canvas.
#[derive(IntoElement)]
#[allow(clippy::type_complexity)]
pub struct WorkflowControls {
    show_zoom: bool,
    show_fit_view: bool,
    show_interactive: bool,
    /// F7 (#149): surface Undo / Redo as discoverable toolbar actions, not
    /// just keyboard shortcuts. Off by default so existing callers don't
    /// suddenly sprout new buttons; opt-in via `show_undo_redo(true)`.
    show_undo_redo: bool,
    /// Whether the Undo button should render enabled. Host supplies this
    /// from `WorkflowCanvas::can_undo()` each render.
    can_undo: bool,
    /// Whether the Redo button should render enabled.
    can_redo: bool,
    interactive: bool,
    position: ControlsPosition,
    orientation: ControlsOrientation,
    accessibility_label: Option<SharedString>,
    fit_view_options: FitViewOptions,
    on_zoom_in: OnClick,
    on_zoom_out: OnClick,
    on_fit_view: Option<Box<dyn Fn(&FitViewOptions, &ClickEvent, &mut Window, &mut App) + 'static>>,
    on_interactive_change: Option<Box<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>>,
    on_undo: OnClick,
    on_redo: OnClick,
    children: Vec<AnyElement>,
}

impl Default for WorkflowControls {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowControls {
    pub fn new() -> Self {
        Self {
            show_zoom: true,
            show_fit_view: true,
            show_interactive: true,
            show_undo_redo: false,
            can_undo: false,
            can_redo: false,
            interactive: true,
            position: ControlsPosition::default(),
            orientation: ControlsOrientation::default(),
            accessibility_label: Some("Workflow controls".into()),
            fit_view_options: FitViewOptions::default(),
            on_zoom_in: None,
            on_zoom_out: None,
            on_fit_view: None,
            on_interactive_change: None,
            on_undo: None,
            on_redo: None,
            children: Vec::new(),
        }
    }

    pub fn show_zoom(mut self, show: bool) -> Self {
        self.show_zoom = show;
        self
    }

    pub fn show_fit_view(mut self, show: bool) -> Self {
        self.show_fit_view = show;
        self
    }

    pub fn show_interactive(mut self, show: bool) -> Self {
        self.show_interactive = show;
        self
    }

    /// Expose Undo / Redo buttons on the controls bar. Required by the HIG
    /// Undo and redo page ("Consider adding Undo and Redo buttons to your
    /// toolbar for content-editing contexts" — F7 from #149). The buttons
    /// read their enabled state from `can_undo` / `can_redo`, which the
    /// host supplies from `WorkflowCanvas::can_undo()` each render so the
    /// affordance gray-outs when the stack is empty, matching HIG guidance.
    pub fn show_undo_redo(mut self, show: bool) -> Self {
        self.show_undo_redo = show;
        self
    }

    /// Tell the bar whether Undo is currently available. Maps directly to
    /// the Undo button's disabled state.
    pub fn can_undo(mut self, can: bool) -> Self {
        self.can_undo = can;
        self
    }

    /// Tell the bar whether Redo is currently available.
    pub fn can_redo(mut self, can: bool) -> Self {
        self.can_redo = can;
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn position(mut self, position: ControlsPosition) -> Self {
        self.position = position;
        self
    }

    pub fn orientation(mut self, orientation: ControlsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn fit_view_options(mut self, options: FitViewOptions) -> Self {
        self.fit_view_options = options;
        self
    }

    /// Sets the accessibility label for the overall controls group.
    ///
    /// The label is attached to the rendered element via [`AccessibleExt`],
    /// which today no-ops pending GPUI's upstream `accessibility_label` API.
    /// See [`crate::foundations::accessibility`] for the tracking note.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn on_zoom_in(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_zoom_in = Some(Box::new(handler));
        self
    }

    pub fn on_zoom_out(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_zoom_out = Some(Box::new(handler));
        self
    }

    pub fn on_fit_view(
        mut self,
        handler: impl Fn(&FitViewOptions, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_fit_view = Some(Box::new(handler));
        self
    }

    pub fn on_interactive_change(
        mut self,
        handler: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_interactive_change = Some(Box::new(handler));
        self
    }

    /// Wire the Undo button (F7).
    pub fn on_undo(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_undo = Some(Box::new(handler));
        self
    }

    /// Wire the Redo button (F7).
    pub fn on_redo(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_redo = Some(Box::new(handler));
        self
    }

    /// Add a custom control button or element.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for WorkflowControls {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let spacing = theme.spacing_sm;

        // Outer positioned wrapper. The accessibility label is attached via
        // `AccessibleExt::with_accessibility`, which is a structural no-op
        // today because GPUI 0.2.2 (`v0.231.1-pre`) has no public
        // `accessibility_label` API. When that lands, `AccessibleExt` is the
        // single wiring point so no changes are needed here.
        let a11y = {
            let mut props = AccessibilityProps::new().role(AccessibilityRole::Group);
            if let Some(label) = self.accessibility_label.clone() {
                props = props.label(label);
            }
            props
        };
        let mut wrapper = div().absolute().with_accessibility(&a11y);
        match self.position {
            ControlsPosition::TopLeft => {
                wrapper = wrapper.top(spacing).left(spacing);
            }
            ControlsPosition::TopRight => {
                wrapper = wrapper.top(spacing).right(spacing);
            }
            ControlsPosition::BottomLeft => {
                wrapper = wrapper.bottom(spacing).left(spacing);
            }
            ControlsPosition::BottomRight => {
                wrapper = wrapper.bottom(spacing).right(spacing);
            }
        }

        // Inner bar — semi-transparent surface. With the Liquid Glass theme,
        // uses Apple shadow-only approach (no border).
        let glass = &theme.glass;
        let bar_bg = glass.accessible_bg(GlassSize::Small, theme.accessibility_mode);
        let mut bar = div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .px(theme.spacing_xs)
            .py(theme.spacing_xs)
            .bg(bar_bg)
            .rounded(theme.radius_full)
            .shadow(glass.shadows(GlassSize::Small).to_vec());
        bar = crate::foundations::materials::apply_high_contrast_border(bar, theme);

        if self.orientation == ControlsOrientation::Vertical {
            bar = bar.flex_col();
        }

        // Track which groups are rendered for dividers
        let mut has_previous_group = false;

        // Undo / Redo group (F7 — rendered leading of the zoom group so
        // Undo is the first affordance a user's eye lands on, matching
        // Keynote / Freeform anatomy).
        if self.show_undo_redo {
            let can_undo = self.can_undo;
            let can_redo = self.can_redo;
            let mut undo_btn = Button::new("wf-ctrl-undo")
                .icon(Icon::new(IconName::RotateCcw))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(!can_undo);
            if let Some(handler) = self.on_undo.take() {
                undo_btn =
                    undo_btn.on_click(move |event, window, cx| handler(event, window, cx));
            }
            bar = bar.child(undo_btn);

            let mut redo_btn = Button::new("wf-ctrl-redo")
                .icon(Icon::new(IconName::RotateCw))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(!can_redo);
            if let Some(handler) = self.on_redo.take() {
                redo_btn =
                    redo_btn.on_click(move |event, window, cx| handler(event, window, cx));
            }
            bar = bar.child(redo_btn);

            has_previous_group = true;
        }

        // Zoom in/out group
        if self.show_zoom {
            if has_previous_group {
                bar = bar.child(self.divider(theme));
            }
            let mut zoom_in = Button::new("wf-ctrl-zoom-in")
                .icon(Icon::new(IconName::Plus))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm);
            if let Some(handler) = self.on_zoom_in.take() {
                zoom_in = zoom_in.on_click(move |event, window, cx| handler(event, window, cx));
            }
            bar = bar.child(zoom_in);

            let mut zoom_out = Button::new("wf-ctrl-zoom-out")
                .icon(Icon::new(IconName::Minus))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm);
            if let Some(handler) = self.on_zoom_out.take() {
                zoom_out = zoom_out.on_click(move |event, window, cx| handler(event, window, cx));
            }
            bar = bar.child(zoom_out);

            has_previous_group = true;
        }

        // Fit-to-view
        if self.show_fit_view {
            if has_previous_group {
                bar = bar.child(self.divider(theme));
            }

            let mut fit_btn = Button::new("wf-ctrl-fit-view")
                .icon(Icon::new(IconName::Maximize))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm);
            if let Some(handler) = self.on_fit_view.take() {
                let opts = self.fit_view_options.clone();
                fit_btn =
                    fit_btn.on_click(move |event, window, cx| handler(&opts, event, window, cx));
            }
            bar = bar.child(fit_btn);

            has_previous_group = true;
        }

        // Interactive toggle
        if self.show_interactive {
            if has_previous_group {
                bar = bar.child(self.divider(theme));
            }

            let icon = if self.interactive {
                IconName::Unlock
            } else {
                IconName::Lock
            };
            let new_state = !self.interactive;

            let mut lock_btn = Button::new("wf-ctrl-interactive")
                .icon(Icon::new(icon))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm);
            if let Some(handler) = self.on_interactive_change.take() {
                lock_btn = lock_btn.on_click(move |event, window, cx| {
                    handler(new_state, event, window, cx);
                });
            }
            bar = bar.child(lock_btn);
        }

        // Custom children
        for child in self.children {
            bar = bar.child(child);
        }

        wrapper.child(bar)
    }
}

impl WorkflowControls {
    fn divider(&self, theme: &TahoeTheme) -> impl IntoElement {
        let d = div().bg(theme.border);
        if self.orientation == ControlsOrientation::Vertical {
            d.w_full().h(px(1.0))
        } else {
            d.h(px(16.0)).w(px(1.0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ControlsOrientation, ControlsPosition, FitViewOptions, WorkflowControls};
    use core::prelude::v1::test;

    #[test]
    fn position_default_is_bottom_left() {
        assert_eq!(ControlsPosition::default(), ControlsPosition::BottomLeft);
    }

    #[test]
    fn orientation_default_is_vertical() {
        assert_eq!(
            ControlsOrientation::default(),
            ControlsOrientation::Vertical
        );
    }

    #[test]
    fn builder_defaults() {
        let c = WorkflowControls::new();
        assert!(c.show_zoom);
        assert!(c.show_fit_view);
        assert!(c.show_interactive);
        assert!(c.interactive);
        assert_eq!(c.position, ControlsPosition::BottomLeft);
        assert_eq!(c.orientation, ControlsOrientation::Vertical);
        assert_eq!(
            c.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Workflow controls")
        );
        assert!(c.children.is_empty());
    }

    #[test]
    fn builder_chaining() {
        let c = WorkflowControls::new()
            .position(ControlsPosition::TopRight)
            .orientation(ControlsOrientation::Horizontal)
            .show_zoom(false)
            .show_fit_view(false)
            .show_interactive(false)
            .interactive(false)
            .accessibility_label("test");
        assert_eq!(c.position, ControlsPosition::TopRight);
        assert_eq!(c.orientation, ControlsOrientation::Horizontal);
        assert!(!c.show_zoom);
        assert!(!c.show_fit_view);
        assert!(!c.show_interactive);
        assert!(!c.interactive);
        assert_eq!(
            c.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("test")
        );
    }

    #[test]
    fn fit_view_options_defaults() {
        let opts = FitViewOptions::default();
        assert_eq!(opts.padding, 50.0);
        assert_eq!(opts.min_zoom, 0.1);
        assert_eq!(opts.max_zoom, 2.0);
    }

    #[test]
    fn fit_view_options_builder() {
        let opts = FitViewOptions {
            padding: 20.0,
            min_zoom: 0.5,
            max_zoom: 1.5,
        };
        let c = WorkflowControls::new().fit_view_options(opts);
        assert_eq!(c.fit_view_options.padding, 20.0);
        assert_eq!(c.fit_view_options.min_zoom, 0.5);
        assert_eq!(c.fit_view_options.max_zoom, 1.5);
    }

    #[test]
    fn fit_view_options_zero_padding() {
        let opts = FitViewOptions {
            padding: 0.0,
            ..FitViewOptions::default()
        };
        assert_eq!(opts.padding, 0.0);
        assert_eq!(opts.min_zoom, 0.1);
    }

    #[test]
    fn fit_view_options_negative_padding_constructible() {
        // Negative padding is allowed at the struct level; validation happens in canvas.
        let opts = FitViewOptions {
            padding: -10.0,
            min_zoom: 0.1,
            max_zoom: 2.0,
        };
        assert_eq!(opts.padding, -10.0);
    }

    #[test]
    fn fit_view_options_inverted_zoom_constructible() {
        // min > max is allowed at the struct level; canvas sanitizes before use.
        let opts = FitViewOptions {
            padding: 50.0,
            min_zoom: 3.0,
            max_zoom: 0.5,
        };
        assert_eq!(opts.min_zoom, 3.0);
        assert_eq!(opts.max_zoom, 0.5);
    }
}
