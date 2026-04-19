//! Canonical macOS Edit menu command set.
//!
//! HIG "Edit menus" (<https://developer.apple.com/design/human-interface-guidelines/edit-menus>)
//! enumerates the standard commands every macOS app is expected to expose
//! in the Edit menu bar slot: Undo, Redo, Cut, Copy, Paste, Paste as,
//! Paste Style, Delete, Select All, and Find … with their canonical
//! keystrokes.
//!
//! This module provides:
//! - [`EditCommand`] — the typed command set, mirroring the HIG table.
//! - [`edit_menu_standard`] — a ready-made [`Vec<ContextMenuEntry>`] for
//!   populating a [`super::context_menu::ContextMenu`] or the Edit slot of
//!   a [`super::menu_bar::MenuBar`].
//!
//! The items carry no `on_click` handlers by default — callers wire up
//! each command to their undo/pasteboard state via the `bind` helper on
//! each [`ContextMenuEntry`] returned.

use gpui::SharedString;

use crate::components::menus_and_actions::context_menu::{
    ContextMenuEntry, ContextMenuItem, ContextMenuItemStyle,
};
use crate::foundations::icons::IconName;
use crate::foundations::keyboard_shortcuts::{MenuShortcut, ModifierKey};

/// The canonical Edit menu commands per HIG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditCommand {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    PasteAsStyle,
    Delete,
    SelectAll,
    Find,
    FindNext,
    FindPrevious,
}

impl EditCommand {
    /// Display label for the command.
    pub fn label(self) -> &'static str {
        match self {
            Self::Undo => "Undo",
            Self::Redo => "Redo",
            Self::Cut => "Cut",
            Self::Copy => "Copy",
            Self::Paste => "Paste",
            Self::PasteAsStyle => "Paste and Match Style",
            Self::Delete => "Delete",
            Self::SelectAll => "Select All",
            Self::Find => "Find\u{2026}",
            Self::FindNext => "Find Next",
            Self::FindPrevious => "Find Previous",
        }
    }

    /// Canonical macOS keyboard shortcut.
    pub fn shortcut(self) -> MenuShortcut {
        use ModifierKey as M;
        match self {
            Self::Undo => MenuShortcut::cmd("Z"),
            Self::Redo => MenuShortcut::new("Z").with_modifiers(&[M::Command, M::Shift]),
            Self::Cut => MenuShortcut::cmd("X"),
            Self::Copy => MenuShortcut::cmd("C"),
            Self::Paste => MenuShortcut::cmd("V"),
            Self::PasteAsStyle => {
                MenuShortcut::new("V").with_modifiers(&[M::Command, M::Option, M::Shift])
            }
            Self::Delete => MenuShortcut::new("Delete"),
            Self::SelectAll => MenuShortcut::cmd("A"),
            Self::Find => MenuShortcut::cmd("F"),
            Self::FindNext => MenuShortcut::cmd("G"),
            Self::FindPrevious => MenuShortcut::new("G").with_modifiers(&[M::Command, M::Shift]),
        }
    }

    /// Optional leading icon — most Edit commands are icon-less per HIG
    /// to keep the text column compact.
    pub fn icon(self) -> Option<IconName> {
        match self {
            Self::Copy => Some(IconName::Copy),
            Self::Find | Self::FindNext | Self::FindPrevious => Some(IconName::Search),
            _ => None,
        }
    }
}

/// Canonical group order for the Edit menu, matching HIG.
const GROUPS: &[&[EditCommand]] = &[
    &[EditCommand::Undo, EditCommand::Redo],
    &[
        EditCommand::Cut,
        EditCommand::Copy,
        EditCommand::Paste,
        EditCommand::PasteAsStyle,
        EditCommand::Delete,
        EditCommand::SelectAll,
    ],
    &[
        EditCommand::Find,
        EditCommand::FindNext,
        EditCommand::FindPrevious,
    ],
];

/// Build a standard Edit menu entry list.
///
/// Each command becomes a `ContextMenuEntry::Item` with its canonical
/// label, shortcut, and (where applicable) leading icon. Groups are
/// separated by `ContextMenuEntry::Separator` per HIG.
///
/// The resulting entries carry no `on_click` handlers — callers wire up
/// each command to their pasteboard / undo stack. The returned Vec can
/// be mutated to add bindings via pattern matching.
pub fn edit_menu_standard() -> Vec<ContextMenuEntry> {
    let mut out = Vec::new();
    for (i, group) in GROUPS.iter().enumerate() {
        if i > 0 {
            out.push(ContextMenuEntry::Separator);
        }
        for cmd in *group {
            let label: SharedString = cmd.label().into();
            let mut item = ContextMenuItem::new(label)
                .style(ContextMenuItemStyle::Default)
                .shortcut(cmd.shortcut());
            if let Some(icon) = cmd.icon() {
                item = item.icon(icon);
            }
            out.push(ContextMenuEntry::Item(item));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{EditCommand, edit_menu_standard};
    use crate::components::menus_and_actions::context_menu::ContextMenuEntry;
    use core::prelude::v1::test;

    #[test]
    fn standard_entries_contain_all_canonical_commands() {
        let entries = edit_menu_standard();
        let labels: Vec<String> = entries
            .iter()
            .filter_map(|e| match e {
                ContextMenuEntry::Item(item) => Some(item.label.to_string()),
                _ => None,
            })
            .collect();
        for cmd in [
            EditCommand::Undo,
            EditCommand::Redo,
            EditCommand::Cut,
            EditCommand::Copy,
            EditCommand::Paste,
            EditCommand::SelectAll,
            EditCommand::Find,
        ] {
            assert!(
                labels.iter().any(|l| l == cmd.label()),
                "missing {} label in standard Edit menu",
                cmd.label()
            );
        }
    }

    #[test]
    fn standard_entries_interleave_separators_between_groups() {
        let entries = edit_menu_standard();
        // We expect exactly 2 separators (3 groups → 2 dividers).
        let sep_count = entries
            .iter()
            .filter(|e| matches!(e, ContextMenuEntry::Separator))
            .count();
        assert_eq!(sep_count, 2);
    }

    #[test]
    fn cmd_z_shortcut_renders_as_command_z() {
        assert_eq!(EditCommand::Undo.shortcut().render(), "\u{2318}Z");
    }

    #[test]
    fn redo_is_command_shift_z() {
        assert_eq!(EditCommand::Redo.shortcut().render(), "\u{21E7}\u{2318}Z");
    }
}
