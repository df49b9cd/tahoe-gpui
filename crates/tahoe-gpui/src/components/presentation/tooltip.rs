//! Tooltip wrapper component.
//!
//! GPUI's `.tooltip()` requires `AnyView`, so we provide a small
//! `TooltipView` Render impl and a convenience `Tooltip` wrapper.

use crate::foundations::materials::{LensEffect, SurfaceContext, glass_lens_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, AnyView, App, ElementId, SharedString, Window, div};

/// HIG hover-to-tooltip delay (500 ms). Apple documents a
/// ~500 ms delay before a tooltip appears; GPUI's `.tooltip()`
/// implementation applies the same value internally (see
/// `TOOLTIP_SHOW_DELAY` in `gpui/src/elements/div.rs`), so callers do
/// not need to stagger presentation. The constant is surfaced here for
/// tests and potential future per-tooltip overrides when GPUI exposes
/// `Tooltip::delay(Duration)`.
pub const TOOLTIP_SHOW_DELAY_MS: u64 = 500;

/// Simple view that renders tooltip text, optionally decorated with a
/// key-binding display on the right.
///
/// The keybinding slot mirrors Zed's tooltip convention (see
/// `crates/ui/src/components/tooltip.rs` — `key_binding` parameter in
/// `tooltip_container`): callers can advertise the keyboard shortcut
/// that triggers the same action so power users discover keybindings
/// without leaving the hover target.
struct TooltipView {
    text: SharedString,
    key_binding: Option<SharedString>,
}

impl Render for TooltipView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Tooltips are subtle-lens Small surfaces — relatively translucent,
        // so readability is most consistent when the label uses the
        // `GlassBright` vibrancy palette (primary level), which is
        // tuned for high contrast against translucent surfaces in both
        // light and dark appearance.
        let effect = LensEffect::subtle(GlassSize::Small, theme);
        let mut el = glass_lens_surface(theme, &effect, GlassSize::Small)
            .px(theme.spacing_sm)
            .py(theme.spacing_xs)
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_sm)
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.label_color(SurfaceContext::GlassBright));

        el = el.child(div().child(self.text.clone()));

        if let Some(binding) = self.key_binding.clone() {
            // Keybinding chip uses the secondary label hierarchy so it
            // reads as an annotation rather than competing with the
            // primary label.
            el = el.child(
                div()
                    .text_color(crate::foundations::materials::resolve_label(
                        theme,
                        SurfaceContext::GlassBright,
                        1,
                    ))
                    .child(binding),
            );
        }

        el
    }
}

/// A convenience wrapper that attaches a text tooltip to a child element.
///
/// The child is wrapped in an interactive div with an id so GPUI
/// can track hover state and show the tooltip.
///
/// # Hover delay
///
/// HIG tooltips appear after a ~500 ms hover. GPUI's
/// `.tooltip()` applies the same [`TOOLTIP_SHOW_DELAY_MS`] internally
/// (`TOOLTIP_SHOW_DELAY` in GPUI) so no additional staggering is
/// needed. When GPUI lands `Tooltip::delay(Duration)`, we'll surface
/// it on this builder.
///
/// # Keybinding slot
///
/// Use [`Tooltip::key_binding`] to add a shortcut annotation next to
/// the tooltip text — matches Zed's tooltip behavior where hover
/// surfaces both describe the action and advertise its keyboard
/// shortcut in one glance. Formatting (e.g. `⌘C`, `⇧⌘K`) is the
/// caller's responsibility so the tooltip stays agnostic of the
/// platform-specific keystroke notation used by the host app.
#[derive(IntoElement)]
pub struct Tooltip {
    text: SharedString,
    child: AnyElement,
    id: ElementId,
    key_binding: Option<SharedString>,
}

impl Tooltip {
    pub fn new(
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        child: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            child: child.into_any_element(),
            key_binding: None,
        }
    }

    /// Attach a keybinding display that renders next to the tooltip
    /// label. Pass the pre-formatted shortcut string (e.g. `"⌘C"`,
    /// `"⇧⌘K"`).
    pub fn key_binding(mut self, binding: impl Into<SharedString>) -> Self {
        self.key_binding = Some(binding.into());
        self
    }

    /// Build a tooltip whose keybinding chip is resolved live from the
    /// window's dispatch tree for `action`. If no binding is registered
    /// (or the binding uses a multi-keystroke chord), the chip is
    /// omitted. Prefer this over hand-typed `.key_binding(...)` strings
    /// so the chip reflects the user's effective keymap.
    pub fn for_action(
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        child: impl IntoElement,
        action: &dyn gpui::Action,
        window: &gpui::Window,
    ) -> Self {
        let mut tip = Self::new(id, text, child);
        if let Some(shortcut) =
            crate::foundations::keyboard_shortcuts::MenuShortcut::for_action(action, window)
        {
            tip = tip.key_binding(SharedString::from(shortcut.render()));
        }
        tip
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .child(self.child)
            .tooltip(text_tooltip_view(self.text, self.key_binding))
    }
}

/// Returns a `.tooltip(...)` closure that renders the canonical tahoe
/// tooltip surface for the given text and optional keybinding glyph.
///
/// Exposed so component builders (e.g.
/// [`Button::tooltip`][crate::components::menus_and_actions::button::Button::tooltip])
/// can attach the canonical tahoe tooltip style without wrapping their
/// element in a full [`Tooltip`] component (which would introduce an extra
/// `div` layer and break ID-based targeting).
pub fn text_tooltip_view(
    text: SharedString,
    key_binding: Option<SharedString>,
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    move |_window, cx| {
        let text = text.clone();
        let key_binding = key_binding.clone();
        cx.new(|_cx| TooltipView { text, key_binding }).into()
    }
}

#[cfg(test)]
mod tests {
    use super::Tooltip;
    use core::prelude::v1::test;
    use gpui::SharedString;

    #[test]
    fn tooltip_delay_matches_hig() {
        // HIG documents a ~500 ms hover delay. GPUI's internal
        // `TOOLTIP_SHOW_DELAY` matches. Assert the public constant
        // stays aligned so future drift is caught at compile time.
        assert_eq!(super::TOOLTIP_SHOW_DELAY_MS, 500);
    }

    #[test]
    fn tooltip_defaults_have_no_keybinding() {
        let t = Tooltip::new("id", "Copy", gpui::div());
        assert!(t.key_binding.is_none());
    }

    #[test]
    fn tooltip_key_binding_stores_formatted_string() {
        let t = Tooltip::new("id", "Copy", gpui::div()).key_binding("⌘C");
        assert_eq!(t.key_binding, Some(SharedString::from("⌘C")));
    }
}
