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
//!
//! # Keyboard
//!
//! Pass an [`Open`-mode][FocusGroupMode::Open] [`FocusGroup`] via
//! [`Toolbar::focus_group`] to enable arrow-key navigation between
//! registered items: Left / Right walk without wrapping (Open mode edges
//! stay put) and Home / End jump to the endpoints. The toolbar's handler
//! ignores any keystroke that carries a modifier, so app-level chords
//! like `Cmd-Right` (end-of-line) and the NavigationSplitView pane-jump
//! chord bubble past the toolbar untouched.
//!
//! **Tab behavior (not yet WAI-ARIA roving tabindex).** Today every
//! registered toolbar item receives a positive `tab_index` via
//! [`FocusGroupExt::focus_group`], so Tab walks through every item in
//! registration order before advancing past the toolbar — it does *not*
//! leave the toolbar on the first press. Full WAI-ARIA APG toolbar
//! semantics (roving tabindex: only the active member exposes
//! `tab_index(0)`, the rest `tab_index(-1)`) is future work; callers who
//! need that today must implement it at the host.
//!
//! The caller owns the group (cheap `Rc<RefCell>` clone) across renders.
//! Two wiring patterns are supported: (a) call
//! [`FocusGroupExt::focus_group`] per item — the idempotent
//! [`FocusGroup::register`] keeps the member list stable across renders
//! — or (b) call [`FocusGroup::set_members`] each render when the host
//! owns the handle list out-of-band or membership changes frame-to-frame.
//! See `examples/gallery/focus_groups.rs` for the reference wiring.
//!
//! [`TextField`](crate::components::selection_and_input::TextField) inputs
//! placed inside a toolbar should **not** be registered as focus-group
//! members. The toolbar's arrow-key handler is guarded by
//! [`FocusGroup::contains_focused`] — a TextField that is not a
//! registered member falls outside that guard and keeps its native
//! Left / Right cursor movement. Callers must still register
//! [`textfield_keybindings()`](crate::textfield_keybindings) on their
//! window so TextField's own cursor actions continue to dispatch.
//!
//! [`FocusGroupExt::focus_group`]: crate::foundations::accessibility::FocusGroupExt::focus_group

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, FontWeight, KeyDownEvent, SharedString, Window, div, px};

use crate::callback_types::OnMutCallback;
use crate::components::menus_and_actions::pulldown_button::{PulldownButton, PulldownItem};
use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, FocusGroup, FocusGroupMode,
};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::{Elevation, Glass, Shape, SurfaceContext, glass_effect_lens};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

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
///
/// # Height
///
/// The toolbar's minimum height is driven by
/// [`TahoeTheme::target_size`](crate::foundations::theme::TahoeTheme::target_size)
/// — 28 pt on macOS, 44 pt on iOS — which matches the HIG control metric for
/// an in-content action bar. Callers that need to render a full-chrome
/// unified toolbar (title bar + toolbar as one region) should size the
/// enclosing container to
/// [`MACOS_TOOLBAR_UNIFIED_HEIGHT`](crate::foundations::layout::MACOS_TOOLBAR_UNIFIED_HEIGHT)
/// (52 pt) and place this `Toolbar` inside it; this component deliberately
/// does not claim that region for itself because it is not limited to the
/// window chrome.
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
    focus_group: Option<FocusGroup>,
    accessibility_label: Option<SharedString>,
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
            focus_group: None,
            accessibility_label: None,
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

    /// Attach a caller-owned [`FocusGroup`] so the toolbar exposes
    /// arrow-key navigation over its items. See the module-level
    /// `# Keyboard` section for the wiring contract.
    ///
    /// The group must be in [`FocusGroupMode::Open`] — HIG toolbars do not
    /// wrap arrow navigation at the edges. Passing a `Cycle` or `Trap`
    /// group panics in both debug and release (silent wrap-around is more
    /// harmful than a loud failure).
    pub fn focus_group(mut self, group: FocusGroup) -> Self {
        assert_eq!(
            group.mode(),
            FocusGroupMode::Open,
            "Toolbar::focus_group requires an Open-mode FocusGroup (Cycle/Trap would wrap \
             arrow navigation at the toolbar edges, which HIG toolbars explicitly do not do)",
        );
        self.focus_group = Some(group);
        self
    }

    /// Set the accessibility label announced when assistive tech enters
    /// the toolbar (e.g. "Documents toolbar"). Paired with the implicit
    /// [`AccessibilityRole::Toolbar`] role the component attaches in
    /// [`render`](RenderOnce::render).
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
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

        // Assemble the bar with a real Liquid Glass lens composite.
        // Floating style uses a Regular/Elevated pill (full Figma lens
        // params); Inline stays Clear/Resting so the render-pass cost of
        // the always-visible chrome is bounded.
        let (glass, shape, elevation) = match self.style {
            ToolbarStyle::Inline => (Glass::Clear, Shape::Default, Elevation::Resting),
            ToolbarStyle::Floating => (Glass::Regular, Shape::Capsule, Elevation::Elevated),
        };

        let mut bar = glass_effect_lens(theme, glass, shape, elevation, None)
            .min_h(px(theme.target_size()))
            .px(theme.spacing_md)
            .flex()
            .flex_row()
            .items_center()
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

        // Toolbar arrow-key navigation. Left / Right walk between
        // registered `FocusGroup` members (Open mode: edges stay put).
        // Home / End jump to endpoints. Two guards bound the handler:
        //
        // (1) Modifier check: any chord (Cmd / Alt / Ctrl / Shift) returns
        //     early so app-level bindings like the NavigationSplitView
        //     pane-jump chord (`Cmd-Opt-[` / `Cmd-Opt-]`) and shell
        //     shortcuts like `Cmd-Right` / `Cmd-End` bubble past the
        //     toolbar untouched. Without this guard a toolbar nested in
        //     an NSV pane would steal the pane-jump chord as soon as a
        //     toolbar item held focus.
        // (2) `contains_focused` check: the handler only fires when a
        //     *registered* item holds focus. Arbitrary descendants (a
        //     TextField whose handle is not a group member, an external
        //     button) keep their native cursor / activation semantics.
        if let Some(group) = self.focus_group.clone() {
            bar = bar.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let m = &event.keystroke.modifiers;
                if m.platform || m.alt || m.control || m.shift || m.function {
                    return;
                }
                if !group.contains_focused(window) {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "left" => {
                        group.focus_previous(window, cx);
                        cx.stop_propagation();
                    }
                    "right" => {
                        group.focus_next(window, cx);
                        cx.stop_propagation();
                    }
                    "home" => {
                        group.focus_first(window, cx);
                        cx.stop_propagation();
                    }
                    "end" => {
                        group.focus_last(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            });
        }

        // Attach AX `role="toolbar"` + optional label. Today this is
        // forward-compat scaffolding (GPUI 0.2.2 has no AX tree API — see
        // `foundations/accessibility/mod.rs`); when the upstream API
        // lands, the single `AccessibleExt::with_accessibility` impl wires
        // every Toolbar to the AX tree in one place.
        //
        // TODO(AX assertion): once GPUI exposes an AX inspector API, add
        // a test that asserts this call attaches `role=Toolbar` (and the
        // optional label) so a refactor that drops the `with_accessibility`
        // line is caught — today the props struct round-trips in unit
        // tests but the render-side wiring has no assertion.
        let mut a11y = AccessibilityProps::new().role(AccessibilityRole::Toolbar);
        if let Some(label) = self.accessibility_label {
            a11y = a11y.label(label);
        }
        bar.with_accessibility(&a11y)
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
        assert!(tb.focus_group.is_none());
        assert!(tb.accessibility_label.is_none());
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

    #[test]
    fn toolbar_focus_group_builder_stores_group() {
        use crate::foundations::accessibility::{FocusGroup, FocusGroupMode};
        let group = FocusGroup::open();
        let tb = Toolbar::new("tb").focus_group(group);
        let stored = tb.focus_group.expect("focus_group should be stored");
        assert_eq!(stored.mode(), FocusGroupMode::Open);
    }

    #[test]
    fn toolbar_accessibility_label_builder_stores_label() {
        let tb = Toolbar::new("tb").accessibility_label("Documents toolbar");
        assert_eq!(
            tb.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Documents toolbar")
        );
    }

    #[test]
    #[should_panic(expected = "Open-mode FocusGroup")]
    fn toolbar_focus_group_rejects_non_open_mode() {
        use crate::foundations::accessibility::FocusGroup;
        let _ = Toolbar::new("tb").focus_group(FocusGroup::cycle());
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, TestAppContext, div};

    use super::Toolbar;
    use crate::foundations::accessibility::{FocusGroup, FocusGroupExt};
    use crate::test_helpers::helpers::{InteractionExt, setup_test_window};

    /// Harness that mints three focus handles tracked inside a caller-owned
    /// `FocusGroup`, plus one untracked handle used to verify the
    /// `contains_focused` guard. The render method wires everything through a
    /// `Toolbar` so GPUI's focus system is live during tests.
    struct ToolbarHarness {
        handles: [FocusHandle; 3],
        untracked: FocusHandle,
        group: FocusGroup,
    }

    impl ToolbarHarness {
        fn new(cx: &mut Context<Self>) -> Self {
            Self {
                handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
                untracked: cx.focus_handle(),
                group: FocusGroup::open(),
            }
        }
    }

    impl Render for ToolbarHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let group = self.group.clone();
            Toolbar::new("tb")
                .focus_group(group)
                .leading(
                    div()
                        .id("item-0")
                        .focus_group(&self.group, &self.handles[0])
                        .child("First"),
                )
                .leading(
                    div()
                        .id("item-1")
                        .focus_group(&self.group, &self.handles[1])
                        .child("Second"),
                )
                .leading(
                    div()
                        .id("untracked")
                        .track_focus(&self.untracked)
                        .child("Untracked"),
                )
                .trailing(
                    div()
                        .id("item-2")
                        .focus_group(&self.group, &self.handles[2])
                        .child("Third"),
                )
        }
    }

    #[gpui::test]
    async fn right_advances_through_members(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("right");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.handles[1].is_focused(window), "right from 0 → 1");
        });
        cx.press("right");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.handles[2].is_focused(window), "right from 1 → 2");
        });
    }

    #[gpui::test]
    async fn right_stops_at_last_in_open_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[2].focus(window, cx);
        });
        cx.press("right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[2].is_focused(window),
                "Open: right past last stays on last"
            );
        });
    }

    #[gpui::test]
    async fn left_retreats_through_members(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[2].focus(window, cx);
        });
        cx.press("left");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.handles[1].is_focused(window), "left from 2 → 1");
        });
        cx.press("left");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.handles[0].is_focused(window), "left from 1 → 0");
        });
    }

    #[gpui::test]
    async fn left_stops_at_first_in_open_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[0].is_focused(window),
                "Open: left past first stays on first"
            );
        });
    }

    #[gpui::test]
    async fn home_and_end_jump_to_endpoints(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        cx.press("home");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.handles[0].is_focused(window), "home → first");
        });
        cx.press("end");
        host.update_in(cx, |host, window, _cx| {
            assert!(host.handles[2].is_focused(window), "end → last");
        });
    }

    #[gpui::test]
    async fn arrow_keys_noop_when_untracked_element_focused(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.untracked.focus(window, cx);
        });
        cx.press("right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.untracked.is_focused(window),
                "right should not move focus from untracked element"
            );
        });
        cx.press("left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.untracked.is_focused(window),
                "left should not move focus from untracked element"
            );
        });
    }

    #[gpui::test]
    async fn cmd_right_on_tracked_member_does_not_advance(cx: &mut TestAppContext) {
        // The modifier-guard at the top of the toolbar's on_key_down must
        // bail on any chord so app-level bindings (end-of-line, window
        // shortcuts) bubble past. Without the guard, focus would advance
        // 0 → 1 and `stop_propagation` would eat the chord.
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("cmd-right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[0].is_focused(window),
                "cmd-right must not advance toolbar focus — chord must bubble"
            );
        });
    }

    #[gpui::test]
    async fn cmd_alt_right_on_tracked_member_does_not_advance(cx: &mut TestAppContext) {
        // Specifically guards against the NavigationSplitView pane-jump
        // collision: a toolbar nested in an NSV pane must let the NSV's
        // `cmd-alt-[`/`cmd-alt-]` chord bubble past. We use `cmd-alt-right`
        // (the old NSV chord) plus `cmd-alt-]` (the new chord) — both must
        // be ignored.
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[0].focus(window, cx);
        });
        cx.press("cmd-alt-right");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[0].is_focused(window),
                "cmd-alt-right must not advance toolbar focus"
            );
        });
        cx.press("cmd-alt-]");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[0].is_focused(window),
                "cmd-alt-] must not advance toolbar focus (pane-jump chord must bubble)"
            );
        });
    }

    #[gpui::test]
    async fn shift_left_on_tracked_member_does_not_retreat(cx: &mut TestAppContext) {
        // Shift+arrows are conventionally used for selection extension;
        // the toolbar must not consume them.
        let (host, cx) = setup_test_window(cx, |_window, cx| ToolbarHarness::new(cx));
        host.update_in(cx, |host, window, cx| {
            host.handles[1].focus(window, cx);
        });
        cx.press("shift-left");
        host.update_in(cx, |host, window, _cx| {
            assert!(
                host.handles[1].is_focused(window),
                "shift-left must not retreat toolbar focus"
            );
        });
    }
}
