//! Stack trace display component.
//!
//! Provides both a stateful `StackTraceView` (Entity-based, for direct use) and
//! composable stateless subcomponents (`StackTraceHeader`, `StackTraceError`,
//! `StackTraceErrorType`, `StackTraceErrorMessage`, `StackTraceActions`,
//! `StackTraceContent`, `StackTraceFrames`) that can be used independently.

#[cfg(test)]
mod tests;
mod types;

pub use types::{ParsedStackTrace, StackFrame, parse_stack_trace};

use crate::callback_types::{OnFileClick, OnFileClickRc, OnToggle};
use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, Entity, FontWeight, KeyDownEvent, SharedString, Window,
    div, px,
};
use std::rc::Rc;

// ─── Stateless subcomponents ────────────────────────────────────────────────

/// Displays the parsed error type (e.g. "TypeError") with semibold weight and error color.
#[derive(IntoElement)]
pub struct StackTraceErrorType {
    error_type: SharedString,
}

impl StackTraceErrorType {
    pub fn new(error_type: impl Into<SharedString>) -> Self {
        Self {
            error_type: error_type.into(),
        }
    }
}

impl RenderOnce for StackTraceErrorType {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
            .text_color(theme.error)
            .flex_shrink_0()
            .child(self.error_type)
    }
}

/// Displays the error message text.
#[derive(IntoElement)]
pub struct StackTraceErrorMessage {
    message: SharedString,
}

impl StackTraceErrorMessage {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl RenderOnce for StackTraceErrorMessage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .text_color(theme.text)
            .overflow_hidden()
            .child(self.message)
    }
}

/// Groups error type and error message in a flex row.
#[derive(IntoElement)]
pub struct StackTraceError {
    error_type: Option<SharedString>,
    message: SharedString,
}

impl StackTraceError {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            error_type: None,
            message: message.into(),
        }
    }

    /// Set the error type (e.g. "TypeError").
    pub fn error_type(mut self, error_type: impl Into<SharedString>) -> Self {
        self.error_type = Some(error_type.into());
        self
    }
}

impl RenderOnce for StackTraceError {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut row = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .flex_1()
            .overflow_hidden();
        if let Some(error_type) = self.error_type {
            row = row.child(StackTraceErrorType::new(error_type));
        }
        row.child(StackTraceErrorMessage::new(self.message))
    }
}

/// Container for action buttons (copy, expand, etc.).
///
/// Type alias for [`crate::components::layout_and_organization::FlexActions`]
/// — a horizontal flex row with gap spacing.
pub type StackTraceActions = crate::components::layout_and_organization::FlexActions;

/// Presentational header showing alert icon, error info, action children, and expand chevron.
///
/// Does NOT own click/keyboard handlers — the parent wraps it.
#[derive(IntoElement)]
pub struct StackTraceHeader {
    error_type: Option<SharedString>,
    message: SharedString,
    is_open: bool,
    children: Vec<AnyElement>,
}

impl StackTraceHeader {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            error_type: None,
            message: message.into(),
            is_open: false,
            children: Vec::new(),
        }
    }

    /// Set the error type displayed before the message.
    pub fn error_type(mut self, error_type: impl Into<SharedString>) -> Self {
        self.error_type = Some(error_type.into());
        self
    }

    /// Set whether the parent collapsible is open (controls chevron direction).
    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Add an action child (e.g. CopyButton).
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for StackTraceHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut error = StackTraceError::new(self.message);
        if let Some(error_type) = self.error_type {
            error = error.error_type(error_type);
        }

        let mut row = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .child(
                Icon::new(IconName::AlertTriangle)
                    .size(theme.icon_size_inline)
                    .color(theme.error),
            )
            .child(error);

        for child in self.children {
            row = row.child(child);
        }

        row.child(
            Icon::new(if self.is_open {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            })
            .size(theme.icon_size_inline)
            .color(theme.text_muted),
        )
    }
}

/// Collapsible content area for stack frames.
#[derive(IntoElement)]
pub struct StackTraceContent {
    children: Vec<AnyElement>,
    max_height: f32,
}

impl Default for StackTraceContent {
    fn default() -> Self {
        Self::new()
    }
}

impl StackTraceContent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            max_height: 400.0,
        }
    }

    /// Set the maximum height (in pixels) for the content area.
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for StackTraceContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut container = div()
            .id(next_element_id("stack-trace-content"))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.hover)
            .max_h(px(self.max_height))
            .overflow_y_scroll();

        for child in self.children {
            container = container.child(child);
        }
        container
    }
}

/// Renders parsed stack frames, filtering internal frames when configured.
#[derive(IntoElement)]
pub struct StackTraceFrames {
    frames: Rc<Vec<StackFrame>>,
    show_internal_frames: bool,
    on_file_click: OnFileClickRc,
}

impl StackTraceFrames {
    pub fn new(frames: impl Into<Rc<Vec<StackFrame>>>) -> Self {
        Self {
            frames: frames.into(),
            show_internal_frames: true,
            on_file_click: None,
        }
    }

    /// Set whether internal frames (node_modules, node:, internal/) are shown.
    pub fn show_internal_frames(mut self, show: bool) -> Self {
        self.show_internal_frames = show;
        self
    }

    /// Set a handler for clickable file paths.
    pub fn on_file_click(
        mut self,
        handler: impl Fn(&str, Option<u32>, Option<u32>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_file_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for StackTraceFrames {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        // HIG Text Views: "Use monospaced fonts for code, file paths, and
        // line references." Applying `font_mono` explicitly here (rather
        // than relying on inheritance from `StackTraceView`) keeps columnar
        // alignment when `StackTraceFrames` is composed outside of the
        // stateful view.
        let mut container = div().flex().flex_col().gap(px(2.0)).font(theme.font_mono());

        for frame in self.frames.iter() {
            if !self.show_internal_frames && frame.is_internal {
                continue;
            }
            let text_color = if frame.is_internal {
                theme.pending
            } else {
                theme.text
            };

            let mut line = div()
                .flex()
                .flex_wrap()
                .text_style(TextStyle::Caption1, theme)
                .child(div().text_color(theme.text_muted).child("at "));

            if let Some(ref func) = frame.function_name {
                line = line.child(div().text_color(text_color).child(format!("{} ", func)));
            }

            if let Some(ref path) = frame.file_path {
                let mut location = path.clone();
                if let Some(ln) = frame.line_number {
                    location.push_str(&format!(":{}", ln));
                    if let Some(col) = frame.column_number {
                        location.push_str(&format!(":{}", col));
                    }
                }
                line = line.child(div().text_color(theme.text_muted).child("("));

                // HIG §Accessibility: affordances must signal interactivity
                // consistently. Always render paths in link-styled text so
                // users can recognise the navigable element; when no click
                // handler is registered the element renders as
                // non-interactive (cursor_default, no click binding) to
                // avoid a misleading affordance. Internal frames stay in
                // the pending (dimmed) colour rather than accent so they
                // read as de-emphasised even when interactive.
                let link_color = if frame.is_internal {
                    theme.pending
                } else {
                    theme.accent
                };

                if let Some(ref handler) = self.on_file_click {
                    let click_path = path.clone();
                    let click_line = frame.line_number;
                    let click_col = frame.column_number;
                    let handler = Rc::clone(handler);
                    line = line.child(
                        div()
                            .id(next_element_id("stack-frame"))
                            .text_color(link_color)
                            .underline()
                            .text_decoration_color(link_color)
                            .cursor_pointer()
                            .on_click(move |_event, window, cx| {
                                handler(&click_path, click_line, click_col, window, cx);
                            })
                            .child(location),
                    );
                } else {
                    line = line.child(
                        div()
                            .text_color(link_color)
                            .underline()
                            .text_decoration_color(link_color)
                            .cursor_default()
                            .child(location),
                    );
                }

                line = line.child(div().text_color(theme.text_muted).child(")"));
            }

            container = container.child(line);
        }

        container
    }
}

// ─── Stateful Entity ────────────────────────────────────────────────────────

/// A stack trace display with collapsible frames.
pub struct StackTraceView {
    element_id: ElementId,
    trace: ParsedStackTrace,
    raw: String,
    is_expanded: bool,
    controlled_open: Option<bool>,
    on_open_change: OnToggle,
    on_file_click: OnFileClick,
    frames_rc: Rc<Vec<StackFrame>>,
    copy_button: Option<Entity<CopyButton>>,
    show_internal_frames: bool,
    max_height: f32,
}

impl StackTraceView {
    pub fn new(trace_str: &str, _cx: &mut Context<Self>) -> Self {
        let trace = parse_stack_trace(trace_str);
        let frames_rc = Rc::new(trace.frames.clone());
        Self {
            element_id: next_element_id("stack-trace"),
            trace,
            raw: trace_str.to_string(),
            is_expanded: false,
            controlled_open: None,
            on_open_change: None,
            on_file_click: None,
            frames_rc,
            copy_button: None,
            show_internal_frames: true,
            max_height: 400.0,
        }
    }

    /// Set the default open state (must be called before first render).
    pub fn set_default_open(&mut self, open: bool) {
        self.is_expanded = open;
    }

    /// Set the controlled open state. When set, the component does not
    /// manage its own expand/collapse state — the parent drives it.
    pub fn set_open(&mut self, open: bool) {
        self.controlled_open = Some(open);
    }

    /// Clear the controlled open state, reverting to uncontrolled mode.
    pub fn clear_open(&mut self) {
        self.controlled_open = None;
    }

    /// Set a callback invoked when the open state changes.
    pub fn set_on_open_change(&mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) {
        self.on_open_change = Some(Box::new(handler));
    }

    pub fn set_on_file_click(
        &mut self,
        handler: impl Fn(&str, Option<u32>, Option<u32>, &mut Window, &mut App) + 'static,
    ) {
        self.on_file_click = Some(Box::new(handler));
    }

    /// Set whether internal frames (node_modules, node:, internal/) are shown.
    /// Default is `true` (shown but dimmed).
    pub fn set_show_internal_frames(&mut self, show: bool) {
        self.show_internal_frames = show;
    }

    /// Set the maximum height (in pixels) for the frames container.
    /// Default is 400.
    pub fn set_max_height(&mut self, height: f32) {
        self.max_height = height;
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let current = self.controlled_open.unwrap_or(self.is_expanded);
        let new_state = !current;

        if self.controlled_open.is_none() {
            self.is_expanded = new_state;
        }

        if let Some(ref handler) = self.on_open_change {
            handler(new_state, window, cx);
        }

        cx.notify();
    }
}

impl Render for StackTraceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_expanded = self.controlled_open.unwrap_or(self.is_expanded);

        // Initialize copy button lazily (needs mutable cx before borrowing theme)
        let copy_button = if let Some(btn) = &self.copy_button {
            btn.clone()
        } else {
            let btn = CopyButton::new(&self.raw, cx);
            self.copy_button = Some(btn.clone());
            btn
        };

        let theme = cx.theme();

        // Build header using stateless subcomponents
        let mut header = StackTraceHeader::new(self.trace.error_message.clone())
            .is_open(is_expanded)
            .child(copy_button);
        if let Some(ref error_type) = self.trace.error_type {
            header = header.error_type(error_type.clone());
        }

        let mut container = div()
            .flex()
            .flex_col()
            .bg(theme.background)
            .rounded(theme.radius_lg)
            .border_1()
            .border_color(theme.border)
            .font(theme.font_mono())
            .text_style(TextStyle::Subheadline, theme)
            .overflow_hidden();

        // Header (always visible, clickable to toggle)
        container = container.child(
            div()
                .id(self.element_id.clone())
                .px(theme.spacing_md)
                .py(theme.spacing_sm)
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover))
                .on_click(cx.listener(|this, _event: &ClickEvent, window, cx| {
                    this.toggle(window, cx);
                }))
                .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                    if crate::foundations::keyboard::is_activation_key(event) {
                        cx.stop_propagation();
                        this.toggle(window, cx);
                    }
                }))
                .child(header),
        );

        // Collapsible content using stateless subcomponents
        if is_expanded && !self.trace.frames.is_empty() {
            let mut frames = StackTraceFrames::new(Rc::clone(&self.frames_rc))
                .show_internal_frames(self.show_internal_frames);

            if self.on_file_click.is_some() {
                let entity = cx.entity().clone();
                frames = frames.on_file_click(move |path, line, col, window, cx| {
                    let path = path.to_string();
                    entity.update(cx, |this, _cx| {
                        if let Some(ref handler) = this.on_file_click {
                            handler(&path, line, col, window, &mut *_cx);
                        }
                    });
                });
            }

            container = container.child(
                StackTraceContent::new()
                    .max_height(self.max_height)
                    .child(frames),
            );
        }

        container
    }
}
