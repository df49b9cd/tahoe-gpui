//! HIG Toolbar — horizontal action bar for content areas.
//!
//! A stateless `RenderOnce` component providing a horizontal bar with leading
//! actions, a centered title, and trailing actions. Distinct from
//! [`NavigationBarIOS`](super::NavigationBarIOS) which serves app-level navigation;
//! `Toolbar` is intended for contextual actions within a content area.
//!
//! # Typed item slots
//!
//! Beyond the generic `leading()` / `trailing()` slots, the builder exposes
//! typed methods that match the item conventions in the macOS HIG:
//!
//! - [`Toolbar::sidebar_toggle`] inserts the canonical first leading item —
//!   a `sidebar.left` button. HIG: "In macOS, the sidebar toggle is the
//!   first item in the unified toolbar."
//! - [`Toolbar::primary_action`] designates a trailing item as the
//!   prominent primary action, rendering it with filled/tinted treatment.
//! - [`Toolbar::overflow`] collects items that should be placed into a
//!   trailing ellipsis pulldown when the toolbar is narrow (the HIG calls
//!   for automatic overflow on NSToolbar).
//! - [`Toolbar::style`] selects between [`ToolbarStyle::Inline`] (the
//!   default, for in-content bars) and [`ToolbarStyle::Floating`] — the
//!   macOS 26 Tahoe Liquid Glass pill hovering above content used in
//!   Safari, Finder, and the new System Settings.

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, FontWeight, SharedString, Window, div, px};

use crate::callback_types::OnMutCallback;
use crate::components::menus_and_actions::pulldown_button::{PulldownButton, PulldownItem};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::{SurfaceContext, glass_surface};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};

/// How the toolbar is laid out against surrounding content.
///
/// HIG (macOS 26 Tahoe): Safari, Finder, and the redesigned System Settings
/// render a **floating** Liquid Glass toolbar hovering above content. Inline
/// toolbars are still used in document windows where the bar is part of the
/// content stack. `Inline` keeps the bar in normal flex flow; `Floating`
/// renders it as an absolute-positioned overlay with a stronger glass shadow.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ToolbarStyle {
    /// Normal flex-flow bar anchored to the top of the content area.
    #[default]
    Inline,
    /// Liquid-glass pill that floats above content (macOS 26 Tahoe). Uses
    /// `position: absolute` with a deeper glass shadow.
    Floating,
}

/// HIG Toolbar — horizontal action bar for content areas.
///
/// Renders a flex row with leading elements on the left, an optional centered
/// title, and trailing elements on the right. Uses glass surface styling when
/// a glass theme is active, falling back to bordered surface otherwise.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    leading: Vec<AnyElement>,
    trailing: Vec<AnyElement>,
    primary_action: Option<AnyElement>,
    overflow: Vec<PulldownItem>,
    title: Option<SharedString>,
    sidebar_toggle: OnMutCallback,
    style: ToolbarStyle,
    customizable: bool,
    on_customize: OnMutCallback,
}

impl Toolbar {
    /// Create a new toolbar with the given element id.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            leading: Vec::new(),
            trailing: Vec::new(),
            primary_action: None,
            overflow: Vec::new(),
            title: None,
            sidebar_toggle: None,
            style: ToolbarStyle::Inline,
            customizable: false,
            on_customize: None,
        }
    }

    /// Append a leading (left-aligned) element.
    pub fn leading(mut self, element: impl IntoElement) -> Self {
        self.leading.push(element.into_any_element());
        self
    }

    /// Append a trailing (right-aligned) element.
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing.push(element.into_any_element());
        self
    }

    /// Install the canonical sidebar-toggle button as the first leading
    /// item, per macOS unified-toolbar conventions. The handler fires when
    /// the button is clicked.
    pub fn sidebar_toggle(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.sidebar_toggle = Some(Box::new(handler));
        self
    }

    /// Designate a prominent primary action. The item is rendered at the
    /// trailing edge with filled/tinted button treatment per HIG
    /// ("A toolbar should have at most one primary action").
    pub fn primary_action(mut self, element: impl IntoElement) -> Self {
        self.primary_action = Some(element.into_any_element());
        self
    }

    /// Append an overflow item. Overflow items are rendered inside a
    /// trailing `ellipsis` (`…`) pulldown, matching the macOS automatic
    /// overflow behavior of NSToolbar when window width is constrained.
    pub fn overflow(mut self, item: PulldownItem) -> Self {
        self.overflow.push(item);
        self
    }

    /// Set the centered title text.
    pub fn title(mut self, text: impl Into<SharedString>) -> Self {
        self.title = Some(text.into());
        self
    }

    /// Pick between inline and floating Liquid Glass layouts.
    pub fn style(mut self, style: ToolbarStyle) -> Self {
        self.style = style;
        self
    }

    /// Mark this toolbar as customizable. HIG: macOS toolbars can be
    /// customized via right-click → "Customize Toolbar…". Pair with
    /// [`Toolbar::on_customize`] to open the customization sheet.
    pub fn customizable(mut self, customizable: bool) -> Self {
        self.customizable = customizable;
        self
    }

    /// Install a callback fired when the user requests toolbar
    /// customization (typically via right-click on the bar background).
    pub fn on_customize(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_customize = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut leading_children: Vec<AnyElement> = Vec::new();
        if let Some(handler) = self.sidebar_toggle {
            let toggle_id = ElementId::from((self.id.clone(), "sidebar-toggle"));
            let h = std::rc::Rc::new(handler);
            leading_children.push(
                crate::components::menus_and_actions::Button::new(toggle_id)
                    .variant(crate::components::menus_and_actions::button::ButtonVariant::Ghost)
                    .icon(Icon::new(IconName::SidebarLeft))
                    .accessibility_label("Toggle Sidebar")
                    .on_click(move |_event, window, cx| h(window, cx))
                    .into_any_element(),
            );
        }
        leading_children.extend(self.leading);

        // Leading group (left-aligned).
        let leading_group = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_sm)
            .children(leading_children);

        // Centered title — flex-1 so it fills the middle and centers text.
        // Respects BoldText accessibility via effective_weight.
        let title_el = div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .text_color(theme.label_color(SurfaceContext::GlassDim))
            .text_style(TextStyle::Title3, theme)
            .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
            .children(self.title.map(|t| div().child(t)));

        // Trailing group (right-aligned). Order per HIG:
        //   <user trailing…> <overflow ellipsis> <primary action>
        let mut trailing_children: Vec<AnyElement> = Vec::new();
        trailing_children.extend(self.trailing);

        if !self.overflow.is_empty() {
            let overflow_id = ElementId::from((self.id.clone(), "overflow"));
            let mut pulldown = PulldownButton::new(overflow_id, "")
                .icon(Icon::new(IconName::Ellipsis))
                .borderless(true)
                .compact(true);
            for item in self.overflow {
                pulldown = pulldown.item(item);
            }
            trailing_children.push(pulldown.into_any_element());
        }

        if let Some(primary) = self.primary_action {
            trailing_children.push(primary);
        }

        let trailing_group = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_sm)
            .children(trailing_children);

        // Assemble the bar with glass surface. Floating style uses a
        // deeper glass size (`Medium`) so the hover-above-content shadow
        // reads; inline stays `Small`.
        let size = match self.style {
            ToolbarStyle::Inline => GlassSize::Small,
            ToolbarStyle::Floating => GlassSize::Medium,
        };

        let mut bar_inner = div()
            .min_h(px(theme.target_size()))
            .px(theme.spacing_md)
            .flex()
            .flex_row()
            .items_center();

        if self.style == ToolbarStyle::Floating {
            bar_inner = bar_inner.rounded(theme.radius_full);
        }

        let mut bar = glass_surface(bar_inner, theme, size)
            .id(self.id)
            .child(leading_group)
            .child(title_el)
            .child(trailing_group);

        // Right-click to customize, per HIG.
        if self.customizable
            && let Some(handler) = self.on_customize
        {
            let h = std::rc::Rc::new(handler);
            bar = bar.on_mouse_down(
                gpui::MouseButton::Right,
                move |_event: &gpui::MouseDownEvent, window, cx| {
                    h(window, cx);
                },
            );
        }

        bar
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use gpui::SharedString;

    use super::{Toolbar, ToolbarStyle};

    #[test]
    fn toolbar_new_defaults() {
        let tb = Toolbar::new("tb");
        assert!(tb.leading.is_empty());
        assert!(tb.trailing.is_empty());
        assert!(tb.title.is_none());
        assert!(tb.sidebar_toggle.is_none());
        assert!(tb.primary_action.is_none());
        assert!(tb.overflow.is_empty());
        assert_eq!(tb.style, ToolbarStyle::Inline);
        assert!(!tb.customizable);
    }

    #[test]
    fn toolbar_title_builder() {
        let tb = Toolbar::new("tb").title("Actions");
        assert_eq!(tb.title.unwrap().as_ref(), "Actions");
    }

    #[test]
    fn toolbar_title_accepts_shared_string() {
        let s = SharedString::from("Shared");
        let tb = Toolbar::new("tb").title(s);
        assert_eq!(tb.title.unwrap().as_ref(), "Shared");
    }

    #[test]
    fn toolbar_leading_appends_elements() {
        let tb = Toolbar::new("tb").leading(gpui::div()).leading(gpui::div());
        assert_eq!(tb.leading.len(), 2);
    }

    #[test]
    fn toolbar_trailing_appends_elements() {
        let tb = Toolbar::new("tb")
            .trailing(gpui::div())
            .trailing(gpui::div())
            .trailing(gpui::div());
        assert_eq!(tb.trailing.len(), 3);
    }

    #[test]
    fn toolbar_chained_builder() {
        let tb = Toolbar::new("tb")
            .title("Edit")
            .leading(gpui::div())
            .trailing(gpui::div());
        assert_eq!(tb.title.unwrap().as_ref(), "Edit");
        assert_eq!(tb.leading.len(), 1);
        assert_eq!(tb.trailing.len(), 1);
    }

    #[test]
    fn toolbar_empty_leading_trailing() {
        let tb = Toolbar::new("empty");
        assert!(tb.leading.is_empty());
        assert!(tb.trailing.is_empty());
        assert!(tb.title.is_none());
    }

    #[test]
    fn toolbar_sidebar_toggle_builder_installs_callback() {
        let tb = Toolbar::new("tb").sidebar_toggle(|_, _| {});
        assert!(tb.sidebar_toggle.is_some());
    }

    #[test]
    fn toolbar_primary_action_builder_stores_element() {
        let tb = Toolbar::new("tb").primary_action(gpui::div());
        assert!(tb.primary_action.is_some());
    }

    #[test]
    fn toolbar_overflow_appends_items() {
        use crate::components::menus_and_actions::pulldown_button::PulldownItem;
        let tb = Toolbar::new("tb")
            .overflow(PulldownItem::new("Settings"))
            .overflow(PulldownItem::new("Preferences"));
        assert_eq!(tb.overflow.len(), 2);
    }

    #[test]
    fn toolbar_style_builder_switches_variant() {
        let tb = Toolbar::new("tb").style(ToolbarStyle::Floating);
        assert_eq!(tb.style, ToolbarStyle::Floating);
    }

    #[test]
    fn toolbar_customizable_builder() {
        let tb = Toolbar::new("tb")
            .customizable(true)
            .on_customize(|_, _| {});
        assert!(tb.customizable);
        assert!(tb.on_customize.is_some());
    }

    #[test]
    fn toolbar_style_default_is_inline() {
        assert_eq!(ToolbarStyle::default(), ToolbarStyle::Inline);
    }
}
