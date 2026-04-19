//! Copy button with visual feedback (checkmark state).
//!
//! Renders `IconName::Copy`, which maps to the canonical SF Symbol
//! `document.on.document` (the SF Symbols 7 name for the glyph
//! historically known as `doc.on.doc`) via [`IconName::sf_asset_path`].
//! The SF Symbol path takes priority over the custom Lucide-derived
//! fallback SVG at `icons/ai-sdk/copy.svg`, so the rendered glyph matches
//! the macOS Edit menu's Copy item.
//!
//! On state change the icon performs a brief opacity cross-fade
//! controlled by [`crate::foundations::motion::MotionRamp::Short`] (HIG
//! micro-interaction guidance, updated December 2025).

use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui::prelude::*;
use gpui::{AnimationExt, AnyElement, App, ClipboardItem, ElementId, Entity, Window, div};

use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::components::menus_and_actions::button_like::ButtonLike;
use crate::foundations::accessibility::effective_duration;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::ActiveTheme;

/// A copy-to-clipboard button that shows a checkmark after copying.
pub struct CopyButton {
    content: String,
    copied: bool,
    copied_at: Option<Instant>,
    timeout: Duration,
    on_copy: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    on_error: Option<Arc<dyn Fn(String) + Send + Sync + 'static>>,
    custom_child: Option<Box<dyn Fn(bool) -> AnyElement + Send + Sync + 'static>>,
    disabled: bool,
}

impl CopyButton {
    pub fn new(content: impl Into<String>, cx: &mut App) -> Entity<Self> {
        cx.new(|_| Self {
            content: content.into(),
            copied: false,
            copied_at: None,
            timeout: Duration::from_millis(2000),
            on_copy: None,
            on_error: None,
            custom_child: None,
            disabled: false,
        })
    }

    /// Set the text to copy.
    pub fn set_content(&mut self, content: String) {
        self.content = content;
    }

    /// Read the current clipboard content. Primarily useful for tests
    /// and for callers that want to mirror the button's state into a
    /// status line.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Set the timeout for the "copied" feedback state.
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Set a callback invoked after a successful copy.
    pub fn set_on_copy(&mut self, on_copy: Arc<dyn Fn() + Send + Sync + 'static>) {
        self.on_copy = Some(on_copy);
    }

    /// Set a callback invoked if copying fails.
    ///
    /// GPUI's `cx.write_to_clipboard()` is infallible, so this will not fire
    /// from the default clipboard path. Exists for API parity with AI SDK
    /// Elements and for use with custom copy mechanisms.
    pub fn set_on_error(&mut self, on_error: Arc<dyn Fn(String) + Send + Sync + 'static>) {
        self.on_error = Some(on_error);
    }

    /// Override the default icon button with custom child content.
    ///
    /// The closure receives the current `copied` state so callers can render
    /// differently after a copy (e.g. "Copy" → "Copied!").
    pub fn set_custom_child(
        &mut self,
        child: Box<dyn Fn(bool) -> AnyElement + Send + Sync + 'static>,
    ) {
        self.custom_child = Some(child);
    }

    /// Set whether the button is disabled.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    fn handle_click(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        // Note: write_to_clipboard is infallible in GPUI, so on_error is not
        // called here. It exists for API parity with AI SDK Elements and for
        // use with custom copy mechanisms.
        let item = ClipboardItem::new_string(self.content.clone());
        cx.write_to_clipboard(item);
        self.copied = true;
        self.copied_at = Some(Instant::now());
        if let Some(on_copy) = &self.on_copy {
            on_copy();
        }
        cx.notify();

        // Schedule a reset after the timeout so the check mark clears even
        // if nothing else triggers a re-render.
        let timeout = self.timeout;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(timeout).await;
            this.update(cx, |this, cx| {
                if this.copied_at.is_some_and(|t| t.elapsed() >= this.timeout) {
                    this.copied = false;
                    this.copied_at = None;
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }
}

impl Render for CopyButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Check if the copied state has expired
        if self.copied
            && let Some(copied_at) = self.copied_at
            && copied_at.elapsed() >= self.timeout
        {
            self.copied = false;
            self.copied_at = None;
        }

        let theme = cx.theme().clone();

        if let Some(ref custom_child) = self.custom_child {
            // Finding 15 adoption: CopyButton's
            // custom-child path used to open-code the disabled / cursor /
            // click / activation-key wiring. ButtonLike now owns that
            // stack — both the click handler and the activation-key
            // handler route through the shared substrate.
            let child = custom_child(self.copied);
            let wrapper = ButtonLike::new("copy-btn")
                .disabled(self.disabled)
                .on_click(cx.listener(|this, _event: &gpui::ClickEvent, window, cx| {
                    this.handle_click(window, cx);
                }))
                .apply(child, &theme, window);
            return wrapper.into_any_element();
        }

        let (icon_name, icon_color) = if self.copied {
            (IconName::Check, Some(theme.success))
        } else {
            (IconName::Copy, None)
        };

        let mut icon = Icon::new(icon_name);
        if let Some(color) = icon_color {
            icon = icon.color(color);
        }

        // State-change cross-fade: 150 ms opacity ramp re-keyed on the
        // `copied` flag so Copy → Check and back each restart the fade.
        // `GlobalElementId` is stable as long as the parent stores this
        // `Entity<CopyButton>` and doesn't reconstruct it every render —
        // callers that do (e.g. building `CopyButton::new(...)` inside a
        // `RenderOnce::render`) churn the entity id and the animation
        // state never persists. In that case the caller is expected to
        // cache the entity and pass it via the surrounding builder's
        // `.copy_button(Entity<CopyButton>)` hook.
        let copied_key = if self.copied { 1 } else { 0 };
        let duration_ms = effective_duration(&theme, 150);
        let animated_icon: AnyElement = if duration_ms == 0 {
            div()
                .id(ElementId::NamedInteger("copy-icon-static".into(), copied_key))
                .child(icon)
                .into_any_element()
        } else {
            div()
                .id(ElementId::NamedInteger("copy-icon-anim".into(), copied_key))
                .child(icon)
                .with_animation(
                    ElementId::NamedInteger("copy-icon-fade".into(), copied_key),
                    gpui::Animation::new(Duration::from_millis(duration_ms)),
                    |el: gpui::Stateful<gpui::Div>, delta: f32| el.opacity(delta),
                )
                .into_any_element()
        };

        let mut btn = Button::new("copy-btn")
            .icon(animated_icon)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .disabled(self.disabled);

        if !self.disabled {
            btn = btn.on_click(cx.listener(|this, _event: &gpui::ClickEvent, window, cx| {
                this.handle_click(window, cx);
            }));
        }

        btn.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::CopyButton;
    use core::prelude::v1::test;
    use gpui::IntoElement;
    use gpui::div;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    fn make_button(content: &str) -> CopyButton {
        CopyButton {
            content: content.into(),
            copied: false,
            copied_at: None,
            timeout: Duration::from_millis(2000),
            on_copy: None,
            on_error: None,
            custom_child: None,
            disabled: false,
        }
    }

    #[test]
    fn initial_state() {
        let btn = make_button("hello");
        assert!(!btn.copied);
        assert!(btn.copied_at.is_none());
        assert_eq!(btn.content, "hello");
    }

    #[test]
    fn timeout_expiry_resets_copied() {
        let mut btn = make_button("hello");
        btn.copied = true;
        btn.copied_at = Some(Instant::now() - Duration::from_millis(3000));

        // Simulate the check that happens during render
        if let Some(copied_at) = btn.copied_at
            && copied_at.elapsed() >= btn.timeout
        {
            btn.copied = false;
            btn.copied_at = None;
        }
        assert!(!btn.copied);
        assert!(btn.copied_at.is_none());
    }

    #[test]
    fn not_expired_stays_copied() {
        let mut btn = make_button("hello");
        btn.copied = true;
        btn.copied_at = Some(Instant::now());

        if let Some(copied_at) = btn.copied_at
            && copied_at.elapsed() >= btn.timeout
        {
            btn.copied = false;
            btn.copied_at = None;
        }
        assert!(btn.copied);
        assert!(btn.copied_at.is_some());
    }

    #[test]
    fn set_content_updates() {
        let mut btn = make_button("old");
        btn.set_content("new".into());
        assert_eq!(btn.content, "new");
    }

    #[test]
    fn set_timeout_updates() {
        let mut btn = make_button("hello");
        btn.set_timeout(Duration::from_millis(5000));
        assert_eq!(btn.timeout, Duration::from_millis(5000));
    }

    #[test]
    fn on_error_stored() {
        let mut btn = make_button("hello");
        assert!(btn.on_error.is_none());
        btn.set_on_error(Arc::new(|_msg| {}));
        assert!(btn.on_error.is_some());
    }

    #[test]
    fn on_error_callback_invoked_with_message() {
        use std::sync::Mutex;
        let captured = Arc::new(Mutex::new(String::new()));
        let captured_clone = captured.clone();
        let mut btn = make_button("hello");
        btn.set_on_error(Arc::new(move |msg| {
            *captured_clone.lock().unwrap() = msg;
        }));
        if let Some(ref on_error) = btn.on_error {
            on_error("clipboard unavailable".to_string());
        }
        assert_eq!(*captured.lock().unwrap(), "clipboard unavailable");
    }

    #[test]
    fn custom_timeout_respected() {
        let mut btn = make_button("hello");
        btn.set_timeout(Duration::from_millis(100));
        btn.copied = true;
        btn.copied_at = Some(Instant::now() - Duration::from_millis(200));

        if let Some(copied_at) = btn.copied_at
            && copied_at.elapsed() >= btn.timeout
        {
            btn.copied = false;
            btn.copied_at = None;
        }
        assert!(!btn.copied);
    }

    #[test]
    fn custom_child_stored() {
        let mut btn = make_button("hello");
        assert!(btn.custom_child.is_none());
        btn.set_custom_child(Box::new(|_copied| div().into_any_element()));
        assert!(btn.custom_child.is_some());
    }

    #[test]
    fn disabled_default_false() {
        let btn = make_button("hello");
        assert!(!btn.disabled);
    }

    #[test]
    fn set_disabled_updates() {
        let mut btn = make_button("hello");
        btn.set_disabled(true);
        assert!(btn.disabled);
    }
}
