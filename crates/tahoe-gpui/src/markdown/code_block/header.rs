//! Code block header and its left/right child components.

use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, SharedString, Window, div};

// -- CodeBlockHeader ----------------------------------------------------------

/// Flex container with `justify-between` layout for the code block header.
///
/// Use `left()` for title/filename elements and `right()` for action buttons.
#[derive(IntoElement, Default)]
pub struct CodeBlockHeader {
    pub(crate) children_left: Vec<AnyElement>,
    pub(crate) children_right: Vec<AnyElement>,
}

impl CodeBlockHeader {
    pub fn new() -> Self {
        Self {
            children_left: Vec::new(),
            children_right: Vec::new(),
        }
    }

    /// Add an element to the left side (title area).
    pub fn left(mut self, child: impl IntoElement) -> Self {
        self.children_left.push(child.into_any_element());
        self
    }

    /// Add an element to the right side (actions area).
    pub fn right(mut self, child: impl IntoElement) -> Self {
        self.children_right.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CodeBlockHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.surface)
            .px(theme.spacing_sm)
            .py(theme.spacing_xs)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .children(self.children_left),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .children(self.children_right),
            )
    }
}

// -- CodeBlockTitle -----------------------------------------------------------

/// Left-aligned container with gap for icon and filename.
#[derive(IntoElement, Default)]
pub struct CodeBlockTitle {
    pub(crate) children: Vec<AnyElement>,
}

impl CodeBlockTitle {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CodeBlockTitle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .children(self.children)
    }
}

// -- CodeBlockFilename --------------------------------------------------------

/// Monospace filename display.
#[derive(IntoElement)]
pub struct CodeBlockFilename {
    pub(crate) name: SharedString,
}

impl CodeBlockFilename {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self { name: name.into() }
    }
}

impl RenderOnce for CodeBlockFilename {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .font(theme.mono_font())
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .child(self.name)
    }
}

// -- CodeBlockActions ---------------------------------------------------------

/// Right-aligned container for action buttons with gap.
#[derive(IntoElement, Default)]
pub struct CodeBlockActions {
    pub(crate) children: Vec<AnyElement>,
}

impl CodeBlockActions {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CodeBlockActions {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .children(self.children)
    }
}
