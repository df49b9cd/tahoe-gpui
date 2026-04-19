//! Standard macOS keyboard shortcuts aligned with the HIG Keyboards page.
//!
//! Comprehensive reference table of standard keyboard shortcuts that macOS
//! applications should support. Components in this crate implement these
//! through GPUI's `KeyBinding` system.
//!
//! # Scope of [`standard_shortcuts`]
//!
//! The table returned by [`standard_shortcuts`] is a **reference document**,
//! not an auto-registration hook. It exists so components can cite the
//! canonical keystroke for an action without hard-coding strings, and so
//! gallery examples can render the full shortcut cheat-sheet.
//!
//! Application-level shortcuts — `cmd-q` (Quit), `cmd-h` (Hide),
//! `cmd-m` (Minimize), `cmd-n` (New), `cmd-w` (Close Window), `cmd-,`
//! (Settings), `cmd-?` (Help), and the window-switching, Spotlight, and
//! screenshot entries in the System and Window categories — are the
//! **host application's responsibility**. Components in this crate MUST
//! NOT bind these keys via `KeyBinding::new(..., None)` (global scope) —
//! doing so would silently shadow the host's menu commands. Components
//! that need text-editing shortcuts (Copy, Paste, Select All, etc.) bind
//! them inside a scoped key context (e.g. `TextField` via
//! `TEXT_FIELD_CONTEXT`) so the bindings only fire when the relevant
//! component is focused.
//!
//! The [`host_owned_keys`] helper below returns the exact list of
//! keystrokes that crate components must not bind at global scope; test
//! suites can pin the contract by asserting their binding list is disjoint
//! from `host_owned_keys()`.
//!
//! # Modifier Key Ordering
//!
//! When listing modifier keys in a shortcut, always use this order per HIG:
//! **Control, Option, Shift, Command** (symbols: `⌃⌥⇧⌘`).
//!
//! # Standard shortcuts (HIG: Keyboards)
//!
//! | Shortcut | Action | Category |
//! |---|---|---|
//! | Cmd+Z | Undo | Editing |
//! | Cmd+Shift+Z | Redo | Editing |
//! | Cmd+X | Cut | Editing |
//! | Cmd+C | Copy | Editing |
//! | Cmd+V | Paste | Editing |
//! | Cmd+A | Select All | Editing |
//! | Cmd+Shift+A | Deselect All | Editing |
//! | Cmd+E | Use Selection for Find | Editing |
//! | Cmd+Shift+V | Paste As | Editing |
//! | Cmd+Alt+V | Paste Style | Editing |
//! | Cmd+B | Bold | Formatting |
//! | Cmd+I | Italic | Formatting |
//! | Cmd+U | Underline | Formatting |
//! | Cmd+T | Show Fonts | Formatting |
//! | Cmd+Shift+C | Show Colors | Formatting |
//! | Cmd+Alt+C | Copy Style | Formatting |
//! | Cmd+{ | Align Left | Formatting |
//! | Cmd+} | Align Right | Formatting |
//! | Cmd+\| | Center Align | Formatting |
//! | Cmd++ | Increase Size | Formatting |
//! | Cmd+- | Decrease Size | Formatting |
//! | Cmd+F | Find | Search |
//! | Cmd+G | Find Next | Search |
//! | Cmd+Shift+G | Find Previous | Search |
//! | Cmd+Alt+F | Jump to Search Field | Search |
//! | Cmd+J | Scroll to Selection | Search |
//! | Cmd+: | Show Spelling | Search |
//! | Cmd+; | Find Misspelled Words | Search |
//! | Cmd+W | Close Window | Window |
//! | Cmd+Shift+W | Close File and Windows | Window |
//! | Cmd+Alt+W | Close All Windows | Window |
//! | Cmd+M | Minimize | Window |
//! | Cmd+Alt+M | Minimize All | Window |
//! | Ctrl+Cmd+F | Enter Full Screen | Window |
//! | Cmd+` | Next Window | Window |
//! | Cmd+Shift+` | Previous Window | Window |
//! | Cmd+Tab | Next App | Window |
//! | Cmd+Shift+Tab | Previous App | Window |
//! | Cmd+Q | Quit | Application |
//! | Cmd+H | Hide App | Application |
//! | Cmd+Alt+H | Hide Others | Application |
//! | Cmd+, | Settings | Application |
//! | Cmd+? | Help | Application |
//! | Cmd+N | New Document | Document |
//! | Cmd+O | Open | Document |
//! | Cmd+S | Save | Document |
//! | Cmd+Shift+S | Save As / Duplicate | Document |
//! | Cmd+P | Print | Document |
//! | Cmd+Shift+P | Page Setup | Document |
//! | Tab | Next Field | Navigation |
//! | Shift+Tab | Previous Field | Navigation |
//! | Escape | Cancel / Dismiss | Navigation |
//! | Cmd+. | Cancel Operation | Navigation |
//! | Ctrl+Tab | Next Tab / Group | Navigation |
//! | Ctrl+Shift+Tab | Previous Tab / Group | Navigation |
//! | Cmd+Right | End of Line | Selection |
//! | Cmd+Left | Beginning of Line | Selection |
//! | Cmd+Up | Beginning of Document | Selection |
//! | Cmd+Down | End of Document | Selection |
//! | Alt+Right | Next Word | Selection |
//! | Alt+Left | Previous Word | Selection |
//! | Shift+Right | Extend Selection Right | Selection |
//! | Shift+Left | Extend Selection Left | Selection |
//! | Shift+Cmd+Right | Select to End of Line | Selection |
//! | Shift+Cmd+Left | Select to Start of Line | Selection |
//! | Shift+Cmd+Up | Select to Beginning | Selection |
//! | Shift+Cmd+Down | Select to End | Selection |
//! | Shift+Alt+Right | Select to End of Word | Selection |
//! | Shift+Alt+Left | Select to Start of Word | Selection |
//! | Cmd+Shift+3 | Screenshot (Full) | System |
//! | Cmd+Shift+4 | Screenshot (Selection) | System |
//! | Cmd+Space | Spotlight | System |
//! | Cmd+F5 | VoiceOver Toggle | System |
//! | F11 | Show Desktop | System |

/// Standard keyboard shortcut categories per HIG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutCategory {
    /// Text editing shortcuts (Copy, Paste, Undo, Bold, Italic, etc.)
    Editing,
    /// Text formatting shortcuts (Bold, Italic, Underline, alignment)
    Formatting,
    /// Search and find shortcuts (Find, Find Next, Find Previous)
    Search,
    /// Window management shortcuts (Close, Minimize, Full Screen, app switching)
    Window,
    /// Application-level shortcuts (Quit, Preferences, Hide)
    Application,
    /// Document management shortcuts (New, Open, Save, Print)
    Document,
    /// Navigation shortcuts (Tab, Escape, focus movement)
    Navigation,
    /// Text selection and cursor movement
    Selection,
    /// System shortcuts (screenshot, zoom, accessibility)
    System,
}

/// A standard macOS keyboard shortcut definition.
#[derive(Debug, Clone)]
pub struct StandardShortcut {
    /// The key combination (e.g., "cmd-c").
    pub keys: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Category for grouping.
    pub category: ShortcutCategory,
}

/// Returns the standard macOS keyboard shortcuts per the HIG Keyboards page.
pub fn standard_shortcuts() -> &'static [StandardShortcut] {
    &[
        // -- Editing ----------------------------------------------------------
        StandardShortcut {
            keys: "cmd-z",
            description: "Undo",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-shift-z",
            description: "Redo",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-x",
            description: "Cut",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-c",
            description: "Copy",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-v",
            description: "Paste",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-a",
            description: "Select All",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-shift-a",
            description: "Deselect All",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-e",
            description: "Use Selection for Find",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-shift-v",
            description: "Paste As",
            category: ShortcutCategory::Editing,
        },
        StandardShortcut {
            keys: "cmd-alt-v",
            description: "Paste Style",
            category: ShortcutCategory::Editing,
        },
        // -- Formatting -------------------------------------------------------
        StandardShortcut {
            keys: "cmd-b",
            description: "Bold",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-i",
            description: "Italic",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-u",
            description: "Underline",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-t",
            description: "Show Fonts",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-shift-c",
            description: "Show Colors",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-alt-c",
            description: "Copy Style",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-{",
            description: "Align Left",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-}",
            description: "Align Right",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-|",
            description: "Center Align",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd-+",
            description: "Increase Size",
            category: ShortcutCategory::Formatting,
        },
        StandardShortcut {
            keys: "cmd--",
            description: "Decrease Size",
            category: ShortcutCategory::Formatting,
        },
        // -- Search -----------------------------------------------------------
        StandardShortcut {
            keys: "cmd-f",
            description: "Find",
            category: ShortcutCategory::Search,
        },
        StandardShortcut {
            keys: "cmd-g",
            description: "Find Next",
            category: ShortcutCategory::Search,
        },
        StandardShortcut {
            keys: "cmd-shift-g",
            description: "Find Previous",
            category: ShortcutCategory::Search,
        },
        StandardShortcut {
            keys: "cmd-alt-f",
            description: "Jump to Search Field",
            category: ShortcutCategory::Search,
        },
        StandardShortcut {
            keys: "cmd-j",
            description: "Scroll to Selection",
            category: ShortcutCategory::Search,
        },
        StandardShortcut {
            keys: "cmd-:",
            description: "Show Spelling",
            category: ShortcutCategory::Search,
        },
        StandardShortcut {
            keys: "cmd-;",
            description: "Find Misspelled Words",
            category: ShortcutCategory::Search,
        },
        // -- Window -----------------------------------------------------------
        StandardShortcut {
            keys: "cmd-w",
            description: "Close Window",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-shift-w",
            description: "Close File and Windows",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-alt-w",
            description: "Close All Windows",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-m",
            description: "Minimize",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-alt-m",
            description: "Minimize All",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "ctrl-cmd-f",
            description: "Enter Full Screen",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-`",
            description: "Next Window",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-shift-`",
            description: "Previous Window",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-tab",
            description: "Next App",
            category: ShortcutCategory::Window,
        },
        StandardShortcut {
            keys: "cmd-shift-tab",
            description: "Previous App",
            category: ShortcutCategory::Window,
        },
        // -- Application ------------------------------------------------------
        StandardShortcut {
            keys: "cmd-q",
            description: "Quit",
            category: ShortcutCategory::Application,
        },
        StandardShortcut {
            keys: "cmd-h",
            description: "Hide App",
            category: ShortcutCategory::Application,
        },
        StandardShortcut {
            keys: "cmd-alt-h",
            description: "Hide Others",
            category: ShortcutCategory::Application,
        },
        StandardShortcut {
            keys: "cmd-,",
            description: "Settings",
            category: ShortcutCategory::Application,
        },
        StandardShortcut {
            keys: "cmd-?",
            description: "Help",
            category: ShortcutCategory::Application,
        },
        // -- Document ---------------------------------------------------------
        StandardShortcut {
            keys: "cmd-n",
            description: "New Document",
            category: ShortcutCategory::Document,
        },
        StandardShortcut {
            keys: "cmd-o",
            description: "Open",
            category: ShortcutCategory::Document,
        },
        StandardShortcut {
            keys: "cmd-s",
            description: "Save",
            category: ShortcutCategory::Document,
        },
        StandardShortcut {
            keys: "cmd-shift-s",
            description: "Save As / Duplicate",
            category: ShortcutCategory::Document,
        },
        StandardShortcut {
            keys: "cmd-p",
            description: "Print",
            category: ShortcutCategory::Document,
        },
        StandardShortcut {
            keys: "cmd-shift-p",
            description: "Page Setup",
            category: ShortcutCategory::Document,
        },
        // -- Navigation -------------------------------------------------------
        StandardShortcut {
            keys: "tab",
            description: "Next Field",
            category: ShortcutCategory::Navigation,
        },
        StandardShortcut {
            keys: "shift-tab",
            description: "Previous Field",
            category: ShortcutCategory::Navigation,
        },
        StandardShortcut {
            keys: "escape",
            description: "Cancel / Dismiss",
            category: ShortcutCategory::Navigation,
        },
        StandardShortcut {
            keys: "cmd-.",
            description: "Cancel Operation",
            category: ShortcutCategory::Navigation,
        },
        StandardShortcut {
            keys: "ctrl-tab",
            description: "Next Tab / Group",
            category: ShortcutCategory::Navigation,
        },
        StandardShortcut {
            keys: "ctrl-shift-tab",
            description: "Previous Tab / Group",
            category: ShortcutCategory::Navigation,
        },
        // -- Selection & Cursor Movement --------------------------------------
        StandardShortcut {
            keys: "cmd-right",
            description: "End of Line",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "cmd-left",
            description: "Beginning of Line",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "cmd-up",
            description: "Beginning of Document",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "cmd-down",
            description: "End of Document",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "alt-right",
            description: "Next Word",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "alt-left",
            description: "Previous Word",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-right",
            description: "Extend Selection Right",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-left",
            description: "Extend Selection Left",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-cmd-right",
            description: "Select to End of Line",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-cmd-left",
            description: "Select to Start of Line",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-cmd-up",
            description: "Select to Beginning",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-cmd-down",
            description: "Select to End",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-alt-right",
            description: "Select to End of Word",
            category: ShortcutCategory::Selection,
        },
        StandardShortcut {
            keys: "shift-alt-left",
            description: "Select to Start of Word",
            category: ShortcutCategory::Selection,
        },
        // -- System -----------------------------------------------------------
        StandardShortcut {
            keys: "cmd-shift-3",
            description: "Screenshot (Full)",
            category: ShortcutCategory::System,
        },
        StandardShortcut {
            keys: "cmd-shift-4",
            description: "Screenshot (Selection)",
            category: ShortcutCategory::System,
        },
        StandardShortcut {
            keys: "cmd-space",
            description: "Spotlight",
            category: ShortcutCategory::System,
        },
        StandardShortcut {
            keys: "cmd-f5",
            description: "VoiceOver Toggle",
            category: ShortcutCategory::System,
        },
        StandardShortcut {
            keys: "f11",
            description: "Show Desktop",
            category: ShortcutCategory::System,
        },
    ]
}

/// Modifier keys per HIG.
///
/// When listing modifier keys in a shortcut, always use this order:
/// Control, Option, Shift, Command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModifierKey {
    /// Control -- avoid as primary modifier (used by system)
    Control,
    /// Option -- use sparingly for less-common commands
    Option,
    /// Shift -- secondary modifier that complements a related shortcut
    Shift,
    /// Command -- prefer as primary modifier
    Command,
}

/// A typed keyboard-shortcut binding rendered as SF Symbol glyph sequences.
///
/// Replaces freeform `"Cmd+D"` strings in menu / shortcut columns with a
/// structured value that renders each modifier + key as a monospaced glyph.
/// The `render()` method returns the canonical macOS rendering:
/// `⌃⌥⇧⌘K` — modifiers in HIG order, then the key.
///
/// # Example
/// ```
/// use tahoe_gpui::foundations::keyboard_shortcuts::{MenuShortcut, ModifierKey};
/// let binding = MenuShortcut::new("D").with_modifiers(&[ModifierKey::Command]);
/// assert_eq!(binding.render(), "\u{2318}D");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuShortcut {
    /// Modifiers in any order — rendered in HIG order
    /// (Control, Option, Shift, Command) regardless of construction order.
    pub modifiers: Vec<ModifierKey>,
    /// The key character or named key (e.g. "D", "Enter", "Return").
    /// Single ASCII letters are upper-cased on display.
    pub key: String,
}

impl MenuShortcut {
    /// Create a shortcut with the given key and no modifiers.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            modifiers: Vec::new(),
            key: key.into(),
        }
    }

    /// Replace the modifier set (order does not matter; display order is
    /// enforced in `render()`).
    pub fn with_modifiers(mut self, modifiers: &[ModifierKey]) -> Self {
        self.modifiers = modifiers.to_vec();
        self
    }

    /// Convenience constructor: Command + key.
    pub fn cmd(key: impl Into<String>) -> Self {
        Self::new(key).with_modifiers(&[ModifierKey::Command])
    }

    /// Convenience constructor: Command + Shift + key.
    pub fn cmd_shift(key: impl Into<String>) -> Self {
        Self::new(key).with_modifiers(&[ModifierKey::Command, ModifierKey::Shift])
    }

    /// Convenience constructor: Command + Option + key.
    pub fn cmd_alt(key: impl Into<String>) -> Self {
        Self::new(key).with_modifiers(&[ModifierKey::Command, ModifierKey::Option])
    }

    /// Render the shortcut as an SF-Symbol modifier glyph sequence:
    /// `⌃⌥⇧⌘K`. Modifiers are sorted into HIG order before rendering.
    pub fn render(&self) -> String {
        let mut mods = self.modifiers.clone();
        mods.sort_by_key(|m| m.order());
        mods.dedup();
        let mut out = String::new();
        for m in &mods {
            out.push_str(m.symbol());
        }
        // Upper-case single-letter ASCII keys; leave named keys as-is.
        if self.key.chars().count() == 1 {
            if let Some(c) = self.key.chars().next() {
                out.extend(c.to_uppercase());
                return out;
            }
        }
        out.push_str(&self.key);
        out
    }
}

impl std::fmt::Display for MenuShortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}

impl From<&str> for MenuShortcut {
    /// Parse a shortcut string like `"Cmd+Shift+D"` or `"Cmd-D"`.
    /// Tokens are split on `+` or `-`; the last token becomes the key,
    /// the rest map to modifiers (case-insensitive). Unknown tokens are
    /// silently dropped.
    fn from(raw: &str) -> Self {
        let parts: Vec<&str> = raw
            .split(['+', '-'])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        let (key_token, mod_tokens) = match parts.as_slice() {
            [] => ("", &[] as &[&str]),
            [rest @ .., last] => (*last, rest),
        };
        let mut modifiers = Vec::new();
        for m in mod_tokens {
            let lower = m.to_ascii_lowercase();
            match lower.as_str() {
                "cmd" | "command" | "meta" | "super" | "\u{2318}" => {
                    modifiers.push(ModifierKey::Command)
                }
                "shift" | "\u{21E7}" => modifiers.push(ModifierKey::Shift),
                "alt" | "option" | "opt" | "\u{2325}" => modifiers.push(ModifierKey::Option),
                "ctrl" | "control" | "\u{2303}" => modifiers.push(ModifierKey::Control),
                _ => {}
            }
        }
        MenuShortcut::new(key_token).with_modifiers(&modifiers)
    }
}

impl From<String> for MenuShortcut {
    fn from(raw: String) -> Self {
        MenuShortcut::from(raw.as_str())
    }
}

impl From<gpui::SharedString> for MenuShortcut {
    fn from(raw: gpui::SharedString) -> Self {
        MenuShortcut::from(raw.as_ref())
    }
}

impl ModifierKey {
    /// Returns the HIG symbol for this modifier key.
    pub fn symbol(self) -> &'static str {
        match self {
            Self::Control => "\u{2303}",
            Self::Option => "\u{2325}",
            Self::Shift => "\u{21E7}",
            Self::Command => "\u{2318}",
        }
    }

    /// Returns the correct ordering index per HIG.
    /// Control=0, Option=1, Shift=2, Command=3.
    pub fn order(self) -> u8 {
        match self {
            Self::Control => 0,
            Self::Option => 1,
            Self::Shift => 2,
            Self::Command => 3,
        }
    }
}

/// The shortcut categories owned by the host application — `KeyBinding`
/// entries in these categories MUST be wired by the host, not the crate.
pub const HOST_OWNED_CATEGORIES: &[ShortcutCategory] = &[
    ShortcutCategory::Application,
    ShortcutCategory::Window,
    ShortcutCategory::System,
];

/// Returns the set of keystrokes a component-level binding must never claim
/// at global scope.
///
/// Useful in component test suites that want a one-line "regression gate"
/// confirming a component hasn't started hijacking `cmd-q`, `cmd-w`, etc.
/// See the module-level doc for the rationale.
pub fn host_owned_keys() -> Vec<&'static str> {
    standard_shortcuts()
        .iter()
        .filter(|entry| HOST_OWNED_CATEGORIES.contains(&entry.category))
        .map(|entry| entry.keys)
        .collect()
}

#[cfg(test)]
mod menu_shortcut_tests {
    use super::{MenuShortcut, ModifierKey};
    use core::prelude::v1::test;

    #[test]
    fn cmd_c_renders_as_command_c() {
        let s = MenuShortcut::cmd("C");
        assert_eq!(s.render(), "\u{2318}C");
    }

    #[test]
    fn lowercase_key_is_uppercased() {
        let s = MenuShortcut::cmd("c");
        assert_eq!(s.render(), "\u{2318}C");
    }

    #[test]
    fn modifiers_sort_into_hig_order() {
        let s = MenuShortcut::new("K").with_modifiers(&[
            ModifierKey::Command,
            ModifierKey::Control,
            ModifierKey::Shift,
        ]);
        assert_eq!(s.render(), "\u{2303}\u{21E7}\u{2318}K");
    }

    #[test]
    fn duplicate_modifiers_deduped() {
        let s =
            MenuShortcut::new("S").with_modifiers(&[ModifierKey::Command, ModifierKey::Command]);
        assert_eq!(s.render(), "\u{2318}S");
    }

    #[test]
    fn named_keys_render_verbatim() {
        let s = MenuShortcut::cmd_shift("Return");
        assert_eq!(s.render(), "\u{21E7}\u{2318}Return");
    }

    #[test]
    fn display_matches_render() {
        let s = MenuShortcut::cmd("V");
        assert_eq!(format!("{}", s), s.render());
    }
}

#[cfg(test)]
mod host_ownership_tests {
    use super::{HOST_OWNED_CATEGORIES, host_owned_keys, standard_shortcuts};
    use core::prelude::v1::test;

    #[test]
    fn host_owned_keys_covers_quit_and_settings() {
        let keys = host_owned_keys();
        assert!(keys.contains(&"cmd-q"), "cmd-q should be host-owned");
        assert!(keys.contains(&"cmd-,"), "cmd-, should be host-owned");
        assert!(keys.contains(&"cmd-w"), "cmd-w should be host-owned");
    }

    #[test]
    fn component_categories_excluded_from_host_list() {
        let keys = host_owned_keys();
        // Editing and Selection are component-scoped — e.g. cmd-c is bound
        // inside the TextField key context. They must NOT appear here.
        assert!(!keys.contains(&"cmd-c"), "cmd-c is component-scoped");
        assert!(!keys.contains(&"cmd-a"), "cmd-a is component-scoped");
    }

    #[test]
    fn every_host_owned_entry_lives_in_host_category() {
        for entry in standard_shortcuts() {
            if host_owned_keys().contains(&entry.keys) {
                assert!(
                    HOST_OWNED_CATEGORIES.contains(&entry.category),
                    "{} should be classified into a host-owned category",
                    entry.keys
                );
            }
        }
    }
}
