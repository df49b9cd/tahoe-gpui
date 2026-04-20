//! Code block component with syntax highlighting and composable sub-components.
//!
//! Provides a code display with tree-sitter syntax highlighting, optional line
//! numbers, copy-to-clipboard, and language selection. Supports both a convenience
//! builder API and compound composition.
//!
//! # Convenience builder
//! ```ignore
//! CodeBlockView::new("fn main() {}")
//!     .language(Some("rust".into()))
//!     .filename("main.rs")
//!     .show_line_numbers(true)
//!     .max_lines(20)
//! ```
//!
//! # Compound composition
//! ```ignore
//! CodeBlockView::from_parts("fn main() {}")
//!     .language(Some("rust".into()))
//!     .header(
//!         CodeBlockHeader::new()
//!             .left(CodeBlockFilename::new("main.rs"))
//!             .right(CodeBlockActions::new().child(copy_button)),
//!     )
//!     .show_line_numbers(true)
//! ```

mod header;
#[cfg(test)]
mod tests;

pub use header::{CodeBlockActions, CodeBlockFilename, CodeBlockHeader, CodeBlockTitle};

use super::syntax;
use crate::callback_types::rc_wrap;
use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::components::presentation::popover::{Popover, PopoverPlacement};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ElementId, Entity, KeyDownEvent, SharedString, StyledText, Window, div, px,
};
use itoa;

type IndexChangeHandler = Box<dyn Fn(usize, &mut Window, &mut App) + 'static>;
type ToggleHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;

// -- LanguageVariant ----------------------------------------------------------

/// A language variant for the language selector.
#[derive(Clone)]
pub struct LanguageVariant {
    pub label: SharedString,
    pub language: String,
    pub code: String,
}

// -- CodeBlockContent ---------------------------------------------------------

/// Low-level syntax-highlighted code body.
///
/// Handles tree-sitter highlighting and optional line numbers. Can be used
/// standalone or inside a `CodeBlockView`.
#[derive(IntoElement)]
pub struct CodeBlockContent {
    code: String,
    language: Option<String>,
    show_line_numbers: bool,
}

impl CodeBlockContent {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            language: None,
            show_line_numbers: false,
        }
    }

    pub fn language(mut self, lang: Option<String>) -> Self {
        self.language = lang;
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }
}

impl RenderOnce for CodeBlockContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let font_mono = theme.font_mono();
        let lang = self.language.as_deref().unwrap_or("");
        let (styled_code, highlights) =
            syntax::build_styled_highlights(&self.code, lang, &theme.syntax);

        // HIG macOS text size minimum is 10 pt; comfortable reading for
        // code is 12–13 pt. `TextStyle::Callout` (12 pt) is the smallest
        // HIG style that clears the comfort floor.
        if self.show_line_numbers {
            let lines: Vec<&str> = self.code.split('\n').collect();
            div()
                .overflow_hidden()
                .p(theme.spacing_md)
                .font(font_mono.clone())
                .text_style(TextStyle::Callout, theme)
                .text_color(theme.text)
                .flex()
                .child(
                    // Line numbers share the code body's TextStyle so
                    // every gutter row lines up vertically with its
                    // corresponding source line. A smaller style here
                    // drifts the numbers out of alignment once the
                    // body scales up (e.g. Dynamic Type).
                    div().flex().flex_col().mr(theme.spacing_md).children(
                        lines.iter().enumerate().map(|(i, _)| {
                            let mut buf = itoa::Buffer::new();
                            let line_num = SharedString::from(buf.format(i + 1).to_owned());
                            div()
                                .text_color(theme.text_muted)
                                .flex()
                                .justify_end()
                                .min_w(px(32.0))
                                .child(line_num)
                        }),
                    ),
                )
                .child(
                    // HIG Layout: long lines need a scroll affordance
                    // rather than silent clipping. `overflow_x_scroll`
                    // lets GPUI expose a horizontal scrollbar and keeps
                    // `whitespace_nowrap` semantics so single-line
                    // tokens don't break mid-line.
                    div()
                        .id("code-block-content")
                        .flex_1()
                        .overflow_x_scroll()
                        .child(StyledText::new(styled_code).with_highlights(highlights)),
                )
                .into_any_element()
        } else {
            div()
                .id("code-block-content-no-gutter")
                .p(theme.spacing_md)
                .font(font_mono.clone())
                .text_style(TextStyle::Callout, theme)
                .text_color(theme.text)
                .overflow_x_scroll()
                .child(StyledText::new(styled_code).with_highlights(highlights))
                .into_any_element()
        }
    }
}

// -- CodeBlockContainer -------------------------------------------------------

/// Outer wrapper for a code block with border, rounded corners, and code
/// background.
///
/// This is the GPUI equivalent of the AI SDK `CodeBlockContainer` which uses
/// CSS `content-visibility` for performance. GPUI does not have an equivalent
/// property, but this struct serves as the extension point for future
/// virtualization.
#[derive(IntoElement, Default)]
pub struct CodeBlockContainer {
    children: Vec<AnyElement>,
}

impl CodeBlockContainer {
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

impl RenderOnce for CodeBlockContainer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .w_full()
            .overflow_hidden()
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border)
            .bg(theme.code_bg)
            .children(self.children)
    }
}

// -- CodeBlockLanguageSelector ------------------------------------------------

/// Dropdown language selector for multi-language code blocks.
///
/// Wraps a `Popover` showing variant labels as a vertical list. The parent
/// manages `is_open` state via `on_toggle`.
#[derive(IntoElement)]
pub struct CodeBlockLanguageSelector {
    id: ElementId,
    variants: Vec<LanguageVariant>,
    active_index: usize,
    is_open: bool,
    on_change: Option<IndexChangeHandler>,
    on_toggle: Option<ToggleHandler>,
}

impl CodeBlockLanguageSelector {
    pub fn new(id: impl Into<ElementId>, variants: Vec<LanguageVariant>) -> Self {
        Self {
            id: id.into(),
            variants,
            active_index: 0,
            is_open: false,
            on_change: None,
            on_toggle: None,
        }
    }

    pub fn active(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    pub fn open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for CodeBlockLanguageSelector {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        if self.variants.is_empty() {
            return div().into_any_element();
        }

        let idx = self.active_index.min(self.variants.len() - 1);
        let active_label = self.variants[idx].label.clone();

        // Trigger: active label + chevron
        let on_toggle = rc_wrap(self.on_toggle);
        let is_open = self.is_open;

        let toggle_for_trigger = on_toggle.clone();
        let toggle_for_trigger_kb = on_toggle.clone();
        let trigger = div()
            .id(ElementId::from((self.id.clone(), "trigger")))
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .cursor_pointer()
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .hover(|s| s.text_color(theme.text))
            .child(active_label)
            .child(
                Icon::new(if is_open {
                    IconName::ChevronUp
                } else {
                    IconName::ChevronDown
                })
                .size(px(12.0)),
            )
            .on_click(move |_event, window, cx| {
                if let Some(ref handler) = toggle_for_trigger {
                    handler(!is_open, window, cx);
                }
            })
            .on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_activation_key(event)
                    && let Some(ref handler) = toggle_for_trigger_kb
                {
                    cx.stop_propagation();
                    handler(!is_open, window, cx);
                }
            });

        // Content: vertical list of variant labels
        let on_change = rc_wrap(self.on_change);
        let toggle_for_dismiss = on_toggle.clone();

        let mut content = div().flex().flex_col().min_w(px(120.0));
        let item_id_prefix: SharedString = format!("{:?}-item", self.id).into();
        for (i, variant) in self.variants.iter().enumerate() {
            let is_active = i == idx;
            let label = variant.label.clone();
            let on_change_ref = on_change.clone();
            let toggle_ref = on_toggle.clone();
            let on_change_ref_kb = on_change.clone();
            let toggle_ref_kb = on_toggle.clone();

            content = content.child(
                div()
                    .id(ElementId::NamedInteger(item_id_prefix.clone(), i as u64))
                    .px(theme.spacing_sm)
                    .py(theme.spacing_xs)
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(if is_active {
                        theme.text
                    } else {
                        theme.text_muted
                    })
                    .cursor_pointer()
                    .hover(|s| s.bg(theme.hover))
                    .when(is_active, |el| el.font_weight(gpui::FontWeight::SEMIBOLD))
                    .child(label)
                    .on_click(move |_event, window, cx| {
                        if let Some(ref handler) = on_change_ref {
                            handler(i, window, cx);
                        }
                        if let Some(ref handler) = toggle_ref {
                            handler(false, window, cx);
                        }
                    })
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        if crate::foundations::keyboard::is_activation_key(event) {
                            cx.stop_propagation();
                            if let Some(ref handler) = on_change_ref_kb {
                                handler(i, window, cx);
                            }
                            if let Some(ref handler) = toggle_ref_kb {
                                handler(false, window, cx);
                            }
                        }
                    }),
            );
        }

        Popover::new(self.id, trigger, content)
            .open(self.is_open)
            .placement(PopoverPlacement::BelowLeft)
            .when_some(toggle_for_dismiss, |popover, handler| {
                popover.on_dismiss(move |window, cx| handler(false, window, cx))
            })
            .into_any_element()
    }
}

// -- CodeBlockView ------------------------------------------------------------

/// A code block with optional language label, line numbers, copy button,
/// and tree-sitter syntax highlighting.
///
/// Supports both convenience builder and compound composition APIs.
#[derive(IntoElement)]
pub struct CodeBlockView {
    code: String,
    language: Option<String>,
    filename: Option<SharedString>,
    show_line_numbers: bool,
    show_header: bool,
    header_actions: Vec<AnyElement>,
    /// Maximum lines to show before collapsing (None = no collapse).
    max_lines: Option<usize>,
    /// Whether the block is currently expanded (overrides max_lines).
    expanded: bool,
    /// Optional pre-created CopyButton entity for persistent state.
    copy_button: Option<Entity<CopyButton>>,
    /// Language variants for language selector.
    language_variants: Vec<LanguageVariant>,
    /// Currently active variant index.
    active_variant_index: usize,
    /// Callback when a language variant label is clicked (convenience API).
    on_variant_change: Option<IndexChangeHandler>,
    /// Custom header element (compound API).
    custom_header: Option<AnyElement>,
    /// Custom footer element (compound API).
    custom_footer: Option<AnyElement>,
}

impl CodeBlockView {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            language: None,
            filename: None,
            show_line_numbers: false,
            show_header: true,
            header_actions: Vec::new(),
            max_lines: None,
            expanded: false,
            copy_button: None,
            language_variants: Vec::new(),
            active_variant_index: 0,
            on_variant_change: None,
            custom_header: None,
            custom_footer: None,
        }
    }

    /// Create a code block for compound composition.
    ///
    /// Functionally identical to `new()` — all builders work with either
    /// constructor. The name signals intent to use `header()` / `footer()`
    /// for custom chrome instead of the convenience API.
    pub fn from_parts(code: impl Into<String>) -> Self {
        Self::new(code)
    }

    pub fn language(mut self, lang: Option<String>) -> Self {
        self.language = lang;
        self
    }

    /// Set the filename displayed in the header (convenience API).
    pub fn filename(mut self, name: impl Into<SharedString>) -> Self {
        self.filename = Some(name.into());
        self
    }

    pub fn show_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn show_header(mut self, show: bool) -> Self {
        self.show_header = show;
        self
    }

    /// Set maximum visible lines before showing "Show more" (None = no limit).
    pub fn max_lines(mut self, max: usize) -> Self {
        self.max_lines = Some(max);
        self
    }

    /// Mark the code block as expanded (overrides max_lines).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Add a custom action element to the header (convenience API).
    pub fn header_action(mut self, action: impl IntoElement) -> Self {
        self.header_actions.push(action.into_any_element());
        self
    }

    /// Provide a pre-created CopyButton entity for persistent copy state.
    pub fn copy_button(mut self, button: Entity<CopyButton>) -> Self {
        self.copy_button = Some(button);
        self
    }

    /// Set language variants for a language selector (convenience API).
    pub fn language_variants(mut self, variants: Vec<LanguageVariant>) -> Self {
        self.language_variants = variants;
        self
    }

    /// Set the active variant index.
    pub fn active_variant_index(mut self, index: usize) -> Self {
        self.active_variant_index = index;
        self
    }

    /// Set a callback for when a language variant label is clicked (convenience API).
    pub fn on_variant_change(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_variant_change = Some(Box::new(handler));
        self
    }

    /// Set a custom header element (compound API).
    ///
    /// When set, the convenience header (filename/language label, header_actions)
    /// is not rendered.
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.custom_header = Some(header.into_any_element());
        self
    }

    /// Set a custom footer element (compound API).
    ///
    /// When set, the auto-generated "Show N more lines" collapse footer is not
    /// rendered.
    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.custom_footer = Some(footer.into_any_element());
        self
    }
}

impl RenderOnce for CodeBlockView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        debug_assert!(
            self.custom_header.is_none()
                || (self.filename.is_none()
                    && self.header_actions.is_empty()
                    && self.language_variants.is_empty()
                    && self.on_variant_change.is_none()),
            "CodeBlockView: use either `header()` (compound) or \
             `filename()`/`header_action()`/`language_variants()`/`on_variant_change()` \
             (convenience), not both"
        );

        // Resolve code/language from active variant if variants are set
        let (code, language) = if !self.language_variants.is_empty() {
            let idx = self
                .active_variant_index
                .min(self.language_variants.len() - 1);
            let variant = &self.language_variants[idx];
            (variant.code.clone(), Some(variant.language.clone()))
        } else {
            (self.code.clone(), self.language.clone())
        };

        // Create or reuse CopyButton (before borrowing theme)
        let copy_button = if let Some(btn) = self.copy_button {
            btn.update(cx, |btn, _cx| btn.set_content(code.clone()));
            btn
        } else {
            CopyButton::new(code.clone(), cx)
        };

        let theme = cx.theme();

        let mut container = CodeBlockContainer::new();

        // Header: prefer custom_header, then auto-generate
        if let Some(custom_header) = self.custom_header {
            container = container.child(custom_header);
        } else if self.show_header {
            let title_text = if let Some(ref filename) = self.filename {
                filename.to_string()
            } else {
                language.as_deref().unwrap_or("text").to_string()
            };

            let mut actions = CodeBlockActions::new();
            for action in self.header_actions {
                actions = actions.child(action);
            }
            actions = actions.child(copy_button);

            // Title area with language variant labels or static label
            let title_element: AnyElement = if !self.language_variants.is_empty() {
                let on_variant_change = rc_wrap(self.on_variant_change);
                let mut selector = div().flex().items_center().gap(theme.spacing_xs);

                for (i, variant) in self.language_variants.iter().enumerate() {
                    let is_active = i
                        == self
                            .active_variant_index
                            .min(self.language_variants.len() - 1);
                    let label_color = if is_active {
                        theme.text
                    } else {
                        theme.text_muted
                    };
                    let label = variant.label.clone();

                    let base = div()
                        .text_style(TextStyle::Caption1, theme)
                        .text_color(label_color)
                        .when(is_active, |el| el.font_weight(gpui::FontWeight::SEMIBOLD))
                        .child(label);

                    let label_el: AnyElement = if let Some(ref handler_rc) = on_variant_change {
                        let handler_click = handler_rc.clone();
                        let handler_kb = handler_rc.clone();
                        base.id(ElementId::NamedInteger("variant-label".into(), i as u64))
                            .cursor_pointer()
                            .on_click(move |_event, window, cx| {
                                handler_click(i, window, cx);
                            })
                            .on_key_down(move |event: &KeyDownEvent, _window, cx| {
                                if crate::foundations::keyboard::is_activation_key(event) {
                                    cx.stop_propagation();
                                    handler_kb(i, _window, cx);
                                }
                            })
                            .into_any_element()
                    } else {
                        base.into_any_element()
                    };

                    selector = selector.child(label_el);
                }

                selector.into_any_element()
            } else {
                let mut title_el = div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted);
                if self.filename.is_some() {
                    title_el = title_el.font(theme.font_mono());
                }
                title_el = title_el.child(title_text);
                title_el.into_any_element()
            };

            let header = CodeBlockHeader::new().left(title_element).right(actions);
            container = container.child(header);
        }

        // Determine display code (truncate if collapsed)
        let all_lines: Vec<&str> = code.split('\n').collect();
        let total_line_count = all_lines.len();
        let should_collapse =
            !self.expanded && self.max_lines.is_some_and(|max| total_line_count > max);
        let visible_line_count = if should_collapse {
            self.max_lines.unwrap_or(total_line_count)
        } else {
            total_line_count
        };

        let display_code = if should_collapse {
            all_lines[..visible_line_count].join("\n")
        } else {
            code.clone()
        };

        // Code body via CodeBlockContent
        container = container.child(
            CodeBlockContent::new(display_code)
                .language(language)
                .show_line_numbers(self.show_line_numbers),
        );

        // Footer: prefer custom_footer, then auto-generate collapse
        if let Some(custom_footer) = self.custom_footer {
            container = container.child(custom_footer);
        } else if should_collapse {
            let hidden_count = total_line_count - visible_line_count;
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(theme.spacing_xs)
                    .border_t_1()
                    .border_color(theme.border)
                    .py(theme.spacing_xs)
                    .cursor_pointer()
                    .hover(|style| style.bg(theme.hover))
                    .child(Icon::new(IconName::ChevronDown))
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child(format!("Show {} more lines", hidden_count)),
                    ),
            );
        }

        container
    }
}
