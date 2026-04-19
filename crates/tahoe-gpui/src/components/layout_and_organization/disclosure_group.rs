//! DisclosureGroup expand-collapse container.

use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::ActiveTheme;
use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, FocusHandle, KeyDownEvent, Window, div, px};

/// A collapsible section with a header and expandable content.
///
/// Since this is a `RenderOnce` component (stateless), the parent view
/// must manage the `is_open` state and provide an `on_toggle` callback
/// to update it.
///
/// # Glyph and chrome
///
/// HIG Disclosure Controls: the sole visual indicator for the open state is
/// the disclosure triangle (`arrowtriangle.right.fill` / `.down.fill`). The
/// header does NOT change background or gain a border when opened — that
/// would double-signal the state and create a false affordance.
///
/// # Keyboard
///
/// Space / Return toggle. Right arrow expands a collapsed group; Left arrow
/// collapses an expanded group.
use crate::callback_types::OnToggle;
#[derive(IntoElement)]
pub struct DisclosureGroup {
    id: ElementId,
    is_open: bool,
    focused: bool,
    /// Optional host-supplied focus handle. Finding 18 in
    /// the Zed cross-reference audit — when set, the focus-ring visibility
    /// comes from `handle.is_focused(window)` and the header threads
    /// `track_focus(&handle)`; otherwise uses the explicit
    /// [`focused`](Self::focused) bool.
    focus_handle: Option<FocusHandle>,
    header: AnyElement,
    body: AnyElement,
    on_toggle: OnToggle,
}

impl DisclosureGroup {
    pub fn new(id: impl Into<ElementId>, header: impl IntoElement, body: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            is_open: false,
            focused: false,
            focus_handle: None,
            header: header.into_any_element(),
            body: body.into_any_element(),
            on_toggle: None,
        }
    }

    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Marks the header as keyboard-focused so a visible focus ring is rendered.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the disclosure group participates in
    /// the host's focus graph. When set, the focus-ring is derived from
    /// `handle.is_focused(window)` and the header threads
    /// `track_focus(&handle)` so Tab-cycling and keyboard shortcuts
    /// scoped to the handle fire correctly. Finding 18 in
    /// the Zed cross-reference audit.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Sets a callback invoked when the header is clicked.
    /// The callback receives the new open state (toggled).
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for DisclosureGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let is_open = self.is_open;
        let new_state = !is_open;
        // Finding 18 in the Zed cross-reference audit.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        let header_selector = format!("disclosure-group-{}", self.id);
        let mut header = div()
            .id(self.id)
            .debug_selector(move || header_selector.clone())
            .focusable()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .cursor_pointer()
            .min_h(px(theme.target_size()))
            .child(
                // HIG disclosure glyph: filled triangle. The triangle's
                // direction is the sole indicator of the open state — no
                // background change, no border change.
                Icon::new(if is_open {
                    IconName::ArrowTriangleDown
                } else {
                    IconName::ArrowTriangleRight
                })
                .size(theme.icon_size_inline),
            )
            .child(self.header);

        if let Some(handle) = self.focus_handle.as_ref() {
            header = header.track_focus(handle);
        }

        header = apply_focus_ring(header, theme, focused, &[]);

        if let Some(handler) = self.on_toggle {
            let handler = std::rc::Rc::new(handler);
            let click_handler = handler.clone();
            header = header
                .on_click(move |_event, window, cx| {
                    click_handler(new_state, window, cx);
                })
                .on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    // Space / Return toggle. Right arrow expands a
                    // collapsed group; Left arrow collapses an expanded
                    // one — matches HIG Accessibility > Keyboard and
                    // `NSOutlineView` arrow-key behaviour.
                    let should_fire = if crate::foundations::keyboard::is_activation_key(event) {
                        true
                    } else {
                        matches!((key, is_open), ("right", false) | ("left", true))
                    };
                    if should_fire {
                        cx.stop_propagation();
                        handler(new_state, window, cx);
                    }
                });
        }

        let mut container = div().flex().flex_col().gap(theme.spacing_sm).child(header);

        if self.is_open {
            container = container.child(self.body);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use crate::components::layout_and_organization::disclosure_group::DisclosureGroup;
    use core::prelude::v1::test;

    #[test]
    fn default_is_closed() {
        let c = DisclosureGroup::new("test", "Header", "Body");
        assert!(!c.is_open);
    }

    #[test]
    fn open_builder() {
        let c = DisclosureGroup::new("test", "Header", "Body").open(true);
        assert!(c.is_open);
    }

    #[test]
    fn on_toggle_default_is_none() {
        let c = DisclosureGroup::new("test", "Header", "Body");
        assert!(c.on_toggle.is_none());
    }

    #[test]
    fn on_toggle_callback_is_some() {
        let c = DisclosureGroup::new("test", "Header", "Body").on_toggle(|_, _, _| {});
        assert!(c.on_toggle.is_some());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, IntoElement, Render, TestAppContext, div};

    use super::DisclosureGroup;
    use crate::test_helpers::helpers::{
        InteractionExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const DISCLOSURE_HEADER: &str = "disclosure-group-disclosure";
    const DISCLOSURE_BODY: &str = "disclosure-group-body";

    struct DisclosureGroupHarness {
        is_open: bool,
        toggles: Vec<bool>,
    }

    impl DisclosureGroupHarness {
        fn new(_cx: &mut Context<Self>) -> Self {
            Self {
                is_open: false,
                toggles: Vec::new(),
            }
        }
    }

    impl Render for DisclosureGroupHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            DisclosureGroup::new(
                "disclosure",
                "Header",
                div()
                    .debug_selector(|| DISCLOSURE_BODY.into())
                    .child("Body"),
            )
            .open(self.is_open)
            .on_toggle(move |is_open, _window, cx| {
                entity.update(cx, |this, cx| {
                    this.is_open = is_open;
                    this.toggles.push(is_open);
                    cx.notify();
                });
            })
        }
    }

    #[gpui::test]
    async fn clicking_and_activation_key_toggle_disclosure(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| DisclosureGroupHarness::new(cx));

        assert_element_absent(cx, DISCLOSURE_BODY);
        cx.click_on(DISCLOSURE_HEADER);

        host.update_in(cx, |host, _window, _cx| {
            assert!(host.is_open);
            assert_eq!(host.toggles, vec![true]);
        });
        assert_element_exists(cx, DISCLOSURE_BODY);

        cx.press("space");

        host.update_in(cx, |host, _window, _cx| {
            assert!(!host.is_open);
            assert_eq!(host.toggles, vec![true, false]);
        });
        assert_element_absent(cx, DISCLOSURE_BODY);
    }

    #[gpui::test]
    async fn arrow_keys_toggle_disclosure_per_hig(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| DisclosureGroupHarness::new(cx));

        // Click once to give the header focus and expand it (starts collapsed).
        cx.click_on(DISCLOSURE_HEADER);
        host.update_in(cx, |host, _window, _cx| {
            assert!(host.is_open);
            assert_eq!(host.toggles, vec![true]);
        });

        // Expanded: right arrow is a no-op, left arrow collapses.
        cx.press("right");
        host.update_in(cx, |host, _window, _cx| {
            assert!(host.is_open, "right on expanded must not collapse");
            assert_eq!(host.toggles, vec![true]);
        });

        cx.press("left");
        host.update_in(cx, |host, _window, _cx| {
            assert!(!host.is_open, "left on expanded must collapse");
            assert_eq!(host.toggles, vec![true, false]);
        });

        // Collapsed: left arrow is a no-op, right arrow expands.
        cx.press("left");
        host.update_in(cx, |host, _window, _cx| {
            assert!(!host.is_open, "left on collapsed must not expand");
            assert_eq!(host.toggles, vec![true, false]);
        });

        cx.press("right");
        host.update_in(cx, |host, _window, _cx| {
            assert!(host.is_open, "right on collapsed must expand");
            assert_eq!(host.toggles, vec![true, false, true]);
        });
    }
}
