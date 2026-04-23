//! HIG Menu Bar — macOS-style horizontal menu bar with pull-down menus.
//!
//! A stateless `RenderOnce` component that renders a horizontal bar of menu
//! titles. When a title is activated, a dropdown menu appears below it.
//!
//! For apps that want centralised menu-state management, use the
//! [`MenuBarController`] entity which owns `open_menu` and
//! `highlighted_index` and exposes focus-follows-hover behaviour.
//!
//! ## Keyboard Navigation
//!
//! When focused:
//! - Left/Right: move between menu titles
//! - Enter/Space/Down: open the selected menu
//! - Escape: close the open menu
//!
//! ## Standard structure
//!
//! macOS apps are expected to expose a canonical set of menus
//! (App, File, Edit, View, Window, Help). [`MenuBar::standard_menus`] and
//! [`MenuBar::validate_standard_structure`] help apps stay conformant. See
//! also [`crate::components::menus_and_actions::edit_menu`] for the
//! canonical Edit menu command set.
//!
//! ## Relationship to the system (AppKit) menu bar
//!
//! GPUI v0.231.1-pre (the version pinned by this workspace) does **not**
//! expose any bridge to AppKit's `NSMenu` / `NSApplication.mainMenu` — it
//! has no safe API for building or installing a system-level menu.
//! [`MenuBar`] therefore renders **only** as an in-window widget: the bar
//! appears inside the host app's GPUI window and is **not mirrored into
//! the macOS system menu bar at the top of the display**. For apps that
//! need a true system menu (the usual macOS expectation), the host must
//! currently reach into AppKit themselves via a separate bridge crate.
//!
//! When GPUI lands the native `NSMenu` API, this module will gain an
//! adapter of the shape `MenuBar::install_as_main_menu(cx)` that walks
//! the same [`Menu`] list and hands each entry to AppKit. Until then,
//! treat [`MenuBar`] as a visual-only component suitable for custom
//! chrome (e.g. a chromeless/kiosk window, or the web-style in-window
//! menu bars shipped on Linux/Windows).

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Context, ElementId, FocusHandle, FontWeight, KeyDownEvent, Pixels,
    SharedString, Window, div, point, px,
};

use crate::callback_types::rc_wrap;
use crate::foundations::accessibility::{AccessibilityProps, AccessibleExt};
use crate::foundations::layout::{DROPDOWN_MAX_HEIGHT, MENU_MIN_WIDTH};
use crate::foundations::materials::{
    Elevation, Glass, Shape, apply_focus_ring, apply_high_contrast_border, glass_effect_lens,
};
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor};
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme, TextStyle, TextStyledExt};

/// A single menu in the menu bar.
pub struct Menu {
    /// Menu title displayed in the bar.
    pub title: SharedString,
    /// Content rendered when this menu is open (typically a list of menu items).
    pub content: AnyElement,
}

impl Menu {
    pub fn new(title: impl Into<SharedString>, content: impl IntoElement) -> Self {
        Self {
            title: title.into(),
            content: content.into_any_element(),
        }
    }

    /// Construct a titled placeholder menu with no content — used by
    /// [`MenuBar::standard_menus`] and host apps that have not yet wired up
    /// every menu. The rendered dropdown is empty; callers can replace the
    /// content later.
    pub fn placeholder(title: impl Into<SharedString>) -> Self {
        Self::new(title, div())
    }
}

/// Compute the new menu index for keyboard navigation.
fn navigate_menu(key: &str, current: usize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        "left" => Some(if current == 0 { count - 1 } else { current - 1 }),
        "right" => Some((current + 1) % count),
        "home" => Some(0),
        "end" => Some(count - 1),
        _ => None,
    }
}

/// Estimate the horizontal width a menu title occupies, given the theme.
///
/// Used by [`MenuBar::render`] to anchor the open-menu dropdown under the
/// activating title rather than flush-left. The estimate uses an average
/// character advance of `0.60 × font_size` for SF Pro at Subheadline scale;
/// real typesetting varies slightly but this is accurate enough for HIG
/// conformance — the actual title click hit region is rendered by GPUI
/// from the same source metric.
pub fn estimated_title_width(title: &str, theme: &TahoeTheme) -> f32 {
    let attrs = TextStyle::Subheadline.attrs();
    let font_size = f32::from(attrs.size);
    let glyph_w = font_size * 0.60;
    let padding = f32::from(theme.spacing_md) * 2.0;
    (title.chars().count() as f32 * glyph_w) + padding
}

/// Offsets for each menu title, measured from the left edge of the bar.
/// Index `i` is the `left` offset for the `i`-th menu's dropdown.
pub fn estimated_title_offsets(titles: &[SharedString], theme: &TahoeTheme) -> Vec<Pixels> {
    let mut offsets = Vec::with_capacity(titles.len());
    let mut cursor = 0.0_f32;
    for title in titles {
        offsets.push(px(cursor));
        cursor += estimated_title_width(title.as_ref(), theme);
    }
    offsets
}

type OnMenuOpen = Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>;

/// A macOS-style horizontal menu bar.
///
/// Stateless `RenderOnce` — the parent manages which menu is open.
/// For centralised state management, pair this with [`MenuBarController`].
#[derive(IntoElement)]
pub struct MenuBar {
    id: ElementId,
    menus: Vec<Menu>,
    /// Index of the currently open menu (None = all closed).
    open_menu: Option<usize>,
    /// Index of the keyboard-highlighted menu title.
    highlighted_index: usize,
    on_open: Option<OnMenuOpen>,
    focused: bool,
    /// Optional focus handle; when set, the bar tracks GPUI's focus
    /// graph and lights the ring reactively. Takes precedence over
    /// [`MenuBar::focused`].
    focus_handle: Option<FocusHandle>,
}

impl MenuBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            menus: Vec::new(),
            open_menu: None,
            highlighted_index: 0,
            on_open: None,
            focused: false,
            focus_handle: None,
        }
    }

    /// Construct a menu bar pre-populated with the canonical macOS
    /// standard menus (App, File, Edit, View, Window, Help) as empty
    /// placeholders. Callers are expected to replace each placeholder
    /// with real content by calling [`MenuBar::menus`] on the result.
    ///
    /// HIG "The menu bar" page enumerates this canonical set — missing
    /// or re-ordered entries are flagged by
    /// [`MenuBar::validate_standard_structure`] in debug builds.
    pub fn standard_menus(id: impl Into<ElementId>, app_name: impl Into<SharedString>) -> Self {
        let app = app_name.into();
        Self::new(id).menus(vec![
            Menu::placeholder(app),
            Menu::placeholder("File"),
            Menu::placeholder("Edit"),
            Menu::placeholder("View"),
            Menu::placeholder("Window"),
            Menu::placeholder("Help"),
        ])
    }

    pub fn menus(mut self, menus: Vec<Menu>) -> Self {
        self.menus = menus;
        self
    }

    pub fn open_menu(mut self, index: Option<usize>) -> Self {
        self.open_menu = index;
        self
    }

    pub fn highlighted_index(mut self, index: usize) -> Self {
        self.highlighted_index = index;
        self
    }

    /// Called when a menu should open or close. `None` = close all.
    pub fn on_open(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_open = Some(Box::new(handler));
        self
    }

    /// Set the explicit `focused` flag. Ignored when a
    /// [`focus_handle`](Self::focus_handle) is supplied — the handle's
    /// reactive state (`handle.is_focused(window)`) takes precedence.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Wire the menu bar into GPUI's focus graph. When set, the focus
    /// ring renders based on `handle.is_focused(window)` — takes
    /// precedence over [`MenuBar::focused`].
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    /// Validate that the menu bar follows the canonical macOS ordering
    /// (App, File, Edit, View, Window, Help). Returns the list of
    /// warnings — an empty vec means the structure is conformant.
    ///
    /// The expected app-name menu is not validated (apps may brand the
    /// first menu however they like); the remaining required titles are
    /// compared case-insensitively.
    pub fn validate_standard_structure(&self) -> Vec<MenuBarWarning> {
        const REQUIRED_AFTER_APP: &[&str] = &["File", "Edit", "View", "Window", "Help"];
        let mut warnings = Vec::new();

        if self.menus.is_empty() {
            warnings.push(MenuBarWarning::Empty);
            return warnings;
        }

        // Skip index 0 (app menu); locate each required title in order.
        let mut cursor = 1;
        for required in REQUIRED_AFTER_APP {
            let found = self
                .menus
                .iter()
                .enumerate()
                .skip(cursor)
                .find_map(|(i, m)| {
                    if m.title.eq_ignore_ascii_case(required) {
                        Some(i)
                    } else {
                        None
                    }
                });
            match found {
                None => warnings.push(MenuBarWarning::Missing { expected: required }),
                Some(i) if i != cursor => {
                    warnings.push(MenuBarWarning::OutOfOrder {
                        expected: required,
                        found_at: i,
                        expected_at: cursor,
                    });
                    cursor = i + 1;
                }
                Some(i) => {
                    cursor = i + 1;
                }
            }
        }
        warnings
    }
}

/// Validation warnings emitted by [`MenuBar::validate_standard_structure`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuBarWarning {
    /// No menus at all.
    Empty,
    /// A required menu (`expected`) is missing entirely.
    Missing { expected: &'static str },
    /// A required menu appears, but at a different index than the canonical
    /// macOS ordering expects.
    OutOfOrder {
        expected: &'static str,
        found_at: usize,
        expected_at: usize,
    },
}

impl RenderOnce for MenuBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_open = rc_wrap(self.on_open);
        let highlighted = self.highlighted_index;

        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);

        // Pre-compute title offsets so the open-menu dropdown anchors under
        // the activating title instead of flush-left. Fixes #142 finding
        // #13 (dropdown always `left_0()`).
        let titles: Vec<SharedString> = self.menus.iter().map(|m| m.title.clone()).collect();
        let title_offsets = estimated_title_offsets(&titles, theme);

        let mut bar = div()
            .id(self.id)
            .focusable()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .overflow_hidden();

        if let Some(handle) = self.focus_handle.as_ref() {
            bar = bar.track_focus(handle);
        }

        // Keyboard navigation
        if let Some(ref handler) = on_open {
            let h = handler.clone();
            let open = self.open_menu;
            let menu_count = self.menus.len();
            let active_index = open.unwrap_or(highlighted);
            bar = bar.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "escape" if open.is_some() => {
                        h(None, window, cx);
                    }
                    "enter" | "space" | "down" => {
                        h(Some(highlighted), window, cx);
                    }
                    "left" | "right" | "home" | "end" => {
                        if let Some(idx) = navigate_menu(key, active_index, menu_count) {
                            h(Some(idx), window, cx);
                        }
                    }
                    _ => {}
                }
            });
        }

        // Focus ring + a11y border sit on the stateful bar; the Liquid
        // Glass lens composite is layered behind the bar in a relative
        // wrapper below so the bar itself stays `Stateful<Div>` for focus
        // handling.
        bar = apply_focus_ring(bar, theme, focused, theme.glass.shadows(GlassSize::Small));
        bar = apply_high_contrast_border(bar, theme);

        // Render menu titles
        let mut open_content: Option<AnyElement> = None;
        let mut open_offset: Pixels = px(0.0);

        for (idx, menu) in self.menus.into_iter().enumerate() {
            let is_open = self.open_menu == Some(idx);
            let is_highlighted = highlighted == idx;

            let mut title = div()
                .id(ElementId::NamedInteger("menu".into(), idx as u64))
                .min_h(px(theme.target_size()))
                .px(theme.spacing_md)
                .flex()
                .items_center()
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(if is_open || is_highlighted {
                    theme.effective_weight(FontWeight::SEMIBOLD)
                } else {
                    theme.effective_weight(FontWeight::NORMAL)
                })
                .text_color(if is_open { theme.accent } else { theme.text })
                .cursor_pointer()
                .hover(|s| s.bg(theme.hover));

            if is_highlighted {
                title = title.bg(theme.hover);
            }

            let a11y_label = menu.title.clone();
            title = title.child(menu.title);

            if let Some(ref handler) = on_open {
                let h = handler.clone();
                let already_open = is_open;
                title = title.on_click(move |_event, window, cx| {
                    if already_open {
                        h(None, window, cx);
                    } else {
                        h(Some(idx), window, cx);
                    }
                });
            }

            let a11y = AccessibilityProps::menu_bar_item(a11y_label);
            title = title.with_accessibility(&a11y);

            bar = bar.child(title);

            if is_open {
                open_content = Some(menu.content);
                open_offset = title_offsets.get(idx).copied().unwrap_or_else(|| px(0.0));
            }
        }

        // Wrap the stateful bar in a relative parent that layers a
        // Glass::Clear lens composite behind it. Clear keeps the material
        // translucent enough that always-visible chrome doesn't dominate.
        // The wrapper becomes the overlay's trigger so `AnchoredOverlay`
        // anchors the dropdown to the bar's bottom edge.
        let bar_with_lens = div()
            .relative()
            .child(
                glass_effect_lens(
                    theme,
                    Glass::Clear,
                    Shape::Default,
                    Elevation::Resting,
                    None,
                )
                .absolute()
                .left_0()
                .top_0()
                .w_full()
                .h(px(theme.target_size())),
            )
            .child(bar);

        // ── Overlay-anchored menu content ───────────────────────────────────
        // `.offset(point(open_offset, 0))` shifts the dropdown horizontally
        // to sit under the activating title; vertical gap is zero so the
        // dropdown meets the bar flush.
        let mut overlay =
            AnchoredOverlay::new(ElementId::Name("menu-bar-overlay".into()), bar_with_lens)
                .anchor(OverlayAnchor::BelowLeft)
                .offset(point(open_offset, px(0.0)));

        if let Some(content) = open_content {
            let dropdown = glass_effect_lens(
                theme,
                Glass::Regular,
                Shape::Default,
                Elevation::Elevated,
                None,
            )
            .min_w(px(MENU_MIN_WIDTH))
            .max_h(px(DROPDOWN_MAX_HEIGHT))
            .flex()
            .flex_col()
            .debug_selector(|| "menu-bar-dropdown".into())
            .child(content);
            overlay = overlay.content(dropdown);
        }

        overlay
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// MenuBarController
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Entity-owned menu-bar state.
///
/// Tracks open-menu index, keyboard highlight, and focus so the parent does
/// not have to re-implement the same boilerplate in every view. The
/// controller's `render` method (via [`gpui::Render`]) produces a [`MenuBar`]
/// wired with the controller's state and its own `on_open` callback so
/// clicking or hovering a title transfers focus correctly.
pub struct MenuBarController {
    menus: Vec<Menu>,
    open_menu: Option<usize>,
    highlighted_index: usize,
    focus_handle: FocusHandle,
}

impl MenuBarController {
    /// Construct an empty controller.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            menus: Vec::new(),
            open_menu: None,
            highlighted_index: 0,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Replace the menu set (this is the usual entry point when rebuilding
    /// menus based on application state).
    pub fn set_menus(&mut self, menus: Vec<Menu>, cx: &mut Context<Self>) {
        self.menus = menus;
        if self
            .open_menu
            .map(|i| i >= self.menus.len())
            .unwrap_or(false)
        {
            self.open_menu = None;
        }
        if self.highlighted_index >= self.menus.len() {
            self.highlighted_index = 0;
        }
        cx.notify();
    }

    /// Open or close a menu by index (`None` closes all).
    pub fn set_open_menu(&mut self, idx: Option<usize>, cx: &mut Context<Self>) {
        self.open_menu = idx;
        if let Some(i) = idx {
            self.highlighted_index = i;
        }
        cx.notify();
    }

    /// Currently open menu index, if any.
    pub fn open_menu(&self) -> Option<usize> {
        self.open_menu
    }

    /// Currently keyboard-highlighted menu index.
    pub fn highlighted_index(&self) -> usize {
        self.highlighted_index
    }

    /// Focus handle — parents can use this to track focus on the menu bar.
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
}

impl Render for MenuBarController {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let menus = std::mem::take(&mut self.menus);
        let weak = cx.weak_entity();
        let bar = MenuBar::new("menu-bar-controller")
            .menus(menus)
            .open_menu(self.open_menu)
            .highlighted_index(self.highlighted_index)
            .focus_handle(&self.focus_handle)
            .on_open(move |idx: Option<usize>, _window, cx| {
                weak.update(cx, |this, cx| {
                    this.open_menu = idx;
                    if let Some(i) = idx {
                        this.highlighted_index = i;
                    }
                    cx.notify();
                })
                .ok();
            });

        // NOTE: the parent is expected to call `set_menus` whenever the
        // menu list changes. GPUI rebuilds the element tree each render
        // so we move `self.menus` out to forward them.
        bar.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{Menu, MenuBar, MenuBarWarning, navigate_menu};
    use core::prelude::v1::test;

    #[test]
    fn menu_bar_defaults() {
        let bar = MenuBar::new("test");
        assert!(bar.menus.is_empty());
        assert!(bar.open_menu.is_none());
        assert_eq!(bar.highlighted_index, 0);
        assert!(!bar.focused);
    }

    #[test]
    fn navigate_left_wraps() {
        assert_eq!(navigate_menu("left", 0, 3), Some(2));
        assert_eq!(navigate_menu("left", 1, 3), Some(0));
    }

    #[test]
    fn navigate_right_wraps() {
        assert_eq!(navigate_menu("right", 2, 3), Some(0));
        assert_eq!(navigate_menu("right", 0, 3), Some(1));
    }

    #[test]
    fn navigate_home_end() {
        assert_eq!(navigate_menu("home", 2, 5), Some(0));
        assert_eq!(navigate_menu("end", 0, 5), Some(4));
    }

    #[test]
    fn navigate_empty_returns_none() {
        assert_eq!(navigate_menu("right", 0, 0), None);
    }

    #[test]
    fn navigate_unknown_key_returns_none() {
        assert_eq!(navigate_menu("space", 0, 3), None);
    }

    #[test]
    fn open_menu_builder() {
        let bar = MenuBar::new("test").open_menu(Some(1));
        assert_eq!(bar.open_menu, Some(1));
    }

    #[test]
    fn focused_builder() {
        let bar = MenuBar::new("test").focused(true);
        assert!(bar.focused);
    }

    #[test]
    fn menu_bar_focus_handle_none_by_default() {
        let bar = MenuBar::new("test");
        assert!(bar.focus_handle.is_none());
    }

    #[gpui::test]
    async fn menu_bar_focus_handle_builder_stores_handle(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let handle = cx.focus_handle();
            let bar = MenuBar::new("test").focus_handle(&handle);
            assert!(
                bar.focus_handle.is_some(),
                "focus_handle(..) must round-trip into the field"
            );
        });
    }

    #[test]
    fn on_open_is_some() {
        let bar = MenuBar::new("test").on_open(|_, _, _| {});
        assert!(bar.on_open.is_some());
    }

    #[test]
    fn standard_menus_populates_canonical_set() {
        let bar = MenuBar::standard_menus("app-bar", "AppName");
        let titles: Vec<_> = bar.menus.iter().map(|m| m.title.as_ref()).collect();
        assert_eq!(
            titles,
            vec!["AppName", "File", "Edit", "View", "Window", "Help"]
        );
    }

    #[test]
    fn validate_standard_structure_flags_empty() {
        let bar = MenuBar::new("empty");
        let warnings = bar.validate_standard_structure();
        assert_eq!(warnings, vec![MenuBarWarning::Empty]);
    }

    #[test]
    fn validate_standard_structure_passes_for_canonical_menus() {
        let bar = MenuBar::standard_menus("app-bar", "AppName");
        assert!(bar.validate_standard_structure().is_empty());
    }

    #[test]
    fn validate_standard_structure_flags_missing_edit() {
        let bar = MenuBar::new("partial").menus(vec![
            Menu::placeholder("App"),
            Menu::placeholder("File"),
            Menu::placeholder("View"),
            Menu::placeholder("Window"),
            Menu::placeholder("Help"),
        ]);
        let warnings = bar.validate_standard_structure();
        assert!(warnings.contains(&MenuBarWarning::Missing { expected: "Edit" }));
    }

    #[test]
    fn estimated_title_offsets_monotonic() {
        use super::estimated_title_offsets;
        use gpui::SharedString;
        let theme = crate::foundations::theme::TahoeTheme::dark();
        let titles: Vec<SharedString> = vec!["App".into(), "File".into(), "Edit".into()];
        let offsets = estimated_title_offsets(&titles, &theme);
        assert_eq!(offsets.len(), 3);
        assert!(offsets[0] < offsets[1], "offsets must be monotonic");
        assert!(offsets[1] < offsets[2], "offsets must be monotonic");
    }
}

#[cfg(test)]
mod clip_escape_tests {
    use gpui::prelude::*;
    use gpui::{Context, IntoElement, Render, TestAppContext, div, px};

    use super::{Menu, MenuBar};
    use crate::test_helpers::helpers::{LocatorExt, setup_test_window};

    /// Nest the menu bar inside a narrow `overflow_hidden()` container
    /// so we can verify `AnchoredOverlay` anchors the dropdown past
    /// the parent's clip region. The bar itself stays inside; the
    /// opened menu must escape.
    struct ClipEscapeHarness;

    impl Render for ClipEscapeHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div().pt(px(40.0)).pl(px(20.0)).child(
                div()
                    .debug_selector(|| "clip-region".into())
                    .w(px(200.0))
                    .h(px(28.0))
                    .overflow_hidden()
                    .child(
                        MenuBar::new("bar")
                            .menus(vec![
                                Menu::new("File", div().w(px(120.0)).h(px(80.0)).child("items")),
                                Menu::new("Edit", div().w(px(120.0)).h(px(80.0)).child("items")),
                            ])
                            .open_menu(Some(1)),
                    ),
            )
        }
    }

    #[gpui::test]
    async fn dropdown_layout_anchors_outside_parent_clip(cx: &mut TestAppContext) {
        let (_host, cx) = setup_test_window(cx, |_window, _cx| ClipEscapeHarness);

        let clip = cx.get_element("clip-region");
        let dropdown = cx.get_element("menu-bar-dropdown");

        // The bar fills the clip vertically (28pt); the dropdown anchors
        // below the bar, so a correctly-escaped overlay has its top at
        // or below the clip's bottom edge.
        assert!(
            dropdown.bounds.top() >= clip.bounds.bottom(),
            "dropdown.top() {:?} should be at or below clip.bottom() {:?}",
            dropdown.bounds.top(),
            clip.bounds.bottom(),
        );
    }

    /// A second menu's dropdown must anchor at a strictly greater
    /// horizontal offset than the first menu's. This locks the
    /// `.offset(point(open_offset, 0))` behaviour: without it, all
    /// menus would collapse to the bar's left edge regardless of
    /// which title was active.
    struct OffsetPreservedHarness {
        open_idx: Option<usize>,
    }

    impl Render for OffsetPreservedHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div().pt(px(40.0)).pl(px(20.0)).child(
                MenuBar::new("bar")
                    .menus(vec![
                        Menu::new("File", div().w(px(120.0)).h(px(80.0)).child("file")),
                        Menu::new("Edit", div().w(px(120.0)).h(px(80.0)).child("edit")),
                    ])
                    .open_menu(self.open_idx),
            )
        }
    }

    #[gpui::test]
    async fn dropdown_horizontal_offset_tracks_active_title(cx: &mut TestAppContext) {
        // Render with the first menu open, capture the dropdown's left
        // edge, then swap to the second menu and assert the dropdown
        // has moved strictly right.
        let (host, cx) = setup_test_window(cx, |_window, _cx| OffsetPreservedHarness {
            open_idx: Some(0),
        });

        let first_left = cx.get_element("menu-bar-dropdown").bounds.left();

        host.update(cx, |host, cx| {
            host.open_idx = Some(1);
            cx.notify();
        });

        let second_left = cx.get_element("menu-bar-dropdown").bounds.left();

        assert!(
            second_left > first_left,
            "dropdown under 'Edit' ({second_left:?}) must be to the right of 'File' ({first_left:?})",
        );
    }
}
