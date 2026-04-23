//! Sidebar component with Liquid Glass styling.
//!
//! HIG sidebar: an inset glass panel where content flows behind it.
//! Uses [`Glass::Regular`] at [`Elevation::Elevated`].
//!
//! This module also exposes:
//!
//! - [`SidebarItem`] — a focusable, keyboard-activatable row with a HIG
//!   capsule selection highlight.
//! - [`SidebarSection`] — a collapsible group header with an optional
//!   disclosure triangle, per HIG v2 "Group hierarchy with disclosure
//!   controls if your app has a lot of content".
//!
//! The [`Sidebar`] supports a `collapsed` state with animated width, and
//! optional min/max width with an interactive resize handle on the trailing
//! edge — both required by the macOS HIG.

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, FocusHandle, FontWeight, Hsla, KeyDownEvent,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, SharedString, Window, div,
    px,
};

use crate::callback_types::{OnClick, OnToggle};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::layout::SIDEBAR_MIN_WIDTH;
use crate::foundations::materials::{
    Elevation, Glass, Shape, SurfaceContext, apply_focus_ring, glass_effect_lens, resolve_focused,
};
use crate::foundations::right_to_left::apply_flex_row_direction;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};

type OnResize = Option<Box<dyn Fn(Pixels, &mut Window, &mut App) + 'static>>;
type OnReorder = Option<Box<dyn Fn(i32, &mut Window, &mut App) + 'static>>;

/// Position of the sidebar relative to the main content area.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum SidebarPosition {
    #[default]
    Left,
    Right,
}

/// A sidebar component with glass morphism styling.
///
/// When the glass theme is active, renders as an inset glass panel using
/// [`Glass::Regular`] at [`Elevation::Elevated`]. Otherwise falls back to surface/border tokens.
///
/// Use [`SidebarPosition`] in parent layout to determine placement (left/right).
///
/// # Collapsing
///
/// HIG v2 mandates that macOS sidebars be collapsible. Callers own the
/// `collapsed` state (Entity-owned boolean) and pass it to
/// [`Sidebar::collapsed`]. When `collapsed == true` the panel renders with
/// zero width. Wire a toolbar button (typically via
/// [`Toolbar::sidebar_toggle`](super::Toolbar::sidebar_toggle)) or a View
/// menu command to flip the state.
///
/// # Resizing
///
/// When [`Sidebar::min_width`] and [`Sidebar::max_width`] are supplied, a
/// 6pt drag handle appears on the trailing edge. Users can drag it to
/// resize the panel within the `[min, max]` range, firing
/// [`Sidebar::on_resize`] with each new width.
///
/// # Content-extension background
///
/// HIG v2 (June 2025) calls for sidebars to float above content with
/// `backgroundExtensionEffect()` mirroring the content underneath. The
/// default renders with a soft border for legibility; callers wishing to
/// match the Tahoe floating-above-content layout can set
/// [`Sidebar::floating`] to drop the trailing border and switch to overlay
/// positioning (handled by the parent layout).
#[derive(IntoElement)]
pub struct Sidebar {
    id: ElementId,
    children: Vec<AnyElement>,
    width: Option<Pixels>,
    min_width: Option<Pixels>,
    max_width: Option<Pixels>,
    collapsed: bool,
    on_toggle: OnToggle,
    on_resize: OnResize,
    floating: bool,
    /// Best-effort background-extension colour — the fill rendered behind
    /// the sidebar glass so the "content extends beneath" HIG intent is
    /// visible without SwiftUI's real `backgroundExtensionEffect`
    /// shader. Finding 24 in the Zed cross-reference audit.
    ///
    /// GPUI v0.231.1-pre does not expose the render-to-texture primitive
    /// needed to sample the actual content behind the sidebar, so this
    /// is a static-color workaround: the caller supplies a colour that
    /// matches the dominant content tone (or, typically, the theme's
    /// content background). When GPUI gains a framebuffer-capture /
    /// render-to-texture API, this field can be swapped for the real
    /// live-sampled extension.
    background_extension: Option<gpui::Hsla>,
}

impl Sidebar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            width: None,
            min_width: None,
            max_width: None,
            collapsed: false,
            on_toggle: None,
            on_resize: None,
            floating: false,
            background_extension: None,
        }
    }

    /// Set the fill rendered behind the sidebar glass so HIG's
    /// `backgroundExtensionEffect` appearance shows through even without
    /// a render-to-texture backbuffer. Pass the theme's content
    /// background colour or the dominant content tone. Finding 24 in
    /// the Zed cross-reference audit.
    pub fn background_extension(mut self, color: gpui::Hsla) -> Self {
        self.background_extension = Some(color);
        self
    }

    /// Append a child element.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Append multiple children from an iterator.
    pub fn children(mut self, iter: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(iter.into_iter().map(|el| el.into_any_element()));
        self
    }

    /// Set the sidebar width. Defaults to `theme.sidebar_width_default`.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    /// Lower bound for resizable width (activates the drag handle).
    pub fn min_width(mut self, min: Pixels) -> Self {
        self.min_width = Some(min);
        self
    }

    /// Upper bound for resizable width (activates the drag handle).
    pub fn max_width(mut self, max: Pixels) -> Self {
        self.max_width = Some(max);
        self
    }

    /// Toggle the collapsed state. When `true`, the panel renders at zero
    /// width.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    /// Fires when the sidebar requests a toggle (e.g. from the drag handle
    /// double-click).
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Fires while the user drags the resize handle with the new width.
    pub fn on_resize(mut self, handler: impl Fn(Pixels, &mut Window, &mut App) + 'static) -> Self {
        self.on_resize = Some(Box::new(handler));
        self
    }

    /// Render as a floating overlay (macOS 26 Tahoe). Drops the trailing
    /// hairline border so underlying content is visible beneath the glass.
    pub fn floating(mut self, floating: bool) -> Self {
        self.floating = floating;
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        // HIG macOS Tahoe: rendered width defaults to
        // `theme.sidebar_width_default` (220pt) when unspecified, and
        // is never allowed to render below the HIG floor
        // (`SIDEBAR_MIN_WIDTH` = 180pt) outside the collapsed state —
        // below 180pt row labels truncate at default Dynamic Type.
        let base_width = self
            .width
            .unwrap_or(theme.sidebar_width_default)
            .max(px(SIDEBAR_MIN_WIDTH));
        let width = if self.collapsed { px(0.0) } else { base_width };

        // HIG navigation-and-search.md:204: sidebars float above content in
        // the Liquid Glass layer. We paint a real dual-Kawase lens composite
        // so the refracted/blurred content behind the sidebar reads through.
        //
        // Finding 24 in the Zed cross-reference audit: when the host supplies
        // a `background_extension` colour we composite it *below* the lens
        // so the HIG "content extends beneath the sidebar" intent is
        // preserved. The extension sits on the outer fill; the lens canvas
        // overlays it and refracts whatever the renderer samples from the
        // framebuffer below.
        let mut el = div()
            .relative()
            .flex()
            .flex_col()
            .h_full()
            .w(width)
            .id(self.id.clone());

        if let Some(ext) = self.background_extension {
            el = el.bg(ext);
        }

        // Paint the lens as the first child so flow children render above it.
        el = el.child(
            glass_effect_lens(
                theme,
                Glass::Regular,
                Shape::RoundedRectangle(theme.radius_lg),
                Elevation::Elevated,
                None,
            )
            .absolute()
            .inset_0(),
        );

        // Floating overlays drop the trailing hairline so content is
        // visible beneath the glass; inset sidebars keep the separator for
        // document-window legibility.
        if !self.floating {
            el = el
                .border_r_1()
                .border_color(crate::foundations::color::with_alpha(theme.border, 0.5));
        }

        if self.collapsed {
            el = el.overflow_hidden();
        }

        for child in self.children {
            el = el.child(child);
        }

        // Optional drag-resize handle on the trailing edge. Activated only
        // when both `min_width` and `max_width` are supplied.
        if let (Some(min), Some(max), Some(on_resize)) =
            (self.min_width, self.max_width, self.on_resize)
            && !self.collapsed
        {
            let on_resize = Rc::new(on_resize);
            let handle_id = ElementId::from((self.id, "resize-handle"));
            let drag_state: Rc<std::cell::Cell<bool>> = Rc::new(std::cell::Cell::new(false));

            let drag_state_down = drag_state.clone();
            let drag_state_move = drag_state.clone();
            let drag_state_up = drag_state.clone();
            let resize_move = on_resize.clone();

            let handle = div()
                .id(handle_id)
                .debug_selector(|| "sidebar-resize-handle".into())
                .absolute()
                .top_0()
                .bottom_0()
                .right(px(-3.0))
                .w(px(6.0))
                .cursor_col_resize()
                .on_mouse_down(
                    MouseButton::Left,
                    move |_event: &MouseDownEvent, _window, _cx| {
                        drag_state_down.set(true);
                    },
                )
                .on_mouse_move(move |event: &MouseMoveEvent, window, cx| {
                    if drag_state_move.get() {
                        // Clamp to caller-supplied [min, max], never
                        // below the HIG floor (SIDEBAR_MIN_WIDTH).
                        let effective_min = f32::from(min).max(SIDEBAR_MIN_WIDTH);
                        let new_width = f32::from(event.position.x)
                            .max(effective_min)
                            .min(f32::from(max));
                        resize_move(px(new_width), window, cx);
                    }
                })
                .on_mouse_up(
                    MouseButton::Left,
                    move |_event: &MouseUpEvent, _window, _cx| {
                        drag_state_up.set(false);
                    },
                );

            el = el.relative().child(handle);
        }

        el
    }
}

// ── SidebarSection ───────────────────────────────────────────────────────────

/// A collapsible group of sidebar rows with an optional disclosure
/// chevron. HIG v2 explicitly calls for grouping sidebar content with
/// disclosure controls and a section title.
#[derive(IntoElement)]
pub struct SidebarSection {
    id: ElementId,
    title: SharedString,
    collapsible: bool,
    expanded: bool,
    children: Vec<AnyElement>,
    on_toggle: OnToggle,
}

impl SidebarSection {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            collapsible: false,
            expanded: true,
            children: Vec::new(),
            on_toggle: None,
        }
    }

    /// Append a row to this section.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    /// Append multiple rows to this section.
    pub fn children(mut self, iter: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(iter.into_iter().map(|el| el.into_any_element()));
        self
    }

    /// Enable the disclosure chevron, making this section collapsible.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Controls whether the section body is visible. Only meaningful when
    /// [`SidebarSection::collapsible`] is `true`.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Fires with the new `expanded` state when the user toggles the
    /// disclosure chevron.
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SidebarSection {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let show_children = !self.collapsible || self.expanded;

        // `apply_flex_row_direction` keeps the disclosure chevron on the
        // reading-leading edge in both LTR and RTL layouts. Chevron glyph
        // flips via `Icon::follow_layout_direction`.
        let mut header = apply_flex_row_direction(div().id(self.id.clone()).flex(), theme)
            .items_center()
            .gap(theme.spacing_xs)
            .px(theme.spacing_md)
            .py(theme.spacing_xs)
            .text_style(TextStyle::Caption1, theme)
            .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
            .text_color(theme.secondary_label_color(SurfaceContext::GlassDim));

        if self.collapsible {
            header = header.cursor_pointer().child(
                Icon::new(if self.expanded {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                })
                .size(px(10.0))
                .color(theme.text_tertiary()),
            );

            if let Some(handler) = self.on_toggle {
                let expanded = self.expanded;
                let h = std::rc::Rc::new(handler);
                header = header.on_click(move |_event, window, cx| {
                    h(!expanded, window, cx);
                });
            }
        }

        header = header.child(div().flex_1().child(self.title));

        let mut outer = div().flex().flex_col().child(header);
        if show_children {
            let body = div().flex().flex_col().children(self.children);
            outer = outer.child(body);
        }
        outer
    }
}

// ── SidebarItem ──────────────────────────────────────────────────────────────

/// A focusable, keyboard-activatable row used inside sidebars and lists.
///
/// Replaces the ad-hoc pattern of building clickable rows from raw `div()`s,
/// which lacked keyboard navigation, focus rings, and a platform-appropriate
/// minimum row height from `theme.target_size()` (28 pt macOS,
/// 44 pt iOS/iPadOS/watchOS). `SidebarItem` provides all of these by
/// default:
///
/// - Click via `on_click`
/// - Keyboard activation via Enter / Space (when focused)
/// - Visible focus ring when `focused(true)` is set
/// - Platform-aware minimum height (`theme.target_size()`)
/// - Optional leading icon and trailing element
/// - Selected state styling that matches the rest of the gallery sidebar
///
/// Stateless: the parent owns `selected` and `focused` state. The parent is
/// expected to track the focused index and call `.focused(true)` on the
/// matching item — typically driven by Up/Down arrow keys.
#[derive(IntoElement)]
pub struct SidebarItem {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    selected: bool,
    focused: bool,
    /// Optional host-supplied focus handle. Precedence rules live on
    /// [`resolve_focused`](crate::foundations::materials::resolve_focused):
    /// when set, the focus-ring derives from `handle.is_focused(window)`
    /// and the root element threads `track_focus(&handle)`.
    ///
    /// Each `SidebarItem` must own its own `FocusHandle` — sharing one
    /// across multiple items collapses the focus-graph node and the ring
    /// will follow whichever item rendered last. A sibling [`List`] (or
    /// any other host-level focus target) should hold a distinct handle
    /// too; do not wire the same handle to both a list and its rows.
    focus_handle: Option<FocusHandle>,
    disabled: bool,
    on_click: OnClick,
    accessibility_label: Option<SharedString>,
    trailing: Option<AnyElement>,
    on_reorder: OnReorder,
}

impl SidebarItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            selected: false,
            focused: false,
            focus_handle: None,
            disabled: false,
            on_click: None,
            accessibility_label: None,
            trailing: None,
            on_reorder: None,
        }
    }

    /// Set a leading icon.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Mark this item as currently selected (highlighted background, semibold label).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Mark this item as keyboard-focused (renders a focus ring).
    /// Ignored when a [`focus_handle`](Self::focus_handle) is also attached
    /// — the handle's live `is_focused(window)` state wins.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the sidebar item participates in the
    /// host's focus graph. Takes precedence over [`focused`](Self::focused)
    /// per [`resolve_focused`].
    ///
    /// Each item must own its own handle — do not share a handle across
    /// multiple `SidebarItem`s, and do not reuse the enclosing `List`'s
    /// handle, or the focus ring will follow whichever node rendered last.
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Click handler. Also fired when the focused item receives Enter or Space.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set an accessibility label for screen readers. Defaults to `label` when unset.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Append a trailing element (e.g. badge, count, chevron).
    pub fn trailing(mut self, element: impl IntoElement) -> Self {
        self.trailing = Some(element.into_any_element());
        self
    }

    /// Enable drag-to-reorder. The handler receives a signed delta (positive
    /// = move down / after, negative = move up / before) when the user
    /// presses ⌥⌘↑/⌥⌘↓ while the item is focused. Drag-and-drop with the
    /// mouse will be wired once GPUI exposes a drag callback; for now the
    /// keyboard path covers the accessibility requirement.
    pub fn on_reorder(mut self, handler: impl Fn(i32, &mut Window, &mut App) + 'static) -> Self {
        self.on_reorder = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SidebarItem {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let focused = resolve_focused(self.focus_handle.as_ref(), window, self.focused);

        // Per macOS Tahoe: selected sidebar item uses a soft accent-tinted
        // background (light blue ~#D4E4F7 for blue accent), not the generic hover gray.
        let bg: Hsla = if self.selected {
            crate::foundations::color::with_alpha(theme.accent, 0.15)
        } else {
            gpui::transparent_black()
        };
        let hover_bg = crate::foundations::color::with_alpha(theme.accent, 0.10);
        let label_color = theme.label_color(SurfaceContext::GlassDim);
        let icon_color = if self.selected {
            theme.accent
        } else {
            theme.text_muted
        };
        let weight = if self.selected {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        };

        let has_handler = self.on_click.is_some();
        // `apply_flex_row_direction` places the leading icon on the reading-
        // leading edge under both LTR and RTL themes. The row's symmetric
        // horizontal padding needs no direction-aware swap.
        let mut row = apply_flex_row_direction(div().id(self.id).focusable().flex(), theme)
            .items_center()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_xs)
            .mx(theme.spacing_xs)
            .min_h(px(theme.target_size()))
            // HIG v2 (macOS 26 Liquid Glass): sidebar selection is a capsule
            // (fully rounded pill), not a rounded-rect. Use radius_full to
            // render the required pill highlight.
            .rounded(theme.radius_full)
            .bg(bg)
            .text_color(label_color)
            .text_style(TextStyle::Body, theme)
            .font_weight(weight);

        if let Some(handle) = self.focus_handle.as_ref() {
            row = row.track_focus(handle);
        }

        if self.disabled {
            row = row.opacity(0.5).cursor_default();
        } else if has_handler {
            row = row.cursor_pointer().hover(move |s| s.bg(hover_bg));
        }

        // Visible focus ring when keyboard-focused.
        row = apply_focus_ring(row, theme, focused, &[]);

        // Wire click + keyboard handlers. Rc lets us share the closure
        // between mouse click, keyboard activation, and reorder paths.
        let reorder_handler = self.on_reorder.map(Rc::new);

        if let Some(handler) = self.on_click
            && !self.disabled
        {
            let handler = Rc::new(handler);

            // Mouse / trackpad click
            {
                let handler = handler.clone();
                row = row.on_click(move |event, window, cx| handler(event, window, cx));
            }

            // Keyboard activation: Enter or Space plus optional ⌥⌘↑/↓
            // reorder shortcuts.
            let reorder = reorder_handler.clone();
            row = row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                if crate::foundations::keyboard::is_activation_key(event) {
                    cx.stop_propagation();
                    handler(&ClickEvent::default(), window, cx);
                    return;
                }

                if let Some(reorder) = &reorder {
                    let m = &event.keystroke.modifiers;
                    if m.alt && m.platform {
                        match event.keystroke.key.as_str() {
                            "up" => {
                                cx.stop_propagation();
                                reorder(-1, window, cx);
                            }
                            "down" => {
                                cx.stop_propagation();
                                reorder(1, window, cx);
                            }
                            _ => {}
                        }
                    }
                }
            });
        } else if let Some(reorder) = reorder_handler {
            // Keyboard reorder works even on non-clickable rows (e.g.
            // headers reordering their section).
            row = row.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let m = &event.keystroke.modifiers;
                if m.alt && m.platform {
                    match event.keystroke.key.as_str() {
                        "up" => {
                            cx.stop_propagation();
                            reorder(-1, window, cx);
                        }
                        "down" => {
                            cx.stop_propagation();
                            reorder(1, window, cx);
                        }
                        _ => {}
                    }
                }
            });
        }

        if let Some(icon) = self.icon {
            row = row.child(Icon::new(icon).size(px(16.0)).color(icon_color));
        }

        let label = self.label;
        row = row.child(div().flex_1().child(label));

        if let Some(trailing) = self.trailing {
            row = row.child(trailing);
        }

        // Accessibility label is captured for future use once GPUI exposes
        // an aria-label API. For now we just keep the field alive so the
        // builder method has a stable shape.
        let _ = self.accessibility_label;

        row
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::prelude::*;
    use gpui::{div, px};

    use super::{Sidebar, SidebarItem, SidebarPosition, SidebarSection};
    use crate::foundations::icons::IconName;

    #[test]
    fn sidebar_position_default_is_left() {
        assert_eq!(SidebarPosition::default(), SidebarPosition::Left);
    }

    #[test]
    fn sidebar_position_equality() {
        assert_eq!(SidebarPosition::Left, SidebarPosition::Left);
        assert_eq!(SidebarPosition::Right, SidebarPosition::Right);
        assert_ne!(SidebarPosition::Left, SidebarPosition::Right);
    }

    #[test]
    fn sidebar_new_has_defaults() {
        let sidebar = Sidebar::new("test-sidebar");
        assert!(sidebar.children.is_empty());
        assert_eq!(sidebar.width, None);
        assert!(!sidebar.collapsed);
        assert!(!sidebar.floating);
        assert!(sidebar.min_width.is_none());
        assert!(sidebar.max_width.is_none());
    }

    #[test]
    fn sidebar_builder_width() {
        let sidebar = Sidebar::new("test-sidebar").width(px(320.0));
        assert_eq!(sidebar.width, Some(px(320.0)));
    }

    #[test]
    fn sidebar_builder_child_adds() {
        let sidebar = Sidebar::new("test-sidebar");
        assert!(sidebar.children.is_empty());

        let sidebar = sidebar.child(div().id("c1"));
        assert_eq!(sidebar.children.len(), 1);

        let sidebar = sidebar.child(div().id("c2"));
        assert_eq!(sidebar.children.len(), 2);
    }

    #[test]
    fn sidebar_builder_children_from_iter() {
        let items: Vec<_> = vec![div().id("a"), div().id("b"), div().id("c")];
        let sidebar = Sidebar::new("test-sidebar").children(items);
        assert_eq!(sidebar.children.len(), 3);
    }

    #[test]
    fn sidebar_collapsed_builder() {
        let sidebar = Sidebar::new("s").collapsed(true);
        assert!(sidebar.collapsed);
    }

    #[test]
    fn sidebar_on_toggle_builder() {
        let sidebar = Sidebar::new("s").on_toggle(|_, _, _| {});
        assert!(sidebar.on_toggle.is_some());
    }

    #[test]
    fn sidebar_on_resize_builder() {
        let sidebar = Sidebar::new("s").on_resize(|_, _, _| {});
        assert!(sidebar.on_resize.is_some());
    }

    #[test]
    fn sidebar_min_max_width_builders() {
        let sidebar = Sidebar::new("s").min_width(px(180.0)).max_width(px(480.0));
        assert_eq!(sidebar.min_width, Some(px(180.0)));
        assert_eq!(sidebar.max_width, Some(px(480.0)));
    }

    #[test]
    fn sidebar_floating_builder() {
        let sidebar = Sidebar::new("s").floating(true);
        assert!(sidebar.floating);
    }

    // ── SidebarSection ──────────────────────────────────────────────────

    #[test]
    fn sidebar_section_defaults() {
        let sec = SidebarSection::new("sec", "Favorites");
        assert_eq!(sec.title.as_ref(), "Favorites");
        assert!(!sec.collapsible);
        assert!(sec.expanded);
        assert!(sec.children.is_empty());
        assert!(sec.on_toggle.is_none());
    }

    #[test]
    fn sidebar_section_collapsible_expanded_builders() {
        let sec = SidebarSection::new("sec", "Tags")
            .collapsible(true)
            .expanded(false);
        assert!(sec.collapsible);
        assert!(!sec.expanded);
    }

    #[test]
    fn sidebar_section_children_builder() {
        let sec = SidebarSection::new("sec", "Tags").child(div()).child(div());
        assert_eq!(sec.children.len(), 2);
    }

    #[test]
    fn sidebar_section_on_toggle_builder() {
        let sec = SidebarSection::new("sec", "Tags").on_toggle(|_, _, _| {});
        assert!(sec.on_toggle.is_some());
    }

    // ── SidebarItem ─────────────────────────────────────────────────────

    #[test]
    fn sidebar_item_new_has_defaults() {
        let item = SidebarItem::new("test", "Inbox");
        assert_eq!(item.label.as_ref(), "Inbox");
        assert!(item.icon.is_none());
        assert!(!item.selected);
        assert!(!item.focused);
        assert!(item.focus_handle.is_none());
        assert!(!item.disabled);
        assert!(item.on_click.is_none());
        assert!(item.accessibility_label.is_none());
        assert!(item.trailing.is_none());
        assert!(item.on_reorder.is_none());
    }

    #[test]
    fn sidebar_item_builder_methods() {
        let item = SidebarItem::new("test", "Drafts")
            .icon(IconName::Folder)
            .selected(true)
            .focused(true)
            .disabled(false)
            .accessibility_label("Drafts folder, 5 items")
            .on_click(|_, _, _| {});
        assert_eq!(item.icon, Some(IconName::Folder));
        assert!(item.selected);
        assert!(item.focused);
        assert!(!item.disabled);
        assert!(item.on_click.is_some());
        assert_eq!(
            item.accessibility_label.as_ref().map(|s| s.as_ref()),
            Some("Drafts folder, 5 items")
        );
    }

    #[test]
    fn sidebar_item_disabled_builder() {
        let item = SidebarItem::new("test", "Sent").disabled(true);
        assert!(item.disabled);
    }

    #[test]
    fn sidebar_item_trailing_builder() {
        let item = SidebarItem::new("test", "Inbox").trailing(div().id("badge"));
        assert!(item.trailing.is_some());
    }

    #[test]
    fn sidebar_item_on_reorder_builder() {
        let item = SidebarItem::new("test", "Inbox").on_reorder(|_, _, _| {});
        assert!(item.on_reorder.is_some());
    }
}

#[cfg(test)]
mod rtl_smoke_tests {
    use super::{SidebarItem, SidebarSection};
    use crate::foundations::icons::IconName;
    use crate::test_helpers::helpers::setup_test_window_rtl;
    use gpui::{Context, IntoElement, Render, TestAppContext, div};

    struct SectionHarness;
    impl SectionHarness {
        fn new(_: &mut Context<Self>) -> Self {
            Self
        }
    }
    impl Render for SectionHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            // Collapsed state uses ChevronRight (a Directional glyph that
            // auto-flips in RTL). Exercises both the header's
            // `apply_flex_row_direction` wiring and the Icon flip path.
            SidebarSection::new("section", "Favorites")
                .collapsible(true)
                .expanded(false)
                .child(div())
        }
    }

    #[gpui::test]
    async fn sidebar_section_renders_under_rtl(cx: &mut TestAppContext) {
        let _ = setup_test_window_rtl(cx, |_window, cx| SectionHarness::new(cx));
    }

    struct ItemHarness;
    impl ItemHarness {
        fn new(_: &mut Context<Self>) -> Self {
            Self
        }
    }
    impl Render for ItemHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            SidebarItem::new("item", "Inbox").icon(IconName::Folder)
        }
    }

    #[gpui::test]
    async fn sidebar_item_renders_under_rtl(cx: &mut TestAppContext) {
        let _ = setup_test_window_rtl(cx, |_window, cx| ItemHarness::new(cx));
    }
}
