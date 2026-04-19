//! Node selection management for the workflow canvas.
//!
//! Also owns the high-level mutation helpers — `delete_selected`,
//! `duplicate_selected`, `nudge_selected`, `extend_selection` — because
//! every one of them lands on the undo stack and shares the same
//! invariant: collect snapshots → fire host callbacks → mutate → push
//! `CanvasCommand`. Centralising them here keeps the command-dispatch
//! order consistent and makes F5/F6/F4 easier to audit against the HIG
//! rules they implement.

use std::collections::HashSet;

use gpui::{Entity, Window};
use gpui::prelude::*;

use super::WorkflowCanvas;
use super::undo::CanvasCommand;
use super::super::connection::Connection;
use super::super::node::WorkflowNode;

impl WorkflowCanvas {
    pub fn selected_nodes(&self) -> &HashSet<usize> {
        &self.selected_nodes
    }

    pub fn select_node(
        &mut self,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        for &prev in &self.selected_nodes {
            if let Some(node) = self.nodes.get(prev) {
                node.update(cx, |node, cx| node.set_selected(false, window, cx));
            }
        }
        self.selected_nodes.clear();

        if let Some(idx) = index {
            if let Some(node) = self.nodes.get(idx) {
                node.update(cx, |node, cx| node.set_selected(true, window, cx));
            }
            self.selected_nodes.insert(idx);
        }

        if let Some(ref handler) = self.on_node_select {
            handler(index, window, cx);
        }
        cx.notify();
    }

    pub fn toggle_node_selection(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_nodes.contains(&index) {
            self.selected_nodes.remove(&index);
            if let Some(node) = self.nodes.get(index) {
                node.update(cx, |node, cx| node.set_selected(false, window, cx));
            }
        } else {
            self.selected_nodes.insert(index);
            if let Some(node) = self.nodes.get(index) {
                node.update(cx, |node, cx| node.set_selected(true, window, cx));
            }
        }
        cx.notify();
    }

    /// F26 (#149): cycle keyboard focus between nodes. Tab (forward) and
    /// Shift+Tab (backward) move the single-node selection through the
    /// node list so the user can navigate a graph without a pointer.
    ///
    /// This is the keyboard half of F26 — it works today without any
    /// GPUI accessibility-tree API. The VoiceOver half, which announces
    /// "Node 2 of 5, selected" on each Tab, still waits on upstream
    /// `accessibility_label` support (see `foundations::accessibility`).
    pub fn cycle_node_focus(
        &mut self,
        forward: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.nodes.is_empty() {
            return;
        }
        let current = self
            .selected_nodes
            .iter()
            .copied()
            .min() // deterministic when multi-selected
            .unwrap_or(usize::MAX);
        let len = self.nodes.len();
        let next = if current == usize::MAX {
            if forward { 0 } else { len - 1 }
        } else if forward {
            (current + 1) % len
        } else {
            (current + len - 1) % len
        };
        self.select_node(Some(next), window, cx);
    }

    /// F5 (#149): Shift-click on a node adds it to the current selection
    /// without dropping existing members. Distinct from `toggle_node_selection`
    /// (Cmd-click): Shift never *removes* a node that was already part of
    /// the selection — matching Finder / Keynote "extend" semantics.
    pub fn extend_selection(
        &mut self,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.selected_nodes.contains(&index) {
            self.selected_nodes.insert(index);
            if let Some(node) = self.nodes.get(index) {
                node.update(cx, |node, cx| node.set_selected(true, window, cx));
            }
            cx.notify();
        }
    }

    /// F3 (#149): nudge every selected node by `(dx, dy)` world units.
    /// Lands on the undo stack as a single batched Move per node so one
    /// Cmd-Z reverses the whole arrow-key press even across a multi-select.
    pub fn nudge_selected(
        &mut self,
        dx: f32,
        dy: f32,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_nodes.is_empty() || (dx == 0.0 && dy == 0.0) {
            return;
        }
        // Snapshot before mutation so each Move command has an accurate from/to.
        let moves: Vec<(String, (f32, f32), (f32, f32))> = self
            .selected_nodes
            .iter()
            .filter_map(|&idx| self.nodes.get(idx))
            .map(|entity| {
                let n = entity.read(cx);
                let id = n.id().to_string();
                let from = n.position();
                let to = (from.0 + dx, from.1 + dy);
                (id, from, to)
            })
            .collect();

        for (id, _from, to) in &moves {
            self.set_node_position_by_id(id, *to, cx);
        }
        for (id, from, to) in moves {
            self.push_history(CanvasCommand::Move {
                node_id: id,
                from,
                to,
            });
        }
        cx.notify();
    }

    /// F4 (#149): duplicate every selected node via the host's factory
    /// callback. No-op when `on_node_duplicate` isn't set (see OQ6).
    pub fn duplicate_selected(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_nodes.is_empty() || self.on_node_duplicate.is_none() {
            return;
        }
        // HIG / Freeform convention: offset the duplicate by (+20, +20)
        // so it's visibly distinct from the original.
        const OFFSET: (f32, f32) = (20.0, 20.0);
        let sources: Vec<(Entity<WorkflowNode>, (f32, f32))> = self
            .selected_nodes
            .iter()
            .filter_map(|&idx| self.nodes.get(idx))
            .map(|e| {
                let pos = e.read(cx).position();
                (e.clone(), (pos.0 + OFFSET.0, pos.1 + OFFSET.1))
            })
            .collect();

        // Take the Option out so we can borrow self mutably inside the loop
        // without running the closure through &mut self.
        let factory = self.on_node_duplicate.take();
        let mut created: Vec<(usize, Entity<WorkflowNode>)> = Vec::new();
        if let Some(ref factory) = factory {
            for (src, target_pos) in sources {
                if let Some(new_entity) = factory(&src, target_pos, window, cx) {
                    // Ensure the factory-provided entity is positioned where
                    // we want it, in case the host didn't apply the offset.
                    new_entity.update(cx, |n, cx| {
                        n.set_position(target_pos.0, target_pos.1, cx);
                    });
                    let idx = self.nodes.len();
                    self.nodes.push(new_entity.clone());
                    created.push((idx, new_entity));
                }
            }
        }
        self.on_node_duplicate = factory;
        if !created.is_empty() {
            // Select the duplicates so the user can immediately move them.
            let new_indices: Vec<usize> = created.iter().map(|(i, _)| *i).collect();
            // Clear old selection UI state.
            for &prev in &self.selected_nodes.clone() {
                if let Some(node) = self.nodes.get(prev) {
                    node.update(cx, |n, cx| n.set_selected(false, window, cx));
                }
            }
            self.selected_nodes.clear();
            for idx in new_indices {
                if let Some(node) = self.nodes.get(idx) {
                    node.update(cx, |n, cx| n.set_selected(true, window, cx));
                }
                self.selected_nodes.insert(idx);
            }
            self.push_history(CanvasCommand::AddNodes { nodes: created });
            cx.notify();
        }
    }

    pub fn delete_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_nodes.is_empty() && self.selected_edges.is_empty() {
            return;
        }

        // Collect all connection IDs to delete BEFORE any mutation.
        let mut conn_ids_to_delete: HashSet<String> = HashSet::new();

        // Explicitly selected edges.
        for &idx in &self.selected_edges {
            if let Some(conn) = self.connections.get(idx) {
                conn_ids_to_delete.insert(conn.id.clone());
            }
        }

        // Edges implicitly removed by node deletion.
        let deleted_node_ids: HashSet<String> = self
            .selected_nodes
            .iter()
            .filter_map(|&idx| self.nodes.get(idx).map(|n| n.read(cx).id().to_string()))
            .collect();

        if !deleted_node_ids.is_empty() {
            for conn in &self.connections {
                if deleted_node_ids.contains(&conn.source.node_id)
                    || deleted_node_ids.contains(&conn.target.node_id)
                {
                    conn_ids_to_delete.insert(conn.id.clone());
                }
            }
        }

        // Fire callbacks BEFORE mutation, while indices and data are still valid.
        if !self.selected_nodes.is_empty() {
            let deleted_indices: Vec<usize> = self.selected_nodes.iter().copied().collect();
            if let Some(ref handler) = self.on_nodes_delete {
                handler(&deleted_indices, window, cx);
            }
        }

        if !conn_ids_to_delete.is_empty() {
            let ids: Vec<String> = conn_ids_to_delete.iter().cloned().collect();
            if let Some(ref handler) = self.on_edges_delete {
                handler(&ids, window, cx);
            }
        }

        // Capture (index, entity) for nodes and (index, connection) for edges
        // so the DeleteNodes command on the undo stack can splice them back
        // in at the same position (F6).
        let mut sorted_node_indices: Vec<usize> = self.selected_nodes.iter().copied().collect();
        sorted_node_indices.sort_unstable();
        let mut deleted_node_entities: Vec<(usize, Entity<WorkflowNode>)> = Vec::new();
        // Remove in reverse order so lower indices stay stable during removal,
        // then reverse the captured vec so it's ordered ascending (the
        // invariant DeleteNodes expects for apply_reverse).
        for idx in sorted_node_indices.iter().rev() {
            if *idx < self.nodes.len() {
                deleted_node_entities.push((*idx, self.nodes.remove(*idx)));
            }
        }
        deleted_node_entities.reverse();

        let mut deleted_edges: Vec<(usize, Connection)> = Vec::new();
        // Walk in reverse so indices remain valid as we remove.
        let mut remaining: Vec<Connection> = Vec::with_capacity(self.connections.len());
        let taken: Vec<Connection> = std::mem::take(&mut self.connections);
        for (idx, conn) in taken.into_iter().enumerate() {
            if conn_ids_to_delete.contains(&conn.id) {
                deleted_edges.push((idx, conn));
            } else {
                remaining.push(conn);
            }
        }
        self.connections = remaining;

        self.selected_nodes.clear();
        self.selected_edges.clear();

        if !deleted_node_entities.is_empty() || !deleted_edges.is_empty() {
            self.push_history(CanvasCommand::DeleteNodes {
                nodes: deleted_node_entities,
                edges: deleted_edges,
            });
        }

        let _ = window;
        cx.notify();
    }
}
