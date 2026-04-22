#![recursion_limit = "512"]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/df49b9cd/tahoe-gpui/main/branding/tahoe-gpui-logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/df49b9cd/tahoe-gpui/main/branding/tahoe-gpui-logo.png"
)]
//! Human Interface Guidelines components for GPUI.
//!
//! This crate provides composable UI components structured around the
//! Human Interface Guidelines taxonomy, built on GPUI (Zed's GPU-accelerated
//! UI framework). It is designed to be reusable independently of any AI SDK.
//!
//! # Module Organization (HIG-aligned)
//!
//! - **[`foundations`]**: Design-system primitives aligned with HIG
//!   Foundations. Implemented modules cover accessibility, app icons, color,
//!   dark mode, icons (incl. SF Symbols), images, keyboard, keyboard
//!   shortcuts, layout, Liquid Glass materials, motion, right-to-left
//!   direction, theme tokens, and typography. Additional HIG foundation
//!   pages (branding, privacy, immersive experiences, inclusion, spatial
//!   layout, writing) are present as documented stubs.
//! - **[`components`]**: Concrete UI controls organized by all 8 HIG
//!   subcategories: content, layout & organization, menus & actions,
//!   navigation & search, presentation, selection & input, status, and
//!   system experiences (stub-only today — Widgets, Notifications, Live
//!   Activities, etc.).
//! - **[`patterns`]**: Design patterns aligned with the 25 HIG pattern
//!   pages. `feedback` and `loading` carry real supporting types; the
//!   remaining 23 pages are tracked as stubs so HIG coverage gaps stay
//!   grep-friendly.
//! - **[`markdown`]**: Streaming markdown renderer with syntax highlighting.
//! - **[`code`]**: Code display components (terminal, file tree, stack traces).
//! - **[`context`]**: Token usage display and model identification.
//! - **[`workflow`]**: Graph canvas components (nodes, edges, connections).
//! - **`voice`**: Audio/speech components (behind `voice` feature).
//! - **[`prelude`]**: Flat re-export of the common public surface, ordered
//!   by HIG subcategory.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines>

pub mod callback_types;
pub mod citation;
pub mod code;
pub mod components;
pub mod context;
pub mod foundations;
pub mod ids;
pub mod markdown;
pub mod navigation_actions;
pub mod patterns;
pub mod prelude;
pub mod text_actions;
pub mod workflow;

#[cfg(feature = "voice")]
pub mod voice;

#[cfg(test)]
pub(crate) mod test_helpers;

pub use foundations::theme::TahoeTheme;

// ── Virtualized list primitives (Finding 6 in the Zed cross-reference audit) ────
//
// Re-exports of GPUI's variable-height `list` element and its `ListState`
// companion so consumers can assemble virtualized outlines / file trees
// without depending on gpui directly. See
// [`components::layout_and_organization::outline_view::OutlineView`] for
// the pattern. `list_element` is named to avoid colliding with Rust
// module-path confusion around the bare function name `list`.
pub use gpui::ListState;
pub use gpui::list as list_element;

/// Returns the crate's shared **modifier-key** text editing bindings
/// (Cmd+C, Alt+Left, Shift+Right, etc.). These are safe to install
/// globally because they don't conflict with component-scoped raw keys.
///
/// The returned set is a strict superset of [`mandatory_keybindings`], so
/// hosts that install this do not additionally need to install the
/// mandatory set.
///
/// ```ignore
/// cx.bind_keys(tahoe_gpui::text_keybindings());
/// ```
pub fn text_keybindings() -> Vec<gpui::KeyBinding> {
    text_actions::keybindings()
}

/// Returns the HIG-mandated keybindings every macOS host MUST install.
///
/// Currently this is `Cmd-Z` → Undo and `Cmd-Shift-Z` → Redo (HIG §Undo
/// and redo: "On macOS, Undo (Command-Z) and Redo (Shift-Command-Z) are
/// expected keyboard shortcuts"). Undo/Redo is declared under the
/// `text_editing` action namespace, but HIG mandates the shortcut for
/// *any* content-editing app — not only those embedding
/// [`TextField`](crate::components::selection_and_input::TextField).
/// Hosts that consume only a canvas, button gallery, or other non-text
/// component must still install this set, otherwise Undo/Redo is
/// unreachable and the app fails the HIG contract.
///
/// ```ignore
/// // Minimum HIG compliance — call this even without TextField.
/// cx.bind_keys(tahoe_gpui::mandatory_keybindings());
/// ```
///
/// Hosts that already register [`text_keybindings`] or
/// [`all_keybindings`] receive these entries via the superset and don't
/// need a second call.
pub fn mandatory_keybindings() -> Vec<gpui::KeyBinding> {
    text_actions::mandatory_keybindings()
}

/// Returns the raw-key bindings scoped to `TextField` (Backspace, Delete,
/// arrow keys). Only needed by apps that embed a `TextField`.
///
/// ```ignore
/// cx.bind_keys(tahoe_gpui::textfield_keybindings());
/// ```
pub fn textfield_keybindings() -> Vec<gpui::KeyBinding> {
    components::selection_and_input::text_field::keybindings()
}

/// Returns the bindings scoped to `TextView` (Cmd+C, Cmd+A). Only needed
/// by apps that embed a [`TextView`](crate::components::content::TextView).
///
/// ```ignore
/// cx.bind_keys(tahoe_gpui::textview_keybindings());
/// ```
pub fn textview_keybindings() -> Vec<gpui::KeyBinding> {
    components::content::text_view::keybindings()
}

/// Returns all standard HIG keybindings for the crate's components.
///
/// Includes:
/// - Modifier-key text editing shortcuts (Cmd+C, Alt+Left, Shift+Right, etc.)
/// - TextField-scoped raw key bindings (Backspace, Delete, arrow keys)
/// - TextView-scoped bindings (Cmd+C, Cmd+A)
///
/// Prefer [`text_keybindings`], [`textfield_keybindings`], or
/// [`textview_keybindings`] when you only embed a subset of the
/// components.
///
/// ```ignore
/// cx.bind_keys(tahoe_gpui::all_keybindings());
/// ```
pub fn all_keybindings() -> Vec<gpui::KeyBinding> {
    let mut bindings = text_keybindings();
    bindings.extend(textfield_keybindings());
    bindings.extend(textview_keybindings());
    bindings
}
