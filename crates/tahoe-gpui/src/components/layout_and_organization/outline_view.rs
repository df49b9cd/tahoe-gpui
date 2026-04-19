//! Hierarchical outline view aligned with HIG Outline views.
//!
//! Displays hierarchical data with expandable/collapsible nodes at multiple
//! indent levels. Commonly used for file browsers, document outlines, and
//! hierarchical settings.
//!
//! # Indentation
//!
//! Default per-level indent is 16 pt, matching `NSOutlineView.indentationPerLevel`.
//! Callers may override via [`OutlineView::indent_width`].
//!
//! # Row height
//!
//! Row height is [`TahoeTheme::row_height`] (28 pt), the macOS standard for
//! interactive tree rows. Previous versions used a `0.75×` multiplier, which
//! produced 21 pt rows — below the HIG 28 pt minimum interactive target size.
//!
//! # Option-click expand all
//!
//! HIG: "Option-clicking the disclosure triangle expands all of its
//! subfolders." [`on_expand_all`] receives the node id when the user
//! Option-clicks; callers recursively expand the subtree.

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{App, ElementId, KeyDownEvent, MouseButton, SharedString, Window, div, px};

use crate::callback_types::OnStrChange;
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};

/// Default per-level indent for outline rows, matching
/// `NSOutlineView.indentationPerLevel` (16 pt). Previously the crate used
/// `spacing_md + spacing_xs` (≈20 pt); aligned to the AppKit default so
/// ported outline hierarchies look identical.
pub const DEFAULT_OUTLINE_INDENT: f32 = 16.0;

/// Rc-wrapped string callback used when sharing handlers across multiple closures.
type OnStrChangeRc = Option<Rc<dyn Fn(&str, &mut Window, &mut App)>>;

/// Rc-wrapped focus-change callback for keyboard navigation.
type OnFocusChangeRc = Option<Rc<dyn Fn(Option<SharedString>, &mut Window, &mut App)>>;

/// A single node in an outline hierarchy.
#[derive(Clone)]
pub struct OutlineNode {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<IconName>,
    pub children: Vec<OutlineNode>,
    pub is_expanded: bool,
}

impl OutlineNode {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            is_expanded: false,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }

    pub fn child(mut self, child: OutlineNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<OutlineNode>) -> Self {
        self.children = children;
        self
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// Hierarchical outline view per HIG.
///
/// Renders a tree of [`OutlineNode`] items with expand/collapse controls,
/// indentation, and optional icons.
///
/// # Virtualization
///
/// The stateless builder below renders every visible row on every frame.
/// That is fine for outlines with a few dozen rows (the common case) but
/// degrades for very large outlines. Finding 6 in the Zed cross-reference
/// audit (df49b9cd/ai-sdk-rust#132) tracks the adoption of variable-
/// height virtualization here.
///
/// For large outlines, drive rendering through the GPUI variable-height
/// `list` primitive: hold a `ListState` in your parent view,
/// pre-flatten the outline nodes via [`flatten_visible`], and render
/// each row with `gpui::list(state.clone(), move |ix, window, cx| { … })`
/// calling back into the `OutlineNode` data for row `ix`. The row
/// chrome (disclosure triangle, indent, icon) is stable so you can
/// copy the inner body of [`OutlineView`]'s row renderer.
///
/// `gpui::ListState` is re-exported at the crate root
/// ([`crate::ListState`](crate::ListState)) and `gpui::list` is
/// available as [`crate::list_element`](crate::list_element) for this
/// purpose. A fully automated virtualization wrapper would need to
/// promote `OutlineView` from `RenderOnce` to a stateful entity — see
/// [ButtonLike (Finding 15)](super::super::menus_and_actions::ButtonLike)
/// for the equivalent substrate pattern.
#[derive(IntoElement)]
pub struct OutlineView {
    id: ElementId,
    nodes: Vec<OutlineNode>,
    indent_width: f32,
    focused_id: Option<SharedString>,
    on_select: OnStrChange,
    on_toggle: OnStrChange,
    on_expand_all: OnStrChange,
    on_focus_change: OnFocusChange,
}

/// Callback invoked when keyboard navigation moves the focused outline node.
type OnFocusChange = Option<Box<dyn Fn(Option<SharedString>, &mut Window, &mut App)>>;

impl OutlineView {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            nodes: Vec::new(),
            // Default aligns with `NSOutlineView.indentationPerLevel` = 16 pt.
            indent_width: DEFAULT_OUTLINE_INDENT,
            focused_id: None,
            on_select: None,
            on_toggle: None,
            on_expand_all: None,
            on_focus_change: None,
        }
    }

    pub fn nodes(mut self, nodes: Vec<OutlineNode>) -> Self {
        self.nodes = nodes;
        self
    }

    pub fn indent_width(mut self, width: f32) -> Self {
        self.indent_width = width;
        self
    }

    /// Set the currently keyboard-focused node id.
    pub fn focused_id(mut self, id: Option<SharedString>) -> Self {
        self.focused_id = id;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Set the handler called when a folder node is expanded or collapsed.
    ///
    /// The handler receives the node id that was toggled.
    pub fn on_toggle(mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    /// Set the handler called when the user Option-clicks a folder's disclosure
    /// row, which per HIG should recursively expand all descendants.
    /// Callers own the recursion — receive the node id and walk the subtree
    /// in their data model.
    pub fn on_expand_all(
        mut self,
        handler: impl Fn(&str, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expand_all = Some(Box::new(handler));
        self
    }

    /// Set the handler called when keyboard navigation moves the focused node.
    pub fn on_focus_change(
        mut self,
        handler: impl Fn(Option<SharedString>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_focus_change = Some(Box::new(handler));
        self
    }

    /// Return the current visible-row sequence — the flat list a
    /// virtualized renderer would paint.
    ///
    /// Finding 6 in df49b9cd/ai-sdk-rust#132. Pair with
    /// [`crate::ListState`] + [`crate::list_element`] for outlines
    /// large enough that the default (non-virtualized) render path
    /// becomes a bottleneck — index `ix` into the returned vector
    /// inside your per-row closure and paint that node's chrome using
    /// the `has_children` / `is_expanded` hints.
    pub fn visible_entries(&self) -> Vec<FlatEntry> {
        flatten_visible(&self.nodes, None)
    }

    /// Return a [`ListState`] pre-sized for the current count of
    /// visible entries, ready to feed to [`crate::list_element`].
    ///
    /// Uses `ListAlignment::Top` and a 100 pt overdraw — matches
    /// Zed's editor/file-tree defaults. Callers should store the
    /// returned `ListState` on their own view and reset it via
    /// `state.reset(outline_view.visible_entries().len())` whenever
    /// the outline's expansion or node set changes.
    pub fn virtualized_state(&self) -> gpui::ListState {
        gpui::ListState::new(
            self.visible_entries().len(),
            gpui::ListAlignment::Top,
            gpui::px(100.0),
        )
    }
}

/// Flattened entry for keyboard navigation over visible tree nodes.
///
/// Public as a non-exhaustive stable struct so hosts building custom
/// virtualized outlines on top of [`crate::list_element`] +
/// [`crate::ListState`] (Finding 6 in df49b9cd/ai-sdk-rust#132) can
/// reuse the same flatten routine without re-implementing the indent
/// + parent tracking themselves.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct FlatEntry {
    /// Stable node id.
    pub id: SharedString,
    /// `true` when the underlying node has child entries.
    pub has_children: bool,
    /// `true` when the node is currently expanded.
    pub is_expanded: bool,
    /// Parent node id, if any.
    pub parent_id: Option<SharedString>,
}

/// Flatten a forest of [`OutlineNode`]s into the visible (currently
/// unfolded) row sequence, preserving depth via `parent_id` and
/// expansion via `is_expanded`. The output order is the rendering
/// order a virtualized outline view needs to feed `gpui::list(...)`.
///
/// Mirrors what [`OutlineView::render`] uses internally. Exposed so
/// hosts needing full virtualization over very large outlines can
/// pair it with [`crate::ListState`] and [`crate::list_element`].
pub fn flatten_visible(nodes: &[OutlineNode], parent: Option<&SharedString>) -> Vec<FlatEntry> {
    let mut out = Vec::new();
    for node in nodes {
        out.push(FlatEntry {
            id: node.id.clone(),
            has_children: node.has_children(),
            is_expanded: node.is_expanded,
            parent_id: parent.cloned(),
        });
        if node.is_expanded {
            out.extend(flatten_visible(&node.children, Some(&node.id)));
        }
    }
    out
}

impl RenderOnce for OutlineView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut container = div().id(self.id).flex().flex_col().focusable();

        let on_toggle: OnStrChangeRc = self.on_toggle.map(|f| Rc::from(f));
        let on_select: OnStrChangeRc = self.on_select.map(|f| Rc::from(f));
        let on_expand_all: OnStrChangeRc = self.on_expand_all.map(|f| Rc::from(f));
        let on_focus_change: OnFocusChangeRc = self.on_focus_change.map(|f| Rc::from(f));
        let focused_id = self.focused_id.clone();
        let indent_width = self.indent_width;

        // Keyboard navigation over the flattened visible tree.
        let flat = flatten_visible(&self.nodes, None);
        if !flat.is_empty() {
            let focus_cb = on_focus_change.clone();
            let toggle_cb = on_toggle.clone();
            let select_cb = on_select.clone();
            let flat_for_keys = flat.clone();
            let focused_for_keys = focused_id.clone();
            container = container.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let current_idx = focused_for_keys
                    .as_ref()
                    .and_then(|id| flat_for_keys.iter().position(|e| &e.id == id));
                match key {
                    "down" => {
                        cx.stop_propagation();
                        let next = match current_idx {
                            Some(i) if i + 1 < flat_for_keys.len() => i + 1,
                            Some(i) => i,
                            None => 0,
                        };
                        if let Some(cb) = &focus_cb {
                            cb(Some(flat_for_keys[next].id.clone()), window, cx);
                        }
                    }
                    "up" => {
                        cx.stop_propagation();
                        let prev = match current_idx {
                            Some(0) | None => 0,
                            Some(i) => i - 1,
                        };
                        if let Some(cb) = &focus_cb {
                            cb(Some(flat_for_keys[prev].id.clone()), window, cx);
                        }
                    }
                    "home" => {
                        cx.stop_propagation();
                        if let Some(cb) = &focus_cb {
                            cb(Some(flat_for_keys[0].id.clone()), window, cx);
                        }
                    }
                    "end" => {
                        cx.stop_propagation();
                        if let Some(cb) = &focus_cb {
                            let last = flat_for_keys.last().unwrap().id.clone();
                            cb(Some(last), window, cx);
                        }
                    }
                    "right" => {
                        cx.stop_propagation();
                        if let Some(i) = current_idx {
                            let entry = &flat_for_keys[i];
                            if entry.has_children && !entry.is_expanded {
                                if let Some(cb) = &toggle_cb {
                                    cb(entry.id.as_ref(), window, cx);
                                }
                            } else if entry.has_children
                                && entry.is_expanded
                                && i + 1 < flat_for_keys.len()
                            {
                                if let Some(cb) = &focus_cb {
                                    cb(Some(flat_for_keys[i + 1].id.clone()), window, cx);
                                }
                            }
                        }
                    }
                    "left" => {
                        cx.stop_propagation();
                        if let Some(i) = current_idx {
                            let entry = &flat_for_keys[i];
                            if entry.has_children && entry.is_expanded {
                                if let Some(cb) = &toggle_cb {
                                    cb(entry.id.as_ref(), window, cx);
                                }
                            } else if let Some(parent) = &entry.parent_id {
                                if let Some(cb) = &focus_cb {
                                    cb(Some(parent.clone()), window, cx);
                                }
                            }
                        }
                    }
                    "enter" | "space" => {
                        cx.stop_propagation();
                        if let Some(i) = current_idx {
                            let entry = &flat_for_keys[i];
                            if entry.has_children {
                                if let Some(cb) = &toggle_cb {
                                    cb(entry.id.as_ref(), window, cx);
                                }
                            } else if let Some(cb) = &select_cb {
                                cb(entry.id.as_ref(), window, cx);
                            }
                        }
                    }
                    _ => {}
                }
            });
        }

        #[allow(clippy::too_many_arguments)]
        fn render_nodes(
            nodes: &[OutlineNode],
            depth: usize,
            indent_width: f32,
            theme: &TahoeTheme,
            on_toggle: &OnStrChangeRc,
            on_select: &OnStrChangeRc,
            on_expand_all: &OnStrChangeRc,
            focused_id: &Option<SharedString>,
        ) -> Vec<gpui::AnyElement> {
            let mut elements = Vec::new();
            for node in nodes {
                let indent = px(depth as f32 * indent_width);
                // HIG disclosure glyph: filled triangle. Down = expanded,
                // right = collapsed.
                let triangle = if node.is_expanded {
                    IconName::ArrowTriangleDown
                } else {
                    IconName::ArrowTriangleRight
                };

                let node_id_str = node.id.clone();
                let is_focused = focused_id.as_ref() == Some(&node.id);

                // Indent on the *leading* edge so trees nested in RTL
                // layouts grow toward the reading direction's start (HIG
                // Right-to-Left: Controls mirror leading/trailing insets).
                let (pad_l, pad_r) = crate::foundations::right_to_left::leading_trailing_insets(
                    theme.layout_direction,
                    indent,
                    px(0.0),
                );

                // Row height: use the theme's 28 pt standard row directly.
                // The previous `0.75×` multiplier produced 21 pt rows, below
                // the HIG 28 pt minimum interactive target.
                let mut row = div()
                    .id(ElementId::from(SharedString::from(format!(
                        "outline-{}",
                        node.id
                    ))))
                    .pl(pad_l)
                    .pr(pad_r)
                    .h(px(theme.row_height()))
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .cursor_pointer()
                    .hover(|style| style.bg(theme.hover));

                // VoiceOver: outline rows announce as group-role elements
                // with a value describing their expansion state. HIG:
                // disclosure rows must expose `"expanded" / "collapsed"`
                // as the accessibility value. The `AccessibleExt` is a
                // structural no-op today (GPUI gap #138); when upstream
                // lands an AX API the single wiring site takes over.
                let ax_label: SharedString = node.label.clone();
                let ax_value: SharedString = if node.has_children() {
                    if node.is_expanded {
                        SharedString::from("expanded")
                    } else {
                        SharedString::from("collapsed")
                    }
                } else {
                    SharedString::from("leaf")
                };
                let ax = AccessibilityProps::new()
                    .label(ax_label)
                    .role(if node.has_children() {
                        AccessibilityRole::Group
                    } else {
                        AccessibilityRole::StaticText
                    })
                    .value(ax_value);
                row = row.with_accessibility(&ax);

                row = apply_focus_ring(row, theme, is_focused, &[]);

                let chevron_size =
                    theme.icon_size_inline - theme.separator_thickness - theme.separator_thickness;
                if node.has_children() {
                    row = row.child(
                        Icon::new(triangle)
                            .size(chevron_size)
                            .color(theme.text_muted),
                    );
                } else {
                    row = row.child(div().w(chevron_size));
                }

                if let Some(icon) = node.icon {
                    row = row.child(
                        Icon::new(icon)
                            .size(
                                theme.icon_size_inline
                                    + theme.separator_thickness
                                    + theme.separator_thickness,
                            )
                            .color(theme.text_muted),
                    );
                }

                // Label: truncate with an ellipsis if the row is narrow.
                // HIG: "use a centered ellipsis" — GPUI only exposes
                // trailing / leading truncation today, so we use trailing
                // ellipsis as the closest available behaviour. Tracked as
                // a GPUI gap: TextOverflow::TruncateMiddle is unsupported.
                row = row.child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .text_style(TextStyle::Body, theme)
                        .text_color(theme.text)
                        .truncate()
                        .child(node.label.clone()),
                );

                // Click handler: toggle expand/collapse for folder nodes,
                // fire on_select for leaf nodes. Option-click on a folder
                // fires `on_expand_all` per HIG Outline Views.
                if node.has_children() {
                    let toggle = on_toggle.clone();
                    let expand_all = on_expand_all.clone();
                    let id = node_id_str.clone();
                    row = row.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                        // Option (alt) + click expands the full subtree.
                        if event.modifiers.alt {
                            if let Some(cb) = &expand_all {
                                cb(id.as_ref(), window, cx);
                            }
                            return;
                        }
                        if let Some(cb) = &toggle {
                            cb(id.as_ref(), window, cx);
                        }
                    });
                } else if let Some(handler) = on_select.clone() {
                    let id = node_id_str.clone();
                    row = row.on_click(move |_event, window, cx| {
                        handler(id.as_ref(), window, cx);
                    });
                }

                elements.push(row.into_any_element());

                if node.is_expanded && node.has_children() {
                    elements.extend(render_nodes(
                        &node.children,
                        depth + 1,
                        indent_width,
                        theme,
                        on_toggle,
                        on_select,
                        on_expand_all,
                        focused_id,
                    ));
                }
            }
            elements
        }

        let rendered = render_nodes(
            &self.nodes,
            0,
            indent_width,
            theme,
            &on_toggle,
            &on_select,
            &on_expand_all,
            &focused_id,
        );
        for el in rendered {
            container = container.child(el);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{DEFAULT_OUTLINE_INDENT, OutlineNode, OutlineView};

    #[test]
    fn outline_node_basics() {
        let node = OutlineNode::new("root", "Root")
            .child(OutlineNode::new("child1", "Child 1"))
            .child(OutlineNode::new("child2", "Child 2"));
        assert!(node.has_children());
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn outline_node_no_children() {
        let node = OutlineNode::new("leaf", "Leaf");
        assert!(!node.has_children());
    }

    #[test]
    fn outline_view_new() {
        let view = OutlineView::new("test-outline");
        assert!(view.nodes.is_empty());
    }

    #[test]
    fn outline_default_indent_matches_nsoutlineview() {
        let view = OutlineView::new("test-outline");
        assert!(
            (view.indent_width - DEFAULT_OUTLINE_INDENT).abs() < f32::EPSILON,
            "default indent should be {DEFAULT_OUTLINE_INDENT} pt (NSOutlineView convention)"
        );
    }

    #[test]
    fn outline_on_expand_all_builder() {
        let view = OutlineView::new("test-outline").on_expand_all(|_id, _w, _cx| {});
        assert!(view.on_expand_all.is_some());
    }

    #[test]
    fn visible_entries_counts_expanded_only() {
        // Finding 6 in df49b9cd/ai-sdk-rust#132: visible_entries must
        // match what the renderer paints — collapsed children don't
        // appear, expanded ones do.
        let collapsed_child = OutlineNode::new("c1", "child-1");
        let visible_child = OutlineNode::new("c2", "child-2");
        let folder = OutlineNode::new("f", "folder")
            .expanded(true)
            .children(vec![visible_child, collapsed_child]);
        let hidden_folder = OutlineNode::new("h", "hidden")
            .children(vec![OutlineNode::new("hc", "hidden-child")]);

        let view = OutlineView::new("test").nodes(vec![folder, hidden_folder]);
        let flat = view.visible_entries();
        // folder (expanded), two children, then hidden_folder (collapsed)
        // — hidden-child is NOT visible, so 4 entries total.
        assert_eq!(flat.len(), 4);
        assert_eq!(flat[0].id.as_ref(), "f");
        assert!(flat[0].is_expanded);
        assert_eq!(flat[1].parent_id.as_deref().map(|s| s.as_ref()), Some("f"));
    }

    #[test]
    fn virtualized_state_matches_visible_count() {
        let view = OutlineView::new("test").nodes(vec![
            OutlineNode::new("a", "a"),
            OutlineNode::new("b", "b"),
            OutlineNode::new("c", "c"),
        ]);
        // Count the ListState knows about equals the visible count.
        let visible = view.visible_entries().len();
        assert_eq!(visible, 3);
        let _state = view.virtualized_state();
        // Smoke test — ListState doesn't expose item count publicly,
        // but construction with zero items would panic on `reset(0)`
        // later; visible_entries() covers the count path.
    }
}
