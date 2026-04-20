//! Workflow node component.
//!
//! A draggable node with input/output ports, rendered as a card with a title bar.

use super::node_toolbar::NodeToolbar;
use crate::callback_types::ElementBuilder;
use crate::foundations::layout::SPACING_12;
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, SharedString, Window, div, px,
};

/// Callback type for select-state change notifications.
type SelectChangeHandler = Box<dyn Fn(bool, &mut Window, &mut App) + 'static>;
/// Callback invoked on double-click of the node. Per HIG Gestures table:
/// "Double tap → Zoom in; secondary action". Canvas hosts wire this to
/// their inline-edit flow (focus a title TextField, open a detail sheet,
/// etc.). No default action when unset.
type DoubleClickHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;

/// Node drag-state opacity while a drag is in flight.
///
/// HIG Drag and drop: "a subtle visual effect — like making the image
/// slightly translucent — conveys that the item is in motion." Matches
/// Freeform's ~0.75 opacity for dragged items.
const DRAG_OPACITY: f32 = 0.75;

/// Snapshot captured at mouse-down so the world-space node position is
/// recovered from screen-space cursor deltas regardless of viewport
/// transform changes mid-drag. Without this, a drag at zoom=2.0 would
/// move the node 2× as fast as the cursor (the old `drag_offset` math
/// conflated screen and world coordinates).
#[derive(Clone, Copy, Debug)]
struct DragAnchor {
    start_world: (f32, f32),
    start_screen: (f32, f32),
}

/// Minimum width of a workflow node (used for port position calculations).
pub(super) const NODE_MIN_WIDTH: f32 = 384.0;
/// Minimum height for an explicitly sized node — derived from the title bar
/// plus bottom padding. Applied during resize so a node can't be crushed
/// below a usable interaction target.
pub(super) const NODE_MIN_HEIGHT: f32 = 80.0;
/// Y offset where the first port starts (below title bar + padding).
pub(super) const PORT_START_Y: f32 = 40.0;
/// Vertical spacing between ports.
pub(super) const PORT_SPACING: f32 = 20.0;

/// Direction of a port on a node.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortType {
    Input,
    Output,
}

/// A port on a workflow node.
#[derive(Debug, Clone)]
pub struct Port {
    /// Display name of the port.
    pub name: String,
    /// Whether this is an input or output port.
    pub port_type: PortType,
}

impl Port {
    pub fn input(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            port_type: PortType::Input,
        }
    }

    pub fn output(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            port_type: PortType::Output,
        }
    }
}

// ─── Stateless sub-components ────────────────────────────────────────────────

/// Stateless header section for a workflow node.
///
/// Renders with a secondary background, bottom border, and rounded top corners.
/// Accepts arbitrary children (typically a `NodeTitle`, optional `NodeDescription`,
/// and optional `NodeAction`).
///
/// # Example
/// ```ignore
/// NodeHeader::new()
///     .child(NodeTitle::new("Process Data"))
///     .child(NodeDescription::new("Transforms input"))
/// ```
#[derive(IntoElement)]
pub struct NodeHeader {
    children: Vec<AnyElement>,
}

impl Default for NodeHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeHeader {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a child element to the header.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for NodeHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut container = div()
            .p(px(SPACING_12))
            .bg(theme.hover)
            .border_b_1()
            .border_color(theme.border)
            .rounded_t(theme.radius_lg)
            .flex()
            .flex_col()
            .gap(px(2.0));
        for child in self.children {
            container = container.child(child);
        }
        container
    }
}

/// Stateless title element for a workflow node header.
///
/// Renders with semibold weight and primary text color.
#[derive(IntoElement)]
pub struct NodeTitle {
    text: SharedString,
}

impl NodeTitle {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for NodeTitle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .text_style(TextStyle::Subheadline, theme)
            .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
            .text_color(theme.text)
            .child(self.text)
    }
}

/// Stateless description element for a workflow node header.
///
/// Renders in a smaller, muted style below the title.
#[derive(IntoElement)]
pub struct NodeDescription {
    text: SharedString,
}

impl NodeDescription {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for NodeDescription {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .child(self.text)
    }
}

/// Stateless action area for a workflow node header.
///
/// Wraps a single child element, intended for buttons or controls
/// placed at the right side of the header.
#[derive(IntoElement)]
pub struct NodeAction {
    child: AnyElement,
}

impl NodeAction {
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
        }
    }
}

impl RenderOnce for NodeAction {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().child(self.child)
    }
}

/// Stateless content area for a workflow node.
///
/// Renders children with padding and a top border separator.
#[derive(IntoElement)]
pub struct NodeContent {
    children: Vec<AnyElement>,
}

impl Default for NodeContent {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeContent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a child element to the content area.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for NodeContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut container = div()
            .border_t_1()
            .border_color(theme.border)
            .p(px(SPACING_12));
        for child in self.children {
            container = container.child(child);
        }
        container
    }
}

/// Stateless footer section for a workflow node.
///
/// Renders with a secondary background, top border, and rounded bottom corners.
#[derive(IntoElement)]
pub struct NodeFooter {
    children: Vec<AnyElement>,
}

impl Default for NodeFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeFooter {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    /// Add a child element to the footer.
    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for NodeFooter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut container = div()
            .border_t_1()
            .border_color(theme.border)
            .bg(theme.hover)
            .rounded_b(theme.radius_lg)
            .p(px(SPACING_12));
        for child in self.children {
            container = container.child(child);
        }
        container
    }
}

// ─── Stateful WorkflowNode (Entity) ────────────────────────────────────────

/// A draggable workflow node with composable sections.
///
/// Supports optional description, content, footer, and action areas
/// matching the AI SDK Elements Node sub-component pattern.
pub struct WorkflowNode {
    element_id: ElementId,
    id: String,
    title: SharedString,
    description: Option<SharedString>,
    position: (f32, f32),
    selected: bool,
    input_ports: Vec<Port>,
    output_ports: Vec<Port>,
    drag_anchor: Option<DragAnchor>,
    /// Cached canvas zoom factor so the node's internal drag math can
    /// translate screen-space cursor deltas into world-space position
    /// deltas. Updated by `WorkflowCanvas` whenever it mutates zoom.
    /// Defaults to 1.0 for standalone use (no canvas).
    viewport_zoom: f32,
    /// Whether to show left (target) handle dot.
    show_target_handle: bool,
    /// Whether to show right (source) handle dot.
    show_source_handle: bool,
    /// Optional content area builder.
    content_builder: ElementBuilder,
    /// Optional footer area builder.
    footer_builder: ElementBuilder,
    /// Optional action area builder (rendered in header, right-aligned).
    action_builder: ElementBuilder,
    /// Optional node-attached toolbar builder.
    toolbar_builder: Option<Box<dyn Fn() -> NodeToolbar>>,
    /// Optional callback invoked when selected state changes.
    on_select: Option<SelectChangeHandler>,
    /// Optional callback invoked on double-click.
    on_double_click: Option<DoubleClickHandler>,
    /// Explicit size override. `None` → render at auto size (the old
    /// behaviour: `NODE_MIN_WIDTH` × content height). `Some((w, h))` →
    /// render at the supplied dimensions. Populated by the canvas when the
    /// user drags a resize handle.
    size: Option<(f32, f32)>,
}

impl WorkflowNode {
    pub fn new(
        cx: &mut Context<Self>,
        id: impl Into<String>,
        title: impl Into<SharedString>,
    ) -> Self {
        let _ = cx;
        Self {
            element_id: next_element_id("workflow-node"),
            id: id.into(),
            title: title.into(),
            description: None,
            position: (0.0, 0.0),
            selected: false,
            input_ports: Vec::new(),
            output_ports: Vec::new(),
            drag_anchor: None,
            viewport_zoom: 1.0,
            show_target_handle: true,
            show_source_handle: true,
            content_builder: None,
            footer_builder: None,
            action_builder: None,
            toolbar_builder: None,
            on_select: None,
            on_double_click: None,
            size: None,
        }
    }

    /// Get the node's string id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Set the position of the node.
    pub fn set_position(&mut self, x: f32, y: f32, cx: &mut Context<Self>) {
        self.position = (x, y);
        cx.notify();
    }

    /// Get the current position.
    pub fn position(&self) -> (f32, f32) {
        self.position
    }

    /// Apply a selected-state change and notify listeners only when the value changes.
    fn update_selected(&mut self, selected: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected == selected {
            return;
        }

        self.selected = selected;
        if let Some(ref cb) = self.on_select {
            cb(self.selected, window, cx);
        }
        cx.notify();
    }

    /// Set whether this node is selected. Fires the `on_select` callback if the value changes.
    pub fn set_selected(&mut self, selected: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.update_selected(selected, window, cx);
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    /// Add an input port.
    pub fn add_input_port(&mut self, name: impl Into<String>, cx: &mut Context<Self>) {
        self.input_ports.push(Port::input(name));
        cx.notify();
    }

    /// Add an output port.
    pub fn add_output_port(&mut self, name: impl Into<String>, cx: &mut Context<Self>) {
        self.output_ports.push(Port::output(name));
        cx.notify();
    }

    /// Set ports in bulk.
    pub fn set_ports(&mut self, inputs: Vec<Port>, outputs: Vec<Port>, cx: &mut Context<Self>) {
        self.input_ports = inputs;
        self.output_ports = outputs;
        cx.notify();
    }

    /// Get input port count.
    pub fn input_port_count(&self) -> usize {
        self.input_ports.len()
    }

    /// Get output port count.
    pub fn output_port_count(&self) -> usize {
        self.output_ports.len()
    }

    /// Set a description shown below the title.
    pub fn set_description(&mut self, desc: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.description = Some(desc.into());
        cx.notify();
    }

    /// Configure handle visibility.
    pub fn set_handles(&mut self, target: bool, source: bool, cx: &mut Context<Self>) {
        self.show_target_handle = target;
        self.show_source_handle = source;
        cx.notify();
    }

    /// Set a builder for the content area (rendered below ports).
    pub fn set_content(
        &mut self,
        builder: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) {
        self.content_builder = Some(Box::new(builder));
        cx.notify();
    }

    /// Set a builder for the footer area.
    pub fn set_footer(
        &mut self,
        builder: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) {
        self.footer_builder = Some(Box::new(builder));
        cx.notify();
    }

    /// Set a builder for the action area (in header, right-aligned).
    pub fn set_action(
        &mut self,
        builder: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
        cx: &mut Context<Self>,
    ) {
        self.action_builder = Some(Box::new(builder));
        cx.notify();
    }

    /// Set a builder for the node-attached toolbar.
    pub fn set_toolbar(
        &mut self,
        builder: impl Fn() -> NodeToolbar + 'static,
        cx: &mut Context<Self>,
    ) {
        self.toolbar_builder = Some(Box::new(builder));
        cx.notify();
    }

    /// Access the toolbar builder (used by canvas for rendering).
    pub(crate) fn toolbar_builder(&self) -> Option<&dyn Fn() -> NodeToolbar> {
        self.toolbar_builder.as_ref().map(|b| b.as_ref())
    }

    /// Set a callback invoked whenever the selected state changes.
    pub fn set_on_select(&mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) {
        self.on_select = Some(Box::new(handler));
    }

    /// Wire a double-click handler. Per HIG Gestures the convention on
    /// macOS canvases (Keynote, Freeform) is that double-click enters an
    /// inline-edit mode on the clicked item — typically focusing a title
    /// field or opening a detail sheet. The node itself makes no UI
    /// assumption; the host provides that behaviour in the handler.
    pub fn set_on_double_click(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_double_click = Some(Box::new(handler));
    }

    /// True while the node is being dragged. Exposed so the canvas can
    /// compose the hover cursor state and multi-drag badges without
    /// reaching into private fields.
    pub fn is_dragging(&self) -> bool {
        self.drag_anchor.is_some()
    }

    /// Update the cached canvas zoom. Called by `WorkflowCanvas` whenever
    /// the viewport transform changes so the node's drag math stays in
    /// world coordinates. Safe to call at any time; the next mouse-move
    /// uses the fresh zoom immediately.
    pub fn set_viewport_zoom(&mut self, zoom: f32) {
        self.viewport_zoom = zoom.max(0.01);
    }

    /// Set an explicit size. `None` reverts to auto sizing.
    pub fn set_size(&mut self, size: Option<(f32, f32)>, cx: &mut Context<Self>) {
        self.size = size.map(|(w, h)| (w.max(NODE_MIN_WIDTH), h.max(NODE_MIN_HEIGHT)));
        cx.notify();
    }

    /// Current explicit size. `None` means "auto-sized"; the rendered
    /// width in that case is `NODE_MIN_WIDTH`, and the height is whatever
    /// the layout resolves to from the node's content.
    pub fn size(&self) -> Option<(f32, f32)> {
        self.size
    }

    /// The size canvas hit-testing and handle-rendering should assume. In
    /// auto-sized mode the height is unknown until layout resolves, so we
    /// fall back to the historical `NODE_HEIGHT` estimate used elsewhere.
    pub(super) fn effective_size(&self) -> (f32, f32) {
        self.size
            .unwrap_or((NODE_MIN_WIDTH, super::canvas::NODE_HEIGHT))
    }

    /// Toggle selected state. Fires the `on_select` callback if set.
    pub fn toggle_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.update_selected(!self.selected, window, cx);
    }

    /// Compute world-space positions for all ports.
    ///
    /// Returns `(port_name, world_x, world_y, port_type)` for each port.
    /// Input ports are on the left edge, output ports on the right edge —
    /// `right` being the node's current effective width (explicit or the
    /// `NODE_MIN_WIDTH` auto fallback). Resized nodes therefore place
    /// their output ports at the new right edge automatically.
    pub fn port_positions(&self) -> Vec<(String, f32, f32, PortType)> {
        let width = self.size.map(|(w, _)| w).unwrap_or(NODE_MIN_WIDTH);
        let mut result = Vec::with_capacity(self.input_ports.len() + self.output_ports.len());
        for (i, port) in self.input_ports.iter().enumerate() {
            result.push((
                port.name.clone(),
                self.position.0,
                self.position.1 + PORT_START_Y + i as f32 * PORT_SPACING,
                PortType::Input,
            ));
        }
        for (i, port) in self.output_ports.iter().enumerate() {
            result.push((
                port.name.clone(),
                self.position.0 + width,
                self.position.1 + PORT_START_Y + (self.input_ports.len() + i) as f32 * PORT_SPACING,
                PortType::Output,
            ));
        }
        result
    }
}

impl Render for WorkflowNode {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let t_surface = theme.surface;
        let t_border = theme.border;
        let t_accent = theme.accent;
        let t_success = theme.success;
        let t_hover = theme.hover;
        let t_text = theme.text;
        let t_text_on_accent = theme.text_on_accent;
        let t_text_muted = theme.text_muted;
        let t_spacing_sm = theme.spacing_sm;
        let t_spacing_xs = theme.spacing_xs;
        let t_radius_lg = theme.radius_lg;
        let t_radius_full = theme.radius_full;

        let border_color = if self.selected { t_accent } else { t_border };
        let is_dragging = self.drag_anchor.is_some();

        let mut card = div()
            .id(self.element_id.clone())
            .min_w(px(NODE_MIN_WIDTH))
            .bg(t_surface)
            .border_1()
            .border_color(border_color)
            .rounded(t_radius_lg)
            .flex()
            .flex_col()
            .overflow_hidden();

        // Apply explicit size when the user has resized the node.
        // Auto-sized nodes keep the `min_w` + flex-content contract
        // above so the title column can expand to fit text.
        if let Some((w, h)) = self.size {
            card = card.w(px(w)).h(px(h));
        }

        // HIG Drag and drop + native Keynote behaviour — a selected
        // object reads as "lifted" with a heavier shadow than its
        // resting state. Keeps the baseline shadow_sm for unselected
        // nodes.
        card = if self.selected {
            card.shadow_md()
        } else {
            card.shadow_sm()
        };

        // Translucent affordance during active drag so the user perceives
        // the item as in motion.
        if is_dragging {
            card = card.opacity(DRAG_OPACITY);
        }

        card = card
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, _window, _cx| {
                    this.drag_anchor = Some(DragAnchor {
                        start_world: this.position,
                        start_screen: (f32::from(event.position.x), f32::from(event.position.y)),
                    });
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, _cx| {
                    this.drag_anchor = None;
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, _cx| {
                    this.drag_anchor = None;
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                if let Some(anchor) = this.drag_anchor {
                    if event.pressed_button == Some(MouseButton::Left) {
                        let zoom = this.viewport_zoom.max(0.01);
                        let dx = (f32::from(event.position.x) - anchor.start_screen.0) / zoom;
                        let dy = (f32::from(event.position.y) - anchor.start_screen.1) / zoom;
                        this.position = (anchor.start_world.0 + dx, anchor.start_world.1 + dy);
                        cx.notify();
                    } else {
                        this.drag_anchor = None;
                    }
                }
            }))
            // Double-click fires the host's inline-edit handler. Single
            // clicks still fall through to the canvas's own click routing
            // for selection, so this never races with selection.
            .on_click(cx.listener(|this, event: &ClickEvent, window, cx| {
                if event.click_count() >= 2
                    && let Some(ref handler) = this.on_double_click
                {
                    cx.stop_propagation();
                    handler(window, cx);
                }
            }));

        // Header: title + description + optional action
        let header_bg = if self.selected { t_accent } else { t_hover };
        let title_color = if self.selected {
            t_text_on_accent
        } else {
            t_text
        };
        let mut header = div()
            .p(px(SPACING_12))
            .bg(header_bg)
            .border_b_1()
            .border_color(t_border)
            .rounded_t(t_radius_lg)
            .flex()
            .items_center()
            .justify_between()
            .hover(move |el| el.opacity(0.9));

        // Title + description column
        let mut title_col = div().flex().flex_col().gap(px(2.0));

        title_col = title_col.child(
            div()
                .text_style(TextStyle::Subheadline, &theme)
                .font_weight(theme.effective_weight(FontWeight::SEMIBOLD))
                .text_color(title_color)
                .child(self.title.clone()),
        );

        if let Some(ref desc) = self.description {
            title_col = title_col.child(
                div()
                    .text_style(TextStyle::Caption1, &theme)
                    .text_color(t_text_muted)
                    .child(desc.clone()),
            );
        }

        header = header.child(title_col);

        if let Some(ref action_builder) = self.action_builder {
            header = header.child(action_builder(window, cx));
        }

        card = card.child(header);

        // Handle dots (left=target, right=source)
        let mut handle_container = div().relative();
        if self.show_target_handle {
            handle_container = handle_container.child(
                div()
                    .absolute()
                    .left(px(-4.0))
                    .top(px(14.0))
                    .size(px(8.0))
                    .rounded(t_radius_full)
                    .bg(t_accent),
            );
        }
        if self.show_source_handle {
            handle_container = handle_container.child(
                div()
                    .absolute()
                    .right(px(-4.0))
                    .top(px(14.0))
                    .size(px(8.0))
                    .rounded(t_radius_full)
                    .bg(t_success),
            );
        }
        card = card.child(handle_container);

        // Ports section
        if !self.input_ports.is_empty() || !self.output_ports.is_empty() {
            let mut ports_container = div()
                .px(t_spacing_sm)
                .py(t_spacing_sm)
                .flex()
                .flex_col()
                .gap(t_spacing_xs);

            for port in &self.input_ports {
                let color = match port.port_type {
                    PortType::Input => t_accent,
                    PortType::Output => t_success,
                };
                ports_container = ports_container.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(t_spacing_xs)
                        .child(div().size(px(8.0)).rounded(t_radius_full).bg(color))
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, &theme)
                                .text_color(t_text_muted)
                                .child(SharedString::from(port.name.clone())),
                        ),
                );
            }
            for port in &self.output_ports {
                let color = match port.port_type {
                    PortType::Input => t_accent,
                    PortType::Output => t_success,
                };
                ports_container = ports_container.child(
                    div().flex().flex_row_reverse().child(
                        div()
                            .flex()
                            .items_center()
                            .gap(t_spacing_xs)
                            .child(div().size(px(8.0)).rounded(t_radius_full).bg(color))
                            .child(
                                div()
                                    .text_style(TextStyle::Caption1, &theme)
                                    .text_color(t_text_muted)
                                    .child(SharedString::from(port.name.clone())),
                            ),
                    ),
                );
            }
            card = card.child(ports_container);
        }

        // Content area (optional, with top border)
        if let Some(ref content_builder) = self.content_builder {
            card = card.child(
                div()
                    .border_t_1()
                    .border_color(t_border)
                    .p(px(SPACING_12))
                    .child(content_builder(window, cx)),
            );
        }

        // Footer area (optional, muted bg with rounded bottom)
        if let Some(ref footer_builder) = self.footer_builder {
            card = card.child(
                div()
                    .border_t_1()
                    .border_color(t_border)
                    .bg(t_hover)
                    .rounded_b(t_radius_lg)
                    .p(px(SPACING_12))
                    .child(footer_builder(window, cx)),
            );
        }

        // Keyboard handler: unmodified Enter/Space toggles selection
        card = card.on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
            let m = &event.keystroke.modifiers;
            let no_modifiers = !m.shift && !m.control && !m.alt && !m.platform;
            if no_modifiers && crate::foundations::keyboard::is_activation_key(event) {
                cx.stop_propagation();
                this.toggle_selected(window, cx);
            }
        }));

        card
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NODE_MIN_WIDTH, NodeAction, NodeContent, NodeDescription, NodeFooter, NodeHeader,
        NodeTitle, PORT_SPACING, PORT_START_Y, Port, PortType,
    };
    use core::prelude::v1::test;
    use gpui::div;

    #[test]
    fn port_type_equality() {
        assert_eq!(PortType::Input, PortType::Input);
        assert_ne!(PortType::Input, PortType::Output);
    }

    #[test]
    fn port_input_constructor() {
        let p = Port::input("data");
        assert_eq!(p.name, "data");
        assert_eq!(p.port_type, PortType::Input);
    }

    #[test]
    fn port_output_constructor() {
        let p = Port::output("result");
        assert_eq!(p.name, "result");
        assert_eq!(p.port_type, PortType::Output);
    }

    #[test]
    fn port_clone() {
        let p = Port::input("x");
        let p2 = p.clone();
        assert_eq!(p.name, p2.name);
        assert_eq!(p.port_type, p2.port_type);
    }

    #[test]
    fn port_type_copy() {
        let t = PortType::Input;
        let t2 = t;
        assert_eq!(t, t2);
    }

    #[test]
    fn port_type_debug() {
        assert!(format!("{:?}", PortType::Input).contains("Input"));
        assert!(format!("{:?}", PortType::Output).contains("Output"));
    }

    #[test]
    fn port_debug() {
        let p = Port::input("data_in");
        let dbg = format!("{:?}", p);
        assert!(dbg.contains("data_in"));
        assert!(dbg.contains("Input"));
    }

    // ── Port position tests ─────────────────────────────────────────

    /// Helper: simulate a WorkflowNode's port_positions logic without Entity.
    fn compute_port_positions(
        pos: (f32, f32),
        inputs: &[&str],
        outputs: &[&str],
    ) -> Vec<(String, f32, f32, PortType)> {
        let mut result = Vec::new();
        for (i, name) in inputs.iter().enumerate() {
            result.push((
                name.to_string(),
                pos.0,
                pos.1 + PORT_START_Y + i as f32 * PORT_SPACING,
                PortType::Input,
            ));
        }
        for (i, name) in outputs.iter().enumerate() {
            result.push((
                name.to_string(),
                pos.0 + NODE_MIN_WIDTH,
                pos.1 + PORT_START_Y + (inputs.len() + i) as f32 * PORT_SPACING,
                PortType::Output,
            ));
        }
        result
    }

    #[test]
    fn port_positions_empty() {
        let positions = compute_port_positions((100.0, 200.0), &[], &[]);
        assert!(positions.is_empty());
    }

    #[test]
    fn port_positions_with_ports() {
        let positions = compute_port_positions((100.0, 200.0), &["in1"], &["out1"]);
        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].0, "in1");
        assert_eq!(positions[0].3, PortType::Input);
        assert_eq!(positions[1].0, "out1");
        assert_eq!(positions[1].3, PortType::Output);
    }

    #[test]
    fn port_position_input_on_left_edge() {
        let positions = compute_port_positions((100.0, 200.0), &["in1"], &[]);
        assert!((positions[0].1 - 100.0).abs() < f32::EPSILON); // x = node.x
    }

    #[test]
    fn port_position_output_on_right_edge() {
        let positions = compute_port_positions((100.0, 200.0), &[], &["out1"]);
        assert!((positions[0].1 - (100.0 + NODE_MIN_WIDTH)).abs() < f32::EPSILON);
    }

    #[test]
    fn port_positions_y_spacing() {
        let positions = compute_port_positions((0.0, 0.0), &["a", "b", "c"], &[]);
        let y0 = positions[0].2;
        let y1 = positions[1].2;
        let y2 = positions[2].2;
        assert!((y1 - y0 - PORT_SPACING).abs() < f32::EPSILON);
        assert!((y2 - y1 - PORT_SPACING).abs() < f32::EPSILON);
    }

    // ── Sub-component constructor tests ─────────────────────────────

    #[test]
    fn node_header_new_creates_empty_children() {
        let header = NodeHeader::new();
        assert!(header.children.is_empty());
    }

    #[test]
    fn node_title_stores_text() {
        let title = NodeTitle::new("Process Data");
        assert_eq!(title.text.as_ref(), "Process Data");
    }

    #[test]
    fn node_description_stores_text() {
        let desc = NodeDescription::new("Transforms input");
        assert_eq!(desc.text.as_ref(), "Transforms input");
    }

    #[test]
    fn node_content_new_creates_empty_children() {
        let content = NodeContent::new();
        assert!(content.children.is_empty());
    }

    #[test]
    fn node_footer_new_creates_empty_children() {
        let footer = NodeFooter::new();
        assert!(footer.children.is_empty());
    }

    #[test]
    fn node_header_child_adds_element() {
        let header = NodeHeader::new().child(div()).child(div());
        assert_eq!(header.children.len(), 2);
    }

    #[test]
    fn node_content_child_adds_element() {
        let content = NodeContent::new().child(div()).child(div()).child(div());
        assert_eq!(content.children.len(), 3);
    }

    #[test]
    fn node_footer_child_adds_element() {
        let footer = NodeFooter::new().child(div());
        assert_eq!(footer.children.len(), 1);
    }

    #[test]
    fn node_action_wraps_child() {
        let _action = NodeAction::new(div());
    }

    // ── Explicit-size contract ────────────────────────────────────
    //
    // These tests pin the arithmetic that `WorkflowNode::set_size` and
    // `effective_size` expose. They intentionally use the module's
    // public API surface so the contract can be rewritten without
    // touching gpui's Entity machinery.

    #[test]
    fn node_min_width_constant_is_384() {
        assert_eq!(super::NODE_MIN_WIDTH, 384.0);
    }

    #[test]
    fn node_min_height_constant_is_80() {
        // Mirrors the NODE_HEIGHT heuristic used in auto-sizing so
        // resize never produces a node smaller than what the layout
        // already expected.
        assert_eq!(super::NODE_MIN_HEIGHT, 80.0);
    }

    #[test]
    fn set_size_clamps_below_minimum() {
        // Pure arithmetic check — replicates the clamp that
        // `set_size` applies:  (w.max(MIN_W), h.max(MIN_H)).
        let w = 100.0_f32.max(super::NODE_MIN_WIDTH);
        let h = 20.0_f32.max(super::NODE_MIN_HEIGHT);
        assert_eq!(w, super::NODE_MIN_WIDTH);
        assert_eq!(h, super::NODE_MIN_HEIGHT);
    }

    #[test]
    fn set_size_preserves_values_above_minimum() {
        let w = 600.0_f32.max(super::NODE_MIN_WIDTH);
        let h = 400.0_f32.max(super::NODE_MIN_HEIGHT);
        assert_eq!(w, 600.0);
        assert_eq!(h, 400.0);
    }
}
