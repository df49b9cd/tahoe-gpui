//! Reusable flex header component for code display panels.
//!
//! Provides a horizontal flex container with configurable layout,
//! commonly used as the header row in terminal, commit, stack trace,
//! and other code display components.

use crate::foundations::theme::{ActiveTheme};
use gpui::prelude::*;
use gpui::{AnyElement, App, Window, div};

/// Vertical alignment for header children.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FlexAlign {
    /// `items-center` (default).
    #[default]
    Center,
    /// `items-start`.
    Start,
}

/// A horizontal flex header with configurable layout.
///
/// Accumulates children into a flex row. Used as the standard header
/// pattern across code display components (terminal, commit, stack trace,
/// test results, artifact, etc.).
///
/// By default renders: `flex + items_center + justify_between + px + py`.
/// Use builder methods to toggle border, gap, padding, and alignment.
#[derive(IntoElement)]
pub struct FlexHeader {
    pub(crate) children: Vec<AnyElement>,
    border: bool,
    gap: bool,
    padding: bool,
    justify_between: bool,
    align: FlexAlign,
}

impl Default for FlexHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl FlexHeader {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            border: false,
            gap: false,
            padding: true,
            justify_between: true,
            align: FlexAlign::Center,
        }
    }

    /// Add a child element to the header.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Add multiple child elements to the header.
    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }

    /// Add a bottom border line (default: false).
    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    /// Add a gap between children (default: false).
    pub fn gap(mut self, gap: bool) -> Self {
        self.gap = gap;
        self
    }

    /// Toggle horizontal padding `px(spacing_md)` and vertical `py(spacing_sm)` (default: true).
    pub fn padding(mut self, padding: bool) -> Self {
        self.padding = padding;
        self
    }

    /// Toggle `justify-between` (default: true).
    pub fn justify_between(mut self, jb: bool) -> Self {
        self.justify_between = jb;
        self
    }

    /// Set vertical alignment (default: [`FlexAlign::Center`]).
    pub fn align(mut self, align: FlexAlign) -> Self {
        self.align = align;
        self
    }
}

impl RenderOnce for FlexHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut el = div().flex();

        // Vertical alignment
        el = match self.align {
            FlexAlign::Center => el.items_center(),
            FlexAlign::Start => el.items_start(),
        };

        // Justify between
        if self.justify_between {
            el = el.justify_between();
        }

        // Gap
        if self.gap {
            el = el.gap(theme.spacing_sm);
        }

        // Padding
        if self.padding {
            el = el.px(theme.spacing_md).py(theme.spacing_sm);
        }

        // Border
        if self.border {
            el = el.border_b_1().border_color(theme.border);
        }

        el.children(self.children)
    }
}

// =============================================================================
// FlexActions
// =============================================================================

/// A horizontal flex container for action buttons.
///
/// Used as the actions row in code display components (terminal, commit,
/// stack trace, artifact). Renders children in a flex row with gap spacing.
#[derive(IntoElement)]
pub struct FlexActions {
    pub(crate) children: Vec<AnyElement>,
}

impl Default for FlexActions {
    fn default() -> Self {
        Self::new()
    }
}

impl FlexActions {
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

impl RenderOnce for FlexActions {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .children(self.children)
    }
}

// =============================================================================
// FlexContent
// =============================================================================

/// A vertical flex container for content sections.
///
/// Used as the content body in code display components. Renders children
/// in a vertical flex column with padding.
#[derive(IntoElement)]
pub struct FlexContent {
    pub(crate) children: Vec<AnyElement>,
}

impl Default for FlexContent {
    fn default() -> Self {
        Self::new()
    }
}

impl FlexContent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for FlexContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .children(self.children)
    }
}
