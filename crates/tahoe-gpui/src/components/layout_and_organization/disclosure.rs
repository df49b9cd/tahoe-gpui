//! HIG Disclosure Control -- standalone expand/collapse arrow.

use gpui::prelude::*;
use gpui::{App, ElementId, KeyDownEvent, Window, div, px};

use crate::callback_types::OnToggle;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::glass_interactive_hover;
use crate::foundations::theme::ActiveTheme;

/// A standalone disclosure indicator (expand/collapse triangle) per HIG.
///
/// Stateless `RenderOnce` -- the parent owns `is_expanded` and provides
/// an `on_toggle` callback to receive the toggled state.
///
/// # Glyph
///
/// HIG Disclosure Controls: "A disclosure triangle points inward from the
/// leading edge when its content is hidden and down when its content is
/// visible." The component renders [`IconName::ArrowTriangleRight`] /
/// [`IconName::ArrowTriangleDown`] — the filled SF Symbol disclosure
/// triangles. Chevrons (`ChevronRight`) are a navigation affordance and
/// intentionally *not* used here.
///
/// # Keyboard
///
/// Space / Return toggle. Right arrow expands a collapsed triangle; Left
/// arrow collapses an expanded triangle — matches HIG Accessibility >
/// Keyboard and `NSOutlineView` behaviour.
///
/// # Example
///
/// ```ignore
/// Disclosure::new("section-1")
///     .expanded(true)
///     .on_toggle(|expanded, _window, cx| { /* update model */ })
/// ```
#[derive(IntoElement)]
pub struct Disclosure {
    id: ElementId,
    is_expanded: bool,
    on_toggle: OnToggle,
}

impl Disclosure {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_expanded: false,
            on_toggle: None,
        }
    }

    pub fn expanded(mut self, is_expanded: bool) -> Self {
        self.is_expanded = is_expanded;
        self
    }

    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Disclosure {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let is_expanded = self.is_expanded;
        let new_state = !is_expanded;

        // HIG disclosure glyph: filled triangle. Down = expanded, right = collapsed.
        let icon_name = if is_expanded {
            IconName::ArrowTriangleDown
        } else {
            IconName::ArrowTriangleRight
        };

        // Apply glass interactive hover before adding id (which makes it Stateful<Div>)
        let base = div()
            .min_w(px(theme.target_size()))
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .justify_center()
            .flex_shrink_0();

        let base = glass_interactive_hover(base, theme);

        let mut el = base
            .id(self.id)
            .focusable()
            .child(Icon::new(icon_name).size(px(16.0)));

        if let Some(handler) = self.on_toggle {
            let handler = std::rc::Rc::new(handler);
            let click_handler = handler.clone();
            el = el
                .cursor_pointer()
                .on_click(move |_event, window, cx| {
                    click_handler(new_state, window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    // Space / Return toggle. Right arrow expands a
                    // collapsed triangle; Left arrow collapses an
                    // expanded one (HIG Accessibility > Keyboard).
                    let should_fire = if crate::foundations::keyboard::is_activation_key(event) {
                        true
                    } else {
                        matches!((key, is_expanded), ("right", false) | ("left", true))
                    };
                    if should_fire {
                        cx.stop_propagation();
                        handler(new_state, window, cx);
                    }
                });
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::Disclosure;
    use core::prelude::v1::test;

    #[test]
    fn disclosure_defaults() {
        let d = Disclosure::new("test");
        assert!(!d.is_expanded);
        assert!(d.on_toggle.is_none());
    }

    #[test]
    fn disclosure_expanded_builder() {
        let d = Disclosure::new("test").expanded(true);
        assert!(d.is_expanded);
    }

    #[test]
    fn disclosure_collapsed_builder() {
        let d = Disclosure::new("test").expanded(false);
        assert!(!d.is_expanded);
    }

    #[test]
    fn disclosure_on_toggle_is_some() {
        let d = Disclosure::new("test").on_toggle(|_, _, _| {});
        assert!(d.on_toggle.is_some());
    }

    #[test]
    fn disclosure_chained_builders() {
        let d = Disclosure::new("test")
            .expanded(true)
            .on_toggle(|_, _, _| {});
        assert!(d.is_expanded);
        assert!(d.on_toggle.is_some());
    }
}
