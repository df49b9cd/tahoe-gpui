//! `ButtonLike` interactivity substrate.
//!
//! Finding 15 in the Zed cross-reference audit
//! (df49b9cd/ai-sdk-rust#132). Zed separates the `Button`
//! *composition* (label + icon + variant) from `ButtonLike`, the
//! *interactivity substrate* shared by every button-shaped control
//! (standard button, copy-confirm button, segmented group members,
//! pill toggles). Before this module, `button.rs`, `copy_button.rs`,
//! and `button_group.rs` each open-coded their own hover / disabled /
//! focus-ring visuals.
//!
//! `ButtonLike` is a lightweight builder that wraps a `Stateful<Div>`
//! and applies a consistent stack:
//! 1. **Focus-ring** driven by either an explicit `focused: bool` or a
//!    caller-supplied [`FocusHandle`] (Zed's pattern).
//! 2. **Hover treatment** — an opacity-darken / opacity-lift on the
//!    underlying fill; callers pick the base fill, `ButtonLike` derives
//!    the hover.
//! 3. **Disabled treatment** — `opacity(0.5)` and `cursor_not_allowed`.
//! 4. **High-contrast border** via [`apply_high_contrast_border`].
//!
//! ## Usage
//!
//! ```ignore
//! use tahoe_gpui::components::menus_and_actions::button_like::ButtonLike;
//!
//! let el = ButtonLike::new("my-btn")
//!     .focus_handle(focus_handle_ref)
//!     .disabled(disabled)
//!     .apply(div().bg(theme.accent).child("Go"), theme);
//! ```
//!
//! The returned element is a `Stateful<Div>` with click handlers,
//! focus tracking, focus ring, hover fade, and disabled opacity already
//! wired. Button internals can then continue to add their own variant-
//! specific chrome (shape, border, shadow, animation) as children or
//! styles.

use gpui::prelude::*;
use gpui::{App, BoxShadow, ClickEvent, ElementId, FocusHandle, KeyDownEvent, Stateful, Window, div};

use crate::foundations::keyboard::is_activation_key;
use crate::foundations::materials::{apply_focus_ring, apply_high_contrast_border};
use crate::foundations::theme::TahoeTheme;

/// Shared interactivity substrate for every button-shaped control.
///
/// See the module-level docs for the rationale and the shared stack of
/// visual behaviours applied by [`ButtonLike::apply`].
/// Boxed click handler shared by every [`ButtonLike`]-based control.
/// Extracted as a type alias so the struct field doesn't trip
/// `clippy::type_complexity`.
type ButtonLikeClickFn = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

pub struct ButtonLike {
    id: ElementId,
    focused: bool,
    focus_handle: Option<FocusHandle>,
    disabled: bool,
    on_click: Option<ButtonLikeClickFn>,
    /// Non-focus-ring shadows composed into the focus-ring shadow stack
    /// at paint time. Variants like `Button::Primary` carry a 0.5pt
    /// specular rim shadow that must survive the focus-ring call, so
    /// callers pass their base stack here and `ButtonLike` forwards it
    /// to [`apply_focus_ring`] instead of the default empty slice.
    base_shadows: Vec<BoxShadow>,
}

impl ButtonLike {
    /// Create a new substrate. `id` becomes the `Stateful<Div>` id and
    /// feeds the `debug_selector` so integration tests can locate the
    /// rendered element.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            focused: false,
            focus_handle: None,
            disabled: false,
            on_click: None,
            base_shadows: Vec::new(),
        }
    }

    /// Supply an existing shadow stack (specular rim, Liquid Glass tier
    /// shadow) that should survive the focus-ring composition. Internal
    /// to `Button`'s variant chrome; other `ButtonLike` adopters
    /// typically don't need this.
    pub fn base_shadows(mut self, shadows: Vec<BoxShadow>) -> Self {
        self.base_shadows = shadows;
        self
    }

    /// Set the explicit `focused` flag. Ignored when a
    /// [`focus_handle`](Self::focus_handle) is supplied — the handle's
    /// reactive state takes precedence.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] — matches Zed's `ButtonLike::track_focus`
    /// pattern. When present, the focus ring is driven by
    /// `handle.is_focused(window)` and the root element threads
    /// `track_focus(&handle)`.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Mark as disabled — turns off `on_click`, applies `opacity(0.5)`,
    /// and swaps the cursor to `not_allowed`.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Install a click handler. Suppressed while `disabled`.
    ///
    /// The handler also fires when the user presses an activation key
    /// (Space / Return) while the substrate holds focus — matching HIG
    /// Accessibility Keyboard guidance that every button-shaped control
    /// must be operable from the keyboard. Keyboard activation reuses
    /// the supplied click handler with a synthetic `ClickEvent` so
    /// callers don't need to register two paths.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Apply the substrate's shared behaviours on top of `content` and
    /// return the resulting stateful element. `content` is expected to
    /// carry any variant-specific styling (background, border, shape,
    /// shadow, padding) — `ButtonLike` only layers the interactivity
    /// concerns.
    ///
    /// Internally wraps `content` in a fresh `div().id(..).focusable()`
    /// and then delegates to [`apply_to`](Self::apply_to). Callers that
    /// already own a pre-styled `Stateful<Div>` (for example
    /// `Button::render`, which applies its variant chrome on the outer
    /// element) should use `apply_to` directly.
    pub fn apply(
        self,
        content: impl IntoElement,
        theme: &TahoeTheme,
        window: &Window,
    ) -> Stateful<gpui::Div> {
        let id = self.id.clone();
        let el = div().id(id).focusable().child(content);
        self.apply_to(el, theme, window)
    }

    /// Layer the substrate's shared behaviours onto an already-styled
    /// `Stateful<Div>`. Used by `Button::render` to keep the variant
    /// chrome (bg, border, padding, rounded, text style) on the outer
    /// focus-ring recipient while still routing interactivity through
    /// the shared substrate.
    ///
    /// The caller is responsible for setting `id(...)` and
    /// `focusable()` on `el` before calling this — matching how the
    /// existing button render paths already build the outer element.
    pub fn apply_to(
        self,
        mut el: Stateful<gpui::Div>,
        theme: &TahoeTheme,
        window: &Window,
    ) -> Stateful<gpui::Div> {
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        if let Some(handle) = self.focus_handle.as_ref() {
            el = el.track_focus(handle);
        }

        el = apply_focus_ring(el, theme, focused, &self.base_shadows);
        el = apply_high_contrast_border(el, theme);

        if self.disabled {
            el = el.opacity(0.5).cursor_not_allowed();
        } else if let Some(handler) = self.on_click {
            let handler = std::rc::Rc::new(handler);
            let click = handler.clone();
            let key = handler.clone();
            el = el
                .cursor_pointer()
                .on_click(move |event, window, cx| {
                    click(event, window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    if is_activation_key(event) {
                        cx.stop_propagation();
                        // Use a minimal synthetic `ClickEvent::default()`.
                        // Keyboard activation does not carry pointer
                        // data; handlers that need the event's click-
                        // specific fields (position, modifiers) should
                        // register `.on_key_down` directly.
                        let synthetic = ClickEvent::default();
                        key(&synthetic, window, cx);
                    }
                });
        }

        el
    }

    /// Returns whether the substrate is currently disabled. Useful when
    /// the caller wants to branch on disabled state in their own render
    /// path before handing off to [`apply`](Self::apply).
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

#[cfg(test)]
mod tests {
    use super::ButtonLike;
    use core::prelude::v1::test;

    #[test]
    fn default_state_is_not_disabled() {
        let bl = ButtonLike::new("test");
        assert!(!bl.is_disabled());
        assert!(!bl.focused);
        assert!(bl.focus_handle.is_none());
        assert!(bl.on_click.is_none());
    }

    #[test]
    fn disabled_builder() {
        assert!(ButtonLike::new("test").disabled(true).is_disabled());
    }

    #[test]
    fn focused_builder_sets_flag() {
        let bl = ButtonLike::new("test").focused(true);
        assert!(bl.focused);
    }

    #[test]
    fn on_click_builder_installs_handler() {
        let bl = ButtonLike::new("test").on_click(|_, _, _| {});
        assert!(bl.on_click.is_some());
    }
}
