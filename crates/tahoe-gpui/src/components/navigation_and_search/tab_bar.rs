//! Tab bar component for organizing content into switchable sections.
//!
//! Stateless `RenderOnce` component — the parent manages the active tab.
//!
//! # HIG alignment
//!
//! HIG v2 treats macOS and iOS tab bars differently:
//!
//! - **macOS (document-style)**: ruled underline beneath the active tab,
//!   per-tab close buttons on hover, optional badges. Used in Safari,
//!   Finder, Xcode. This is the default when [`TabBarStyle::Document`] is
//!   selected.
//! - **iOS / iPadOS (segmented-style)**: centered labels with a capsule /
//!   filled highlight, tightly clustered. Selected via
//!   [`TabBarStyle::Segmented`].
//! - **macOS 26 Tahoe (floating)**: Liquid Glass pill floating above
//!   content. Selected via [`TabBarStyle::Floating`].
//!
//! macOS callers should pick [`TabBarStyle::Document`] or
//! [`TabBarStyle::Floating`]; iOS / iPadOS callers pick
//! [`TabBarStyle::Segmented`].

use crate::callback_types::{OnSharedStringChange, rc_wrap};
use crate::components::content::badge::BadgeVariant;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::{SurfaceContext, apply_focus_ring, apply_high_contrast_border};
use crate::foundations::theme::{ActiveTheme, GlassSize, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ElementId, FocusHandle, FontWeight, KeyDownEvent, SharedString, Window, div,
    px,
};

/// Visual style of a [`TabBar`]. Controls indicator shape (capsule vs
/// underline) and surround treatment (flat vs glass pill). See the module
/// docs for HIG alignment.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TabBarStyle {
    /// macOS document-style tabs: underline indicator beneath the active
    /// label, flat surround. HIG default for macOS.
    #[default]
    Document,
    /// iOS / iPadOS segmented-style tabs: centered labels with a capsule
    /// highlight. The original pre-v2 rendering.
    Segmented,
    /// macOS 26 Tahoe floating Liquid Glass pill hovering above content.
    Floating,
}

/// A single tab item with label and body content.
pub struct TabItem {
    /// Unique identifier for this tab.
    pub id: SharedString,
    /// Label element shown in the tab bar.
    pub label: AnyElement,
    /// Body content shown when this tab is active.
    pub body: AnyElement,
    /// Whether this tab shows an `xmark` close button on hover. Per HIG
    /// macOS document tabs (Safari, Finder) close via an `xmark` that
    /// appears on hover — set this to `true` and wire
    /// [`TabBar::on_close`] to receive close requests.
    pub closable: bool,
    /// Optional notification badge drawn at the trailing edge of the tab.
    /// HIG v2 recommends badges on tabs to indicate critical information.
    pub badge: Option<BadgeVariant>,
    /// Optional per-tab focus handle. When every [`TabItem`] in a [`TabBar`]
    /// carries one, the bar switches to the WAI-ARIA tablist focus model:
    /// each tab is individually focusable, Enter/Space activate the focused
    /// tab, and arrow-key navigation moves focus in lock-step with the
    /// active-tab change. Callers mint the handles via
    /// `cx.focus_handle()` and persist them across renders in their entity
    /// state.
    pub focus_handle: Option<FocusHandle>,
}

impl TabItem {
    pub fn new(
        id: impl Into<SharedString>,
        label: impl IntoElement,
        body: impl IntoElement,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into_any_element(),
            body: body.into_any_element(),
            closable: false,
            badge: None,
            focus_handle: None,
        }
    }

    /// Mark this tab as closable. On hover, the tab reveals an `xmark`
    /// affordance that fires [`TabBar::on_close`] when clicked.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Attach a notification badge to this tab. Renders at the trailing
    /// edge of the tab label using the supplied [`BadgeVariant`]. Use
    /// [`BadgeVariant::Notification`] for a red count badge or
    /// [`BadgeVariant::Dot`] for a silent unread indicator.
    pub fn badge(mut self, badge: BadgeVariant) -> Self {
        self.badge = Some(badge);
        self
    }

    /// Attach a [`FocusHandle`] so this individual tab participates in the
    /// host's focus graph. When every [`TabItem`] in the [`TabBar`] carries
    /// a handle, the bar follows the WAI-ARIA tablist pattern: Enter/Space
    /// activate the focused tab, and arrow-key navigation moves focus in
    /// lock-step with the active-tab change. Mixing tabs with and without
    /// handles falls back to the legacy bar-level focus model.
    ///
    /// # Example
    ///
    /// Store handles on entity state and reuse them across renders —
    /// minting a fresh handle inside `render()` destroys focus tracking
    /// every frame:
    ///
    /// ```ignore
    /// struct MyView {
    ///     active: SharedString,
    ///     tab_handles: [FocusHandle; 3],
    /// }
    ///
    /// impl MyView {
    ///     fn new(cx: &mut Context<Self>) -> Self {
    ///         Self {
    ///             active: "home".into(),
    ///             tab_handles: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
    ///         }
    ///     }
    /// }
    ///
    /// impl Render for MyView {
    ///     fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
    ///         TabBar::new("tabs").items(vec![
    ///             TabItem::new("home", "Home", "…").focus_handle(&self.tab_handles[0]),
    ///             TabItem::new("docs", "Docs", "…").focus_handle(&self.tab_handles[1]),
    ///             TabItem::new("help", "Help", "…").focus_handle(&self.tab_handles[2]),
    ///         ])
    ///     }
    /// }
    /// ```
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }
}

/// Compute the new tab index for a keyboard navigation action.
///
/// Handles: Left/Right (wrapping; swapped when `rtl`), Home/End, ⌃Tab
/// (next, wrapping), ⌃⇧Tab (previous, wrapping), and ⌘1..=⌘9 (jump to
/// Nth tab). Mirrors `SegmentedControl`'s RTL arrow-swap.
pub(super) fn navigate_tab(
    key: &str,
    modifiers_platform: bool,
    modifiers_control: bool,
    modifiers_shift: bool,
    rtl: bool,
    active_tab: &SharedString,
    tab_ids: &[SharedString],
) -> Option<usize> {
    let count = tab_ids.len();
    if count == 0 {
        return None;
    }
    let current = tab_ids.iter().position(|id| id == active_tab).unwrap_or(0);

    // ⌘1..=⌘9 → jump to Nth tab (clamped to count)
    if modifiers_platform
        && let Some(digit) = key.chars().next().and_then(|c| c.to_digit(10))
        && (1..=9).contains(&digit)
    {
        let idx = (digit as usize - 1).min(count - 1);
        return Some(idx);
    }

    // ⌃Tab / ⌃⇧Tab → next/previous (wrapping)
    if modifiers_control && key == "tab" {
        return Some(if modifiers_shift {
            if current == 0 { count - 1 } else { current - 1 }
        } else {
            (current + 1) % count
        });
    }

    // Visual-leading motion: under RTL the leading edge is on the right,
    // so physical Right means "previous" and Left means "next". Home/End
    // stay absolute.
    let logical_key = match key {
        "left" if rtl => "right",
        "right" if rtl => "left",
        other => other,
    };

    match logical_key {
        "left" => Some(if current == 0 { count - 1 } else { current - 1 }),
        "right" => Some((current + 1) % count),
        "home" => Some(0),
        "end" => Some(count - 1),
        _ => None,
    }
}

type OnCloseTab = Option<Box<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>;

/// A horizontal tab bar with switchable content panels.
///
/// The parent manages the `active_tab` state and provides an `on_change`
/// callback to update it when the user clicks a tab.
///
/// ## Keyboard Navigation
///
/// When focused: Left/Right (wrapping; swapped under RTL), Home/End,
/// ⌃Tab / ⌃⇧Tab for next/previous, ⌘1..⌘9 to jump to a specific tab.
///
/// ### Per-tab (WAI-ARIA tablist) focus mode
///
/// When every [`TabItem`] supplies a [`FocusHandle`] via
/// [`TabItem::focus_handle`], the bar switches to the WAI-ARIA tablist
/// focus model: each tab is individually focusable, Enter/Space activate
/// the focused tab, and arrow-key navigation moves focus in lock-step
/// with the active-tab change ("selection follows focus"). The bar
/// container itself drops out of the document Tab sequence so Tab/Shift-
/// Tab cross the widget at a single roving-tabindex stop (the active
/// tab), matching ARIA Authoring Practices. Mixing tabs with and without
/// handles falls back to the legacy bar-level focus model.
#[derive(IntoElement)]
pub struct TabBar {
    id: ElementId,
    items: Vec<TabItem>,
    active_tab: SharedString,
    on_change: OnSharedStringChange,
    on_close: OnCloseTab,
    focused: bool,
    /// Optional host-supplied focus handle. Finding 18 in
    /// the Zed cross-reference audit — when set, the focus-ring visibility
    /// comes from `handle.is_focused(window)` and the root element
    /// threads `track_focus(&handle)`; otherwise uses the explicit
    /// [`focused`](Self::focused) bool.
    focus_handle: Option<FocusHandle>,
    style: TabBarStyle,
}

impl TabBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            active_tab: SharedString::default(),
            on_change: None,
            on_close: None,
            focused: false,
            focus_handle: None,
            style: TabBarStyle::default(),
        }
    }

    /// Set the tab items.
    pub fn items(mut self, items: Vec<TabItem>) -> Self {
        self.items = items;
        self
    }

    /// Set which tab is currently active.
    pub fn active(mut self, tab_id: impl Into<SharedString>) -> Self {
        self.active_tab = tab_id.into();
        self
    }

    /// Set the callback invoked when a tab is clicked.
    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Set the callback invoked when a tab's `xmark` close affordance is
    /// clicked. Only fires for tabs with [`TabItem::closable`] set to
    /// `true`.
    pub fn on_close(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }

    /// Marks this tab bar as keyboard-focused, showing a visible focus ring.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the tab bar participates in the
    /// host's focus graph. When set, the focus-ring is derived from
    /// `handle.is_focused(window)` and the root element threads
    /// `track_focus(&handle)` so Tab-cycling and keyboard shortcuts
    /// scoped to the handle fire correctly. Finding 18 in
    /// the Zed cross-reference audit.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Pick the visual style per [`TabBarStyle`]. Default:
    /// [`TabBarStyle::Document`] (macOS document tabs with underline).
    pub fn style(mut self, style: TabBarStyle) -> Self {
        self.style = style;
        self
    }
}

impl RenderOnce for TabBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let selector_id = self.id.to_string();
        let tab_bar_selector = format!("tab-bar-{selector_id}");
        // Finding 18 in the Zed cross-reference audit.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        // Collect tab IDs before consuming items (needed for keyboard nav)
        let tab_ids: Vec<SharedString> = self.items.iter().map(|item| item.id.clone()).collect();
        // Per-tab focus handles: enters WAI-ARIA tablist mode iff every
        // `TabItem` supplied one. `Option<Vec<_>>::collect` returns
        // `Some(v)` iff every element was `Some`.
        let per_tab_handles: Option<Vec<FocusHandle>> = self
            .items
            .iter()
            .map(|item| item.focus_handle.clone())
            .collect();
        let active_for_keys = self.active_tab.clone();
        let style = self.style;
        let rtl = theme.is_rtl();

        let mut tab_headers = Vec::new();
        let mut active_body: Option<AnyElement> = None;
        let on_change = rc_wrap(self.on_change);
        let on_close = rc_wrap(self.on_close);

        for (idx, item) in self.items.into_iter().enumerate() {
            let is_active = item.id == self.active_tab;
            let closable = item.closable;
            let badge = item.badge;
            if is_active {
                active_body = Some(item.body);
            }

            // WAI-ARIA tab role + aria-selected equivalent. `TabItem::label`
            // is `AnyElement` so no VoiceOver label is attached here — the
            // host supplies it via the label element itself. Currently a
            // structural no-op (GPUI has no AX tree API); see
            // `foundations::accessibility::voiceover` for the upstream wait.
            let ax = AccessibilityProps::new()
                .role(AccessibilityRole::Tab)
                .value(if is_active {
                    SharedString::from("selected")
                } else {
                    SharedString::from("unselected")
                });

            let tab_id = item.id.clone();
            let tab_selector = format!("tab-bar-{selector_id}-tab-{idx}");
            let mut tab = div()
                .id(ElementId::NamedInteger("tab".into(), idx as u64))
                .debug_selector(move || tab_selector.clone())
                .cursor_pointer()
                .min_h(px(theme.target_size()))
                .flex()
                .flex_row()
                .items_center()
                .gap(theme.spacing_xs)
                .px(theme.spacing_md)
                .py(theme.spacing_sm)
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(if is_active {
                    theme.effective_weight(FontWeight::SEMIBOLD)
                } else {
                    theme.effective_weight(FontWeight::NORMAL)
                })
                .text_color(if is_active {
                    theme.label_color(SurfaceContext::GlassDim)
                } else {
                    theme.secondary_label_color(SurfaceContext::GlassDim)
                });

            // Style-specific active-tab indicator.
            match style {
                TabBarStyle::Document => {
                    // Document tabs use a 2pt underline — macOS Safari /
                    // Finder / Xcode pattern.
                    if is_active {
                        tab = tab.border_b_2().border_color(theme.accent);
                    } else {
                        tab = tab.border_b_2().border_color(gpui::transparent_black());
                    }
                }
                TabBarStyle::Segmented | TabBarStyle::Floating => {
                    if is_active {
                        tab = tab.bg(theme.glass.hover_bg).rounded(theme.radius_full);
                    }
                }
            }

            tab = tab
                .hover(|s| s.text_color(theme.label_color(SurfaceContext::GlassDim)))
                .child(item.label);

            // Trailing badge, if configured.
            if let Some(variant) = badge {
                tab = tab.child(crate::components::content::badge::Badge::new("").variant(variant));
            }

            // Per-tab close affordance, shown on hover.
            if closable && let Some(ref close_handler) = on_close {
                let close_handler = close_handler.clone();
                let close_tab_id = tab_id.clone();
                let close_id = ElementId::NamedInteger("tab-close".into(), idx as u64);
                tab = tab.child(
                    div()
                        .id(close_id)
                        .debug_selector({
                            let id = tab_id.clone();
                            move || format!("tab-bar-close-{id}")
                        })
                        .ml(theme.spacing_xs)
                        .cursor_pointer()
                        .opacity(0.0)
                        .hover(|s| s.opacity(1.0))
                        .on_click(move |_event, window, cx| {
                            cx.stop_propagation();
                            close_handler(close_tab_id.clone(), window, cx);
                        })
                        .child(
                            Icon::new(IconName::X)
                                .size(px(10.0))
                                .color(theme.text_muted),
                        ),
                );
            }

            if let Some(ref handler) = on_change {
                let click_handler = handler.clone();
                let click_id = tab_id.clone();
                // In per-tab mode, clicks must also move keyboard focus to
                // the clicked tab so a subsequent arrow-key press reads
                // from the correct origin (belt-and-suspenders for GPUI's
                // auto-focus on `.focusable()` elements).
                let click_focus = per_tab_handles.as_ref().map(|h| h[idx].clone());
                tab = tab.on_click(move |_event, window, cx| {
                    if let Some(ref h) = click_focus {
                        h.focus(window, cx);
                    }
                    click_handler(click_id.clone(), window, cx);
                });
            }

            // WAI-ARIA tablist per-tab focus: each tab is individually
            // focusable and handles Enter/Space activation. Follows the
            // `focusable()` + `track_focus()` + `on_key_down(is_activation_key)`
            // shape used by `ButtonLike::apply_to` and `SidebarItem`;
            // `on_change` takes `SharedString` rather than `&ClickEvent`,
            // so no synthetic event is synthesized. Only activation keys
            // stop propagation — arrows / Home / End / ⌃Tab / ⌘-digit must
            // bubble to the bar-level handler at the end of this method.
            if let Some(handles) = per_tab_handles.as_ref() {
                let handle = &handles[idx];
                let tab_focused = handle.is_focused(window);
                tab = tab.focusable().track_focus(handle);

                // Focus ring must be distinguishable from the active-tab
                // selection indicator (WCAG 2.4.7 Focus Visible). Match
                // bar-level treatment: flat for document tabs, glass
                // shadow slice for segmented/floating.
                tab = match style {
                    TabBarStyle::Document => apply_focus_ring(tab, theme, tab_focused, &[]),
                    TabBarStyle::Segmented | TabBarStyle::Floating => apply_focus_ring(
                        tab,
                        theme,
                        tab_focused,
                        theme.glass.shadows(GlassSize::Small),
                    ),
                };

                if let Some(ref handler) = on_change {
                    let key_handler = handler.clone();
                    let key_id = tab_id;
                    tab = tab.on_key_down(move |event: &KeyDownEvent, window, cx| {
                        if crate::foundations::keyboard::is_activation_key(event) {
                            cx.stop_propagation();
                            key_handler(key_id.clone(), window, cx);
                        }
                    });
                }
            }

            tab = tab.with_accessibility(&ax);
            tab_headers.push(tab);
        }

        // Bar is a Tab-sequence stop only in legacy (non-per-tab) keyboard
        // mode — WAI-ARIA roving tabindex requires per-tab mode to expose
        // a single stop (the active tab), not N+1 (bar + every tab). The
        // bar-level `on_key_down` below still fires in per-tab mode via
        // event bubbling from the focused tab, so non-activation keys
        // (arrows, Home / End, ⌃Tab, ⌘-digit) continue to route here.
        let bar_focusable = per_tab_handles.is_none() || self.focus_handle.is_some();
        let mut tab_bar = div()
            .id(self.id)
            .debug_selector(move || tab_bar_selector.clone())
            .when(bar_focusable, |el| el.focusable())
            .flex()
            .children(tab_headers);

        if let Some(handle) = self.focus_handle.as_ref() {
            tab_bar = tab_bar.track_focus(handle);
        }

        // Keyboard navigation: Left/Right, Home/End, ⌃Tab/⌃⇧Tab, ⌘1..⌘9.
        if let Some(ref handler) = on_change {
            let key_handler = handler.clone();
            let key_tab_ids = tab_ids;
            // When per-tab focus mode is active, arrow-nav must move focus
            // in lock-step with the active-tab change so Enter/Space on
            // the newly-focused tab re-confirms (rather than reverts) the
            // selection — the ARIA "selection follows focus" pattern.
            let nav_handles = per_tab_handles;
            tab_bar = tab_bar.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if let Some(new_index) = navigate_tab(
                    event.keystroke.key.as_str(),
                    event.keystroke.modifiers.platform,
                    event.keystroke.modifiers.control,
                    event.keystroke.modifiers.shift,
                    rtl,
                    &active_for_keys,
                    &key_tab_ids,
                ) {
                    if let Some(ref handles) = nav_handles {
                        handles[new_index].focus(window, cx);
                    }
                    key_handler(key_tab_ids[new_index].clone(), window, cx);
                }
            });
        }

        match style {
            TabBarStyle::Document => {
                // Document tabs: flat row with a bottom hairline separator
                // aligned with the inactive tabs' transparent underline.
                tab_bar = tab_bar
                    .border_b_1()
                    .border_color(crate::foundations::color::with_alpha(theme.border, 0.5));
                tab_bar = apply_focus_ring(tab_bar, theme, focused, &[]);
            }
            TabBarStyle::Segmented | TabBarStyle::Floating => {
                let glass = &theme.glass;
                tab_bar = tab_bar
                    .bg(glass.accessible_bg(GlassSize::Small, theme.accessibility_mode))
                    .rounded(if style == TabBarStyle::Floating {
                        theme.radius_full
                    } else {
                        glass.radius(GlassSize::Small)
                    })
                    .overflow_hidden();
                tab_bar =
                    apply_focus_ring(tab_bar, theme, focused, glass.shadows(GlassSize::Small));
                tab_bar = apply_high_contrast_border(tab_bar, theme);
            }
        }

        let mut container = div().flex().flex_col().child(tab_bar);

        if let Some(body) = active_body {
            container = container.child(div().pt(theme.spacing_sm).child(body));
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::{TabBar, TabBarStyle, TabItem};
    use crate::components::content::badge::BadgeVariant;
    use core::prelude::v1::test;
    use gpui::SharedString;

    #[test]
    fn default_active_tab_is_empty() {
        let tabs = TabBar::new("test");
        assert_eq!(tabs.active_tab, SharedString::default());
    }

    #[test]
    fn items_builder() {
        let tabs = TabBar::new("test").items(vec![
            TabItem::new("a", "Label A", "Body A"),
            TabItem::new("b", "Label B", "Body B"),
        ]);
        assert_eq!(tabs.items.len(), 2);
    }

    #[test]
    fn active_tab_builder() {
        let tabs = TabBar::new("test").active("tab-2");
        assert_eq!(tabs.active_tab.as_ref(), "tab-2");
    }

    #[test]
    fn on_change_is_some() {
        let tabs = TabBar::new("test").on_change(|_, _, _| {});
        assert!(tabs.on_change.is_some());
    }

    #[test]
    fn on_close_is_some() {
        let tabs = TabBar::new("test").on_close(|_, _, _| {});
        assert!(tabs.on_close.is_some());
    }

    #[test]
    fn items_can_be_empty() {
        let tabs = TabBar::new("test");
        assert_eq!(tabs.items.len(), 0);
    }

    #[test]
    fn focused_defaults_false() {
        let tabs = TabBar::new("test");
        assert!(!tabs.focused);
    }

    #[test]
    fn focused_builder() {
        let tabs = TabBar::new("test").focused(true);
        assert!(tabs.focused);
    }

    #[test]
    fn style_default_is_document() {
        assert_eq!(TabBarStyle::default(), TabBarStyle::Document);
    }

    #[test]
    fn style_builder_switches_variant() {
        let tabs = TabBar::new("test").style(TabBarStyle::Floating);
        assert_eq!(tabs.style, TabBarStyle::Floating);
    }

    #[test]
    fn tab_item_closable_builder() {
        let item = TabItem::new("a", "A", "Body").closable(true);
        assert!(item.closable);
    }

    #[test]
    fn tab_item_badge_builder() {
        let item = TabItem::new("a", "A", "Body").badge(BadgeVariant::Dot);
        assert_eq!(item.badge, Some(BadgeVariant::Dot));
    }

    #[test]
    fn navigate_left_wraps() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            navigate_tab("left", false, false, false, false, &"a".into(), &ids),
            Some(2)
        );
        assert_eq!(
            navigate_tab("left", false, false, false, false, &"b".into(), &ids),
            Some(0)
        );
    }

    #[test]
    fn navigate_right_wraps() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            navigate_tab("right", false, false, false, false, &"c".into(), &ids),
            Some(0)
        );
        assert_eq!(
            navigate_tab("right", false, false, false, false, &"a".into(), &ids),
            Some(1)
        );
    }

    #[test]
    fn navigate_home_end() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            navigate_tab("home", false, false, false, false, &"c".into(), &ids),
            Some(0)
        );
        assert_eq!(
            navigate_tab("end", false, false, false, false, &"a".into(), &ids),
            Some(2)
        );
    }

    #[test]
    fn navigate_empty_returns_none() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec![];
        assert_eq!(
            navigate_tab("right", false, false, false, false, &"a".into(), &ids),
            None
        );
    }

    #[test]
    fn navigate_unknown_key_returns_none() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into()];
        assert_eq!(
            navigate_tab("space", false, false, false, false, &"a".into(), &ids),
            None
        );
    }

    #[test]
    fn navigate_ctrl_tab_advances() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            navigate_tab("tab", false, true, false, false, &"a".into(), &ids),
            Some(1)
        );
        assert_eq!(
            navigate_tab("tab", false, true, false, false, &"c".into(), &ids),
            Some(0)
        );
    }

    #[test]
    fn navigate_ctrl_shift_tab_reverses() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            navigate_tab("tab", false, true, true, false, &"a".into(), &ids),
            Some(2)
        );
        assert_eq!(
            navigate_tab("tab", false, true, true, false, &"b".into(), &ids),
            Some(0)
        );
    }

    #[test]
    fn navigate_cmd_digit_jumps() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into(), "c".into()];
        assert_eq!(
            navigate_tab("1", true, false, false, false, &"c".into(), &ids),
            Some(0)
        );
        assert_eq!(
            navigate_tab("2", true, false, false, false, &"a".into(), &ids),
            Some(1)
        );
        // Digit beyond tab count clamps to last.
        assert_eq!(
            navigate_tab("9", true, false, false, false, &"a".into(), &ids),
            Some(2)
        );
    }

    #[test]
    fn navigate_rtl_swaps_left_right() {
        use super::navigate_tab;
        let ids: Vec<SharedString> = vec!["a".into(), "b".into(), "c".into()];
        // In RTL, physical Right moves to the previous tab (visual-leading)
        // and Left moves to the next. Home/End stay absolute.
        assert_eq!(
            navigate_tab("right", false, false, false, true, &"b".into(), &ids),
            Some(0),
            "RTL Right should decrement"
        );
        assert_eq!(
            navigate_tab("left", false, false, false, true, &"b".into(), &ids),
            Some(2),
            "RTL Left should increment"
        );
        // Wrapping still works under RTL.
        assert_eq!(
            navigate_tab("right", false, false, false, true, &"a".into(), &ids),
            Some(2),
            "RTL Right wraps from first to last"
        );
        assert_eq!(
            navigate_tab("left", false, false, false, true, &"c".into(), &ids),
            Some(0),
            "RTL Left wraps from last to first"
        );
        // Home/End unchanged under RTL.
        assert_eq!(
            navigate_tab("home", false, false, false, true, &"c".into(), &ids),
            Some(0)
        );
        assert_eq!(
            navigate_tab("end", false, false, false, true, &"a".into(), &ids),
            Some(2)
        );
    }
}

#[cfg(test)]
mod interaction_tests {
    use gpui::prelude::*;
    use gpui::{Context, FocusHandle, IntoElement, Render, SharedString, TestAppContext, div};

    use super::{TabBar, TabItem};
    use crate::test_helpers::helpers::{
        InteractionExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const TAB_HOME: &str = "tab-bar-tabs-tab-0";
    const TAB_SETTINGS: &str = "tab-bar-tabs-tab-1";
    const PANEL_HOME: &str = "tab-panel-home";
    const PANEL_SETTINGS: &str = "tab-panel-settings";
    const PANEL_PROFILE: &str = "tab-panel-profile";
    const TAB_CLOSE_SETTINGS: &str = "tab-bar-close-settings";

    struct TabBarHarness {
        active: SharedString,
        changes: Vec<SharedString>,
        closed: Vec<SharedString>,
        closable: bool,
        /// When `Some`, the bar renders in WAI-ARIA tablist mode with per-tab
        /// focus handles. When `None`, falls back to the legacy bar-level
        /// focus model.
        tab_handles: Option<[FocusHandle; 3]>,
        /// When true and `tab_handles` is `Some`, the middle tab's handle
        /// is dropped at render time — simulating the mixed-mode contract
        /// where the bar falls back to legacy keyboard mode because not
        /// every item has a handle.
        mixed_mode: bool,
        /// Host-supplied bar focus handle. Required for legacy-mode tests
        /// that need a real focused element to route key events into —
        /// without it, `cx.press(...)` hits a default sink after a click.
        bar_handle: Option<FocusHandle>,
    }

    impl TabBarHarness {
        fn new(_cx: &mut Context<Self>, active: impl Into<SharedString>) -> Self {
            Self {
                active: active.into(),
                changes: Vec::new(),
                closed: Vec::new(),
                closable: false,
                tab_handles: None,
                mixed_mode: false,
                bar_handle: None,
            }
        }

        fn new_with_handles(cx: &mut Context<Self>, active: impl Into<SharedString>) -> Self {
            Self {
                active: active.into(),
                changes: Vec::new(),
                closed: Vec::new(),
                closable: false,
                tab_handles: Some([cx.focus_handle(), cx.focus_handle(), cx.focus_handle()]),
                mixed_mode: false,
                bar_handle: None,
            }
        }
    }

    fn items(closable: bool, handles: Option<&[FocusHandle; 3]>, mixed_mode: bool) -> Vec<TabItem> {
        let mut home = TabItem::new(
            "home",
            "Home",
            div()
                .debug_selector(|| PANEL_HOME.into())
                .child("Home body"),
        );
        let mut settings = TabItem::new(
            "settings",
            "Settings",
            div()
                .debug_selector(|| PANEL_SETTINGS.into())
                .child("Settings body"),
        )
        .closable(closable);
        let mut profile = TabItem::new(
            "profile",
            "Profile",
            div()
                .debug_selector(|| PANEL_PROFILE.into())
                .child("Profile body"),
        );

        if let Some(handles) = handles {
            home = home.focus_handle(&handles[0]);
            // Dropping the middle handle triggers the mixed-mode fallback:
            // `Option::<Vec<_>>::collect` on `item.focus_handle` yields
            // `None`, which puts the bar back in legacy keyboard mode.
            if !mixed_mode {
                settings = settings.focus_handle(&handles[1]);
            }
            profile = profile.focus_handle(&handles[2]);
        }

        vec![home, settings, profile]
    }

    impl Render for TabBarHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut Context<Self>,
        ) -> impl IntoElement {
            let entity = cx.entity().clone();
            let close_entity = cx.entity().clone();
            let mut bar = TabBar::new("tabs")
                .items(items(
                    self.closable,
                    self.tab_handles.as_ref(),
                    self.mixed_mode,
                ))
                .active(self.active.clone())
                .on_change(move |tab, _window, cx| {
                    entity.update(cx, |this, cx| {
                        this.active = tab.clone();
                        this.changes.push(tab.clone());
                        cx.notify();
                    });
                })
                .on_close(move |tab, _window, cx| {
                    close_entity.update(cx, |this, cx| {
                        this.closed.push(tab.clone());
                        cx.notify();
                    });
                });
            if let Some(ref h) = self.bar_handle {
                bar = bar.focus_handle(h);
            }
            bar
        }
    }

    #[gpui::test]
    async fn clicking_tab_updates_active_panel(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| TabBarHarness::new(cx, "home"));

        assert_element_exists(cx, PANEL_HOME);
        assert_element_absent(cx, PANEL_SETTINGS);
        cx.click_on(TAB_SETTINGS);

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "settings");
            assert_eq!(
                host.changes.last().map(SharedString::as_ref),
                Some("settings")
            );
        });
        assert_element_absent(cx, PANEL_HOME);
        assert_element_exists(cx, PANEL_SETTINGS);
    }

    #[gpui::test]
    async fn arrow_keys_move_between_tabs(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| TabBarHarness::new(cx, "settings"));

        cx.click_on(TAB_SETTINGS);
        cx.press("left");
        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "home");
            assert_eq!(host.changes.last().map(SharedString::as_ref), Some("home"));
        });

        cx.click_on(TAB_SETTINGS);
        cx.press("right");
        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "profile");
            assert_eq!(
                host.changes.last().map(SharedString::as_ref),
                Some("profile")
            );
        });

        cx.click_on(TAB_HOME);
        cx.press("left");
        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "profile");
            assert_eq!(
                host.changes.last().map(SharedString::as_ref),
                Some("profile")
            );
        });
    }

    #[gpui::test]
    async fn home_and_end_keys_jump_to_tab_edges(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| TabBarHarness::new(cx, "settings"));

        cx.click_on(TAB_SETTINGS);
        cx.press("home");
        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "home");
            assert_eq!(host.changes.last().map(SharedString::as_ref), Some("home"));
        });

        cx.click_on(TAB_SETTINGS);
        cx.press("end");
        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "profile");
            assert_eq!(
                host.changes.last().map(SharedString::as_ref),
                Some("profile")
            );
        });
        assert_element_exists(cx, PANEL_PROFILE);
    }

    #[gpui::test]
    async fn closable_tab_fires_on_close(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            let mut harness = TabBarHarness::new(cx, "home");
            harness.closable = true;
            harness
        });

        assert_element_exists(cx, TAB_CLOSE_SETTINGS);
        cx.click_on(TAB_CLOSE_SETTINGS);

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(
                host.closed.last().map(SharedString::as_ref),
                Some("settings")
            );
        });
    }

    // ── WAI-ARIA tablist per-tab focus mode ──────────────────────────

    #[gpui::test]
    async fn enter_activates_focused_tab(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "home")
        });

        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[1].focus(window, cx);
        });
        cx.press("enter");

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "settings");
            assert_eq!(
                host.changes.last().map(SharedString::as_ref),
                Some("settings")
            );
        });
    }

    #[gpui::test]
    async fn space_activates_focused_tab(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "home")
        });

        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[2].focus(window, cx);
        });
        cx.press("space");

        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "profile");
            assert_eq!(
                host.changes.last().map(SharedString::as_ref),
                Some("profile")
            );
        });
    }

    #[gpui::test]
    async fn arrow_right_moves_focus_to_next_tab(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "home")
        });

        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[0].focus(window, cx);
        });
        cx.press("right");

        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "settings");
            assert!(
                host.tab_handles.as_ref().unwrap()[1].is_focused(window),
                "expected settings tab to receive focus after arrow-right"
            );
        });
    }

    #[gpui::test]
    async fn arrow_left_wraps_focus(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "home")
        });

        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[0].focus(window, cx);
        });
        cx.press("left");

        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "profile");
            assert!(
                host.tab_handles.as_ref().unwrap()[2].is_focused(window),
                "expected profile tab to receive focus after arrow-left wraparound"
            );
        });
    }

    #[gpui::test]
    async fn home_end_move_focus(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "settings")
        });

        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[1].focus(window, cx);
        });
        cx.press("home");

        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "home");
            assert!(host.tab_handles.as_ref().unwrap()[0].is_focused(window));
        });

        cx.press("end");
        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "profile");
            assert!(host.tab_handles.as_ref().unwrap()[2].is_focused(window));
        });
    }

    #[gpui::test]
    async fn backward_compat_enter_without_handles_is_noop(cx: &mut TestAppContext) {
        // Without per-tab handles, Enter/Space do not activate a tab —
        // only clicks and the bar-level arrow/Home/End handler fire
        // `on_change`. Regression guard so future edits don't accidentally
        // wire per-tab activation to the bar-level focus.
        //
        // Bar handle is required here: without it, `cx.press(...)` after a
        // click lands on a default sink rather than the bar's
        // `on_key_down`, and the test would pass trivially (events routing
        // nowhere) rather than proving `navigate_tab` returns `None` for
        // Enter/Space.
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            let mut harness = TabBarHarness::new(cx, "home");
            harness.bar_handle = Some(cx.focus_handle());
            harness
        });

        host.update_in(cx, |host, window, cx| {
            host.bar_handle.as_ref().unwrap().focus(window, cx);
        });

        cx.press("enter");
        cx.press("space");

        host.update_in(cx, |host, _window, _cx| {
            assert!(
                host.changes.is_empty(),
                "Enter/Space must not fire on_change when per-tab handles are absent"
            );
            assert_eq!(host.active.as_ref(), "home");
        });
    }

    // ── Mixed-mode fallback (pre-tab-focus regression guard) ─────────

    #[gpui::test]
    async fn mixed_handles_fall_back_to_bar_level(cx: &mut TestAppContext) {
        // Contract: the bar switches to per-tab WAI-ARIA mode *only when
        // every* `TabItem` carries a handle. If any handle is missing,
        // the bar reverts to legacy bar-level keyboard mode — Enter/Space
        // no longer activate a focused tab, and arrows still advance the
        // selection via the bar handler.
        //
        // Regression guard for `Option::<Vec<_>>::collect` vs e.g.
        // `any()` — the former is the contract, the latter would flip
        // "iff every" to "iff any" silently.
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            let mut harness = TabBarHarness::new_with_handles(cx, "home");
            harness.mixed_mode = true;
            harness.bar_handle = Some(cx.focus_handle());
            harness
        });

        // Focus one of the two attached handles. In legacy mode, its
        // Enter/Space don't fire on_change because the per-tab handler
        // was never wired.
        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[0].focus(window, cx);
        });
        cx.press("enter");
        cx.press("space");

        host.update_in(cx, |host, _window, _cx| {
            assert!(
                host.changes.is_empty(),
                "mixed mode must not wire per-tab Enter/Space activation"
            );
        });

        // Arrows still advance via the bar-level handler: focus the bar
        // handle, press Right, and the active tab must move.
        host.update_in(cx, |host, window, cx| {
            host.bar_handle.as_ref().unwrap().focus(window, cx);
        });
        cx.press("right");
        host.update_in(cx, |host, _window, _cx| {
            assert_eq!(host.active.as_ref(), "settings");
            assert_eq!(
                host.changes.last().map(SharedString::as_ref),
                Some("settings")
            );
        });
    }

    // ── Click focus-sync + Ctrl-Tab / Cmd-digit focus-sync ───────────

    #[gpui::test]
    async fn clicking_tab_in_per_tab_mode_moves_focus(cx: &mut TestAppContext) {
        // Explicit guard that the per-tab on_click handler calls
        // `handle.focus(...)` — not just on_change. A regression here
        // would leave keyboard focus stuck on a previously-focused
        // element after the user clicks a different tab.
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "home")
        });

        cx.click_on(TAB_SETTINGS);
        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "settings");
            assert!(
                host.tab_handles.as_ref().unwrap()[1].is_focused(window),
                "click must move focus to the clicked tab in per-tab mode"
            );
        });
    }

    #[gpui::test]
    async fn ctrl_tab_moves_focus_in_per_tab_mode(cx: &mut TestAppContext) {
        // `handles[new_index].focus(...)` fires for every `navigate_tab`
        // `Some` return — not only arrow / Home / End. A future edit that
        // narrows focus-sync to the arrow branch would silently regress
        // Ctrl-Tab's focus-sync guarantee.
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "home")
        });

        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[0].focus(window, cx);
        });
        cx.press("ctrl-tab");
        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "settings");
            assert!(
                host.tab_handles.as_ref().unwrap()[1].is_focused(window),
                "Ctrl-Tab must move focus to the next tab in per-tab mode"
            );
        });

        cx.press("ctrl-shift-tab");
        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "home");
            assert!(
                host.tab_handles.as_ref().unwrap()[0].is_focused(window),
                "Ctrl-Shift-Tab must move focus to the previous tab"
            );
        });
    }

    #[gpui::test]
    async fn cmd_digit_moves_focus_in_per_tab_mode(cx: &mut TestAppContext) {
        let (host, cx) = setup_test_window(cx, |_window, cx| {
            TabBarHarness::new_with_handles(cx, "home")
        });

        host.update_in(cx, |host, window, cx| {
            host.tab_handles.as_ref().unwrap()[0].focus(window, cx);
        });
        cx.press("cmd-3");
        host.update_in(cx, |host, window, _cx| {
            assert_eq!(host.active.as_ref(), "profile");
            assert!(
                host.tab_handles.as_ref().unwrap()[2].is_focused(window),
                "Cmd-3 must move focus to the third tab"
            );
        });
    }
}
