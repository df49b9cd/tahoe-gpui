//! Undo and redo pattern (HIG Undo and redo).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/undo-and-redo>
//!
//! # Where the runtime lives
//!
//! The canvas-level implementation is in
//! [`crate::workflow::canvas`]: it owns an [`UndoStack`] of
//! `CanvasCommand` values and wires `Cmd+Z` / `Shift+Cmd+Z` plus optional
//! toolbar buttons (via [`crate::workflow::WorkflowControls::show_undo_redo`]).
//! Findings F6 and F7 in issue #149 drove that work.
//!
//! # When to reach for this pattern
//!
//! Every content-modifying user action is a candidate: node moves,
//! deletions, connections, duplicates. HIG specifically calls out three
//! expectations:
//!
//! 1. **Multi-level undo** — not just the most recent operation. The
//!    canvas default is 200 entries, matching Keynote's session depth.
//! 2. **Named undo entries** — the menu should say "Undo Move" rather
//!    than "Undo". `CanvasCommand::label()` supplies those.
//! 3. **Discoverable toolbar buttons** — the HIG explicitly notes
//!    "Consider adding Undo and Redo buttons to your toolbar for
//!    content-editing contexts." `WorkflowControls::show_undo_redo(true)`
//!    turns them on; pass the canvas's `can_undo()` / `can_redo()` so
//!    the buttons disable correctly when the stack is empty.
//!
//! # Host integration notes
//!
//! Hosts that mirror canvas state in an external model (persistent DB,
//! collaborative CRDT) should register both halves of the lifecycle —
//! `set_on_nodes_delete` and `set_on_nodes_restore` — so undo restores
//! don't leave the host model out of sync. The canvas fires
//! `on_nodes_restore` / `on_edges_restore` *after* it has resurrected
//! the entities so host reads see a consistent world.
