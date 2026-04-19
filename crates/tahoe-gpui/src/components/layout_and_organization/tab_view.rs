//! Tab view (HIG Tab Views — macOS `NSTabView`).
//!
//! Presents multiple mutually exclusive panes of content in the same area.
//! A horizontal tab-strip sits above a bordered content pane. Distinct
//! from [`TabBar`](crate::components::navigation_and_search::tab_bar::TabBar),
//! which is the iOS-style bottom navigation bar.
//!
//! # Platform
//!
//! macOS only. HIG caps a tab view at **six tabs** — past that the
//! structure gets hard to scan. The component emits a debug assertion
//! when more than six tabs are configured.
//!
//! # Hidden tab control
//!
//! Callers doing programmatic pane switching (for example driving the
//! selection from outside the component) can hide the tab strip with
//! [`TabView::hide_control`]. The bordered content pane continues to
//! render the selected tab's body.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/tab-views>

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, FontWeight, KeyDownEvent, SharedString, Window, div, px};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

/// HIG-recommended maximum tab count for a `NSTabView`-style tab strip.
pub const MAX_TAB_VIEW_TABS: usize = 6;

/// One tab within a [`TabView`].
pub struct Tab {
    pub id: SharedString,
    pub label: SharedString,
    pub body: AnyElement,
}

impl Tab {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        body: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            body: body.into_any_element(),
        }
    }
}

type OnSelect = Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>;

/// A macOS `NSTabView`-style tab view.
#[derive(IntoElement)]
pub struct TabView {
    id: ElementId,
    tabs: Vec<Tab>,
    selected_id: Option<SharedString>,
    on_select: OnSelect,
    hide_control: bool,
    focused: bool,
}

impl TabView {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            tabs: Vec::new(),
            selected_id: None,
            on_select: None,
            hide_control: false,
            focused: false,
        }
    }

    pub fn tabs(mut self, tabs: Vec<Tab>) -> Self {
        debug_assert!(
            tabs.len() <= MAX_TAB_VIEW_TABS,
            "HIG: tab views should have at most {MAX_TAB_VIEW_TABS} tabs; got {}",
            tabs.len()
        );
        self.tabs = tabs;
        self
    }

    pub fn selected_id(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Hide the tab-strip control. Useful when selection is driven
    /// programmatically (e.g. a wizard). The bordered content pane
    /// continues to render the selected tab's body.
    pub fn hide_control(mut self, hide: bool) -> Self {
        self.hide_control = hide;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl RenderOnce for TabView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let selected_idx = self
            .selected_id
            .as_ref()
            .and_then(|id| self.tabs.iter().position(|t| &t.id == id))
            .or(if self.tabs.is_empty() { None } else { Some(0) });

        let on_select = self.on_select.clone();
        let tabs_len = self.tabs.len();
        let tab_ids: Vec<SharedString> = self.tabs.iter().map(|t| t.id.clone()).collect();

        // ── Tab strip ───────────────────────────────────────────────────────
        let mut control = div()
            .flex()
            .flex_row()
            .items_end()
            .gap(theme.spacing_xs)
            .pl(theme.spacing_md);

        if !self.hide_control {
            for (idx, tab) in self.tabs.iter().enumerate() {
                let is_selected = selected_idx == Some(idx);
                let weight = if is_selected {
                    theme.effective_weight(FontWeight::SEMIBOLD)
                } else {
                    theme.effective_weight(FontWeight::MEDIUM)
                };

                let ax = AccessibilityProps::new()
                    .label(tab.label.clone())
                    .role(AccessibilityRole::Tab)
                    .value(if is_selected {
                        SharedString::from("selected")
                    } else {
                        SharedString::from("unselected")
                    });

                let mut button = div()
                    .id(ElementId::Name(SharedString::from(format!(
                        "tabview-tab-{}",
                        tab.id
                    ))))
                    .px(theme.spacing_sm)
                    .py(theme.spacing_xs)
                    .min_h(px(theme.target_size()))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .with_accessibility(&ax)
                    .text_style(TextStyle::Subheadline, theme)
                    .font_weight(weight)
                    .text_color(if is_selected {
                        theme.text
                    } else {
                        theme.text_muted
                    })
                    // The "selected" tab visually merges with the content
                    // pane below — surface background + no bottom border.
                    .rounded_t(theme.radius_md)
                    .child(tab.label.clone());

                if is_selected {
                    button = button
                        .bg(theme.surface)
                        .border_t_1()
                        .border_l_1()
                        .border_r_1()
                        .border_color(theme.border);
                }

                if let Some(cb) = on_select.clone() {
                    let tab_id = tab.id.clone();
                    button =
                        button
                            .hover(|s| s.bg(theme.hover))
                            .on_click(move |_event, window, cx| {
                                cb(tab_id.clone(), window, cx);
                            });
                }

                control = control.child(button);
            }
        }

        // ── Content pane ────────────────────────────────────────────────────
        let mut body_container = div()
            .flex_1()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md)
            .p(theme.spacing_md);

        if let Some(idx) = selected_idx {
            if let Some(tab) = self.tabs.into_iter().nth(idx) {
                body_container = body_container.child(tab.body);
            }
        }

        let mut container = div()
            .id(self.id)
            .focusable()
            .flex()
            .flex_col()
            .w_full()
            .h_full();

        // Arrow-key navigation through the tab strip. HIG: tabs
        // behave as a radio group, so arrow keys cycle and wrap.
        if !self.hide_control && tabs_len > 1 {
            if let Some(cb) = on_select.clone() {
                let ids = tab_ids;
                container = container.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let delta: i32 = match key {
                        "right" => 1,
                        "left" => -1,
                        _ => return,
                    };
                    cx.stop_propagation();
                    let len = ids.len() as i32;
                    let current = selected_idx.unwrap_or(0) as i32;
                    let next = ((current + delta).rem_euclid(len)) as usize;
                    cb(ids[next].clone(), window, cx);
                });
            }
        }

        container = container.child(control).child(body_container);
        container = apply_focus_ring(container, theme, self.focused, &[]);

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::div;

    use super::{MAX_TAB_VIEW_TABS, Tab, TabView};

    #[test]
    fn tab_new() {
        let tab = Tab::new("a", "A", div());
        assert_eq!(tab.id.as_ref(), "a");
        assert_eq!(tab.label.as_ref(), "A");
    }

    #[test]
    fn tab_view_defaults() {
        let view = TabView::new("tv");
        assert!(view.tabs.is_empty());
        assert!(view.selected_id.is_none());
        assert!(view.on_select.is_none());
        assert!(!view.hide_control);
        assert!(!view.focused);
    }

    #[test]
    fn tab_view_accepts_up_to_max_tabs() {
        let tabs: Vec<_> = (0..MAX_TAB_VIEW_TABS)
            .map(|i| Tab::new(format!("t{i}"), format!("T{i}"), div()))
            .collect();
        let view = TabView::new("tv").tabs(tabs);
        assert_eq!(view.tabs.len(), MAX_TAB_VIEW_TABS);
    }

    #[test]
    fn tab_view_hide_control_builder() {
        let view = TabView::new("tv").hide_control(true);
        assert!(view.hide_control);
    }

    #[test]
    fn tab_view_selected_id_builder() {
        let view = TabView::new("tv").selected_id("main");
        assert_eq!(view.selected_id.as_ref().map(|s| s.as_ref()), Some("main"));
    }
}
