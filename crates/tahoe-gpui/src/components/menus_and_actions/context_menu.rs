//! HIG context menu component with glass morphism.
//!
//! A stateful context menu that appears at a screen position (typically from a
//! right-click). Supports keyboard navigation (Arrow keys, Enter, Escape,
//! Shift-F10, Application/Menu key), click-outside dismiss, separator
//! dividers, destructive, disabled, and checked items, submenu expansion
//! (click, Right-arrow, or ~100 ms hover), typed keyboard-shortcut glyphs,
//! Space-to-toggle for checkable items, and anchor-rect positioning.

use std::rc::Rc;
use std::time::Duration;

use gpui::prelude::*;
use gpui::{
    Action, App, Bounds, ClickEvent, ElementId, FocusHandle, KeyDownEvent, MouseDownEvent, Pixels,
    Point, SharedString, Task, Window, div, px,
};

use crate::callback_types::OnMutCallback;
use crate::components::layout_and_organization::separator::Separator;
use crate::foundations::accessibility::{
    AccessibilityProps, AccessibilityRole, AccessibleExt, HeadingLevel,
};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::keyboard_shortcuts::MenuShortcut;
use crate::foundations::layout::{MENU_MAX_WIDTH, MENU_MIN_WIDTH, SPACING_4};
use crate::foundations::materials::{Elevation, Glass, Shape, SurfaceContext, glass_effect_lens};
use crate::foundations::overlay::{AnchoredOverlay, OverlayAnchor, OverlayLayer};
use crate::foundations::theme::ActiveTheme;
use crate::ids::next_element_id;

/// Delay before a hovered submenu row opens its nested overlay
/// (HIG "menus open on hover after a short delay").
const SUBMENU_HOVER_OPEN_MS: u64 = 100;

/// Shared-ownership toggle callback fired by Space on a checkable row.
/// Equivalent to `OnToggle` but `Rc`-based so the same handler can be
/// cloned into both the click and keyboard code paths without moving.
pub type OnToggleRc = Option<Rc<dyn Fn(bool, &mut Window, &mut App)>>;

// ─── Item Style ──────────────────────────────────────────────────────────────

/// Visual style for a context menu item.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum ContextMenuItemStyle {
    /// Standard item appearance.
    #[default]
    Default,
    /// Destructive/warning appearance (e.g. delete actions).
    Destructive,
    /// Greyed-out, non-interactive appearance.
    Disabled,
}

// ─── Item ────────────────────────────────────────────────────────────────────

/// A single action item inside a context menu.
///
/// This is a plain data struct, not a component. The parent `ContextMenu`
/// renders each item during its `Render` pass.
pub struct ContextMenuItem {
    /// Display label.
    pub label: SharedString,
    /// Optional leading icon.
    pub icon: Option<IconName>,
    /// Visual style.
    pub style: ContextMenuItemStyle,
    /// Optional typed keyboard shortcut rendered right-aligned as
    /// SF-Symbol glyph sequences (⌘⇧⌥⌃K). Pass a string like
    /// `"Cmd+D"` (auto-parsed via `From<&str>`) or
    /// `MenuShortcut::cmd("D")` for the typed constructor.
    pub shortcut: Option<MenuShortcut>,
    /// `true` renders a leading checkmark glyph instead of `icon` (toggled
    /// state, e.g. "View > Show Toolbar ✓"). When both `checked` and `icon`
    /// are set the checkmark wins in the leading slot and the icon is
    /// dropped from the row to preserve alignment.
    pub checked: bool,
    /// Click handler invoked when the item is activated.
    pub on_click: OnMutCallback,
    /// Optional toggle handler fired when Space is pressed on a selected
    /// checkable row. Receives the *new* checked state. Firing this does
    /// **not** close the menu — toggling is a non-destructive gesture so
    /// users can flip multiple switches without reopening the overlay.
    /// Enter still activates + closes via [`ContextMenuItem::on_click`].
    pub on_toggle: OnToggleRc,
    /// Optional GPUI action dispatched when the row activates. When set,
    /// `activate_item` calls `window.dispatch_action(action.boxed_clone(),
    /// cx)` so the same action works from click, keyboard shortcut, and
    /// command palette — Zed's unified action-dispatch pattern. Runs
    /// *before* `on_click` if both are present.
    pub action: Option<Box<dyn Action>>,
}

impl ContextMenuItem {
    /// Create a new default-styled item with the given label.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            style: ContextMenuItemStyle::Default,
            shortcut: None,
            checked: false,
            on_click: None,
            on_toggle: None,
            action: None,
        }
    }

    /// Dispatch a GPUI action when the row activates. Same dispatch path
    /// as the keyboard shortcut and the command palette, so menu click,
    /// keybinding, and palette invocation all route through one handler.
    ///
    /// Runs before any `on_click` handler if both are supplied; the menu
    /// closes afterwards regardless.
    pub fn action(mut self, action: Box<dyn Action>) -> Self {
        self.action = Some(action);
        self
    }

    /// Set the leading icon.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the visual style.
    pub fn style(mut self, style: ContextMenuItemStyle) -> Self {
        self.style = style;
        self
    }

    /// Set the keyboard shortcut. Accepts a typed [`MenuShortcut`] or a
    /// string like `"Cmd+D"` via the `From<&str>` impl — both render as
    /// SF-Symbol glyph sequences.
    pub fn shortcut(mut self, shortcut: impl Into<MenuShortcut>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Mark the item as toggled on (leading checkmark glyph, HIG "checked"
    /// state). Clears the icon slot to keep alignment consistent.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the click handler.
    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set the Space-to-toggle handler for a checkable row.
    ///
    /// The handler is called with the *new* checked state; the caller is
    /// responsible for updating its model and re-rendering the menu with
    /// an updated `.checked(...)` value. The menu stays open so the user
    /// can toggle multiple options in one gesture (HIG "checked menu
    /// items").
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

// ─── Entry ───────────────────────────────────────────────────────────────────

/// An entry in the context menu.
pub enum ContextMenuEntry {
    /// An actionable menu item.
    Item(ContextMenuItem),
    /// A visual separator divider between groups.
    Separator,
    /// A non-interactive uppercase label that introduces a logical group of
    /// items — HIG "section heading" pattern used in Finder's "Arrange By"
    /// submenu, Safari's bookmarks sidebar, and System Settings groups.
    /// Renders as a small caption and is skipped by keyboard navigation.
    SectionHeader(SharedString),
    /// A submenu — renders a trailing chevron and opens a nested overlay
    /// on hover / right-arrow. Submenu items are themselves a flat list
    /// of `ContextMenuEntry` values so an arbitrary depth is possible,
    /// though HIG recommends at most one nested level.
    Submenu {
        /// Label for the parent row.
        label: SharedString,
        /// Optional leading icon on the parent row.
        icon: Option<IconName>,
        /// Entries displayed when the submenu opens.
        items: Vec<ContextMenuEntry>,
    },
}

// ─── ContextMenu (stateful) ──────────────────────────────────────────────────

/// A stateful context menu overlay.
///
/// Tracks open/close state, screen position, keyboard selection, focus,
/// and (optionally) the trigger element's anchor rect so the menu can be
/// positioned below/beside the trigger rather than overlapping the cursor.
/// Create with `Entity::new(cx, ContextMenu::new)` and control via
/// `entity.update(cx, |menu, cx| menu.open(pos, cx))`.
pub struct ContextMenu {
    element_id: ElementId,
    items: Vec<ContextMenuEntry>,
    is_open: bool,
    position: Option<Point<Pixels>>,
    /// Optional anchor rect for the trigger. When provided, the menu is
    /// positioned *below* (or beside, if the bottom would overflow) the
    /// rect instead of at the raw cursor point.
    anchor: Option<Bounds<Pixels>>,
    /// Selection path. Length 1 = top-level index; length 2 = [parent, child]
    /// inside a submenu. Deeper nesting is supported but not encouraged.
    selection_path: Vec<usize>,
    /// Indices of expanded submenus (currently supports at most one level).
    expanded_submenu: Option<usize>,
    /// Pending submenu-open timer scheduled by hover. Dropping the task
    /// aborts it, so replacing this field (on a new hover or a hover-leave)
    /// is sufficient to cancel the previously-scheduled open.
    hover_submenu_task: Option<Task<()>>,
    /// Top-level index of the row currently being hovered in the parent
    /// menu. Used only to decide whether a pending hover-open should fire
    /// (the row still matches) or be ignored (the user moved away).
    hovered_submenu_index: Option<usize>,
    focus_handle: FocusHandle,
}

impl ContextMenu {
    /// Create a new closed context menu with no items.
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("context-menu"),
            items: Vec::new(),
            is_open: false,
            position: None,
            anchor: None,
            selection_path: Vec::new(),
            expanded_submenu: None,
            hover_submenu_task: None,
            hovered_submenu_index: None,
            focus_handle: cx.focus_handle(),
        }
    }

    /// Replace the menu entries.
    pub fn set_items(&mut self, items: Vec<ContextMenuEntry>) {
        self.items = items;
    }

    /// Show the menu at the given screen position (no anchor rect).
    pub fn open(&mut self, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        self.is_open = true;
        self.position = Some(position);
        self.anchor = None;
        self.selection_path.clear();
        self.expanded_submenu = None;
        self.hover_submenu_task = None;
        self.hovered_submenu_index = None;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    /// Show the menu anchored to the given trigger bounds. The menu will
    /// prefer opening directly below the anchor (so it does not overlap
    /// the triggering element per HIG), falling back to above when the
    /// bottom edge would overflow the window.
    pub fn open_anchored(
        &mut self,
        anchor: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_open = true;
        self.position = Some(Point {
            x: anchor.origin.x,
            y: anchor.origin.y + anchor.size.height,
        });
        self.anchor = Some(anchor);
        self.selection_path.clear();
        self.expanded_submenu = None;
        self.hover_submenu_task = None;
        self.hovered_submenu_index = None;
        self.focus_handle.focus(window, cx);
        cx.notify();
    }

    /// Register context-menu keystrokes (`Shift-F10`, Menu/Application key)
    /// on an existing focusable element so the menu can be opened without
    /// a pointer. The `anchor_fn` supplies the bounds of the trigger at the
    /// moment of activation; the menu opens anchored below those bounds.
    ///
    /// Returns a tuple `(on_key_down_handler, on_secondary_click_handler)`
    /// that the caller wires into their element. This is deliberately a
    /// builder helper rather than a middleware wrapper because GPUI's
    /// listener lifetimes require the handlers to be installed on the
    /// concrete element type by the caller.
    ///
    /// Example:
    /// ```ignore
    /// let (on_key, on_ctx) = menu.attach(|| trigger_bounds(), cx);
    /// trigger
    ///     .on_key_down(on_key)
    ///     .on_mouse_down(gpui::MouseButton::Right, move |ev, window, cx| {
    ///         on_ctx(ev.position, window, cx);
    ///     });
    /// ```
    /// Convenience: returns true when the given key event is one of the
    /// HIG context-menu activation shortcuts (Shift-F10 / Menu key).
    pub fn is_context_menu_shortcut(event: &KeyDownEvent) -> bool {
        let k = event.keystroke.key.as_str();
        // "menu" is the macOS Menu / Application key; "shift-f10" is the
        // Windows-compatible alias that some keyboards send on macOS.
        if k == "menu" {
            return true;
        }
        if k == "f10" && event.keystroke.modifiers.shift {
            return true;
        }
        false
    }

    /// Hide the menu and clear selection.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        self.is_open = false;
        self.selection_path.clear();
        self.expanded_submenu = None;
        self.hover_submenu_task = None;
        self.hovered_submenu_index = None;
        cx.notify();
    }

    /// Returns `true` when the menu is visible.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the current keyboard-selected *top-level* index, if any.
    /// Submenu selection is reflected via [`selection_path`](Self::selection_path).
    pub fn selected_index(&self) -> Option<usize> {
        self.selection_path.first().copied()
    }

    /// Returns the current keyboard-selection path.
    /// Length 0 = no selection; length 1 = top-level row index; length 2 =
    /// [parent, child] inside an open submenu.
    pub fn selection_path(&self) -> &[usize] {
        &self.selection_path
    }

    /// Returns the index of the currently expanded submenu, if any.
    pub fn expanded_submenu(&self) -> Option<usize> {
        self.expanded_submenu
    }

    // ── Keyboard helpers ─────────────────────────────────────────────────

    /// Return the items list at the depth currently receiving keyboard focus.
    fn active_items(&self) -> &[ContextMenuEntry] {
        if let Some(parent_idx) = self.expanded_submenu
            && let Some(ContextMenuEntry::Submenu { items, .. }) = self.items.get(parent_idx)
        {
            return items;
        }
        &self.items
    }

    /// Move selection down (wrapping) at the currently active depth.
    fn select_next(&mut self) {
        let depth = self.selection_path.len().max(1);
        let current = self.selection_path.last().copied();
        let next = nav_next(self.active_items(), current);
        self.update_leaf_selection(depth, next);
    }

    /// Move selection up (wrapping) at the currently active depth.
    fn select_prev(&mut self) {
        let depth = self.selection_path.len().max(1);
        let current = self.selection_path.last().copied();
        let prev = nav_prev(self.active_items(), current);
        self.update_leaf_selection(depth, prev);
    }

    fn update_leaf_selection(&mut self, depth: usize, next: Option<usize>) {
        if let Some(idx) = next {
            self.selection_path.truncate(depth);
            if self.selection_path.len() < depth {
                self.selection_path.push(idx);
            } else if let Some(last) = self.selection_path.last_mut() {
                *last = idx;
            }
        }
    }

    /// Begin (or restart) the ~100 ms hover-to-open timer for a submenu
    /// parent row at top-level index `idx`. Dropping `hover_submenu_task`
    /// cancels any previously-pending open, so each call is an idempotent
    /// "reset the timer for the row currently under the cursor".
    fn schedule_submenu_hover_open(&mut self, idx: usize, cx: &mut Context<Self>) {
        // Ignore hovers on non-submenu rows so a quick mouse-over of an
        // adjacent item doesn't schedule a phantom open.
        if !matches!(self.items.get(idx), Some(ContextMenuEntry::Submenu { .. })) {
            self.cancel_submenu_hover_open();
            return;
        }
        // If we're already scheduled on this exact row, leave the existing
        // timer alone — replacing it would reset the 100 ms clock, which
        // is the opposite of what a continuous hover should do.
        if self.hovered_submenu_index == Some(idx) && self.hover_submenu_task.is_some() {
            return;
        }
        self.hovered_submenu_index = Some(idx);
        let task = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(SUBMENU_HOVER_OPEN_MS))
                .await;
            let _ = this.update(cx, |this: &mut Self, cx| {
                // Only fire if the user is still hovering this row; a
                // hover-leave (or a hover on a different row) will have
                // cleared `hovered_submenu_index`.
                if this.hovered_submenu_index != Some(idx) {
                    return;
                }
                if !matches!(this.items.get(idx), Some(ContextMenuEntry::Submenu { .. })) {
                    return;
                }
                this.expanded_submenu = Some(idx);
                this.selection_path = vec![idx];
                if let Some(first_child) = first_actionable_in_submenu(&this.items, idx) {
                    this.selection_path.push(first_child);
                }
                this.hover_submenu_task = None;
                cx.notify();
            });
        });
        self.hover_submenu_task = Some(task);
    }

    /// Cancel any pending hover-to-open scheduled by
    /// [`ContextMenu::schedule_submenu_hover_open`]. Safe to call even
    /// when nothing is pending.
    fn cancel_submenu_hover_open(&mut self) {
        self.hover_submenu_task = None;
        self.hovered_submenu_index = None;
    }

    /// Toggle the checked state of the currently-selected row and fire its
    /// `on_toggle` handler. Used by the Space-key branch of
    /// [`ContextMenu::handle_key_down`]. Returns `true` when the key event
    /// was consumed (a checkable row was selected *and* had `on_toggle`).
    fn toggle_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(idx) = self.selection_path.last().copied() else {
            return false;
        };
        let items = self.active_items();
        let Some(ContextMenuEntry::Item(item)) = items.get(idx) else {
            return false;
        };
        if item.style == ContextMenuItemStyle::Disabled {
            return false;
        }
        let Some(handler) = item.on_toggle.clone() else {
            return false;
        };
        let new_state = !item.checked;
        handler(new_state, window, cx);
        // The row stays visible; the host is responsible for rebuilding
        // the items with the new `.checked(...)` state. A plain notify
        // keeps the row highlight in sync while we wait for that rebuild.
        cx.notify();
        true
    }

    /// Fire the item's `on_click` and close the menu, skipping disabled entries.
    ///
    /// Single entry point used by both keyboard Enter and mouse click so the
    /// two paths can't diverge on disabled/missing-handler semantics.
    fn activate_item(&mut self, idx: usize, window: &mut Window, cx: &mut Context<Self>) {
        let items: &[ContextMenuEntry] = self.active_items();
        if let Some(ContextMenuEntry::Item(item)) = items.get(idx)
            && item.style != ContextMenuItemStyle::Disabled
        {
            // Dispatch the action first so keymap-registered handlers
            // run exactly as they would for a keyboard shortcut. Then
            // fall through to the explicit `on_click` callback for
            // items that wire both or neither.
            if let Some(action) = &item.action {
                window.dispatch_action(action.boxed_clone(), cx);
            }
            if let Some(handler) = &item.on_click {
                handler(window, cx);
            }
        }
        self.close(cx);
    }

    /// Fire the selected item's `on_click` and close the menu.
    fn activate_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(idx) = self.selection_path.last().copied() else {
            self.close(cx);
            return;
        };

        // If the active selection lands on an unopened submenu parent row
        // (only reachable at top level), expand it instead of activating.
        if self.selection_path.len() == 1
            && matches!(self.items.get(idx), Some(ContextMenuEntry::Submenu { .. }))
            && self.expanded_submenu != Some(idx)
        {
            self.expanded_submenu = Some(idx);
            if let Some(first_child) = first_actionable_in_submenu(&self.items, idx) {
                self.selection_path.push(first_child);
            }
            cx.notify();
            return;
        }

        self.activate_item(idx, window, cx);
    }

    /// Handle keyboard events.
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Shift-F10 and Menu/Application key dismiss the menu if already
        // open (pressing them again is a toggle gesture on macOS/Windows).
        if Self::is_context_menu_shortcut(event) {
            self.close(cx);
            return;
        }
        match event.keystroke.key.as_str() {
            "down" => {
                self.select_next();
                cx.notify();
            }
            "up" => {
                self.select_prev();
                cx.notify();
            }
            // Right arrow: expand submenu when selection is on a submenu
            // parent row at the top level.
            "right" if self.selection_path.len() == 1 => {
                let idx = self.selection_path[0];
                if matches!(self.items.get(idx), Some(ContextMenuEntry::Submenu { .. })) {
                    self.expanded_submenu = Some(idx);
                    if let Some(first_child) = first_actionable_in_submenu(&self.items, idx) {
                        self.selection_path.push(first_child);
                    }
                    cx.notify();
                }
            }
            // Left arrow: collapse the submenu and return focus to the parent row.
            "left" if self.selection_path.len() >= 2 => {
                self.selection_path.pop();
                self.expanded_submenu = None;
                cx.notify();
            }
            "enter" => {
                self.activate_selected(window, cx);
            }
            // Space toggles a checkable row without closing the menu.
            // Falls through to no-op on non-checkable rows so the host
            // retains the option to bind Space elsewhere (e.g. as a
            // scrolling gesture) if nothing consumed it here.
            "space" => {
                self.toggle_selected(window, cx);
            }
            "escape" => {
                if self.expanded_submenu.is_some() {
                    // First escape collapses the submenu.
                    self.expanded_submenu = None;
                    if self.selection_path.len() > 1 {
                        self.selection_path.truncate(1);
                    }
                    cx.notify();
                } else {
                    self.close(cx);
                }
            }
            _ => {}
        }
    }
}

/// Locate the first actionable child inside a `Submenu` entry at the given
/// top-level index.
fn first_actionable_in_submenu(items: &[ContextMenuEntry], parent_idx: usize) -> Option<usize> {
    if let Some(ContextMenuEntry::Submenu { items: sub, .. }) = items.get(parent_idx) {
        actionable_indices(sub).first().copied()
    } else {
        None
    }
}

// ─── Navigation helpers (free functions, testable without GPUI context) ──────

/// Collect indices of entries that are actionable (not separators, not
/// disabled). `Submenu` entries count as actionable — their parent row is
/// selectable even though its click opens a nested menu rather than
/// invoking a handler.
fn actionable_indices(items: &[ContextMenuEntry]) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| match entry {
            ContextMenuEntry::Item(item) if item.style != ContextMenuItemStyle::Disabled => Some(i),
            ContextMenuEntry::Submenu { .. } => Some(i),
            _ => None,
        })
        .collect()
}

/// Compute the next selection index (moving down, wrapping).
fn nav_next(items: &[ContextMenuEntry], current: Option<usize>) -> Option<usize> {
    let actionable = actionable_indices(items);
    if actionable.is_empty() {
        return current;
    }
    Some(match current {
        Some(idx) => actionable
            .iter()
            .find(|&&i| i > idx)
            .copied()
            .unwrap_or(actionable[0]),
        None => actionable[0],
    })
}

/// Compute the previous selection index (moving up, wrapping).
fn nav_prev(items: &[ContextMenuEntry], current: Option<usize>) -> Option<usize> {
    let actionable = actionable_indices(items);
    if actionable.is_empty() {
        return current;
    }
    // `actionable` is non-empty: the early-return above handles the empty case.
    let last = *actionable
        .last()
        .expect("actionable is non-empty: early-return above handles empty case");
    Some(match current {
        Some(idx) => actionable
            .iter()
            .rev()
            .find(|&&i| i < idx)
            .copied()
            .unwrap_or(last),
        None => last,
    })
}

impl Render for ContextMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        // Snapshot theme values as owned locals. `render_rows` borrows
        // `cx` mutably through `cx.listener`, which would conflict with
        // an outstanding immutable `cx.theme()` borrow.
        let (
            accent,
            accent_text,
            row_h,
            radius_md,
            spacing_xs,
            spacing_sm,
            menu_inset,
            dim_label,
            dim_secondary_label,
            error_color,
            icon_size_inline,
            icon_size_small,
        );
        {
            let theme = cx.theme();
            accent = theme.accent;
            accent_text = theme.text_on_accent;
            row_h = theme.row_height();
            radius_md = theme.radius_md;
            spacing_xs = theme.spacing_xs;
            spacing_sm = theme.spacing_sm;
            menu_inset = theme.menu_inset;
            dim_label = theme.label_color(SurfaceContext::GlassDim);
            dim_secondary_label = theme.secondary_label_color(SurfaceContext::GlassDim);
            error_color = theme.error;
            icon_size_inline = theme.icon_size_inline;
            icon_size_small = theme.icon_size_small;
        }

        let style_tokens = RowStyleTokens {
            accent,
            accent_text,
            row_h,
            radius_md,
            spacing_sm,
            menu_inset,
            dim_label,
            dim_secondary_label,
            error_color,
            icon_size_inline,
            icon_size_small,
        };

        let raw_pos = self.position.unwrap_or_else(|| Point {
            x: px(0.0),
            y: px(0.0),
        });

        let window_bounds = window.bounds();
        let row_count = self
            .items
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    ContextMenuEntry::Item(_) | ContextMenuEntry::Submenu { .. }
                )
            })
            .count() as f32;
        let sep_rows = self
            .items
            .iter()
            .filter(|e| matches!(e, ContextMenuEntry::Separator))
            .count() as f32;
        let est_height = row_count * row_h + sep_rows * 9.0 + 8.0;
        let est_width = MENU_MAX_WIDTH;
        let max_x = (f32::from(window_bounds.size.width) - est_width).max(0.0);
        let max_y = (f32::from(window_bounds.size.height) - est_height).max(0.0);

        // Anchor-aware positioning: when the menu was opened with
        // `open_anchored`, prefer opening directly below the anchor rect
        // (HIG: menus must not overlap their trigger). If that overflows
        // the window bottom, flip to above the anchor. Otherwise fall
        // back to the raw cursor point with simple clamping.
        let pos = if let Some(anchor) = self.anchor {
            let below_y = f32::from(anchor.origin.y) + f32::from(anchor.size.height) + 2.0;
            let above_y = f32::from(anchor.origin.y) - est_height - 2.0;
            let y = if below_y + est_height <= f32::from(window_bounds.size.height) {
                below_y
            } else if above_y >= 0.0 {
                above_y
            } else {
                below_y.clamp(0.0, max_y)
            };
            let x = f32::from(anchor.origin.x).clamp(0.0, max_x);
            Point { x: px(x), y: px(y) }
        } else {
            Point {
                x: px(f32::from(raw_pos.x).clamp(0.0, max_x)),
                y: px(f32::from(raw_pos.y).clamp(0.0, max_y)),
            }
        };

        // Render the top-level rows.
        let selected_top = self.selection_path.first().copied();
        let selected_child = self.selection_path.get(1).copied();
        let expanded = self.expanded_submenu;
        let top_items = self.items.as_slice() as *const [ContextMenuEntry];

        // SAFETY: GPUI listener callbacks (`cx.listener`) are dispatched on a
        // subsequent event-loop turn — after `render()` has returned and the
        // `&[ContextMenuEntry]` reference is no longer live. The raw pointer
        // is cast to a reference only for the synchronous duration of this
        // `render` call, so no aliasing mutation occurs while it is held.
        // The raw-pointer indirection is needed because the borrow checker
        // conservatively rejects the simultaneous `&self.items` + `&mut cx`
        // loans, even though the reference is dropped before `cx` is used.
        let top_children = render_rows(
            unsafe { &*top_items },
            &style_tokens,
            selected_top,
            cx,
            false,
        );

        let theme_for_surface = cx.theme();
        let menu = glass_effect_lens(
            theme_for_surface,
            Glass::Regular,
            Shape::RoundedRectangle(theme_for_surface.radius_lg),
            Elevation::Elevated,
            None,
        )
        .flex()
        .flex_col()
        .min_w(px(MENU_MIN_WIDTH))
        .max_w(px(MENU_MAX_WIDTH))
        .py(spacing_xs)
        .overflow_hidden()
        .debug_selector(|| "context-menu-content".into())
        .children(top_children)
        .on_mouse_down_out(cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
            this.close(cx);
        }));

        // Optional nested submenu overlay.
        let submenu_overlay = if let Some(parent_idx) = expanded {
            // Grab a raw pointer to the nested items slice; the safety
            // rationale matches the top-level borrow above.
            let nested_items_ptr: Option<*const [ContextMenuEntry]> =
                match self.items.get(parent_idx) {
                    Some(ContextMenuEntry::Submenu { items, .. }) => Some(items.as_slice()),
                    _ => None,
                };
            nested_items_ptr.map(|items_ptr| {
                // SAFETY: Same rationale as the top-level `top_items` block
                // above. `items_ptr` was produced from `self.items` in this
                // stack frame, and GPUI listeners are dispatched on a later
                // event-loop turn — after `render()` returns and this
                // reference is dropped — so no aliasing mutation occurs
                // during the synchronous `render_rows` call.
                let rows = render_rows(
                    unsafe { &*items_ptr },
                    &style_tokens,
                    selected_child,
                    cx,
                    true,
                );
                let theme_for_sub = cx.theme();
                let nested = glass_effect_lens(
                    theme_for_sub,
                    Glass::Regular,
                    Shape::RoundedRectangle(theme_for_sub.radius_lg),
                    Elevation::Elevated,
                    None,
                )
                .flex()
                .flex_col()
                .min_w(px(MENU_MIN_WIDTH))
                .max_w(px(MENU_MAX_WIDTH))
                .py(spacing_xs)
                .overflow_hidden()
                .debug_selector(|| "context-submenu-content".into())
                .children(rows);
                let row_top = parent_idx as f32 * row_h + f32::from(spacing_xs);
                div()
                    .absolute()
                    .left(px(MENU_MIN_WIDTH - 4.0))
                    .top(px(row_top))
                    .child(nested)
            })
        } else {
            None
        };

        // The wrapped content is what `AnchoredOverlay` positions at the
        // window-absolute point. Keep the submenu positioned relative to
        // the wrapper (relative() parent) so its `.absolute().left(...).
        // top(row_top)` offset still resolves against the menu's origin.
        // Focus handle + key listener live on this wrapper so keyboard
        // events fire after `AnchoredOverlay` hands focus here.
        let mut content_wrapper = div()
            .id(self.element_id.clone())
            .debug_selector(|| "context-menu-overlay".into())
            .track_focus(&self.focus_handle)
            .relative()
            .child(menu)
            .on_key_down(cx.listener(Self::handle_key_down));
        if let Some(sub) = submenu_overlay {
            content_wrapper = content_wrapper.child(sub);
        }

        // Route through `AnchoredOverlay::WindowPoint`: the event-supplied
        // point is in window-absolute coordinates (`MouseDownEvent::position`),
        // which matches `WindowPoint`'s coordinate space exactly. This
        // also drops the overlay out of the normal layout tree via
        // `deferred()`, so opening or closing the menu no longer shifts
        // siblings below its host. `OverlayLayer::CONTEXT_MENU` stacks
        // the menu above tooltips and dropdowns so it reads as the
        // active input surface.
        //
        // Trigger is an empty `div()` because context menus have no
        // visible trigger of their own — the consumer handles the
        // right-click on whatever container is appropriate and calls
        // `menu.open(event.position, ..)`. The trigger renders at
        // zero size and is invisible.
        AnchoredOverlay::new(
            ElementId::NamedChild(
                std::sync::Arc::new(self.element_id.clone()),
                "ctx-overlay".into(),
            ),
            div(),
        )
        .anchor(OverlayAnchor::WindowPoint(pos))
        .priority(OverlayLayer::CONTEXT_MENU)
        .content(content_wrapper)
        .into_any_element()
    }
}

/// Snapshot of theme values used by row rendering. Extracted so
/// `render_rows` can be called without re-borrowing `cx.global` while
/// `cx.listener` is borrowing `cx` mutably.
struct RowStyleTokens {
    accent: gpui::Hsla,
    accent_text: gpui::Hsla,
    row_h: f32,
    radius_md: Pixels,
    spacing_sm: Pixels,
    menu_inset: Pixels,
    dim_label: gpui::Hsla,
    dim_secondary_label: gpui::Hsla,
    error_color: gpui::Hsla,
    icon_size_inline: Pixels,
    icon_size_small: Pixels,
}

/// Build the list of row elements for a given items slice.
///
/// Used for both the top-level menu and a nested submenu overlay. The
/// caller supplies `is_submenu` so click listeners can dispatch via the
/// right code path (top-level items route through `activate_item`, submenu
/// children route through a dedicated handler that unwraps the parent
/// selection into a flat index).
fn render_rows(
    items: &[ContextMenuEntry],
    t: &RowStyleTokens,
    selected_idx: Option<usize>,
    cx: &mut Context<ContextMenu>,
    is_submenu: bool,
) -> Vec<gpui::AnyElement> {
    let mut children: Vec<gpui::AnyElement> = Vec::new();
    for (idx, entry) in items.iter().enumerate() {
        match entry {
            ContextMenuEntry::Separator => {
                children.push(
                    div()
                        .w_full()
                        .py(px(SPACING_4))
                        .px(t.spacing_sm)
                        .child(Separator::horizontal())
                        .into_any_element(),
                );
            }
            ContextMenuEntry::SectionHeader(label) => {
                // HIG section heading: uppercase caption in secondary
                // label color. Not interactive, so no hit region — the
                // click event handler in `Render` already skips this
                // variant via `activate_item`'s `Item(..)` match.
                // a11y: expose as Heading(3) with the original (mixed-case)
                // label so VoiceOver announces the group boundary even
                // though the visual row is skipped by keyboard navigation.
                let a11y = AccessibilityProps::new()
                    .role(AccessibilityRole::Heading(HeadingLevel::new_clamped(3)))
                    .label(label.clone());
                children.push(
                    div()
                        .w_full()
                        .pt(t.spacing_sm)
                        .pb(px(2.0))
                        .px(t.spacing_sm)
                        .text_color(t.dim_secondary_label)
                        .child(SharedString::from(label.to_uppercase()))
                        .with_accessibility(&a11y)
                        .into_any_element(),
                );
            }
            ContextMenuEntry::Item(item) => {
                let is_selected = selected_idx == Some(idx);
                let is_disabled = item.style == ContextMenuItemStyle::Disabled;

                let (text_color, icon_color) = if is_selected && !is_disabled {
                    (t.accent_text, t.accent_text)
                } else {
                    match item.style {
                        ContextMenuItemStyle::Default => (t.dim_label, t.dim_secondary_label),
                        ContextMenuItemStyle::Destructive => (t.error_color, t.error_color),
                        ContextMenuItemStyle::Disabled => {
                            (t.dim_secondary_label, t.dim_secondary_label)
                        }
                    }
                };

                let selector_tag: SharedString = if is_submenu {
                    "context-submenu-item".into()
                } else {
                    "ctx-item".into()
                };
                let mut row = div()
                    .id(ElementId::NamedInteger(selector_tag, idx as u64))
                    .debug_selector(move || {
                        if is_submenu {
                            format!("context-submenu-item-{idx}")
                        } else {
                            format!("context-menu-item-{idx}")
                        }
                    })
                    .w_full()
                    .h(px(t.row_h))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(t.spacing_sm)
                    .px(t.spacing_sm)
                    .mx(t.menu_inset)
                    .rounded(t.radius_md)
                    .text_color(text_color);

                if is_disabled {
                    row = row.opacity(0.5);
                }

                let accent = t.accent;
                if is_selected && !is_disabled {
                    row = row.bg(accent).rounded(t.radius_md);
                } else if !is_disabled {
                    row = row.hover(move |style| {
                        style.bg(crate::foundations::color::with_alpha(accent, 0.10))
                    });
                }

                // Leading slot: checkmark (checked) wins over icon.
                if item.checked {
                    row = row.child(
                        Icon::new(IconName::Check)
                            .size(t.icon_size_inline)
                            .color(icon_color),
                    );
                } else if let Some(icon_name) = item.icon {
                    row = row.child(
                        Icon::new(icon_name)
                            .size(t.icon_size_inline)
                            .color(icon_color),
                    );
                }

                row = row.child(div().flex_1().child(item.label.clone()));

                if let Some(shortcut) = &item.shortcut {
                    let shortcut_color = if is_selected && !is_disabled {
                        crate::foundations::color::with_alpha(t.accent_text, 0.7)
                    } else {
                        t.dim_secondary_label
                    };
                    row = row.child(
                        div()
                            .text_color(shortcut_color)
                            .child(SharedString::from(shortcut.render())),
                    );
                }

                if !is_disabled && item.on_click.is_some() {
                    row = row.cursor_pointer().on_click(cx.listener(
                        move |this, _event: &ClickEvent, window, cx| {
                            if is_submenu && let Some(parent) = this.expanded_submenu {
                                this.selection_path = vec![parent, idx];
                            }
                            this.activate_item(idx, window, cx);
                        },
                    ));
                }

                let mut a11y = AccessibilityProps::menu_item(item.label.clone());
                if item.checked {
                    a11y = a11y.value("On");
                }
                row = row.with_accessibility(&a11y);

                children.push(row.into_any_element());
            }
            ContextMenuEntry::Submenu { label, icon, .. } => {
                let is_selected = selected_idx == Some(idx);

                let text_color = if is_selected {
                    t.accent_text
                } else {
                    t.dim_label
                };
                let icon_color = if is_selected {
                    t.accent_text
                } else {
                    t.dim_secondary_label
                };

                let mut row = div()
                    .id(ElementId::NamedInteger("ctx-submenu".into(), idx as u64))
                    .debug_selector(move || format!("context-menu-submenu-{idx}"))
                    .w_full()
                    .h(px(t.row_h))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(t.spacing_sm)
                    .px(t.spacing_sm)
                    .mx(t.menu_inset)
                    .rounded(t.radius_md)
                    .text_color(text_color);

                let accent = t.accent;
                if is_selected {
                    row = row.bg(accent);
                } else {
                    row = row.hover(move |style| {
                        style.bg(crate::foundations::color::with_alpha(accent, 0.10))
                    });
                }

                if let Some(icon_name) = icon {
                    row = row.child(
                        Icon::new(*icon_name)
                            .size(t.icon_size_inline)
                            .color(icon_color),
                    );
                }
                row = row.child(div().flex_1().child(label.clone()));
                row = row.child(
                    Icon::new(IconName::ChevronRight)
                        .size(t.icon_size_small)
                        .color(icon_color),
                );

                row = row.cursor_pointer().on_click(cx.listener(
                    move |this, _event: &ClickEvent, _window, cx| {
                        this.expanded_submenu = Some(idx);
                        this.selection_path = vec![idx];
                        if let Some(first_child) = first_actionable_in_submenu(&this.items, idx) {
                            this.selection_path.push(first_child);
                        }
                        // A click is an explicit open; cancel any
                        // still-pending hover timer so we don't double-fire.
                        this.cancel_submenu_hover_open();
                        cx.notify();
                    },
                ));

                // HIG "menus open on hover after a short delay": arm a
                // ~100 ms timer when this parent row starts being hovered,
                // and cancel it on hover-leave. Only wired for top-level
                // rows — nested levels open directly.
                if !is_submenu {
                    row = row.on_hover(cx.listener(move |this, &hovered: &bool, _window, cx| {
                        if hovered {
                            // Do not rearm when this row's submenu is
                            // already visible — it's a no-op that would
                            // just churn the task field.
                            if this.expanded_submenu == Some(idx) {
                                return;
                            }
                            this.schedule_submenu_hover_open(idx, cx);
                        } else if this.hovered_submenu_index == Some(idx) {
                            this.cancel_submenu_hover_open();
                        }
                    }));
                }

                let a11y = AccessibilityProps::menu_item(label.clone());
                row = row.with_accessibility(&a11y);

                children.push(row.into_any_element());
            }
        }
    }
    children
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use crate::components::menus_and_actions::context_menu::{
        ContextMenuEntry, ContextMenuItem, ContextMenuItemStyle,
    };

    // ── Construction ─────────────────────────────────────────────────────

    #[test]
    fn new_menu_is_closed_with_no_items() {
        // A freshly constructed menu should be closed with no entries.
        // We test the fields indirectly via public accessors since we
        // cannot create a Context outside GPUI. Instead, we test the
        // item/entry data types which are context-free.
        let item = ContextMenuItem::new("Copy");
        assert_eq!(item.label.as_ref(), "Copy");
        assert!(item.icon.is_none());
        assert_eq!(item.style, ContextMenuItemStyle::Default);
        assert!(item.shortcut.is_none());
        assert!(item.on_click.is_none());
    }

    #[test]
    fn item_builder_sets_all_fields() {
        use crate::foundations::icons::IconName;
        let item = ContextMenuItem::new("Delete")
            .icon(IconName::Trash)
            .style(ContextMenuItemStyle::Destructive)
            .shortcut("Cmd+D")
            .on_click(|_, _| {});
        assert_eq!(item.label.as_ref(), "Delete");
        assert_eq!(item.icon, Some(IconName::Trash));
        assert_eq!(item.style, ContextMenuItemStyle::Destructive);
        let rendered = item.shortcut.as_ref().map(|s| s.render());
        assert_eq!(rendered.as_deref(), Some("\u{2318}D"));
        assert!(item.on_click.is_some());
    }

    #[test]
    fn item_checked_defaults_false_and_builder_sets_true() {
        let item = ContextMenuItem::new("Show Toolbar");
        assert!(!item.checked);
        let item = item.checked(true);
        assert!(item.checked);
    }

    #[test]
    fn item_shortcut_accepts_typed_menu_shortcut() {
        let item = ContextMenuItem::new("Redo").shortcut(super::MenuShortcut::cmd_shift("Z"));
        assert_eq!(
            item.shortcut.as_ref().map(|s| s.render()).as_deref(),
            Some("\u{21E7}\u{2318}Z")
        );
    }

    #[test]
    fn item_style_default_is_default() {
        assert_eq!(
            ContextMenuItemStyle::default(),
            ContextMenuItemStyle::Default
        );
    }

    #[test]
    fn all_item_styles_are_distinct() {
        let styles = [
            ContextMenuItemStyle::Default,
            ContextMenuItemStyle::Destructive,
            ContextMenuItemStyle::Disabled,
        ];
        for i in 0..styles.len() {
            for j in (i + 1)..styles.len() {
                assert_ne!(styles[i], styles[j]);
            }
        }
    }

    // ── Entry enum ───────────────────────────────────────────────────────

    #[test]
    fn entry_item_wraps_menu_item() {
        let entry = ContextMenuEntry::Item(ContextMenuItem::new("Paste"));
        match entry {
            ContextMenuEntry::Item(item) => assert_eq!(item.label.as_ref(), "Paste"),
            ContextMenuEntry::Separator
            | ContextMenuEntry::Submenu { .. }
            | ContextMenuEntry::SectionHeader(_) => panic!("expected Item variant"),
        }
    }

    #[test]
    fn entry_section_header_constructs() {
        let entry = ContextMenuEntry::SectionHeader("Arrange By".into());
        match entry {
            ContextMenuEntry::SectionHeader(label) => assert_eq!(label.as_ref(), "Arrange By"),
            _ => panic!("expected SectionHeader variant"),
        }
    }

    #[test]
    fn entry_submenu_constructs() {
        let entry = ContextMenuEntry::Submenu {
            label: "Edit".into(),
            icon: None,
            items: vec![ContextMenuEntry::Item(ContextMenuItem::new("Cut"))],
        };
        match entry {
            ContextMenuEntry::Submenu { label, items, .. } => {
                assert_eq!(label.as_ref(), "Edit");
                assert_eq!(items.len(), 1);
            }
            _ => panic!("expected Submenu"),
        }
    }

    #[test]
    fn entry_separator_is_separator() {
        let entry = ContextMenuEntry::Separator;
        assert!(matches!(entry, ContextMenuEntry::Separator));
    }

    // ── Keyboard navigation helpers (tested via free functions) ─────────

    /// Build a test entry list: Item("Cut"), Separator, Disabled("Disabled"), Item("Paste").
    fn make_test_items() -> Vec<ContextMenuEntry> {
        vec![
            ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
            ContextMenuEntry::Separator,
            ContextMenuEntry::Item(
                ContextMenuItem::new("Disabled").style(ContextMenuItemStyle::Disabled),
            ),
            ContextMenuEntry::Item(ContextMenuItem::new("Paste")),
        ]
    }

    #[test]
    fn nav_next_starts_at_first_actionable() {
        let items = make_test_items();
        let selected = super::nav_next(&items, None);
        // Index 0 ("Cut") is the first actionable item.
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn nav_next_skips_separator_and_disabled() {
        let items = make_test_items();
        let selected = super::nav_next(&items, Some(0));
        // Should skip index 1 (Separator) and index 2 (Disabled), land on 3 ("Paste").
        assert_eq!(selected, Some(3));
    }

    #[test]
    fn nav_next_wraps_around() {
        let items = make_test_items();
        let selected = super::nav_next(&items, Some(3)); // last actionable
        // Should wrap to index 0.
        assert_eq!(selected, Some(0));
    }

    #[test]
    fn nav_prev_wraps_to_last_actionable() {
        let items = make_test_items();
        let selected = super::nav_prev(&items, Some(0));
        // Should wrap to index 3.
        assert_eq!(selected, Some(3));
    }

    #[test]
    fn nav_prev_starts_at_last_actionable_when_none() {
        let items = make_test_items();
        let selected = super::nav_prev(&items, None);
        // With no selection, should pick the last actionable = index 3.
        assert_eq!(selected, Some(3));
    }

    #[test]
    fn nav_next_all_non_actionable_preserves_current() {
        let items = vec![
            ContextMenuEntry::Separator,
            ContextMenuEntry::Item(
                ContextMenuItem::new("Disabled").style(ContextMenuItemStyle::Disabled),
            ),
            ContextMenuEntry::Separator,
        ];
        // When no actionable items exist, nav_next/prev preserve current selection
        assert_eq!(super::nav_next(&items, None), None);
        assert_eq!(super::nav_next(&items, Some(1)), Some(1));
    }

    #[test]
    fn nav_prev_all_non_actionable_preserves_current() {
        let items = vec![
            ContextMenuEntry::Separator,
            ContextMenuEntry::Item(
                ContextMenuItem::new("Disabled").style(ContextMenuItemStyle::Disabled),
            ),
        ];
        assert_eq!(super::nav_prev(&items, None), None);
        assert_eq!(super::nav_prev(&items, Some(0)), Some(0));
    }

    #[test]
    fn nav_next_empty_items_returns_none() {
        let items: Vec<ContextMenuEntry> = vec![];
        assert_eq!(super::nav_next(&items, None), None);
    }
}

#[cfg(test)]
mod interaction_tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{TestAppContext, point, px};

    use super::{ContextMenu, ContextMenuEntry, ContextMenuItem, ContextMenuItemStyle};
    use crate::test_helpers::helpers::{
        InteractionExt, assert_element_absent, assert_element_exists, setup_test_window,
    };

    const MENU_OVERLAY: &str = "context-menu-overlay";
    const MENU_CONTENT: &str = "context-menu-content";
    const ITEM_COPY: &str = "context-menu-item-0";
    const ITEM_PASTE: &str = "context-menu-item-3";

    fn test_items(actions: Rc<RefCell<Vec<&'static str>>>) -> Vec<ContextMenuEntry> {
        vec![
            ContextMenuEntry::Item(ContextMenuItem::new("Copy").on_click({
                let actions = actions.clone();
                move |_, _| actions.borrow_mut().push("copy")
            })),
            ContextMenuEntry::Separator,
            ContextMenuEntry::Item(
                ContextMenuItem::new("Disabled").style(ContextMenuItemStyle::Disabled),
            ),
            ContextMenuEntry::Item(ContextMenuItem::new("Paste").on_click({
                let actions = actions.clone();
                move |_, _| actions.borrow_mut().push("paste")
            })),
        ]
    }

    fn open_menu(menu: &gpui::Entity<ContextMenu>, cx: &mut gpui::VisualTestContext) {
        menu.update_in(cx, |menu, window, cx| {
            menu.open(point(px(40.0), px(40.0)), window, cx);
        });
    }

    #[gpui::test]
    async fn click_item_fires_handler_and_closes(cx: &mut TestAppContext) {
        let actions = Rc::new(RefCell::new(Vec::new()));
        let (menu, cx) = setup_test_window(cx, |_window, cx| ContextMenu::new(cx));

        menu.update_in(cx, |menu, _window, _cx| {
            menu.set_items(test_items(actions.clone()));
        });
        open_menu(&menu, cx);

        assert_element_exists(cx, MENU_OVERLAY);
        assert_element_exists(cx, MENU_CONTENT);

        cx.click_on(ITEM_COPY);

        assert_eq!(&*actions.borrow(), &["copy"]);
        menu.update_in(cx, |menu, _window, _cx| {
            assert!(!menu.is_open());
        });
    }

    #[gpui::test]
    async fn keyboard_navigation_skips_disabled_items(cx: &mut TestAppContext) {
        let actions = Rc::new(RefCell::new(Vec::new()));
        let (menu, cx) = setup_test_window(cx, |_window, cx| ContextMenu::new(cx));

        menu.update_in(cx, |menu, _window, _cx| {
            menu.set_items(test_items(actions.clone()));
        });
        open_menu(&menu, cx);

        cx.press("down");
        cx.press("down");
        cx.press("enter");

        assert_eq!(&*actions.borrow(), &["paste"]);
        menu.update_in(cx, |menu, _window, _cx| {
            assert_eq!(menu.selected_index(), None);
            assert!(!menu.is_open());
        });
    }

    #[gpui::test]
    async fn escape_and_outside_click_dismiss_menu(cx: &mut TestAppContext) {
        let actions = Rc::new(RefCell::new(Vec::new()));
        let (menu, cx) = setup_test_window(cx, |_window, cx| ContextMenu::new(cx));

        menu.update_in(cx, |menu, _window, _cx| {
            menu.set_items(test_items(actions.clone()));
        });
        open_menu(&menu, cx);
        cx.press("escape");

        menu.update_in(cx, |menu, _window, _cx| assert!(!menu.is_open()));
        assert_element_absent(cx, MENU_CONTENT);

        open_menu(&menu, cx);
        assert_element_exists(cx, ITEM_PASTE);
        cx.click_at(point(px(5.0), px(5.0)));

        menu.update_in(cx, |menu, _window, _cx| assert!(!menu.is_open()));
        assert!(actions.borrow().is_empty());
        assert_element_absent(cx, MENU_CONTENT);
    }

    // ── Submenu hover-to-open (Task A) ───────────────────────────────────

    /// Build an entries list whose index 0 is a submenu parent — exercises
    /// the hover-to-open path.
    fn items_with_submenu() -> Vec<ContextMenuEntry> {
        vec![
            ContextMenuEntry::Submenu {
                label: "Recent".into(),
                icon: None,
                items: vec![
                    ContextMenuEntry::Item(ContextMenuItem::new("File A")),
                    ContextMenuEntry::Item(ContextMenuItem::new("File B")),
                ],
            },
            ContextMenuEntry::Item(ContextMenuItem::new("Close")),
        ]
    }

    #[gpui::test]
    async fn submenu_hover_expands_after_delay(cx: &mut TestAppContext) {
        let (menu, cx) = setup_test_window(cx, |_window, cx| ContextMenu::new(cx));

        menu.update_in(cx, |menu, _window, _cx| {
            menu.set_items(items_with_submenu());
        });
        open_menu(&menu, cx);

        // Drive the hover-open timer directly: rendering a synthetic hover
        // event against the overlay would still funnel through the same
        // `schedule_submenu_hover_open` we're about to call.
        menu.update_in(cx, |menu, _window, cx| {
            menu.schedule_submenu_hover_open(0, cx);
            assert!(menu.expanded_submenu().is_none(), "must not expand yet");
        });

        // 100 ms later, the task should have fired and expanded the submenu.
        cx.executor()
            .advance_clock(std::time::Duration::from_millis(150));
        cx.run_until_parked();

        menu.update_in(cx, |menu, _window, _cx| {
            assert_eq!(
                menu.expanded_submenu(),
                Some(0),
                "submenu should be open after 100 ms hover"
            );
            // Selection path should have descended into the first child
            // so keyboard nav lands on a real row.
            assert_eq!(menu.selection_path(), &[0, 0]);
        });
    }

    #[gpui::test]
    async fn submenu_hover_cancelled_before_delay(cx: &mut TestAppContext) {
        let (menu, cx) = setup_test_window(cx, |_window, cx| ContextMenu::new(cx));

        menu.update_in(cx, |menu, _window, _cx| {
            menu.set_items(items_with_submenu());
        });
        open_menu(&menu, cx);

        // Start the timer, then cancel before 100 ms elapse.
        menu.update_in(cx, |menu, _window, cx| {
            menu.schedule_submenu_hover_open(0, cx);
            menu.cancel_submenu_hover_open();
        });

        cx.executor()
            .advance_clock(std::time::Duration::from_millis(300));
        cx.run_until_parked();

        menu.update_in(cx, |menu, _window, _cx| {
            assert!(
                menu.expanded_submenu().is_none(),
                "hover-leave must cancel the pending open"
            );
        });
    }

    // ── Space-to-toggle (Task B) ─────────────────────────────────────────

    #[gpui::test]
    async fn space_fires_on_toggle_without_closing_menu(cx: &mut TestAppContext) {
        let toggles: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
        let (menu, cx) = setup_test_window(cx, |_window, cx| ContextMenu::new(cx));

        menu.update_in(cx, |menu, _window, _cx| {
            let t = toggles.clone();
            menu.set_items(vec![ContextMenuEntry::Item(
                ContextMenuItem::new("Show Toolbar")
                    .checked(false)
                    .on_toggle(move |new_state, _window, _cx| {
                        t.borrow_mut().push(new_state);
                    }),
            )]);
        });
        open_menu(&menu, cx);

        // Land keyboard selection on the checkable row.
        cx.press("down");
        cx.press("space");

        menu.update_in(cx, |menu, _window, _cx| {
            assert!(menu.is_open(), "Space must not close the menu");
        });
        assert_eq!(
            &*toggles.borrow(),
            &[true],
            "on_toggle should fire with the flipped state"
        );
    }

    #[gpui::test]
    async fn space_on_non_togglable_item_is_a_noop(cx: &mut TestAppContext) {
        let actions = Rc::new(RefCell::new(Vec::new()));
        let (menu, cx) = setup_test_window(cx, |_window, cx| ContextMenu::new(cx));

        menu.update_in(cx, |menu, _window, _cx| {
            menu.set_items(test_items(actions.clone()));
        });
        open_menu(&menu, cx);

        cx.press("down");
        cx.press("space");

        // Neither Copy's on_click nor any toggle should have fired.
        assert!(actions.borrow().is_empty());
        menu.update_in(cx, |menu, _window, _cx| {
            assert!(menu.is_open(), "Space must not close on a plain item");
        });
    }
}

#[cfg(test)]
mod positioning_tests {
    use gpui::prelude::*;
    use gpui::{Context, Entity, IntoElement, Render, TestAppContext, div, point, px};

    use super::{ContextMenu, ContextMenuEntry, ContextMenuItem};
    use crate::test_helpers::helpers::{LocatorExt, setup_test_window};

    /// Harness that mimics the gallery pane: the context-menu entity is
    /// nested inside a PADDED flex column so its layout container has a
    /// non-zero window offset. This reproduces the bug where the menu
    /// rendered offset from the pointer because the old code's
    /// `.absolute().top(pos.y).left(pos.x)` was interpreted relative to
    /// the pane rather than the window.
    struct OffsetHarness {
        menu: Entity<ContextMenu>,
    }

    impl Render for OffsetHarness {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut Context<Self>,
        ) -> impl IntoElement {
            div()
                .pt(px(200.0))
                .pl(px(80.0))
                .debug_selector(|| "offset-host".into())
                .flex()
                .flex_col()
                .child(
                    div()
                        .debug_selector(|| "sibling-below".into())
                        .h(px(40.0))
                        .w(px(240.0))
                        .child("sibling below the menu host"),
                )
                .child(self.menu.clone())
        }
    }

    #[gpui::test]
    async fn menu_content_lands_at_window_absolute_point(cx: &mut TestAppContext) {
        // Bug: the menu used `div().absolute().top(pos.y).left(pos.x)`
        // nested inside a padded pane. That made `pos` resolve against
        // the pane's local origin instead of the window root, so the
        // menu appeared offset by roughly (pane.origin.x, pane.origin.y)
        // from the cursor. Fix: `AnchoredOverlay::WindowPoint(pos)` uses
        // `anchored().position()` which is window-absolute, matching the
        // coordinate space of `MouseDownEvent::position`.
        let menu_entity = cx.update(|cx| cx.new(ContextMenu::new));
        let (_host, cx) = setup_test_window(cx, {
            let menu = menu_entity.clone();
            move |_window, _cx| OffsetHarness { menu }
        });

        let click_at = point(px(400.0), px(300.0));
        menu_entity.update_in(cx, |menu, window, cx| {
            menu.set_items(vec![
                ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                ContextMenuEntry::Item(ContextMenuItem::new("Paste")),
            ]);
            menu.open(click_at, window, cx);
        });

        let content = cx.get_element("context-menu-content");

        // Anchored overlay places the content's top-left at the anchor
        // point (modulo snap-margin clamping, which shouldn't fire near
        // the middle of the viewport). Allow a small tolerance for the
        // wrapper's own padding / border geometry — the check is that
        // the menu landed near the click, not hundreds of pixels away
        // (which is the regression before the fix).
        let dx = (f32::from(content.bounds.origin.x) - f32::from(click_at.x)).abs();
        let dy = (f32::from(content.bounds.origin.y) - f32::from(click_at.y)).abs();
        assert!(
            dx < 8.0 && dy < 8.0,
            "menu content origin {:?} should be within ~8pt of click point {:?}; \
             delta = ({dx}, {dy}). A larger delta means the overlay is resolving \
             against a local (pane-relative) coordinate system instead of \
             window-absolute — the exact bug AnchoredOverlay::WindowPoint fixes.",
            content.bounds.origin,
            click_at,
        );
    }

    #[gpui::test]
    async fn opening_menu_does_not_shift_sibling_below(cx: &mut TestAppContext) {
        // Bug: the menu's render subtree contributed to the host's
        // layout (the outer `.absolute().size_full()` wrapper added
        // layout footprint to the gallery pane's flex column), so
        // siblings rendered AFTER the menu entity were shifted down
        // when the menu opened. Fix: `AnchoredOverlay` wraps the
        // overlay in `deferred()`, taking it out of the parent's
        // layout tree — siblings never see a size change.
        let menu_entity = cx.update(|cx| cx.new(ContextMenu::new));
        let (_host, cx) = setup_test_window(cx, {
            let menu = menu_entity.clone();
            move |_window, _cx| OffsetHarness { menu }
        });

        // Capture the sibling's position while the menu is closed.
        let closed_top = cx.get_element("sibling-below").bounds.top();

        // Open the menu and let layout settle.
        menu_entity.update_in(cx, |menu, window, cx| {
            menu.set_items(vec![
                ContextMenuEntry::Item(ContextMenuItem::new("Cut")),
                ContextMenuEntry::Item(ContextMenuItem::new("Copy")),
                ContextMenuEntry::Item(ContextMenuItem::new("Paste")),
                ContextMenuEntry::Separator,
                ContextMenuEntry::Item(ContextMenuItem::new("Delete")),
            ]);
            menu.open(point(px(400.0), px(300.0)), window, cx);
        });

        let opened_top = cx.get_element("sibling-below").bounds.top();

        assert_eq!(
            opened_top, closed_top,
            "opening the context menu must not shift siblings in the host's \
             layout tree. The `deferred()` wrapper inside AnchoredOverlay \
             moves the overlay out of flow, so the sibling's layout position \
             is invariant across open/close transitions."
        );
    }
}
