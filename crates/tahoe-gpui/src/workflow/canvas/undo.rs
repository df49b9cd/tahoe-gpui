//! Undo/redo support for the workflow canvas.
//!
//! Implements a bounded two-stack undo history (HIG §Undo and redo),
//! so that every user-initiated mutation — moves, deletes, connect — has a
//! reversible shadow. Each command carries just enough data to both apply
//! and revert itself; the canvas never mutates `nodes` / `connections`
//! directly from event handlers anymore, it pushes a command and dispatches.
//!
//! # Architecture (answers Open Question 3 from #149)
//!
//! The stack lives **inside** `WorkflowCanvas`. Findings F6/F7 called out
//! that `delete_selected` mutates state with zero reversibility, and the
//! HIG mandates multi-level undo on macOS; keeping the stack internal means
//! every host gets that out of the box without wiring. Hosts that own a
//! richer model (persistent DB, collaborative CRDT) can still observe
//! mutations via the existing `on_nodes_delete` / `on_edges_delete` /
//! `on_connect` callbacks — those fire during both the initial command and
//! its replay, keeping the host model consistent with canvas-local undo.
//!
//! Entities are retained on the stack rather than snapshot-then-rebuilt,
//! because `WorkflowNode` holds non-`Clone` closures (`content_builder`,
//! `toolbar_builder`, `on_select`). Storing the `Entity<WorkflowNode>`
//! handle keeps those callbacks intact across undo/redo round-trips.

use gpui::Entity;

use super::super::connection::Connection;
use super::super::node::WorkflowNode;

/// One reversible canvas mutation.
///
/// Each variant stores the data required to both apply the forward action
/// and revert it. Entity handles are retained on the stack; deleted nodes
/// are parked in the command rather than dropped.
#[allow(dead_code)]
pub(super) enum CanvasCommand {
    /// A node was moved from `from` to `to`.
    Move {
        node_id: String,
        from: (f32, f32),
        to: (f32, f32),
    },
    /// Nodes (and any edges incident to them) were deleted.
    ///
    /// Entities are held in the command so undo can splice them back in at
    /// the same indices. Edges are kept as a grouped set so implicitly
    /// removed connections come back in the same undo step.
    DeleteNodes {
        /// Parked entity handles, paired with the index they came from.
        /// Ordered by index ascending so undo can reinsert in order.
        nodes: Vec<(usize, Entity<WorkflowNode>)>,
        /// Edges that were removed as part of the same deletion.
        /// Each tuple keeps the original connection and its original index.
        edges: Vec<(usize, Connection)>,
    },
    /// A connection was added (via port drag).
    AddConnection { connection: Connection },
    /// A connection was removed (explicit edge selection + delete).
    DeleteConnection { index: usize, connection: Connection },
    /// Nodes were duplicated (⌥-drag copy or ⌘D).
    ///
    /// Stores the new entities so redo can re-attach them and undo can pull
    /// them out again.
    AddNodes {
        /// Each entity, paired with the index it was inserted at.
        nodes: Vec<(usize, Entity<WorkflowNode>)>,
    },
    /// A node was resized by dragging one of its handles (F28). Position
    /// and size are captured together because corner/edge handles change
    /// both atomically — reverting only one would leave the node visually
    /// "teleporting" mid-undo.
    Resize {
        node_id: String,
        from_pos: (f32, f32),
        to_pos: (f32, f32),
        from_size: (f32, f32),
        to_size: (f32, f32),
    },
}

impl CanvasCommand {
    /// Human-readable name for the "Undo X" / "Redo X" menu item per HIG.
    pub(super) fn label(&self) -> &'static str {
        match self {
            CanvasCommand::Move { .. } => "Move",
            CanvasCommand::DeleteNodes { .. } => "Delete",
            CanvasCommand::AddConnection { .. } => "Connect",
            CanvasCommand::DeleteConnection { .. } => "Disconnect",
            CanvasCommand::AddNodes { .. } => "Duplicate",
            CanvasCommand::Resize { .. } => "Resize",
        }
    }
}

/// Two-stack undo history with a bounded depth.
///
/// The bound (`capacity`) caps memory for long editing sessions — once the
/// undo stack hits it, the oldest command is discarded. This matches the
/// HIG recommendation of multi-level undo without unbounded growth.
pub(super) struct UndoStack {
    undo: Vec<CanvasCommand>,
    redo: Vec<CanvasCommand>,
    capacity: usize,
}

impl UndoStack {
    /// Default depth. Matches what Keynote and Freeform expose to users
    /// before the "Undo" menu item grays out during a single session.
    pub(super) const DEFAULT_CAPACITY: usize = 200;

    pub(super) fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            capacity: Self::DEFAULT_CAPACITY,
        }
    }

    /// Record a new user-initiated command. Clears any pending redo — the
    /// usual "new branch off history" behaviour so redo never replays a
    /// command that was superseded by fresh user input.
    pub(super) fn push(&mut self, cmd: CanvasCommand) {
        self.redo.clear();
        if self.undo.len() >= self.capacity {
            self.undo.remove(0);
        }
        self.undo.push(cmd);
    }

    /// Pop the most recent command for the canvas to revert. The caller
    /// must push the reverted command onto the redo stack via [`Self::push_redo`].
    pub(super) fn pop_undo(&mut self) -> Option<CanvasCommand> {
        self.undo.pop()
    }

    /// Pop the most recent redo entry for the canvas to re-apply.
    pub(super) fn pop_redo(&mut self) -> Option<CanvasCommand> {
        self.redo.pop()
    }

    /// After a successful undo, the reverted command moves to the redo
    /// stack so Shift-⌘-Z can replay it.
    pub(super) fn push_redo(&mut self, cmd: CanvasCommand) {
        self.redo.push(cmd);
    }

    /// After a successful redo, re-arm the undo path without clearing the
    /// redo stack (which `push` would do).
    pub(super) fn push_undo_after_redo(&mut self, cmd: CanvasCommand) {
        if self.undo.len() >= self.capacity {
            self.undo.remove(0);
        }
        self.undo.push(cmd);
    }

    pub(super) fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub(super) fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Peek at the top of the undo stack to surface the label in a menu.
    /// Returns `None` when the stack is empty so callers can render
    /// "Undo" rather than "Undo _".
    pub(super) fn peek_undo_label(&self) -> Option<&'static str> {
        self.undo.last().map(CanvasCommand::label)
    }

    pub(super) fn peek_redo_label(&self) -> Option<&'static str> {
        self.redo.last().map(CanvasCommand::label)
    }

    /// Drop all history — used by tests and by `reset_view` style
    /// full-replace operations where a partial undo would produce a
    /// confusing partial state.
    #[cfg(test)]
    pub(super) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    #[cfg(test)]
    pub(super) fn undo_len(&self) -> usize {
        self.undo.len()
    }

    #[cfg(test)]
    pub(super) fn redo_len(&self) -> usize {
        self.redo.len()
    }
}

impl Default for UndoStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    fn sample_move(id: &str) -> CanvasCommand {
        CanvasCommand::Move {
            node_id: id.to_string(),
            from: (0.0, 0.0),
            to: (10.0, 10.0),
        }
    }

    #[test]
    fn push_appends_and_clears_redo() {
        let mut s = UndoStack::new();
        s.push(sample_move("n1"));
        s.push_redo(sample_move("redo-before"));
        assert!(s.can_redo());

        s.push(sample_move("n2"));
        assert!(!s.can_redo(), "new push must clear redo stack");
        assert_eq!(s.undo_len(), 2);
    }

    #[test]
    fn pop_undo_returns_newest_first() {
        let mut s = UndoStack::new();
        s.push(sample_move("a"));
        s.push(sample_move("b"));

        match s.pop_undo().expect("expected b") {
            CanvasCommand::Move { node_id, .. } => assert_eq!(node_id, "b"),
            _ => panic!("wrong variant"),
        }
        match s.pop_undo().expect("expected a") {
            CanvasCommand::Move { node_id, .. } => assert_eq!(node_id, "a"),
            _ => panic!("wrong variant"),
        }
        assert!(s.pop_undo().is_none());
    }

    #[test]
    fn capacity_drops_oldest() {
        let mut s = UndoStack {
            undo: Vec::new(),
            redo: Vec::new(),
            capacity: 3,
        };
        s.push(sample_move("a"));
        s.push(sample_move("b"));
        s.push(sample_move("c"));
        s.push(sample_move("d"));
        assert_eq!(s.undo_len(), 3);
        // "a" should be dropped; the bottom should now be "b".
        let first = s.pop_undo().unwrap();
        let _ = first; // top = d
        let _ = s.pop_undo().unwrap(); // c
        match s.pop_undo().expect("should have b") {
            CanvasCommand::Move { node_id, .. } => assert_eq!(node_id, "b"),
            _ => panic!("wrong variant"),
        }
        assert!(s.pop_undo().is_none());
    }

    #[test]
    fn push_redo_preserves_undo_path() {
        let mut s = UndoStack::new();
        s.push(sample_move("a"));
        let popped = s.pop_undo().unwrap();
        s.push_redo(popped);
        assert!(s.can_redo());
        assert!(!s.can_undo());
        assert_eq!(s.redo_len(), 1);

        let popped_redo = s.pop_redo().unwrap();
        s.push_undo_after_redo(popped_redo);
        assert!(s.can_undo());
        assert!(!s.can_redo());
        assert_eq!(s.redo_len(), 0);
    }

    #[test]
    fn push_undo_after_redo_preserves_redo_stack() {
        // `push_undo_after_redo` is the "don't clear redo" variant used
        // during a redo round-trip so the user can keep pressing Shift-⌘-Z.
        let mut s = UndoStack::new();
        s.push(sample_move("a"));
        s.push(sample_move("b"));
        let popped_b = s.pop_undo().unwrap();
        s.push_redo(popped_b);
        let popped_a = s.pop_undo().unwrap();
        s.push_redo(popped_a);
        assert_eq!(s.redo_len(), 2);

        // Now redo "a" — it goes back on undo, redo stack keeps "b".
        let re_a = s.pop_redo().unwrap();
        s.push_undo_after_redo(re_a);
        assert_eq!(s.undo_len(), 1);
        assert_eq!(s.redo_len(), 1);
    }

    #[test]
    fn labels_match_variants() {
        assert_eq!(sample_move("x").label(), "Move");
        assert_eq!(
            CanvasCommand::AddConnection {
                connection: crate::workflow::Connection::new(
                    "c",
                    crate::workflow::PortId::new("a", "o"),
                    crate::workflow::PortId::new("b", "i")
                )
            }
            .label(),
            "Connect"
        );
        assert_eq!(
            CanvasCommand::Resize {
                node_id: "n".into(),
                from_pos: (0.0, 0.0),
                to_pos: (10.0, 10.0),
                from_size: (400.0, 100.0),
                to_size: (500.0, 120.0),
            }
            .label(),
            "Resize"
        );
    }

    #[test]
    fn peek_labels_report_top() {
        let mut s = UndoStack::new();
        assert_eq!(s.peek_undo_label(), None);
        s.push(sample_move("a"));
        assert_eq!(s.peek_undo_label(), Some("Move"));
        s.push_redo(sample_move("b"));
        assert_eq!(s.peek_redo_label(), Some("Move"));
    }

    #[test]
    fn clear_empties_both_stacks() {
        let mut s = UndoStack::new();
        s.push(sample_move("a"));
        s.push_redo(sample_move("b"));
        s.clear();
        assert!(!s.can_undo());
        assert!(!s.can_redo());
    }
}
