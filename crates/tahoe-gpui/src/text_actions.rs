//! Shared keyboard editing actions for text input components.
//!
//! These GPUI actions represent generic text-editing operations shared by
//! [`TextField`](crate::components::selection_and_input::TextField) and
//! other text input components. Chatbot-specific actions (Submit, Newline, etc.)
//! remain in the binding crate.
//!
//! **Keybinding strategy per HIG and GPUI best practices:**
//!
//! Only **modifier-key combinations** (Cmd+C, Alt+Left, Shift+Right, etc.) are
//! registered as global keybindings. Raw keys (arrows, Backspace, Delete, Enter,
//! Space, Escape) are NOT globally bound because:
//! 1. Arrow keys have component-specific meanings (text cursor vs tab switch vs slider)
//! 2. Enter/Space mean different things in different contexts (submit vs activate vs newline)
//! 3. Escape dismiss behavior varies per component type
//!
//! Components handle raw keys via `on_key_down()` or scoped `KeyBinding` contexts.
//!
//! # Selection-to-boundary actions
//!
//! macOS distinguishes line-boundary selection (`Cmd-Shift-Left/Right`) from
//! document-boundary selection (`Cmd-Shift-Up/Down`). Historically this module
//! exported a single `SelectToStart` / `SelectToEnd` pair bound to both
//! gestures, which conflated the two meanings. The four
//! [`SelectToLineStart`] / [`SelectToLineEnd`] / [`SelectToDocStart`] /
//! [`SelectToDocEnd`] actions now track the HIG convention exactly.
//! Single-line inputs (e.g. [`TextField`](crate::components::selection_and_input::TextField))
//! treat all four as equivalent because line-start == doc-start for a
//! one-line field; multi-line editors can differentiate.
//!
//! # Mandatory HIG keybindings
//!
//! [`mandatory_keybindings`] returns the subset HIG §Undo-and-redo lists as
//! required on every macOS app: `Cmd-Z` (Undo) and `Cmd-Shift-Z` (Redo).
//! Hosts that wire a subset of the crate's components *must* still install
//! these globally, otherwise Undo/Redo is unreachable outside the text-entry
//! scope. See [`crate::mandatory_keybindings`] for the crate-level export.

use gpui::{KeyBinding, actions};

actions!(
    text_editing,
    [
        Backspace,
        Copy,
        Cut,
        Delete,
        End,
        Home,
        Left,
        Paste,
        Right,
        SelectAll,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
        WordLeft,
        WordRight,
        // Standard macOS editing actions
        Undo,
        Redo,
        /// Extend selection to the start of the current line
        /// (Cmd-Shift-Left on macOS).
        SelectToLineStart,
        /// Extend selection to the end of the current line
        /// (Cmd-Shift-Right on macOS).
        SelectToLineEnd,
        /// Extend selection to the start of the document
        /// (Cmd-Shift-Up on macOS).
        SelectToDocStart,
        /// Extend selection to the end of the document
        /// (Cmd-Shift-Down on macOS).
        SelectToDocEnd,
        MoveToStart,
        MoveToEnd,
        DeleteWord,
        DeleteToEnd,
        DeleteToStart,
        /// Up arrow — line-up navigation. TextView binds this to
        /// scroll-up-by-one-line when scrollable; future multi-line
        /// editors may bind it to cursor-move-up.
        Up,
        /// Down arrow — line-down navigation.
        Down,
        /// Page Up — scroll or jump by one viewport height.
        PageUp,
        /// Page Down — scroll or jump by one viewport height.
        PageDown,
    ]
);

/// Returns the HIG-mandated global keybindings that *every* macOS host
/// must install, regardless of which components are embedded.
///
/// Currently this is `Cmd-Z` → [`Undo`] and `Cmd-Shift-Z` → [`Redo`] per
/// HIG §Undo-and-redo: "On macOS, Undo (Command-Z) and Redo
/// (Shift-Command-Z) are expected keyboard shortcuts."
///
/// [`keybindings`] also contains these entries, but a host that only
/// consumes a non-text component (e.g. a canvas or button gallery) still
/// needs Undo/Redo reachable — hence the separate export. Callers
/// typically install the full set via [`crate::mandatory_keybindings`].
pub fn mandatory_keybindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("cmd-z", Undo, None),
        KeyBinding::new("cmd-shift-z", Redo, None),
    ]
}

/// Returns the standard macOS text editing keybindings per HIG.
///
/// These are **modifier-key combinations only** — no raw arrow keys, Enter,
/// Space, Escape, or Backspace. Raw keys are handled per-component via
/// `on_key_down()` or scoped keybindings to avoid global conflicts.
///
/// Includes the [`mandatory_keybindings`] set (Undo / Redo) plus clipboard,
/// word navigation, and selection-to-boundary bindings.
///
/// Register during app initialization:
/// ```ignore
/// cx.bind_keys(tahoe_gpui::text_actions::keybindings());
/// ```
pub fn keybindings() -> Vec<KeyBinding> {
    vec![
        // ── Mandatory (HIG §Undo-and-redo) ───────────────────────
        KeyBinding::new("cmd-z", Undo, None),
        KeyBinding::new("cmd-shift-z", Redo, None),
        // ── Clipboard (HIG §Editing) ─────────────────────────────
        KeyBinding::new("cmd-c", Copy, None),
        KeyBinding::new("cmd-x", Cut, None),
        KeyBinding::new("cmd-v", Paste, None),
        KeyBinding::new("cmd-a", SelectAll, None),
        // ── Word navigation (HIG §Selection) ─────────────────────
        // Modifier+arrow: safe as global because they don't conflict
        // with raw arrow keys used for component navigation.
        KeyBinding::new("alt-left", WordLeft, None),
        KeyBinding::new("alt-right", WordRight, None),
        KeyBinding::new("cmd-left", Home, None),
        KeyBinding::new("cmd-right", End, None),
        KeyBinding::new("cmd-up", MoveToStart, None),
        KeyBinding::new("cmd-down", MoveToEnd, None),
        // ── Selection with modifiers (HIG §Selection) ────────────
        KeyBinding::new("shift-left", SelectLeft, None),
        KeyBinding::new("shift-right", SelectRight, None),
        KeyBinding::new("alt-shift-left", SelectWordLeft, None),
        KeyBinding::new("alt-shift-right", SelectWordRight, None),
        // macOS line-boundary vs document-boundary selection:
        // cmd-shift-left/right → line; cmd-shift-up/down → document.
        KeyBinding::new("cmd-shift-left", SelectToLineStart, None),
        KeyBinding::new("cmd-shift-right", SelectToLineEnd, None),
        KeyBinding::new("cmd-shift-up", SelectToDocStart, None),
        KeyBinding::new("cmd-shift-down", SelectToDocEnd, None),
        // ── Deletion with modifiers (HIG §Editing) ───────────────
        KeyBinding::new("alt-backspace", DeleteWord, None),
        KeyBinding::new("cmd-backspace", DeleteToStart, None),
        KeyBinding::new("cmd-delete", DeleteToEnd, None),
    ]
}

#[cfg(test)]
mod tests {
    use super::{keybindings, mandatory_keybindings};
    use core::prelude::v1::test;

    #[test]
    fn mandatory_set_contains_exactly_two_bindings() {
        // HIG §Undo-and-redo mandates Undo / Redo. Guard against accidental
        // additions that would widen the "mandatory" contract without
        // explicit review.
        assert_eq!(mandatory_keybindings().len(), 2);
    }

    #[test]
    fn full_keybindings_superset_of_mandatory() {
        // The global set must still include every mandatory binding so
        // hosts that register only `keybindings()` stay HIG-compliant.
        let full = keybindings().len();
        assert!(full > mandatory_keybindings().len());
    }
}
