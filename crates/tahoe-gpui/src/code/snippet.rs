//! Snippet component — a copyable code snippet display.

use std::sync::Arc;
use std::time::Duration;

use gpui::prelude::*;
use gpui::{AnyElement, App, Entity, SharedString, Window, div};

use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// A read-only code snippet with copy button.
///
/// Displays a short code reference (e.g. a terminal command) with optional
/// prefix text and a copy-to-clipboard button. Built on the same design
/// tokens as `CodeBlockView` but optimised for single-line inline display.
///
/// # Examples
/// ```ignore
/// Snippet::new("npm install @ai-sdk/react")
///     .prefix("$")
/// ```
#[derive(IntoElement)]
pub struct Snippet {
    code: SharedString,
    prefix: Option<SharedString>,
    language: Option<SharedString>,
    wrap: bool,
    copy_button: Option<Entity<CopyButton>>,
    timeout: Option<Duration>,
    on_copy: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    on_error: Option<Arc<dyn Fn(String) + Send + Sync + 'static>>,
    addons: Vec<AnyElement>,
    copy_button_child: Option<Box<dyn Fn(bool) -> AnyElement + Send + Sync + 'static>>,
    disabled: bool,
}

impl Snippet {
    pub fn new(code: impl Into<SharedString>) -> Self {
        Self {
            code: code.into(),
            prefix: None,
            language: None,
            wrap: false,
            copy_button: None,
            timeout: None,
            on_copy: None,
            on_error: None,
            addons: Vec::new(),
            copy_button_child: None,
            disabled: false,
        }
    }

    /// Optional language label displayed in the right gutter as a muted
    /// `Caption1` tag. Mirrors `CodeBlockView`'s language header for
    /// short inline snippets that otherwise omit the chrome.
    pub fn language(mut self, language: impl Into<SharedString>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// When `true`, long snippets wrap (`whitespace_pre_wrap`) instead of
    /// truncating via `text_ellipsis` + `whitespace_nowrap`. Default is
    /// `false` to preserve the pre-existing single-line layout.
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Optional prefix displayed before the code in muted text (e.g. `$`).
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Provide a pre-created `CopyButton` entity for persistent copy state.
    ///
    /// Without this, a new `CopyButton` is created each render and the
    /// "copied" checkmark feedback is lost if the parent re-renders.
    pub fn copy_button(mut self, button: Entity<CopyButton>) -> Self {
        self.copy_button = Some(button);
        self
    }

    /// Override the duration of the "copied" feedback state (default 2 000 ms).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Callback invoked after a successful copy.
    pub fn on_copy(mut self, callback: Arc<dyn Fn() + Send + Sync + 'static>) -> Self {
        self.on_copy = Some(callback);
        self
    }

    /// Callback invoked if copying fails.
    ///
    /// See [`CopyButton::set_on_error`] for details on when this fires.
    pub fn on_error(mut self, callback: Arc<dyn Fn(String) + Send + Sync + 'static>) -> Self {
        self.on_error = Some(callback);
        self
    }

    /// Add addon content rendered between the prefix and the code text.
    ///
    /// Can be called multiple times to add multiple addons. This is the GPUI
    /// equivalent of the web `<SnippetAddon />` / `<SnippetText />`
    /// composable slots.
    pub fn addon(mut self, addon: impl IntoElement) -> Self {
        self.addons.push(addon.into_any_element());
        self
    }

    /// Override the CopyButton's default icon with custom child content.
    ///
    /// The closure receives the current `copied` state so callers can render
    /// conditionally (e.g. "Copy" → "Copied!").
    pub fn copy_button_child(
        mut self,
        child: Box<dyn Fn(bool) -> AnyElement + Send + Sync + 'static>,
    ) -> Self {
        self.copy_button_child = Some(child);
        self
    }

    /// Whether the snippet (and its copy button) are disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Snippet {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let mono_font = theme.mono_font();

        // Reuse the provided CopyButton entity, or create a fresh one.
        let copy_button = if let Some(btn) = self.copy_button {
            btn.update(cx, |btn, _cx| {
                btn.set_content(self.code.to_string());
                if let Some(timeout) = self.timeout {
                    btn.set_timeout(timeout);
                }
                if let Some(on_copy) = self.on_copy {
                    btn.set_on_copy(on_copy);
                }
                if let Some(on_error) = self.on_error {
                    btn.set_on_error(on_error);
                }
                if let Some(child) = self.copy_button_child {
                    btn.set_custom_child(child);
                }
                btn.set_disabled(self.disabled);
            });
            btn
        } else {
            let btn = CopyButton::new(self.code.to_string(), cx);
            btn.update(cx, |btn, _cx| {
                if let Some(timeout) = self.timeout {
                    btn.set_timeout(timeout);
                }
                if let Some(on_copy) = self.on_copy {
                    btn.set_on_copy(on_copy);
                }
                if let Some(on_error) = self.on_error {
                    btn.set_on_error(on_error);
                }
                if let Some(child) = self.copy_button_child {
                    btn.set_custom_child(child);
                }
                btn.set_disabled(self.disabled);
            });
            btn
        };

        let mut container = div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .bg(theme.code_bg)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border);

        if self.disabled {
            container = container.cursor_default();
        }

        let code_div = if self.wrap {
            div().text_color(theme.text).whitespace_normal()
        } else {
            div()
                .text_color(theme.text)
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
        };

        let mut result = container.child({
            let mut row = div()
                .flex()
                .items_center()
                .gap(theme.spacing_sm)
                .flex_1()
                .font(mono_font.clone())
                .text_style(TextStyle::Subheadline, &theme)
                .text_color(theme.text_muted);

            if let Some(prefix) = self.prefix {
                row = row.child(div().text_color(theme.text_muted).child(prefix));
            }

            if !self.addons.is_empty() {
                row = row.children(self.addons);
            }

            row.child(code_div.child(self.code))
        });

        if let Some(language) = self.language {
            result = result.child(
                div()
                    .font(mono_font.clone())
                    .text_style(TextStyle::Caption1, &theme)
                    .text_color(theme.text_muted)
                    .child(language),
            );
        }

        result.child(copy_button)
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use std::sync::Arc;
    use std::time::Duration;

    use gpui::div;
    use gpui::prelude::{IntoElement, ParentElement};

    use super::Snippet;

    #[test]
    fn snippet_builder_defaults() {
        let snippet = Snippet::new("test");
        assert_eq!(snippet.code.as_ref(), "test");
        assert!(snippet.prefix.is_none());
        assert!(snippet.language.is_none());
        assert!(!snippet.wrap);
        assert!(snippet.copy_button.is_none());
        assert!(snippet.timeout.is_none());
        assert!(snippet.on_copy.is_none());
        assert!(snippet.on_error.is_none());
        assert!(snippet.addons.is_empty());
        assert!(snippet.copy_button_child.is_none());
        assert!(!snippet.disabled);
    }

    #[test]
    fn snippet_language_stored() {
        let s = Snippet::new("a").language("bash");
        assert_eq!(s.language.as_ref().map(|s| s.as_ref()), Some("bash"));
    }

    #[test]
    fn snippet_wrap_toggle() {
        assert!(Snippet::new("x").wrap(true).wrap);
        assert!(!Snippet::new("x").wrap(false).wrap);
    }

    #[test]
    fn snippet_builder_chain() {
        let snippet = Snippet::new("cargo build")
            .prefix("$")
            .timeout(Duration::from_secs(5));
        assert_eq!(snippet.code.as_ref(), "cargo build");
        assert_eq!(snippet.prefix.as_ref().map(|s| s.as_ref()), Some("$"));
        assert_eq!(snippet.timeout, Some(Duration::from_secs(5)));
    }

    #[test]
    fn snippet_on_copy_stored() {
        let snippet = Snippet::new("code").on_copy(Arc::new(|| {}));
        assert!(snippet.on_copy.is_some());
    }

    #[test]
    fn snippet_addon_stored() {
        let snippet = Snippet::new("value").addon(div().child("let x = "));
        assert_eq!(snippet.addons.len(), 1);
    }

    #[test]
    fn snippet_multiple_addons() {
        let snippet = Snippet::new("value")
            .addon(div().child("a"))
            .addon(div().child("b"));
        assert_eq!(snippet.addons.len(), 2);
    }

    #[test]
    fn snippet_on_error_stored() {
        let snippet = Snippet::new("code").on_error(Arc::new(|_err| {}));
        assert!(snippet.on_error.is_some());
    }

    #[test]
    fn snippet_disabled_default_false() {
        let snippet = Snippet::new("code");
        assert!(!snippet.disabled);
    }

    #[test]
    fn snippet_disabled_set() {
        let snippet = Snippet::new("code").disabled(true);
        assert!(snippet.disabled);
    }

    #[test]
    fn snippet_copy_button_child_stored() {
        let snippet =
            Snippet::new("code").copy_button_child(Box::new(|_copied| div().into_any_element()));
        assert!(snippet.copy_button_child.is_some());
    }
}
