//! Dock menu component (HIG "Dock menus", macOS-only).
//!
//! HIG: <https://developer.apple.com/design/human-interface-guidelines/dock-menus>
//!
//! macOS apps can add custom items to the Dock menu (Control-click the app
//! icon). AppKit exposes this via
//! `NSApplicationDelegate.applicationDockMenu(_:)`.
//!
//! GPUI does not currently provide an AppKit bridge for Dock menus. The
//! [`DockMenu`] type below is therefore a **declarative container**: it
//! captures the items the host wants to expose, and a host-level adapter
//! (outside this crate) is expected to hand them to AppKit when GPUI
//! gains the API. Until then, callers can still preview the menu via
//! [`DockMenu::as_context_menu_entries`] — the same items render in a
//! regular context menu for in-app simulation.
//!
//! # Platform support
//!
//! Dock menus are a **macOS-only** HIG surface. On Linux and Windows, the
//! system dock / taskbar either does not exist or exposes an incompatible
//! per-platform API. Callers targeting non-macOS platforms should not
//! instantiate [`DockMenu`]: the type still compiles (it is a pure data
//! container so that cross-platform test suites can at least reference
//! it), but there is no path to install it into the OS dock — that
//! requires an AppKit bridge which ships with the macOS build of GPUI.
//! The crate suppresses `dead_code` warnings on non-macOS so the
//! placeholder surface does not pollute builds that never call it.

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use gpui::SharedString;

use crate::components::menus_and_actions::context_menu::{ContextMenuEntry, ContextMenuItem};
use crate::foundations::icons::IconName;

/// A single item in a Dock menu. Mirrors HIG Dock menu item semantics —
/// a label, optional icon, optional submenu, and an action handler.
pub struct DockMenuItem {
    pub label: SharedString,
    pub icon: Option<IconName>,
    /// When `true`, the item is greyed out in the Dock menu.
    pub disabled: bool,
    /// Nested items — HIG allows one level of submenu in Dock menus.
    pub submenu: Vec<DockMenuItem>,
}

impl DockMenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
            submenu: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_submenu(mut self, items: Vec<DockMenuItem>) -> Self {
        self.submenu = items;
        self
    }
}

/// Declarative container for a macOS Dock menu.
///
/// Not a `RenderOnce` — Dock menus live outside the app window's view
/// hierarchy. Use [`DockMenu::as_context_menu_entries`] if you need to
/// preview the items in an in-app menu.
pub struct DockMenu {
    pub items: Vec<DockMenuItem>,
}

impl DockMenu {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn item(mut self, item: DockMenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: Vec<DockMenuItem>) -> Self {
        self.items = items;
        self
    }

    /// Convert the Dock menu definition into a flat list of context-menu
    /// entries (for in-app simulation — not wired to the real Dock). The
    /// conversion preserves icons, disabled state, and nested submenus as
    /// `ContextMenuEntry::Submenu` variants.
    pub fn as_context_menu_entries(&self) -> Vec<ContextMenuEntry> {
        self.items.iter().map(entry_for).collect()
    }
}

impl Default for DockMenu {
    fn default() -> Self {
        Self::new()
    }
}

fn entry_for(item: &DockMenuItem) -> ContextMenuEntry {
    if !item.submenu.is_empty() {
        return ContextMenuEntry::Submenu {
            label: item.label.clone(),
            icon: item.icon,
            items: item.submenu.iter().map(entry_for).collect(),
        };
    }
    let mut ci = ContextMenuItem::new(item.label.clone());
    if let Some(icon) = item.icon {
        ci = ci.icon(icon);
    }
    if item.disabled {
        ci = ci.style(
            crate::components::menus_and_actions::context_menu::ContextMenuItemStyle::Disabled,
        );
    }
    ContextMenuEntry::Item(ci)
}

#[cfg(test)]
mod tests {
    use super::{DockMenu, DockMenuItem};
    use crate::components::menus_and_actions::context_menu::ContextMenuEntry;
    use core::prelude::v1::test;

    #[test]
    fn dock_menu_empty_by_default() {
        let menu = DockMenu::new();
        assert!(menu.items.is_empty());
    }

    #[test]
    fn item_builder_chains() {
        let item = DockMenuItem::new("Options")
            .icon(crate::foundations::icons::IconName::Settings)
            .disabled(true);
        assert_eq!(item.label.as_ref(), "Options");
        assert!(item.disabled);
        assert!(item.icon.is_some());
    }

    #[test]
    fn submenu_items_become_submenu_entries() {
        let menu = DockMenu::new().item(DockMenuItem::new("Recent Files").with_submenu(vec![
            DockMenuItem::new("File 1"),
            DockMenuItem::new("File 2"),
        ]));
        let entries = menu.as_context_menu_entries();
        assert_eq!(entries.len(), 1);
        match &entries[0] {
            ContextMenuEntry::Submenu { label, items, .. } => {
                assert_eq!(label.as_ref(), "Recent Files");
                assert_eq!(items.len(), 2);
            }
            _ => panic!("expected submenu"),
        }
    }

    #[test]
    fn disabled_items_map_to_disabled_context_menu_items() {
        let menu = DockMenu::new().item(DockMenuItem::new("Unavailable").disabled(true));
        let entries = menu.as_context_menu_entries();
        match &entries[0] {
            ContextMenuEntry::Item(item) => {
                assert_eq!(
                    item.style,
                    crate::components::menus_and_actions::context_menu::ContextMenuItemStyle::Disabled
                );
            }
            _ => panic!("expected item"),
        }
    }
}
