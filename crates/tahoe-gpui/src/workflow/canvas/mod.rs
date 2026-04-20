//! Workflow canvas component.
//!
//! A pannable, zoomable container that holds workflow nodes and draws edges
//! between them based on connection data. Matches the AI SDK Elements Canvas
//! interaction model: scroll-to-pan, Ctrl/Cmd+scroll to zoom, drag-to-select,
//! and Delete/Backspace to remove selected nodes.
//!
//! # HIG coverage
//!
//! This module aligns with the macOS Tahoe HIG pages for Drag and drop,
//! Gestures, Pointing devices, Keyboards, Undo and redo, and Materials.
//! Concrete call sites: [`WorkflowCanvas::handle_key_down`] for ⌘+/⌘−/⌘0
//! zoom and arrow-key nudge, [`super::controls::WorkflowControls::show_undo_redo`]
//! for toolbar undo/redo, [`undo::UndoStack`] + [`undo::CanvasCommand`]
//! for multi-level undo, [`PORT_HIT_RADIUS_SCREEN_PX`] +
//! [`EDGE_HIT_TOLERANCE_SCREEN_PX`] for 44 pt HIG hit targets, and the
//! [`resize`] module for resize-handle geometry. `WorkflowToolbar` and
//! `NodeToolbar` render on Liquid Glass per HIG Materials.
//!
//! ## Upstream-blocked accessibility
//!
//! GPUI `v0.231.1-pre` exposes no accessibility / AX tree API (grep of
//! `crates/gpui/src/**` for `accessibility`, `AXRole`, or `NSAccessibility`
//! yields no matches — verified in the working tree). The crate carries
//! [`crate::foundations::accessibility::AccessibilityProps`] on every
//! labelled element so the day GPUI lands the API, the single
//! `AccessibleExt::with_accessibility` site will wire labels / roles /
//! values without per-component changes. Keyboard graph navigation —
//! Tab / Shift-Tab — is implemented in `cycle_node_focus` and works
//! today.

mod viewport;
use viewport::compute_fit_zoom_and_center;
mod resize;
use resize::{HANDLE_HIT_RADIUS, HANDLE_VISUAL_SIZE, ResizeHandle, apply_handle_delta, handle_at};
mod selection;
mod undo;
use undo::{CanvasCommand, UndoStack};

use std::collections::HashSet;
use std::time::Instant;

use crate::foundations::theme::ActiveTheme;
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{
    App, Bounds, ClickEvent, CursorStyle, ElementId, Entity, FocusHandle, Focusable, Hsla,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PinchEvent,
    ScrollDelta, ScrollWheelEvent, Window, canvas, div, fill, point, px, size,
};

use super::connection::{Connection, PortId};
use super::connection_line::ConnectionLine;
use super::edge::{EdgeElement, EdgeStyle};
use super::node::{PortType, WorkflowNode};
use super::util::HandlePosition;
use super::util::{point_to_cubic_bezier_distance, point_to_quadratic_bezier_distance};

/// Screen-space radius used for port hit-testing.
///
/// The visual handle (8 px) combined with a 12 px hit tolerance produces a
/// ~24 px effective target — well below the HIG 44 pt minimum for a
/// touch/pointer target. We keep the visual handle small (ports are visually
/// unobtrusive) but expand the hit circle to 22 px screen-space (44 pt
/// diameter) so reach matches Apple's minimum regardless of zoom. The
/// tolerance is applied in screen space so low-zoom sessions don't collapse
/// the effective target.
pub(super) const PORT_HIT_RADIUS_SCREEN_PX: f32 = 22.0;
/// Screen-space radius for edge hit-testing.
///
/// The old 8 px literal was applied after the world→screen transform,
/// meaning zoom changed the *effective canvas-space* tolerance instead of
/// leaving the pointer target constant. 11 px screen-space ≈ 22 px diameter
/// which, for a 1-pt line, is a comfortable 44 pt pointer target along the
/// minor axis without swallowing nearby clicks.
pub(super) const EDGE_HIT_TOLERANCE_SCREEN_PX: f32 = 11.0;
/// Distance (screen px) the pointer must move between mouse-down and mouse-up
/// for a drag to register as a Move command on the undo stack. Stops a bare
/// click-to-select from polluting the history with a zero-delta move.
const MOVE_COMMIT_THRESHOLD: f32 = 1.0;
/// Extra padding (screen px) inside the viewport that triggers auto-pan while
/// dragging a node near the edge. Per HIG Drag and drop §"Scroll contents of
/// destination when necessary".
const AUTO_PAN_MARGIN: f32 = 40.0;
/// Auto-pan velocity in screen-pixels per frame when the drag is pinned to
/// the edge. Small enough that fast mouse movement still feels responsive.
const AUTO_PAN_STEP: f32 = 12.0;

/// In-flight resize operation driven by one of the 8 node handles.
struct ResizeState {
    node_id: String,
    handle: ResizeHandle,
    /// World-space position when the resize started.
    start_pos: (f32, f32),
    /// Size when the resize started. This is the size the canvas
    /// observes; auto-sized nodes seed it with `NODE_MIN_WIDTH × NODE_HEIGHT`
    /// so the first drag still produces a meaningful explicit size.
    start_size: (f32, f32),
    /// World-space mouse position at `mouse_down`. Stored in world space
    /// so zoom changes mid-drag don't skew the delta.
    start_mouse_world: (f32, f32),
}

/// Pre-resolved edge data for rendering.
struct ResolvedEdge {
    index: usize,
    from: (f32, f32),
    to: (f32, f32),
    style: EdgeStyle,
    source_position: HandlePosition,
    target_position: HandlePosition,
    label: Option<String>,
}

/// Default width assumed for node position calculations.
pub(super) const NODE_WIDTH: f32 = 384.0;
/// Default height assumed per node for edge connection points.
pub(super) const NODE_TITLE_HEIGHT: f32 = 32.0;
/// Approximate node height for bounding box calculations.
pub(super) const NODE_HEIGHT: f32 = 80.0;
/// Minimum zoom level for manual zoom operations (set_zoom, scroll, zoom_in/out).
const MIN_ZOOM: f32 = 0.1;
/// Maximum zoom level for manual zoom operations (set_zoom, scroll, zoom_in/out).
/// Note: `FitViewOptions::default().max_zoom` is 2.0 — fit-to-view intentionally
/// caps lower than manual zoom for a less jarring auto-fit experience.
const MAX_ZOOM: f32 = 3.0;

/// A pannable, zoomable canvas that renders workflow nodes and edges.
#[allow(clippy::type_complexity)]
pub struct WorkflowCanvas {
    element_id: ElementId,
    focus_handle: FocusHandle,
    nodes: Vec<Entity<WorkflowNode>>,
    connections: Vec<Connection>,
    pan_offset: (f32, f32),
    zoom: f32,
    selected_nodes: HashSet<usize>,
    dragging_node: bool,
    selection_start: Option<(f32, f32)>,
    selection_end: Option<(f32, f32)>,
    viewport_size: Option<(f32, f32)>,
    has_auto_fitted: bool,
    animation_t: f32,
    last_animation_time: Option<Instant>,
    interactive: bool,
    show_grid: bool,
    grid_spacing: Option<f32>,
    grid_dot_size: Option<f32>,
    selected_edges: HashSet<usize>,
    on_node_select: Option<Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>>,
    on_nodes_delete: Option<Box<dyn Fn(&[usize], &mut Window, &mut App) + 'static>>,
    on_edge_select: Option<Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>>,
    on_edges_delete: Option<Box<dyn Fn(&[String], &mut Window, &mut App) + 'static>>,
    connecting_from: Option<(PortId, PortType, (f32, f32))>,
    connecting_mouse: Option<(f32, f32)>,
    on_connect: Option<Box<dyn Fn(PortId, PortId, &mut Window, &mut App) + 'static>>,
    /// Captured on mouse-down over a node so the Move command gets both ends.
    /// Paired with the node id (indices are unstable across deletes).
    drag_initial_pos: Option<(String, (f32, f32))>,
    /// Active resize state. Populated when the user clicks a resize handle
    /// and cleared on mouse-up. Holds everything needed to compute the
    /// incremental new position / size and build the matching `Resize`
    /// command on commit.
    resizing: Option<ResizeState>,
    /// Index of the node currently under the pointer — drives the hover
    /// cursor and shadow-lift hover hint. `None` when pointer is over empty
    /// canvas or a port / edge hit instead.
    hovered_node: Option<usize>,
    /// Index of the edge currently under the pointer — drives edge hover
    /// thickening. Falls back to `None` when the pointer hits a node or
    /// empty canvas.
    hovered_edge: Option<usize>,
    /// Id of the port currently under the pointer while a connection drag
    /// is in-flight. Drives the drop-zone highlight.
    hovered_port: Option<PortId>,
    /// Which of the 8 resize handles (if any) the pointer sits over —
    /// drives the cursor shape when hovering a selected node's handles.
    /// Set only when `selected_nodes.len() == 1`.
    hovered_resize_handle: Option<ResizeHandle>,
    /// Reversible history. See `undo::UndoStack`.
    history: UndoStack,
    /// Optional callback fired when undo/redo restores nodes the host had
    /// observed as deleted — lets the host re-register them in its source
    /// model. Receives the restored entities.
    on_nodes_restore: Option<Box<dyn Fn(&[Entity<WorkflowNode>], &mut Window, &mut App) + 'static>>,
    /// Optional callback fired when undo/redo restores edges that had fired
    /// an `on_edges_delete` earlier. Host re-adds them to its model.
    on_edges_restore: Option<Box<dyn Fn(&[Connection], &mut Window, &mut App) + 'static>>,
    /// Optional host-provided duplicate factory (⌘D, ⌥-drag copy).
    /// Called per source node; host returns a fresh entity or `None` to
    /// reject the duplicate request.
    #[allow(clippy::type_complexity)]
    on_node_duplicate: Option<
        Box<
            dyn Fn(
                    &Entity<WorkflowNode>,
                    (f32, f32),
                    &mut Window,
                    &mut App,
                ) -> Option<Entity<WorkflowNode>>
                + 'static,
        >,
    >,
}

impl WorkflowCanvas {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("workflow-canvas"),
            focus_handle: cx.focus_handle(),
            nodes: Vec::new(),
            connections: Vec::new(),
            pan_offset: (0.0, 0.0),
            zoom: 1.0,
            selected_nodes: HashSet::new(),
            dragging_node: false,
            selection_start: None,
            selection_end: None,
            viewport_size: None,
            has_auto_fitted: false,
            animation_t: 0.0,
            last_animation_time: None,
            interactive: true,
            show_grid: true,
            grid_spacing: None,
            grid_dot_size: None,
            selected_edges: HashSet::new(),
            on_node_select: None,
            on_nodes_delete: None,
            on_edge_select: None,
            on_edges_delete: None,
            connecting_from: None,
            connecting_mouse: None,
            on_connect: None,
            drag_initial_pos: None,
            resizing: None,
            hovered_node: None,
            hovered_edge: None,
            hovered_port: None,
            hovered_resize_handle: None,
            history: UndoStack::new(),
            on_nodes_restore: None,
            on_edges_restore: None,
            on_node_duplicate: None,
        }
    }

    pub fn add_node(&mut self, node: Entity<WorkflowNode>, cx: &mut Context<Self>) {
        let zoom = self.zoom;
        node.update(cx, |n, _| n.set_viewport_zoom(zoom));
        self.nodes.push(node);
        cx.notify();
    }

    pub fn add_connection(&mut self, connection: Connection, cx: &mut Context<Self>) {
        self.connections.push(connection);
        cx.notify();
    }

    pub fn nodes(&self) -> &[Entity<WorkflowNode>] {
        &self.nodes
    }
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    pub fn set_pan(&mut self, x: f32, y: f32, cx: &mut Context<Self>) {
        self.pan_offset = (x, y);
        cx.notify();
    }

    pub fn pan_offset(&self) -> (f32, f32) {
        self.pan_offset
    }

    pub fn set_zoom(&mut self, zoom: f32, cx: &mut Context<Self>) {
        self.zoom = zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        self.propagate_viewport_zoom(cx);
        cx.notify();
    }

    /// Push the current zoom to every child node so their internal drag
    /// math stays in world coordinates. Cheap: each node's setter is a
    /// single `f32` write + `max`, and we only notify the canvas.
    fn propagate_viewport_zoom(&mut self, cx: &mut Context<Self>) {
        let zoom = self.zoom;
        for node_entity in &self.nodes {
            node_entity.update(cx, |node, _| node.set_viewport_zoom(zoom));
        }
    }

    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    pub fn viewport_size(&self) -> Option<(f32, f32)> {
        self.viewport_size
    }

    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.set_zoom(self.zoom + 0.1, cx);
    }

    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.set_zoom(self.zoom - 0.1, cx);
    }

    pub fn reset_view(&mut self, cx: &mut Context<Self>) {
        self.pan_offset = (0.0, 0.0);
        self.zoom = 1.0;
        self.propagate_viewport_zoom(cx);
        cx.notify();
    }

    pub fn fit_view(&mut self, viewport_w: f32, viewport_h: f32, cx: &mut Context<Self>) {
        self.fit_view_with_options(
            viewport_w,
            viewport_h,
            &super::controls::FitViewOptions::default(),
            cx,
        );
    }

    pub fn fit_view_with_options(
        &mut self,
        viewport_w: f32,
        viewport_h: f32,
        opts: &super::controls::FitViewOptions,
        cx: &mut Context<Self>,
    ) {
        if self.nodes.is_empty() {
            return;
        }

        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for node_entity in &self.nodes {
            let pos = node_entity.read(cx).position();
            min_x = min_x.min(pos.0);
            min_y = min_y.min(pos.1);
            max_x = max_x.max(pos.0 + NODE_WIDTH);
            max_y = max_y.max(pos.1 + NODE_HEIGHT);
        }

        if let Some((zoom, pan)) = compute_fit_zoom_and_center(
            (min_x, min_y, max_x, max_y),
            (viewport_w, viewport_h),
            opts,
        ) {
            self.zoom = zoom;
            self.pan_offset = pan;
            self.propagate_viewport_zoom(cx);
            cx.notify();
        }
    }

    /// Compute the world-space bounding box of all nodes.
    /// Returns `(min_x, min_y, max_x, max_y)` or `None` if there are no nodes.
    pub fn world_bounds(&self, cx: &App) -> Option<(f32, f32, f32, f32)> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for node_entity in &self.nodes {
            let pos = node_entity.read(cx).position();
            min_x = min_x.min(pos.0);
            min_y = min_y.min(pos.1);
            max_x = max_x.max(pos.0 + NODE_WIDTH);
            max_y = max_y.max(pos.1 + NODE_HEIGHT);
        }
        Some((min_x, min_y, max_x, max_y))
    }

    pub fn set_on_node_select(
        &mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) {
        self.on_node_select = Some(Box::new(handler));
    }

    pub fn set_on_nodes_delete(
        &mut self,
        handler: impl Fn(&[usize], &mut Window, &mut App) + 'static,
    ) {
        self.on_nodes_delete = Some(Box::new(handler));
    }

    pub fn set_interactive(&mut self, interactive: bool, cx: &mut Context<Self>) {
        self.interactive = interactive;
        cx.notify();
    }

    pub fn is_interactive(&self) -> bool {
        self.interactive
    }

    pub fn set_show_grid(&mut self, show: bool, cx: &mut Context<Self>) {
        self.show_grid = show;
        cx.notify();
    }

    pub fn set_grid_spacing(&mut self, spacing: f32, cx: &mut Context<Self>) {
        self.grid_spacing = Some(spacing);
        cx.notify();
    }

    pub fn set_grid_dot_size(&mut self, size: f32, cx: &mut Context<Self>) {
        self.grid_dot_size = Some(size);
        cx.notify();
    }

    pub fn set_node_dragging(&mut self, dragging: bool) {
        self.dragging_node = dragging;
    }
    pub fn selected_edges(&self) -> &HashSet<usize> {
        &self.selected_edges
    }
    pub fn animation_t(&self) -> f32 {
        self.animation_t
    }

    pub fn set_on_edge_select(
        &mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) {
        self.on_edge_select = Some(Box::new(handler));
    }

    pub fn set_on_edges_delete(
        &mut self,
        handler: impl Fn(&[String], &mut Window, &mut App) + 'static,
    ) {
        self.on_edges_delete = Some(Box::new(handler));
    }

    pub fn select_edge(
        &mut self,
        index: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.selected_edges.clear();
        if let Some(idx) = index {
            self.selected_edges.insert(idx);
        }
        if let Some(ref handler) = self.on_edge_select {
            handler(index, window, cx);
        }
        cx.notify();
    }

    /// Hit-test edges at a screen point. Returns the index of the closest edge
    /// within `tolerance` pixels, or `None`.
    fn edge_at_screen_point(&self, screen_x: f32, screen_y: f32, cx: &App) -> Option<usize> {
        let pan = self.pan_offset;
        let zoom = self.zoom;
        // Edge tolerance is expressed in screen-space pixels, so zoom doesn't
        // distort the pointer target. With the bezier projection below
        // already in screen coordinates, a constant 11 px ≈ 44 pt diameter
        // target, matching the HIG pointer-target minimum.
        let tolerance = EDGE_HIT_TOLERANCE_SCREEN_PX;
        let mut best: Option<(usize, f32)> = None;

        for (idx, conn) in self.connections.iter().enumerate() {
            let from = self
                .port_position(&conn.source, PortType::Output, cx)
                .or_else(|| {
                    self.node_position_by_id(&conn.source.node_id, cx)
                        .map(|src| (src.0 + NODE_WIDTH, src.1 + NODE_TITLE_HEIGHT / 2.0))
                });
            let to = self
                .port_position(&conn.target, PortType::Input, cx)
                .or_else(|| {
                    self.node_position_by_id(&conn.target.node_id, cx)
                        .map(|tgt| (tgt.0, tgt.1 + NODE_TITLE_HEIGHT / 2.0))
                });

            if let (Some(from), Some(to)) = (from, to) {
                // Transform to screen space
                let sf = (from.0 * zoom + pan.0, from.1 * zoom + pan.1);
                let st = (to.0 * zoom + pan.0, to.1 * zoom + pan.1);

                let is_dashed = conn.edge_style == EdgeStyle::Dashed;
                let ctrl_x = (sf.0 + st.0) / 2.0;
                let dist = if is_dashed {
                    let ctrl = (ctrl_x, (sf.1 + st.1) / 2.0);
                    point_to_quadratic_bezier_distance((screen_x, screen_y), sf, ctrl, st, 48)
                } else {
                    point_to_cubic_bezier_distance(
                        (screen_x, screen_y),
                        sf,
                        (ctrl_x, sf.1),
                        (ctrl_x, st.1),
                        st,
                        48,
                    )
                };

                if dist <= tolerance && best.is_none_or(|(_, b_dist)| dist < b_dist) {
                    best = Some((idx, dist));
                }
            }
        }

        best.map(|(idx, _)| idx)
    }

    pub fn set_on_connect(
        &mut self,
        handler: impl Fn(PortId, PortId, &mut Window, &mut App) + 'static,
    ) {
        self.on_connect = Some(Box::new(handler));
    }

    /// Register a callback fired when undo/redo resurrects previously deleted
    /// nodes. Hosts that mirror the canvas in an external model use this to
    /// re-insert the corresponding records. The callback fires AFTER the
    /// canvas itself has restored the entities, so host reads see them live.
    pub fn set_on_nodes_restore(
        &mut self,
        handler: impl Fn(&[Entity<WorkflowNode>], &mut Window, &mut App) + 'static,
    ) {
        self.on_nodes_restore = Some(Box::new(handler));
    }

    /// Register a callback fired when undo/redo resurrects previously deleted
    /// connections.
    pub fn set_on_edges_restore(
        &mut self,
        handler: impl Fn(&[Connection], &mut Window, &mut App) + 'static,
    ) {
        self.on_edges_restore = Some(Box::new(handler));
    }

    /// Register a duplicate factory — used by `Cmd+D` and ⌥-drag copy.
    ///
    /// The host receives the source entity and the target world-space position
    /// and returns a fresh `Entity<WorkflowNode>` (or `None` to decline).
    /// Duplicate is delegated because `WorkflowNode` holds non-cloneable
    /// closures (`content_builder`, `toolbar_builder`, `on_select`), which
    /// only the host knows how to reconstitute.
    #[allow(clippy::type_complexity)]
    pub fn set_on_node_duplicate(
        &mut self,
        handler: impl Fn(
            &Entity<WorkflowNode>,
            (f32, f32),
            &mut Window,
            &mut App,
        ) -> Option<Entity<WorkflowNode>>
        + 'static,
    ) {
        self.on_node_duplicate = Some(Box::new(handler));
    }

    /// True when `undo()` would have an effect. Drives the Undo toolbar
    /// button's disabled state per HIG.
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// True when `redo()` would have an effect.
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Short label for the top of the undo stack (e.g. "Move", "Delete")
    /// so hosts can render "Undo `<Label>`" menu items per HIG.
    pub fn undo_label(&self) -> Option<&'static str> {
        self.history.peek_undo_label()
    }

    pub fn redo_label(&self) -> Option<&'static str> {
        self.history.peek_redo_label()
    }

    /// Revert the most recent user-initiated mutation. No-op when the undo
    /// stack is empty.
    pub fn undo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(cmd) = self.history.pop_undo() {
            let reverted = self.apply_reverse(cmd, window, cx);
            self.history.push_redo(reverted);
            cx.notify();
        }
    }

    /// Replay the most recently undone mutation. No-op when the redo stack
    /// is empty.
    pub fn redo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(cmd) = self.history.pop_redo() {
            let reverted = self.apply_forward(cmd, window, cx);
            self.history.push_undo_after_redo(reverted);
            cx.notify();
        }
    }

    /// Revert a command; return the "inverse" command that `redo()` should
    /// replay. Having `apply_reverse` and `apply_forward` emit each other's
    /// inverses keeps both directions symmetric without bespoke invert logic.
    fn apply_reverse(
        &mut self,
        cmd: CanvasCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> CanvasCommand {
        match cmd {
            CanvasCommand::Move { node_id, from, to } => {
                // Reverse: move back to `from`. Forward (for redo) is identical
                // but swapped.
                self.set_node_position_by_id(&node_id, from, cx);
                CanvasCommand::Move {
                    node_id,
                    from: to,
                    to: from,
                }
            }
            CanvasCommand::DeleteNodes { nodes, edges } => {
                // Restore nodes at their original indices.
                // Vec was ordered ascending by idx at delete-time.
                for (idx, entity) in &nodes {
                    let clamped = (*idx).min(self.nodes.len());
                    self.nodes.insert(clamped, entity.clone());
                }
                for (idx, conn) in &edges {
                    let clamped = (*idx).min(self.connections.len());
                    self.connections.insert(clamped, conn.clone());
                }
                let entities: Vec<Entity<WorkflowNode>> =
                    nodes.iter().map(|(_, e)| e.clone()).collect();
                let conns: Vec<Connection> = edges.iter().map(|(_, c)| c.clone()).collect();
                if let Some(ref handler) = self.on_nodes_restore {
                    handler(&entities, window, cx);
                }
                if !conns.is_empty()
                    && let Some(ref handler) = self.on_edges_restore
                {
                    handler(&conns, window, cx);
                }
                CanvasCommand::DeleteNodes { nodes, edges }
            }
            CanvasCommand::AddConnection { connection } => {
                // Reverse = remove.
                let idx = self
                    .connections
                    .iter()
                    .position(|c| c.id == connection.id)
                    .unwrap_or(self.connections.len());
                self.connections.retain(|c| c.id != connection.id);
                CanvasCommand::DeleteConnection {
                    index: idx,
                    connection,
                }
            }
            CanvasCommand::DeleteConnection { index, connection } => {
                let clamped = index.min(self.connections.len());
                self.connections.insert(clamped, connection.clone());
                if let Some(ref handler) = self.on_edges_restore {
                    handler(std::slice::from_ref(&connection), window, cx);
                }
                CanvasCommand::AddConnection { connection }
            }
            CanvasCommand::AddNodes { nodes } => {
                // Reverse = remove by id (index may have shifted).
                let ids: HashSet<String> = nodes
                    .iter()
                    .map(|(_, e)| e.read(cx).id().to_string())
                    .collect();
                // Capture indices before removal so redo can restore order.
                let snapshot: Vec<(usize, Entity<WorkflowNode>)> = self
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| ids.contains(e.read(cx).id()))
                    .map(|(i, e)| (i, e.clone()))
                    .collect();
                self.nodes.retain(|e| !ids.contains(e.read(cx).id()));
                // Notify host so external model can drop duplicates.
                if let Some(ref handler) = self.on_nodes_delete {
                    let indices: Vec<usize> = snapshot.iter().map(|(i, _)| *i).collect();
                    handler(&indices, window, cx);
                }
                CanvasCommand::DeleteNodes {
                    nodes: snapshot,
                    edges: Vec::new(),
                }
            }
            CanvasCommand::Resize {
                node_id,
                from_pos,
                to_pos,
                from_size,
                to_size,
            } => {
                // Reverse = restore start pos + size, then emit the inverse
                // Resize so redo reapplies the original change.
                self.set_node_position_by_id(&node_id, from_pos, cx);
                self.set_node_size_by_id(&node_id, Some(from_size), cx);
                CanvasCommand::Resize {
                    node_id,
                    from_pos: to_pos,
                    to_pos: from_pos,
                    from_size: to_size,
                    to_size: from_size,
                }
            }
        }
    }

    /// Forward-apply an inverse command during redo. Mirrors `apply_reverse`.
    fn apply_forward(
        &mut self,
        cmd: CanvasCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> CanvasCommand {
        // apply_reverse already produces the inverse command structure we
        // need for redo; running it again moves us forward one step and
        // emits the *original* shape for the undo stack.
        self.apply_reverse(cmd, window, cx)
    }

    /// Write a node's position by looking it up by id. Used by undo of Move.
    pub(in crate::workflow::canvas) fn set_node_position_by_id(
        &mut self,
        node_id: &str,
        pos: (f32, f32),
        cx: &mut Context<Self>,
    ) {
        for entity in &self.nodes {
            if entity.read(cx).id() == node_id {
                entity.update(cx, |n, cx| n.set_position(pos.0, pos.1, cx));
                break;
            }
        }
    }

    /// Write a node's explicit size by id. `None` reverts to auto-sized.
    /// Used by Resize command apply/revert.
    pub(in crate::workflow::canvas) fn set_node_size_by_id(
        &mut self,
        node_id: &str,
        size: Option<(f32, f32)>,
        cx: &mut Context<Self>,
    ) {
        for entity in &self.nodes {
            if entity.read(cx).id() == node_id {
                entity.update(cx, |n, cx| n.set_size(size, cx));
                break;
            }
        }
    }

    /// Produce a duplicate at a specific world-space point and record it on
    /// the undo stack. No-op when the host hasn't registered an
    /// `on_node_duplicate` factory.
    fn duplicate_node_at(
        &mut self,
        source_node_id: &str,
        pos: (f32, f32),
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let source_entity = self
            .nodes
            .iter()
            .find(|e| e.read(cx).id() == source_node_id)
            .cloned();
        let Some(source) = source_entity else {
            return;
        };
        let factory = self.on_node_duplicate.take();
        if let Some(ref factory) = factory
            && let Some(new_entity) = factory(&source, pos, window, cx)
        {
            let zoom = self.zoom;
            new_entity.update(cx, |n, cx| {
                n.set_position(pos.0, pos.1, cx);
                n.set_viewport_zoom(zoom);
            });
            let idx = self.nodes.len();
            self.nodes.push(new_entity.clone());
            self.history.push(CanvasCommand::AddNodes {
                nodes: vec![(idx, new_entity)],
            });
        }
        self.on_node_duplicate = factory;
    }

    /// Push an explicit command onto the undo stack. Used by the sub-modules
    /// (selection, drag handler) that mutate state outside the render pass.
    /// Scope is deliberately narrower than `pub(super)` so the `CanvasCommand`
    /// type stays purely internal to the canvas module group.
    pub(in crate::workflow::canvas) fn push_history(&mut self, cmd: CanvasCommand) {
        self.history.push(cmd);
    }

    /// Hit-test all ports at a screen point. Returns the port ID and type if within
    /// `tolerance` pixels of a port handle.
    fn port_at_screen_point(
        &self,
        screen_x: f32,
        screen_y: f32,
        cx: &App,
    ) -> Option<(PortId, PortType, (f32, f32))> {
        let pan = self.pan_offset;
        let zoom = self.zoom;
        // 22 px ≈ 44 pt diameter HIG minimum pointer target. Applied in
        // screen space so low zoom doesn't shrink the hit area below the
        // minimum even though the visible handle is smaller.
        let tolerance = PORT_HIT_RADIUS_SCREEN_PX;

        for node_entity in &self.nodes {
            let node = node_entity.read(cx);
            for (name, wx, wy, pt) in node.port_positions() {
                let sx = wx * zoom + pan.0;
                let sy = wy * zoom + pan.1;
                let dx = screen_x - sx;
                let dy = screen_y - sy;
                if (dx * dx + dy * dy).sqrt() <= tolerance {
                    return Some((PortId::new(node.id(), name), pt, (sx, sy)));
                }
            }
        }
        None
    }

    fn finish_connection(
        &mut self,
        screen_x: f32,
        screen_y: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let from = self.connecting_from.take();
        self.connecting_mouse = None;

        if let Some((source_port, source_type, _)) = from
            && let Some((target_port, target_type, _)) =
                self.port_at_screen_point(screen_x, screen_y, cx)
        {
            // Validate: must be different nodes, opposite port types.
            let valid = source_port.node_id != target_port.node_id && source_type != target_type;

            if valid {
                // Normalize: always source=Output, target=Input.
                let (src, tgt) = if source_type == PortType::Output {
                    (source_port, target_port)
                } else {
                    (target_port, source_port)
                };
                // Snapshot the connection list so we can see what (if
                // anything) the host added, then register that connection
                // on the undo stack. Hosts that don't insert into
                // `self.connections` (e.g. purely external models) still
                // get the on_connect callback but no undo entry — their
                // model owns reversibility.
                let before: HashSet<String> =
                    self.connections.iter().map(|c| c.id.clone()).collect();
                if let Some(ref handler) = self.on_connect {
                    handler(src, tgt, window, cx);
                }
                if let Some(new_conn) = self
                    .connections
                    .iter()
                    .find(|c| !before.contains(&c.id))
                    .cloned()
                {
                    self.history.push(CanvasCommand::AddConnection {
                        connection: new_conn,
                    });
                }
            }
        }
        cx.notify();
    }

    fn node_position_by_id(&self, node_id: &str, cx: &App) -> Option<(f32, f32)> {
        for node_entity in &self.nodes {
            let node = node_entity.read(cx);
            if node.id() == node_id {
                return Some(node.position());
            }
        }
        None
    }

    /// Look up the world position of a specific port on a node.
    fn port_position(&self, port_id: &PortId, port_type: PortType, cx: &App) -> Option<(f32, f32)> {
        for node_entity in &self.nodes {
            let node = node_entity.read(cx);
            if node.id() == port_id.node_id {
                return node
                    .port_positions()
                    .into_iter()
                    .find(|(name, _, _, pt)| name == &port_id.port_name && *pt == port_type)
                    .map(|(_, x, y, _)| (x, y));
            }
        }
        None
    }

    /// Single entry point for every HIG-defined canvas shortcut (zoom,
    /// nudge, duplicate, undo/redo).
    ///
    /// Laid out as a flat branch cascade because the HIG tables for
    /// Keyboards and Gestures are equally flat — reordering here should
    /// read like reordering rows in those tables.
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key: &str = event.keystroke.key.as_ref();
        let m = &event.keystroke.modifiers;
        let cmd = m.platform;
        let shift = m.shift;
        let no_mods = !m.platform && !m.control && !m.alt && !m.shift;

        // Delete / Backspace — existing contract.
        if (key == "backspace" || key == "delete") && no_mods {
            self.delete_selected(window, cx);
            return;
        }

        // Cmd-Z / Shift-Cmd-Z — HIG-mandated macOS undo/redo.
        if cmd && key == "z" {
            if shift {
                self.redo(window, cx);
            } else {
                self.undo(window, cx);
            }
            return;
        }

        // Cmd-+, Cmd--, Cmd-0 zoom shortcuts per HIG Keyboards.
        // Several keyboards/layouts surface "+" as "=" (same physical key),
        // so we accept either.
        if cmd && (key == "=" || key == "+") {
            self.zoom_in(cx);
            return;
        }
        if cmd && key == "-" {
            self.zoom_out(cx);
            return;
        }
        if cmd && key == "0" {
            if let Some((vw, vh)) = self.viewport_size {
                self.fit_view(vw, vh, cx);
            }
            return;
        }

        // Cmd-D duplicate. `on_node_duplicate` host callback is required —
        // if absent, we intentionally no-op rather than guess a broken
        // duplicate.
        if cmd && key == "d" {
            self.duplicate_selected(window, cx);
            return;
        }

        // Keyboard graph navigation. Tab / Shift-Tab cycle through nodes
        // in insertion order, setting selection to the landed node so both
        // the visual focus ring and the VoiceOver announcement (once
        // GPUI's AX API lands) track the same target. The VoiceOver half
        // waits on upstream (see `foundations::accessibility` for status).
        if key == "tab" && !cmd && !m.control && !m.alt {
            self.cycle_node_focus(!shift, window, cx);
            return;
        }

        // Arrow-key nudge. 1 pt without modifier, 10 pt with Shift —
        // matches Freeform / Keynote. Only applies to nodes; rect
        // selection never owns focus while arrow nudging is expected.
        if no_mods || (shift && !cmd && !m.control && !m.alt) {
            let step = if shift { 10.0 } else { 1.0 };
            let delta = match key {
                "left" => Some((-step, 0.0)),
                "right" => Some((step, 0.0)),
                "up" => Some((0.0, -step)),
                "down" => Some((0.0, step)),
                _ => None,
            };
            if let Some((dx, dy)) = delta
                && !self.selected_nodes.is_empty()
            {
                self.nudge_selected(dx, dy, window, cx);
            }
        }
    }

    /// Test the screen point against the 8 resize handles of each
    /// currently selected node. Returns the initial `ResizeState` the
    /// canvas should keep in-flight until mouse-up.
    fn resize_handle_hit(&self, screen_x: f32, screen_y: f32, cx: &App) -> Option<ResizeState> {
        if self.selected_nodes.len() != 1 {
            // Handles only render for single-node selection, so the
            // hit-test gates on the same invariant.
            return None;
        }
        let pan = self.pan_offset;
        let zoom = self.zoom;
        let idx = *self.selected_nodes.iter().next()?;
        let entity = self.nodes.get(idx)?;
        let node = entity.read(cx);
        let (nw, nh) = node.effective_size();
        let pos = node.position();
        let sx = pos.0 * zoom + pan.0;
        let sy = pos.1 * zoom + pan.1;
        let sw = nw * zoom;
        let sh = nh * zoom;
        let handle = handle_at(screen_x, screen_y, sx, sy, sw, sh)?;
        // Translate mouse back into world space so the delta stays
        // independent of zoom changes during the drag.
        let world_mouse = self.screen_to_world((screen_x, screen_y));
        Some(ResizeState {
            node_id: node.id().to_string(),
            handle,
            start_pos: pos,
            start_size: (nw, nh),
            start_mouse_world: world_mouse,
        })
    }

    /// Apply an in-flight resize to the node under the handle.
    fn update_resize(&mut self, screen_x: f32, screen_y: f32, cx: &mut Context<Self>) {
        let Some(state) = self.resizing.as_ref() else {
            return;
        };
        let world = self.screen_to_world((screen_x, screen_y));
        let delta = (
            world.0 - state.start_mouse_world.0,
            world.1 - state.start_mouse_world.1,
        );
        let (new_pos, new_size) =
            apply_handle_delta(state.handle, state.start_pos, state.start_size, delta);
        let id = state.node_id.clone();
        self.set_node_position_by_id(&id, new_pos, cx);
        self.set_node_size_by_id(&id, Some(new_size), cx);
        cx.notify();
    }

    /// Commit the completed resize to the undo stack. Empty-delta resizes
    /// (a click on the handle with no drag) leave the stack alone.
    fn finish_resize(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(state) = self.resizing.take() else {
            return;
        };
        // Read the end state from the live node so we capture whatever
        // intermediate `update_resize` last wrote — this way the
        // recorded Resize command matches exactly what the user sees.
        let Some(entity) = self.nodes.iter().find(|e| e.read(cx).id() == state.node_id) else {
            return;
        };
        let n = entity.read(cx);
        let end_pos = n.position();
        let end_size = n.effective_size();

        if (end_pos.0 - state.start_pos.0).abs() < 0.5
            && (end_pos.1 - state.start_pos.1).abs() < 0.5
            && (end_size.0 - state.start_size.0).abs() < 0.5
            && (end_size.1 - state.start_size.1).abs() < 0.5
        {
            return;
        }

        self.history.push(CanvasCommand::Resize {
            node_id: state.node_id,
            from_pos: state.start_pos,
            to_pos: end_pos,
            from_size: state.start_size,
            to_size: end_size,
        });
        cx.notify();
    }

    /// Inverse of the render-time world→screen transform. Handy for
    /// keeping drag gestures anchored to the same world point as zoom or
    /// pan changes mid-drag.
    fn screen_to_world(&self, screen: (f32, f32)) -> (f32, f32) {
        let zoom = self.zoom;
        let pan = self.pan_offset;
        ((screen.0 - pan.0) / zoom, (screen.1 - pan.1) / zoom)
    }

    fn node_at_screen_point(&self, screen_x: f32, screen_y: f32, cx: &App) -> Option<usize> {
        let pan = self.pan_offset;
        let zoom = self.zoom;
        for (idx, node_entity) in self.nodes.iter().enumerate().rev() {
            let node = node_entity.read(cx);
            let pos = node.position();
            let (ew, eh) = node.effective_size();
            let nx = pos.0 * zoom + pan.0;
            let ny = pos.1 * zoom + pan.1;
            let nw = ew * zoom;
            let nh = eh * zoom;
            if screen_x >= nx && screen_x <= nx + nw && screen_y >= ny && screen_y <= ny + nh {
                return Some(idx);
            }
        }
        None
    }
}

impl Focusable for WorkflowCanvas {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for WorkflowCanvas {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let vs = window.viewport_size();
        let vw = f32::from(vs.width);
        let vh = f32::from(vs.height);
        self.viewport_size = Some((vw, vh));

        if !self.has_auto_fitted && !self.nodes.is_empty() && vw > 0.0 && vh > 0.0 {
            self.has_auto_fitted = true;
            self.fit_view(vw, vh, cx);
        }

        let theme = cx.theme();
        let edge_color = theme.text_muted;
        let grid_color = theme.border;
        let bg = theme.background;
        let accent = theme.accent;
        let selection_fill = Hsla { a: 0.15, ..accent };

        let pan = self.pan_offset;
        let zoom = self.zoom;

        let mut edge_data: Vec<ResolvedEdge> = Vec::new();
        for (conn_idx, conn) in self.connections.iter().enumerate() {
            // Try port-aware positions first, fall back to fixed offsets
            let from = self
                .port_position(&conn.source, PortType::Output, cx)
                .or_else(|| {
                    self.node_position_by_id(&conn.source.node_id, cx)
                        .map(|src| (src.0 + NODE_WIDTH, src.1 + NODE_TITLE_HEIGHT / 2.0))
                });
            let to = self
                .port_position(&conn.target, PortType::Input, cx)
                .or_else(|| {
                    self.node_position_by_id(&conn.target.node_id, cx)
                        .map(|tgt| (tgt.0, tgt.1 + NODE_TITLE_HEIGHT / 2.0))
                });
            if let (Some(from), Some(to)) = (from, to) {
                edge_data.push(ResolvedEdge {
                    index: conn_idx,
                    from,
                    to,
                    style: conn.edge_style,
                    source_position: conn.source_position,
                    target_position: conn.target_position,
                    label: conn.label.clone(),
                });
            }
        }

        if !edge_data.is_empty() {
            let now = Instant::now();
            if let Some(last) = self.last_animation_time {
                let dt = now.duration_since(last).as_secs_f32();
                self.animation_t = (self.animation_t + dt / 2.0) % 1.0;
            }
            self.last_animation_time = Some(now);
            window.request_animation_frame();
        }

        let anim_t = self.animation_t;
        let selection_rect = match (self.selection_start, self.selection_end) {
            (Some(s), Some(e)) => Some((
                s.0.min(e.0),
                s.1.min(e.1),
                (s.0 - e.0).abs(),
                (s.1 - e.1).abs(),
            )),
            _ => None,
        };

        // HIG pointer-shape table maps "open hand" to "hoverable draggable
        // content" and "closed hand" to "actively dragging". The canvas
        // applies the cursor here because the container captures all
        // mouse hover / drag events — the nodes themselves only see
        // clicks that land on them.
        //
        // Resize handles take priority: their cursor tells the user
        // they're about to resize, not drag.
        let cursor_style = if let Some(state) = &self.resizing {
            state.handle.cursor()
        } else if let Some(handle) = self.hovered_resize_handle {
            handle.cursor()
        } else if self.connecting_from.is_some() {
            CursorStyle::Crosshair
        } else if self.drag_initial_pos.is_some() {
            CursorStyle::ClosedHand
        } else if self.hovered_node.is_some() {
            CursorStyle::OpenHand
        } else {
            CursorStyle::Arrow
        };

        let mut container = div()
            .id(self.element_id.clone())
            .track_focus(&self.focus_handle)
            .flex_1()
            .w_full()
            .h_full()
            .relative()
            .overflow_hidden()
            .bg(bg)
            .cursor(cursor_style)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.focus_handle.focus(window, cx);
                    if !this.interactive {
                        return;
                    }
                    let mx = f32::from(event.position.x);
                    let my = f32::from(event.position.y);

                    // Resize handles win before port hits and node clicks,
                    // because a handle painted at the corner sits on top
                    // of the node's own interactive area.
                    if let Some(state) = this.resize_handle_hit(mx, my, cx) {
                        this.resizing = Some(state);
                        return;
                    }

                    // Check for port hit first (connection drag).
                    if let Some((port_id, port_type, screen_pos)) =
                        this.port_at_screen_point(mx, my, cx)
                    {
                        this.connecting_from = Some((port_id, port_type, screen_pos));
                        this.connecting_mouse = Some((mx, my));
                        return;
                    }
                    if let Some(idx) = this.node_at_screen_point(mx, my, cx) {
                        // macOS convention is Cmd (`platform`) for toggle,
                        // Shift for range extend. Using Shift for toggle
                        // (prior behaviour) conflicted with every native
                        // canvas tool (Finder, Keynote, Freeform).
                        if event.modifiers.platform {
                            this.toggle_node_selection(idx, window, cx);
                        } else if event.modifiers.shift {
                            // Shift on a node: additive range select — grow
                            // the selection to include this node without
                            // dropping prior ones.
                            this.extend_selection(idx, window, cx);
                        } else {
                            this.select_node(Some(idx), window, cx);
                        }
                        this.select_edge(None, window, cx);
                        // Snapshot drag start for Move-command generation.
                        if let Some(node) = this.nodes.get(idx) {
                            let n = node.read(cx);
                            this.drag_initial_pos = Some((n.id().to_string(), n.position()));
                        }
                    } else if let Some(edge_idx) = this.edge_at_screen_point(mx, my, cx) {
                        this.select_node(None, window, cx);
                        this.select_edge(Some(edge_idx), window, cx);
                    } else if !this.dragging_node {
                        this.selection_start = Some((mx, my));
                        this.selection_end = Some((mx, my));
                        if !event.modifiers.platform && !event.modifiers.shift {
                            this.select_node(None, window, cx);
                        }
                        this.select_edge(None, window, cx);
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &MouseUpEvent, window, cx| {
                    if this.connecting_from.is_some() {
                        let mx = f32::from(event.position.x);
                        let my = f32::from(event.position.y);
                        this.finish_connection(mx, my, window, cx);
                        return;
                    }
                    // Commit any active resize to the undo stack.
                    if this.resizing.is_some() {
                        this.finish_resize(window, cx);
                        return;
                    }
                    // Resolve the node drag that's just finished. Two
                    // outcomes:
                    //  - Alt held → ⌥-drag copy: restore the original
                    //    position, then ask the host factory for a fresh
                    //    duplicate at the drag-end location.
                    //  - otherwise → commit a Move command so ⌘Z can put
                    //    it back. A zero-delta "drag" (pure click for
                    //    selection) is filtered out by
                    //    MOVE_COMMIT_THRESHOLD.
                    if let Some((id, start)) = this.drag_initial_pos.take()
                        && let Some((_, end)) = this
                            .nodes
                            .iter()
                            .map(|e| {
                                let n = e.read(cx);
                                (n.id().to_string(), n.position())
                            })
                            .find(|(nid, _)| *nid == id)
                    {
                        let dx = start.0 - end.0;
                        let dy = start.1 - end.1;
                        let moved =
                            dx.abs() > MOVE_COMMIT_THRESHOLD || dy.abs() > MOVE_COMMIT_THRESHOLD;
                        if moved && event.modifiers.alt {
                            // Restore the original position — the drag
                            // was a copy-gesture, not a move.
                            this.set_node_position_by_id(&id, start, cx);
                            this.duplicate_node_at(&id, end, window, cx);
                        } else if moved {
                            this.history.push(CanvasCommand::Move {
                                node_id: id,
                                from: start,
                                to: end,
                            });
                        }
                    }
                    this.selection_start = None;
                    this.selection_end = None;
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    // Cancel any in-progress connection drag.
                    this.connecting_from = None;
                    this.connecting_mouse = None;
                    // Restore on failed drop — a drag that leaves the
                    // canvas bounds is treated as a cancellation, so the
                    // node snaps back to where it started. HIG Drag and
                    // drop: "Preserve transparency of operations by
                    // restoring content if the drag fails."
                    if let Some((id, start)) = this.drag_initial_pos.take() {
                        this.set_node_position_by_id(&id, start, cx);
                    }
                    // Same contract for a resize that leaves the canvas:
                    // restore the pre-drag size + position so the user is
                    // never stuck with a half-resized node.
                    if let Some(state) = this.resizing.take() {
                        this.set_node_position_by_id(&state.node_id, state.start_pos, cx);
                        this.set_node_size_by_id(&state.node_id, Some(state.start_size), cx);
                    }
                    this.selection_start = None;
                    this.selection_end = None;
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, window, cx| {
                let mx = f32::from(event.position.x);
                let my = f32::from(event.position.y);

                // Active resize pulls everything else to a stop — no
                // hover tracking, no selection rect, no auto-pan.
                if this.resizing.is_some() {
                    this.update_resize(mx, my, cx);
                    return;
                }

                // Connection drag tracking.
                if this.connecting_from.is_some() {
                    this.connecting_mouse = Some((mx, my));
                    // Highlight the port under the pointer while a
                    // connection is in flight so compatible drop targets
                    // are discoverable.
                    let new_port_hover = this.port_at_screen_point(mx, my, cx).map(|(id, _, _)| id);
                    if new_port_hover != this.hovered_port {
                        this.hovered_port = new_port_hover;
                    }
                    cx.notify();
                    return;
                }

                // Hover tracking drives the open/closed hand cursor,
                // subtle hover affordance, edge thickening, and resize
                // cursor. Only recomputes when any of the tracked hovers
                // changed so notify() stays rate-limited.
                let new_resize_hover = this.resize_handle_hit(mx, my, cx).map(|state| state.handle);
                let new_node_hover = if new_resize_hover.is_none() {
                    this.node_at_screen_point(mx, my, cx)
                } else {
                    None
                };
                let new_edge_hover = if new_node_hover.is_none() && new_resize_hover.is_none() {
                    this.edge_at_screen_point(mx, my, cx)
                } else {
                    None
                };
                if new_node_hover != this.hovered_node
                    || new_edge_hover != this.hovered_edge
                    || new_resize_hover != this.hovered_resize_handle
                {
                    this.hovered_node = new_node_hover;
                    this.hovered_edge = new_edge_hover;
                    this.hovered_resize_handle = new_resize_hover;
                    cx.notify();
                }
                if this.hovered_port.is_some() {
                    this.hovered_port = None;
                    cx.notify();
                }

                // Auto-pan when dragging a node near a viewport edge.
                // The HIG mandates "Scroll contents of destination when
                // necessary" during drag. We drive pan_offset by a constant
                // step per frame so the effect is perceptibly smooth and
                // predictable even on fast drags.
                if this.drag_initial_pos.is_some()
                    && event.pressed_button == Some(MouseButton::Left)
                    && let Some((vw, vh)) = this.viewport_size
                {
                    let mut dx = 0.0_f32;
                    let mut dy = 0.0_f32;
                    if mx < AUTO_PAN_MARGIN {
                        dx = AUTO_PAN_STEP;
                    } else if mx > vw - AUTO_PAN_MARGIN {
                        dx = -AUTO_PAN_STEP;
                    }
                    if my < AUTO_PAN_MARGIN {
                        dy = AUTO_PAN_STEP;
                    } else if my > vh - AUTO_PAN_MARGIN {
                        dy = -AUTO_PAN_STEP;
                    }
                    if dx != 0.0 || dy != 0.0 {
                        this.pan_offset.0 += dx;
                        this.pan_offset.1 += dy;
                        cx.notify();
                    }
                }

                if this.selection_start.is_some() && !this.dragging_node {
                    this.selection_end = Some((mx, my));

                    if let (Some(start), Some(end)) = (this.selection_start, this.selection_end) {
                        let rect_x = start.0.min(end.0);
                        let rect_y = start.1.min(end.1);
                        let rect_w = (start.0 - end.0).abs();
                        let rect_h = (start.1 - end.1).abs();
                        let pan = this.pan_offset;
                        let zoom = this.zoom;

                        for &prev in &this.selected_nodes {
                            if let Some(node) = this.nodes.get(prev) {
                                node.update(cx, |n, cx| n.set_selected(false, window, cx));
                            }
                        }
                        this.selected_nodes.clear();

                        for (idx, node_entity) in this.nodes.iter().enumerate() {
                            let node = node_entity.read(cx);
                            let pos = node.position();
                            let (ew, eh) = node.effective_size();
                            let nx = pos.0 * zoom + pan.0;
                            let ny = pos.1 * zoom + pan.1;
                            let nw = ew * zoom;
                            let nh = eh * zoom;
                            if nx + nw >= rect_x
                                && nx <= rect_x + rect_w
                                && ny + nh >= rect_y
                                && ny <= rect_y + rect_h
                            {
                                this.selected_nodes.insert(idx);
                                node_entity.update(cx, |n, cx| n.set_selected(true, window, cx));
                            }
                        }
                    }
                    cx.notify();
                }
            }))
            // Native pinch-gesture support. GPUI surfaces
            // NSMagnificationGesture as a dedicated `PinchEvent`, so zoom
            // reads the normalised `delta` field (0.1 = 10% zoom-in per the
            // GPUI docs) multiplicatively. Cmd+scroll remains wired below as
            // a fallback for mice without a pinch gesture — Freeform and
            // Safari honour both paths, so we do too.
            .on_pinch(cx.listener(|this, event: &PinchEvent, _window, cx| {
                if !this.interactive {
                    return;
                }
                // Multiplicative so the zoom step scales with the current
                // level — 0.1 delta at zoom 2.0 advances to 2.2, at zoom
                // 0.5 to 0.55, preserving the perceived pinch feel.
                let next = this.zoom * (1.0 + event.delta);
                this.zoom = next.clamp(MIN_ZOOM, MAX_ZOOM);
                this.propagate_viewport_zoom(cx);
                cx.notify();
            }))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _window, cx| {
                if !this.interactive {
                    return;
                }
                if event.modifiers.platform || event.modifiers.control {
                    let delta = match event.delta {
                        ScrollDelta::Lines(d) => d.y * 0.05,
                        ScrollDelta::Pixels(d) => f32::from(d.y) * 0.002,
                    };
                    this.zoom = (this.zoom + delta).clamp(MIN_ZOOM, MAX_ZOOM);
                    this.propagate_viewport_zoom(cx);
                } else {
                    let (dx, dy) = match event.delta {
                        ScrollDelta::Lines(d) => (d.x * 40.0, d.y * 40.0),
                        ScrollDelta::Pixels(d) => (f32::from(d.x), f32::from(d.y)),
                    };
                    this.pan_offset.0 += dx;
                    this.pan_offset.1 += dy;
                }
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if !this.interactive {
                    return;
                }
                this.handle_key_down(event, window, cx);
            }));

        // Background dot grid
        let show_grid = self.show_grid;
        let base_grid_spacing = self.grid_spacing.unwrap_or(40.0);
        let dot_size = self.grid_dot_size.unwrap_or(2.0);
        if show_grid {
            container = container.child(
                canvas(
                    move |_bounds, _window, _cx| {},
                    move |bounds, _, window, _cx| {
                        let grid_spacing = base_grid_spacing * zoom;
                        let ox = f32::from(bounds.origin.x);
                        let oy = f32::from(bounds.origin.y);
                        let w = f32::from(bounds.size.width);
                        let h = f32::from(bounds.size.height);
                        let start_x = (pan.0 % grid_spacing) - grid_spacing;
                        let start_y = (pan.1 % grid_spacing) - grid_spacing;
                        if grid_spacing < 8.0 {
                            return;
                        }
                        let mut x = start_x;
                        while x < w + grid_spacing {
                            let mut y = start_y;
                            while y < h + grid_spacing {
                                window.paint_quad(fill(
                                    Bounds {
                                        origin: point(px(ox + x), px(oy + y)),
                                        size: size(px(dot_size), px(dot_size)),
                                    },
                                    grid_color,
                                ));
                                y += grid_spacing;
                            }
                            x += grid_spacing;
                        }
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full(),
            );
        }

        // Edges
        for edge in edge_data {
            let af = (edge.from.0 * zoom + pan.0, edge.from.1 * zoom + pan.1);
            let at = (edge.to.0 * zoom + pan.0, edge.to.1 * zoom + pan.1);
            let mut el = EdgeElement::new(af, at)
                .color(edge_color)
                .style(edge.style)
                .selected(self.selected_edges.contains(&edge.index))
                .hovered(self.hovered_edge == Some(edge.index))
                .source_position(edge.source_position)
                .target_position(edge.target_position)
                .target_indicator(true)
                .animation_t(anim_t);
            if let Some(label) = edge.label {
                el = el.label(label);
            }
            container = container.child(el);
        }

        // Connection drag preview
        if let (Some((_, _, from_screen)), Some(mouse)) =
            (&self.connecting_from, self.connecting_mouse)
        {
            container = container.child(ConnectionLine::new(*from_screen, mouse));
        }

        // Drop-zone highlight during port-connection drag. Every port on
        // a different node with the opposite type gets an accent ring so
        // the user sees the valid drop targets at a glance. The hovered
        // port (if any) gets a brighter ring; incompatible ports are left
        // alone rather than marked with a "no" glyph to keep the canvas
        // calm during drag.
        if let Some((source_id, source_type, _)) = self.connecting_from.clone() {
            let accent = theme.accent;
            let border_soft = Hsla { a: 0.4, ..accent };
            let border_strong = accent;
            for node_entity in &self.nodes {
                let node = node_entity.read(cx);
                if node.id() == source_id.node_id {
                    // Same node: never a valid drop target.
                    continue;
                }
                for (port_name, wx, wy, pt) in node.port_positions() {
                    if pt == source_type {
                        continue;
                    }
                    let sx = wx * zoom + pan.0;
                    let sy = wy * zoom + pan.1;
                    let is_hovered = self
                        .hovered_port
                        .as_ref()
                        .is_some_and(|p| p.node_id == node.id() && p.port_name == port_name);
                    let ring_color = if is_hovered {
                        border_strong
                    } else {
                        border_soft
                    };
                    // Outer ring = 14 px diameter (visible affordance above
                    // the 8 px handle), positioned at the port centre.
                    let ring = div()
                        .absolute()
                        .left(px(sx - 8.0))
                        .top(px(sy - 8.0))
                        .size(px(16.0))
                        .border_2()
                        .border_color(ring_color)
                        .rounded_full();
                    container = container.child(ring);
                }
            }
        }

        // Nodes
        let drag_in_flight = self.drag_initial_pos.is_some();
        let multi_count = self.selected_nodes.len();
        let text_on_accent = theme.text_on_accent;
        for (idx, node_entity) in self.nodes.iter().enumerate() {
            let pos = node_entity.read(cx).position();
            let x = pos.0 * zoom + pan.0;
            let y = pos.1 * zoom + pan.1;
            let node_idx = idx;

            let mut slot = div()
                .id(ElementId::NamedInteger("wf-node-slot".into(), idx as u64))
                .absolute()
                .left(px(x))
                .top(px(y))
                .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                    // Cmd toggles, Shift extends; plain click replaces.
                    if event.modifiers().platform {
                        this.toggle_node_selection(node_idx, window, cx);
                    } else if event.modifiers().shift {
                        this.extend_selection(node_idx, window, cx);
                    } else {
                        let new_sel = if this.selected_nodes.contains(&node_idx)
                            && this.selected_nodes.len() == 1
                        {
                            None
                        } else {
                            Some(node_idx)
                        };
                        this.select_node(new_sel, window, cx);
                    }
                }))
                .child(node_entity.clone());

            // Multi-item drag badge. When the user drags a node that is
            // part of a selection of two or more, show a count pill
            // anchored to the lead node's top-right corner — HIG Drag
            // and drop: "Support multiple simultaneous drags … badge
            // during multi-item drag operations."
            if drag_in_flight && multi_count > 1 && self.selected_nodes.contains(&idx) {
                let badge = div()
                    .absolute()
                    .top(px(-8.0))
                    .right(px(-8.0))
                    .px(px(6.0))
                    .py(px(2.0))
                    .bg(accent)
                    .text_color(text_on_accent)
                    .rounded(theme.radius_full)
                    .shadow_sm()
                    .child(gpui::SharedString::from(multi_count.to_string()));
                slot = slot.child(badge);
            }

            container = container.child(slot);
        }

        // Node toolbars (second pass, above nodes for z-ordering)
        let multi_selected = self.selected_nodes.len() > 1;
        let hovered_idx = self.hovered_node;
        for (idx, node_entity) in self.nodes.iter().enumerate() {
            let node = node_entity.read(cx);
            if let Some(builder) = node.toolbar_builder() {
                let pos = node.position();
                let (ew, eh) = node.effective_size();
                let screen_x = pos.0 * zoom + pan.0;
                let screen_y = pos.1 * zoom + pan.1;
                let toolbar = builder().with_node_context(
                    screen_x,
                    screen_y,
                    ew * zoom,
                    eh * zoom,
                    node.is_selected(),
                    hovered_idx == Some(idx),
                    multi_selected,
                );
                container = container.child(toolbar);
            }
        }

        // Resize handles painted on the single selected node. Skipped
        // during multi-select because resize semantics across a group
        // are ambiguous (do we scale? translate? grow individually?) —
        // Keynote and Freeform both suppress handles in the same case.
        if let Some(&idx) = self.selected_nodes.iter().next()
            && self.selected_nodes.len() == 1
            && let Some(entity) = self.nodes.get(idx)
        {
            let node = entity.read(cx);
            let pos = node.position();
            let (ew, eh) = node.effective_size();
            let sx = pos.0 * zoom + pan.0;
            let sy = pos.1 * zoom + pan.1;
            let sw = ew * zoom;
            let sh = eh * zoom;
            let handle_half = HANDLE_VISUAL_SIZE / 2.0;
            let _ = HANDLE_HIT_RADIUS; // tied here so imports stay live
            for handle in ResizeHandle::ALL {
                let (hx, hy) = handle.centre(sx, sy, sw, sh);
                container = container.child(
                    div()
                        .absolute()
                        .left(px(hx - handle_half))
                        .top(px(hy - handle_half))
                        .size(px(HANDLE_VISUAL_SIZE))
                        .bg(theme.surface)
                        .border_1()
                        .border_color(theme.accent)
                        .rounded(px(2.0)),
                );
            }
        }

        // Selection rectangle
        if let Some((rx, ry, rw, rh)) = selection_rect
            && (rw > 2.0 || rh > 2.0)
        {
            container = container.child(
                canvas(
                    move |_bounds, _window, _cx| {},
                    move |_bounds, _, window, _cx| {
                        let rect = Bounds {
                            origin: point(px(rx), px(ry)),
                            size: size(px(rw), px(rh)),
                        };
                        window.paint_quad(fill(rect, selection_fill));
                        let lw = 1.0;
                        window.paint_quad(fill(
                            Bounds {
                                origin: point(px(rx), px(ry)),
                                size: size(px(rw), px(lw)),
                            },
                            accent,
                        ));
                        window.paint_quad(fill(
                            Bounds {
                                origin: point(px(rx), px(ry + rh - lw)),
                                size: size(px(rw), px(lw)),
                            },
                            accent,
                        ));
                        window.paint_quad(fill(
                            Bounds {
                                origin: point(px(rx), px(ry)),
                                size: size(px(lw), px(rh)),
                            },
                            accent,
                        ));
                        window.paint_quad(fill(
                            Bounds {
                                origin: point(px(rx + rw - lw), px(ry)),
                                size: size(px(lw), px(rh)),
                            },
                            accent,
                        ));
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full(),
            );
        }

        container
    }
}

#[cfg(test)]
mod tests;
