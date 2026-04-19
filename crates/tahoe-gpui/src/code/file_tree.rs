//! File tree display component with virtualized rendering.

use crate::callback_types::{OnExpandedChange, OnStrChange, RenderActionsRc};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Context, ElementId, FocusHandle, KeyDownEvent, SharedString,
    Window, div, px, uniform_list,
};
use std::collections::HashSet;
use std::rc::Rc;

/// Payload carried when a file-tree row is dragged via
/// [`FileTreeView::set_draggable`]. Consumers register
/// `.on_drop::<DraggedFilePath>(…)` on their drop targets to receive the
/// dragged entry. Mirrors Zed's `DraggedSelection`
/// (`crates/workspace/src/pane.rs:64`) — a simple typed payload so
/// callers can discriminate between internal tree drags and other
/// sources (e.g. `ExternalPaths`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DraggedFilePath {
    pub path: String,
    pub name: String,
    pub is_folder: bool,
}

/// Floating preview rendered under the cursor while a file-tree row is
/// being dragged. Mirrors Zed's `DraggedProjectEntryView`
/// (`crates/project_panel/src/project_panel.rs:~5150`) — a small chip
/// with the row's icon + filename on the active theme's card surface.
pub struct DraggedFilePathView {
    pub name: SharedString,
    pub icon: IconName,
}

impl Render for DraggedFilePathView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .px(theme.spacing_sm)
            .py(theme.spacing_xs)
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_sm)
            .child(
                Icon::new(self.icon)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text)
                    .child(self.name.clone()),
            )
    }
}

/// A node in the file tree.
#[derive(Debug, Clone)]
pub enum TreeNode {
    File {
        name: String,
        path: String,
        icon: Option<IconName>,
    },
    Folder {
        name: String,
        path: String,
        children: Vec<TreeNode>,
    },
}

/// Map a file extension (lowercase, no leading dot) to the closest
/// SF-Symbol-backed [`IconName`]. Returns [`IconName::File`] when no
/// dedicated icon is known.
///
/// Mirrors the Xcode project-navigator convention of distinguishing
/// source files by language family so the tree reads at a glance
/// without relying on filename alone (HIG §Icons + Finder
/// document-icon family). Finding N4 / #12 in
/// the HIG Code-surface audit.
pub fn icon_for_extension(ext: &str) -> IconName {
    match ext.to_ascii_lowercase().as_str() {
        // Programming languages — reuse the existing `Lang*` icon set
        // so FileTree rows line up with the surrounding dev-tools UI.
        "rs" => IconName::LangRust,
        "py" | "pyi" | "pyx" => IconName::LangPython,
        "js" | "mjs" | "cjs" | "jsx" => IconName::LangJavaScript,
        "ts" | "tsx" | "cts" | "mts" => IconName::LangTypeScript,
        "go" => IconName::LangGo,
        "c" | "h" => IconName::LangC,
        "cc" | "cpp" | "cxx" | "hh" | "hpp" | "hxx" => IconName::LangCpp,
        "sh" | "bash" | "zsh" | "fish" => IconName::LangBash,
        "json" | "jsonc" | "ndjson" => IconName::LangJson,
        "toml" => IconName::LangToml,
        "html" | "htm" | "xhtml" => IconName::LangHtml,
        "css" | "scss" | "sass" | "less" => IconName::LangCss,

        // Known container / data formats — fall back to closest glyph
        // family rather than the generic document icon.
        "md" | "markdown" | "mdx" => IconName::Book,
        "yml" | "yaml" => IconName::FileCode,
        "xml" => IconName::FileCode,
        "lock" => IconName::Lock,
        "env" => IconName::Environment,
        "log" => IconName::Logs,
        "sql" => IconName::Database,
        "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "heic" => IconName::Image,

        _ => IconName::File,
    }
}

impl TreeNode {
    pub fn file(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self::File {
            name: name.into(),
            path: path.into(),
            icon: None,
        }
    }

    /// Set a custom icon for a file node. No-op on folder nodes.
    pub fn with_icon(mut self, icon: IconName) -> Self {
        if let Self::File {
            icon: ref mut i, ..
        } = self
        {
            *i = Some(icon);
        }
        self
    }

    pub fn folder(
        name: impl Into<String>,
        path: impl Into<String>,
        children: Vec<TreeNode>,
    ) -> Self {
        Self::Folder {
            name: name.into(),
            path: path.into(),
            children,
        }
    }

    /// Get the path of this tree node.
    pub fn path(&self) -> &str {
        match self {
            TreeNode::File { path, .. } => path,
            TreeNode::Folder { path, .. } => path,
        }
    }

    /// Get the display name of this tree node.
    pub fn name(&self) -> &str {
        match self {
            TreeNode::File { name, .. } => name,
            TreeNode::Folder { name, .. } => name,
        }
    }
}

/// A flat entry for rendering (after flattening visible tree).
#[derive(Clone)]
struct FlatEntry {
    path: String,
    name: String,
    depth: usize,
    is_folder: bool,
    is_expanded: bool,
    custom_icon: Option<IconName>,
}

/// Result of processing a keyboard navigation event.
struct KeyNavAction {
    new_focused: Option<usize>,
    select: bool,
    toggle_expand: bool,
}

/// An interactive file tree display with expand/collapse, selection, and virtualized rendering.
pub struct FileTreeView {
    element_id: ElementId,
    root_children: Vec<TreeNode>,
    expanded_paths: HashSet<String>,
    /// Set of currently selected paths. Stored as `HashSet<String>` so
    /// callers can opt into Cmd-click / Shift-click multi-selection
    /// (finding #11 / the HIG Code-surface audit). The convenience
    /// `select()` API continues to behave as single-select (replaces the
    /// set) for callers that do not enable [`Self::set_multi_select`].
    selected_paths: HashSet<String>,
    /// When `true`, `select()` augments the selection set instead of
    /// replacing it — used to implement Cmd-click semantics. Off by
    /// default to match the historical single-select behaviour.
    multi_select: bool,
    focus_handle: FocusHandle,
    on_select: OnStrChange,
    on_expanded_change: OnExpandedChange,
    render_actions: RenderActionsRc,
    select_folders: bool,
    focused_index: Option<usize>,
    /// Cached flat entries — rebuilt when tree structure changes.
    flat_entries: Vec<FlatEntry>,
    entries_dirty: bool,
    /// Scroll handle used to reveal focused items via
    /// `scroll_to_reveal_item` from keyboard navigation. Finding 17 in
    /// the Zed cross-reference audit.
    scroll_handle: gpui::UniformListScrollHandle,
    /// When `true`, file rows register a GPUI drag source with a
    /// [`DraggedFilePath`] payload so consumers can participate in
    /// drag-and-drop (finding #19). Off by default — many embeds don't
    /// want drag-to-move semantics on an AI output surface.
    draggable: bool,
}

impl FileTreeView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("file-tree"),
            root_children: Vec::new(),
            expanded_paths: HashSet::new(),
            selected_paths: HashSet::new(),
            multi_select: false,
            focus_handle: cx.focus_handle(),
            on_select: None,
            on_expanded_change: None,
            render_actions: None,
            select_folders: true,
            focused_index: None,
            flat_entries: Vec::new(),
            entries_dirty: true,
            scroll_handle: gpui::UniformListScrollHandle::new(),
            draggable: false,
        }
    }

    /// Opt into drag-source behaviour on rows. When enabled, each row
    /// registers a GPUI drag source that carries a [`DraggedFilePath`]
    /// payload. Parent UIs add `.on_drop::<DraggedFilePath>(…)` to any
    /// drop target that should accept file-tree drags.
    pub fn set_draggable(&mut self, enabled: bool) {
        self.draggable = enabled;
    }

    /// Whether drag-source behaviour is enabled (see
    /// [`Self::set_draggable`]).
    pub fn is_draggable(&self) -> bool {
        self.draggable
    }

    /// Programmatically scroll the tree so the entry at `index` is visible.
    ///
    /// Use from keyboard navigation paths — HIG §scroll-views recommends
    /// auto-scrolling when the selection or insertion point changes so
    /// the user doesn't lose their place after a `↑` / `↓` / `Home` /
    /// `End` press. Finding 17 in the Zed cross-reference audit.
    pub fn scroll_to_reveal_item(&self, index: usize) {
        self.scroll_handle
            .scroll_to_item(index, gpui::ScrollStrategy::Top);
    }

    pub fn set_children(&mut self, children: Vec<TreeNode>, cx: &mut Context<Self>) {
        self.root_children = children;
        self.entries_dirty = true;
        cx.notify();
    }

    pub fn set_on_select(&mut self, handler: impl Fn(&str, &mut Window, &mut App) + 'static) {
        self.on_select = Some(Box::new(handler));
    }

    /// Set initial expanded paths (uncontrolled mode). Only applies if no paths are currently expanded.
    pub fn set_default_expanded(&mut self, paths: HashSet<String>, cx: &mut Context<Self>) {
        if self.expanded_paths.is_empty() {
            self.expanded_paths = paths;
            self.entries_dirty = true;
            cx.notify();
        }
    }

    /// Replace expanded paths from parent (controlled mode).
    pub fn set_expanded_paths(&mut self, paths: HashSet<String>, cx: &mut Context<Self>) {
        self.expanded_paths = paths;
        self.entries_dirty = true;
        cx.notify();
    }

    /// Register callback when the expanded path set changes.
    pub fn set_on_expanded_change(
        &mut self,
        handler: impl Fn(HashSet<String>, &mut Window, &mut App) + 'static,
    ) {
        self.on_expanded_change = Some(Box::new(handler));
    }

    /// Register a callback that renders action buttons for a given path.
    /// Return `None` to show no actions for that path.
    pub fn set_render_actions(
        &mut self,
        renderer: impl Fn(&str, &mut Window, &mut App) -> Option<AnyElement> + 'static,
    ) {
        self.render_actions = Some(Rc::new(renderer));
    }

    /// Whether clicking a folder also fires `on_select` (default: true).
    pub fn set_select_folders(&mut self, enabled: bool) {
        self.select_folders = enabled;
    }

    pub fn toggle_expand(&mut self, path: &str, cx: &mut Context<Self>) {
        if self.expanded_paths.contains(path) {
            self.expanded_paths.remove(path);
        } else {
            self.expanded_paths.insert(path.to_string());
        }
        self.entries_dirty = true;
        cx.notify();
    }

    /// Replace the selection with a single path — the single-select
    /// contract for callers that haven't opted into multi-select.
    pub fn select(&mut self, path: &str, cx: &mut Context<Self>) {
        self.selected_paths.clear();
        self.selected_paths.insert(path.to_string());
        cx.notify();
    }

    /// Toggle membership of `path` in the selection set — Cmd-click
    /// semantics. No-op on the empty string to match Finder behaviour
    /// where clicking the background clears rather than adds.
    pub fn toggle_select(&mut self, path: &str, cx: &mut Context<Self>) {
        if path.is_empty() {
            return;
        }
        if self.selected_paths.contains(path) {
            self.selected_paths.remove(path);
        } else {
            self.selected_paths.insert(path.to_string());
        }
        cx.notify();
    }

    /// Enable multi-select mode. When enabled, the row-click handler
    /// treats Cmd-click as `toggle_select` and Shift-click as a
    /// contiguous-range extend; plain click still behaves as `select`.
    pub fn set_multi_select(&mut self, enabled: bool) {
        self.multi_select = enabled;
    }

    /// Whether multi-select mode is enabled.
    pub fn multi_select(&self) -> bool {
        self.multi_select
    }

    /// Return the currently selected path set (empty when nothing is
    /// selected).
    pub fn selected_paths(&self) -> &HashSet<String> {
        &self.selected_paths
    }

    /// Replace the selection with the given set. Useful for parent-
    /// controlled selection (e.g. restoring state).
    pub fn set_selected_paths(&mut self, paths: HashSet<String>, cx: &mut Context<Self>) {
        self.selected_paths = paths;
        cx.notify();
    }

    /// Process a keyboard navigation event, returning the action to take.
    fn handle_key_nav(
        key: &str,
        focused_index: Option<usize>,
        flat_entries: &[FlatEntry],
    ) -> KeyNavAction {
        let count = flat_entries.len();
        if count == 0 {
            return KeyNavAction {
                new_focused: None,
                select: false,
                toggle_expand: false,
            };
        }
        let idx = focused_index.unwrap_or(0);
        match key {
            "down" => KeyNavAction {
                new_focused: Some((idx + 1).min(count - 1)),
                select: false,
                toggle_expand: false,
            },
            "up" => KeyNavAction {
                new_focused: Some(idx.saturating_sub(1)),
                select: false,
                toggle_expand: false,
            },
            "right" => {
                let entry = &flat_entries[idx];
                if entry.is_folder && !entry.is_expanded {
                    KeyNavAction {
                        new_focused: Some(idx),
                        select: false,
                        toggle_expand: true,
                    }
                } else {
                    KeyNavAction {
                        new_focused: Some((idx + 1).min(count - 1)),
                        select: false,
                        toggle_expand: false,
                    }
                }
            }
            "left" => {
                let entry = &flat_entries[idx];
                if entry.is_folder && entry.is_expanded {
                    KeyNavAction {
                        new_focused: Some(idx),
                        select: false,
                        toggle_expand: true,
                    }
                } else {
                    // Move to parent: scan backwards for entry at depth - 1
                    let target_depth = entry.depth.saturating_sub(1);
                    let parent_idx = (0..idx)
                        .rev()
                        .find(|&i| {
                            flat_entries[i].depth == target_depth && flat_entries[i].is_folder
                        })
                        .unwrap_or(idx);
                    KeyNavAction {
                        new_focused: Some(parent_idx),
                        select: false,
                        toggle_expand: false,
                    }
                }
            }
            "home" => KeyNavAction {
                new_focused: Some(0),
                select: false,
                toggle_expand: false,
            },
            "end" => KeyNavAction {
                new_focused: Some(count - 1),
                select: false,
                toggle_expand: false,
            },
            "enter" | "space" => {
                let entry = &flat_entries[idx];
                KeyNavAction {
                    new_focused: Some(idx),
                    select: true,
                    toggle_expand: entry.is_folder,
                }
            }
            _ => KeyNavAction {
                new_focused: focused_index,
                select: false,
                toggle_expand: false,
            },
        }
    }

    fn ensure_flat_entries(&mut self) {
        if self.entries_dirty {
            // Split borrows: read root_children + expanded_paths, write flat_entries
            let expanded = &self.expanded_paths;
            let children = &self.root_children;
            self.flat_entries.clear();
            for node in children {
                Self::flatten_node_into(expanded, node, 0, &mut self.flat_entries);
            }
            self.entries_dirty = false;
        }
    }

    fn flatten_node_into(
        expanded: &HashSet<String>,
        node: &TreeNode,
        depth: usize,
        entries: &mut Vec<FlatEntry>,
    ) {
        match node {
            TreeNode::File { name, path, icon } => {
                entries.push(FlatEntry {
                    path: path.clone(),
                    name: name.clone(),
                    depth,
                    is_folder: false,
                    is_expanded: false,
                    custom_icon: *icon,
                });
            }
            TreeNode::Folder {
                name,
                path,
                children,
            } => {
                let is_expanded = expanded.contains(path.as_str());
                entries.push(FlatEntry {
                    path: path.clone(),
                    name: name.clone(),
                    depth,
                    is_folder: true,
                    is_expanded,
                    custom_icon: None,
                });
                if is_expanded {
                    for child in children {
                        Self::flatten_node_into(expanded, child, depth + 1, entries);
                    }
                }
            }
        }
    }
}

impl Render for FileTreeView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        self.ensure_flat_entries();

        let entry_count = self.flat_entries.len();
        let entries_snapshot = self.flat_entries.clone();
        let selected_paths = self.selected_paths.clone();
        let focused_index = self.focused_index;
        let select_folders = self.select_folders;
        let draggable = self.draggable;
        let entity_handle = cx.entity().clone();
        let render_actions = self.render_actions.clone();
        let bg_color = theme.background;
        let hover_color = theme.hover;
        let selected_bg = theme.selected_bg;
        let accent_color = theme.accent;
        let text_color = theme.text;
        let text_muted = theme.text_muted;
        let info_color = theme.info;
        let connector_line = theme.connector_line;
        let spacing_xs = theme.spacing_xs;
        let spacing_sm = theme.spacing_sm;
        let icon_size = theme.icon_size_inline;
        let indent_width = px(16.0);

        let keyboard_handle = cx.entity().clone();

        div()
            .id(self.element_id.clone())
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .bg(bg_color)
            .rounded(theme.radius_lg)
            .border_1()
            .border_color(theme.border)
            .font_family(theme.font_mono.clone())
            .text_style(TextStyle::Subheadline, theme)
            .py(spacing_sm)
            .on_key_down(move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                let key = event.keystroke.key.as_str();
                keyboard_handle.update(cx, |tree, cx| {
                    tree.ensure_flat_entries();
                    let action = Self::handle_key_nav(key, tree.focused_index, &tree.flat_entries);
                    if action.new_focused != tree.focused_index {
                        tree.focused_index = action.new_focused;
                        cx.notify();
                    }
                    if let Some(idx) = action.new_focused {
                        let path = tree.flat_entries[idx].path.clone();
                        if action.toggle_expand {
                            tree.toggle_expand(&path, cx);
                            if let Some(cb) = tree.on_expanded_change.take() {
                                cb(tree.expanded_paths.clone(), window, &mut *cx);
                                tree.on_expanded_change = Some(cb);
                            }
                        }
                        if action.select {
                            tree.select(&path, cx);
                            if let Some(cb) = tree.on_select.take() {
                                cb(&path, window, &mut *cx);
                                tree.on_select = Some(cb);
                            }
                        }
                    }
                });
            })
            .child(
                uniform_list(
                    "file-tree-entries",
                    entry_count,
                    cx.processor(move |_this: &mut Self, range: std::ops::Range<usize>, window, cx| {
                        range
                            .into_iter()
                            .map(|ix| {
                                let entry = &entries_snapshot[ix];
                                let is_selected = selected_paths.contains(&entry.path);
                                let is_focused = focused_index == Some(ix);
                                let path = entry.path.clone();
                                let path_click = entry.path.clone();
                                let is_folder = entry.is_folder;
                                let is_expanded = entry.is_expanded;

                                let icon = if is_folder {
                                    if is_expanded {
                                        Icon::new(IconName::FolderOpen).size(icon_size).color(info_color)
                                    } else {
                                        Icon::new(IconName::Folder).size(icon_size).color(info_color)
                                    }
                                } else {
                                    let icon_name = entry.custom_icon.unwrap_or_else(|| {
                                        // Derive an SF-Symbol-backed icon from the
                                        // extension when none is explicitly set —
                                        // mirrors Xcode's project navigator.
                                        entry
                                            .name
                                            .rsplit_once('.')
                                            .map(|(_, ext)| icon_for_extension(ext))
                                            .unwrap_or(IconName::File)
                                    });
                                    Icon::new(icon_name).size(icon_size).color(text_muted)
                                };

                                let chevron: Option<Icon> = if is_folder {
                                    Some(
                                        Icon::new(if is_expanded {
                                            IconName::ChevronDown
                                        } else {
                                            IconName::ChevronRight
                                        })
                                        .size(px(12.0))
                                        .color(text_muted),
                                    )
                                } else {
                                    None
                                };

                                // Render per-node action buttons if callback is set
                                let actions_el = render_actions.as_ref().and_then(|ra| {
                                    ra(&entry.path, window, cx)
                                });

                                // Build indent guides: one border-left per depth level
                                let content_id = if is_folder {
                                    format!("folder-{}", path_click)
                                } else {
                                    format!("file-{}", path_click)
                                };
                                let mut content = div()
                                    .id(ElementId::from(SharedString::from(content_id)))
                                    .flex()
                                    .items_center()
                                    .gap(spacing_xs)
                                    .pr(spacing_sm)
                                    .py(px(2.0))
                                    .cursor_pointer()
                                    // HIG §Color: selection and hover must be
                                    // visually distinct. `selected_bg` is a
                                    // tinted accent fill (mirrors Finder's
                                    // `selectedContentBackgroundColor`); `hover`
                                    // stays the neutral grey.
                                    .bg(if is_selected { selected_bg } else { bg_color })
                                    .hover(|s| s.bg(if is_selected { selected_bg } else { hover_color }));

                                // For folders: chevron toggles expand, name area selects
                                // For files: entire row selects
                                if is_folder {
                                    // Chevron: toggle expand only
                                    content = content.child(
                                        div()
                                            .id(ElementId::from(SharedString::from(
                                                format!("chevron-{}", path_click),
                                            )))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .min_w(px(20.0))
                                            .min_h(px(20.0))
                                            .cursor_pointer()
                                            .on_click({
                                                let handle = entity_handle.clone();
                                                let path_click = path_click.clone();
                                                move |_event: &ClickEvent, window: &mut Window, cx: &mut App| {
                                                    handle.update(cx, |tree, cx| {
                                                        tree.toggle_expand(&path_click, cx);
                                                        if let Some(cb) = tree.on_expanded_change.take() {
                                                            cb(tree.expanded_paths.clone(), window, &mut *cx);
                                                            tree.on_expanded_change = Some(cb);
                                                        }
                                                    });
                                                }
                                            })
                                            .children(chevron),
                                    );

                                    // Icon + Name area: select only
                                    let mut name_area = div()
                                        .id(ElementId::from(SharedString::from(
                                            format!("name-{}", path_click),
                                        )))
                                        .flex()
                                        .flex_1()
                                        .items_center()
                                        .gap(spacing_xs)
                                        .cursor_pointer()
                                        .child(icon);

                                    if select_folders {
                                        name_area = name_area.on_click({
                                            let handle = entity_handle.clone();
                                            let path_click = path_click.clone();
                                            move |_event: &ClickEvent, window: &mut Window, cx: &mut App| {
                                                handle.update(cx, |tree, cx| {
                                                    tree.select(&path_click, cx);
                                                    if let Some(cb) = tree.on_select.take() {
                                                        cb(&path_click, window, &mut *cx);
                                                        tree.on_select = Some(cb);
                                                    }
                                                });
                                            }
                                        });
                                    } else {
                                        // When folders aren't selectable, clicking the name area
                                        // toggles expand (restoring the old full-row behavior)
                                        name_area = name_area.on_click({
                                            let handle = entity_handle.clone();
                                            let path_click = path_click.clone();
                                            move |_event: &ClickEvent, window: &mut Window, cx: &mut App| {
                                                handle.update(cx, |tree, cx| {
                                                    tree.toggle_expand(&path_click, cx);
                                                    if let Some(cb) = tree.on_expanded_change.take() {
                                                        cb(tree.expanded_paths.clone(), window, &mut *cx);
                                                        tree.on_expanded_change = Some(cb);
                                                    }
                                                });
                                            }
                                        });
                                    }

                                    name_area = name_area.child(
                                        div()
                                            .flex_1()
                                            .overflow_hidden()
                                            .text_ellipsis()
                                            .whitespace_nowrap()
                                            .text_color(text_color)
                                            .child(entry.name.clone()),
                                    );

                                    content = content.child(name_area);
                                } else {
                                    // File: entire content area selects
                                    // Spacer to align with folder chevron
                                    content = content
                                        .child(
                                            div().w(px(20.0)).flex_shrink_0(),
                                        )
                                        .on_click({
                                            let handle = entity_handle.clone();
                                            let path_click = path_click.clone();
                                            move |_event: &ClickEvent, window: &mut Window, cx: &mut App| {
                                                handle.update(cx, |tree, cx| {
                                                    tree.select(&path_click, cx);
                                                    if let Some(cb) = tree.on_select.take() {
                                                        cb(&path_click, window, &mut *cx);
                                                        tree.on_select = Some(cb);
                                                    }
                                                });
                                            }
                                        })
                                        .child(icon)
                                        .child(
                                            div()
                                                .flex_1()
                                                .overflow_hidden()
                                                .text_ellipsis()
                                                .whitespace_nowrap()
                                                .text_color(text_color)
                                                .child(entry.name.clone()),
                                        );
                                }

                                // Append action buttons (pushed to the right via flex_1 on name)
                                if let Some(actions) = actions_el {
                                    content = content.child(
                                        div()
                                            .id(ElementId::from(SharedString::from(
                                                format!("actions-{}", path_click),
                                            )))
                                            .flex()
                                            .items_center()
                                            .on_click(|_: &ClickEvent, _: &mut Window, _: &mut App| {
                                                // Consume click to prevent row selection
                                            })
                                            .child(actions),
                                    );
                                }

                                // Focus ring for keyboard navigation (always reserve border space to avoid layout shift)
                                content = content.border_1().border_color(
                                    if is_focused { accent_color } else { gpui::transparent_black() }
                                );

                                // Opt-in drag source. Mirrors Zed's
                                // project-panel pattern: the payload is
                                // cheap to clone (`DraggedFilePath`) and
                                // the preview is a small chip reusing the
                                // row icon + filename. Callers wire a
                                // matching `.on_drop::<DraggedFilePath>`
                                // on their drop target. Finding #19 in
                                // the HIG Code-surface audit.
                                if draggable {
                                    let drag_payload = DraggedFilePath {
                                        path: path_click.clone(),
                                        name: entry.name.clone(),
                                        is_folder,
                                    };
                                    let icon_for_preview = if is_folder {
                                        if is_expanded {
                                            IconName::FolderOpen
                                        } else {
                                            IconName::Folder
                                        }
                                    } else {
                                        entry.custom_icon.unwrap_or_else(|| {
                                            entry
                                                .name
                                                .rsplit_once('.')
                                                .map(|(_, ext)| icon_for_extension(ext))
                                                .unwrap_or(IconName::File)
                                        })
                                    };
                                    content = content.on_drag(
                                        drag_payload,
                                        move |payload, _offset, _window, cx| {
                                            cx.new(|_| DraggedFilePathView {
                                                name: payload.name.clone().into(),
                                                icon: icon_for_preview,
                                            })
                                        },
                                    );
                                }

                                // Wrap with indent guides
                                let depth = entry.depth;
                                let row = div()
                                    .id(ElementId::from(SharedString::from(path)))
                                    .flex()
                                    .pl(spacing_sm);

                                // Nest indent guide containers for each depth level
                                if depth == 0 {
                                    row.child(content)
                                } else {
                                    // Build nested indent wrappers from innermost to outermost
                                    let mut nested: AnyElement = content.into_any_element();
                                    for _ in 0..depth {
                                        nested = div()
                                            .flex_1()
                                            .ml(indent_width)
                                            .border_l_1()
                                            .border_color(connector_line)
                                            .child(nested)
                                            .into_any_element();
                                    }
                                    row.child(nested)
                                }
                            })
                            .collect()
                    }),
                )
                .track_scroll(&self.scroll_handle)
                .flex_1()
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{FileTreeView, FlatEntry, TreeNode, icon_for_extension};
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;
    use std::collections::HashSet;

    // ─── icon_for_extension ────────────────────────────────────────────────

    #[test]
    fn icon_for_extension_rust() {
        assert_eq!(icon_for_extension("rs"), IconName::LangRust);
    }

    #[test]
    fn icon_for_extension_ts_and_tsx() {
        assert_eq!(icon_for_extension("ts"), IconName::LangTypeScript);
        assert_eq!(icon_for_extension("tsx"), IconName::LangTypeScript);
    }

    #[test]
    fn icon_for_extension_js_variants() {
        assert_eq!(icon_for_extension("js"), IconName::LangJavaScript);
        assert_eq!(icon_for_extension("mjs"), IconName::LangJavaScript);
        assert_eq!(icon_for_extension("jsx"), IconName::LangJavaScript);
    }

    #[test]
    fn icon_for_extension_json_and_toml() {
        assert_eq!(icon_for_extension("json"), IconName::LangJson);
        assert_eq!(icon_for_extension("toml"), IconName::LangToml);
    }

    #[test]
    fn icon_for_extension_shell_scripts() {
        for ext in ["sh", "bash", "zsh", "fish"] {
            assert_eq!(icon_for_extension(ext), IconName::LangBash, "ext: {}", ext);
        }
    }

    #[test]
    fn icon_for_extension_markdown() {
        assert_eq!(icon_for_extension("md"), IconName::Book);
        assert_eq!(icon_for_extension("markdown"), IconName::Book);
        assert_eq!(icon_for_extension("mdx"), IconName::Book);
    }

    #[test]
    fn icon_for_extension_images() {
        for ext in ["png", "jpg", "jpeg", "svg", "gif", "webp"] {
            assert_eq!(icon_for_extension(ext), IconName::Image, "ext: {}", ext);
        }
    }

    #[test]
    fn icon_for_extension_case_insensitive() {
        assert_eq!(icon_for_extension("RS"), IconName::LangRust);
        assert_eq!(icon_for_extension("JSON"), IconName::LangJson);
    }

    #[test]
    fn icon_for_extension_unknown_falls_back_to_file() {
        assert_eq!(icon_for_extension("xyz"), IconName::File);
        assert_eq!(icon_for_extension(""), IconName::File);
    }

    #[test]
    fn file_constructor() {
        let node = TreeNode::file("main.rs", "src/main.rs");
        assert_eq!(node.name(), "main.rs");
        assert_eq!(node.path(), "src/main.rs");
    }

    #[test]
    fn folder_constructor() {
        let node = TreeNode::folder("src", "src", vec![TreeNode::file("lib.rs", "src/lib.rs")]);
        assert_eq!(node.name(), "src");
        assert_eq!(node.path(), "src");
    }

    #[test]
    fn flatten_single_file() {
        let expanded = HashSet::new();
        let mut entries = Vec::new();
        let node = TreeNode::file("main.rs", "src/main.rs");
        FileTreeView::flatten_node_into(&expanded, &node, 0, &mut entries);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "main.rs");
        assert_eq!(entries[0].depth, 0);
        assert!(!entries[0].is_folder);
    }

    #[test]
    fn flatten_collapsed_folder_hides_children() {
        let expanded = HashSet::new(); // nothing expanded
        let mut entries = Vec::new();
        let node = TreeNode::folder(
            "src",
            "src",
            vec![
                TreeNode::file("lib.rs", "src/lib.rs"),
                TreeNode::file("main.rs", "src/main.rs"),
            ],
        );
        FileTreeView::flatten_node_into(&expanded, &node, 0, &mut entries);
        assert_eq!(entries.len(), 1); // only the folder
        assert!(entries[0].is_folder);
        assert!(!entries[0].is_expanded);
    }

    #[test]
    fn flatten_expanded_folder_shows_children() {
        let mut expanded = HashSet::new();
        expanded.insert("src".to_string());
        let mut entries = Vec::new();
        let node = TreeNode::folder(
            "src",
            "src",
            vec![
                TreeNode::file("lib.rs", "src/lib.rs"),
                TreeNode::file("main.rs", "src/main.rs"),
            ],
        );
        FileTreeView::flatten_node_into(&expanded, &node, 0, &mut entries);
        assert_eq!(entries.len(), 3); // folder + 2 files
        assert!(entries[0].is_folder);
        assert!(entries[0].is_expanded);
        assert_eq!(entries[1].depth, 1);
        assert_eq!(entries[2].depth, 1);
    }

    #[test]
    fn flatten_nested_depth_tracking() {
        let mut expanded = HashSet::new();
        expanded.insert("root".to_string());
        expanded.insert("root/src".to_string());
        let mut entries = Vec::new();
        let tree = TreeNode::folder(
            "root",
            "root",
            vec![TreeNode::folder(
                "src",
                "root/src",
                vec![TreeNode::file("lib.rs", "root/src/lib.rs")],
            )],
        );
        FileTreeView::flatten_node_into(&expanded, &tree, 0, &mut entries);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].depth, 0); // root
        assert_eq!(entries[1].depth, 1); // src
        assert_eq!(entries[2].depth, 2); // lib.rs
    }

    #[test]
    fn flatten_empty_folder() {
        let mut expanded = HashSet::new();
        expanded.insert("empty".to_string());
        let mut entries = Vec::new();
        let node = TreeNode::folder("empty", "empty", vec![]);
        FileTreeView::flatten_node_into(&expanded, &node, 0, &mut entries);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_folder);
        assert!(entries[0].is_expanded);
    }

    #[test]
    fn flatten_propagates_custom_icon() {
        let expanded = HashSet::new();
        let mut entries = Vec::new();
        let node = TreeNode::file("main.rs", "src/main.rs").with_icon(IconName::Code);
        FileTreeView::flatten_node_into(&expanded, &node, 0, &mut entries);
        assert_eq!(entries[0].custom_icon, Some(IconName::Code));
    }

    #[test]
    fn flatten_default_icon_is_none() {
        let expanded = HashSet::new();
        let mut entries = Vec::new();
        let node = TreeNode::file("readme.txt", "readme.txt");
        FileTreeView::flatten_node_into(&expanded, &node, 0, &mut entries);
        assert_eq!(entries[0].custom_icon, None);
    }

    #[test]
    fn with_icon_on_folder_is_noop() {
        let node = TreeNode::folder("src", "src", vec![]).with_icon(IconName::Code);
        // Should still be a folder (with_icon is a no-op on folders)
        assert!(matches!(node, TreeNode::Folder { .. }));
    }

    // --- Keyboard navigation tests ---

    fn make_test_entries() -> Vec<FlatEntry> {
        // Simulate: src/ (expanded), src/lib.rs, src/main.rs, README.md
        vec![
            FlatEntry {
                path: "src".into(),
                name: "src".into(),
                depth: 0,
                is_folder: true,
                is_expanded: true,
                custom_icon: None,
            },
            FlatEntry {
                path: "src/lib.rs".into(),
                name: "lib.rs".into(),
                depth: 1,
                is_folder: false,
                is_expanded: false,
                custom_icon: None,
            },
            FlatEntry {
                path: "src/main.rs".into(),
                name: "main.rs".into(),
                depth: 1,
                is_folder: false,
                is_expanded: false,
                custom_icon: None,
            },
            FlatEntry {
                path: "README.md".into(),
                name: "README.md".into(),
                depth: 0,
                is_folder: false,
                is_expanded: false,
                custom_icon: None,
            },
        ]
    }

    #[test]
    fn key_nav_down() {
        let entries = make_test_entries();
        let action = FileTreeView::handle_key_nav("down", Some(0), &entries);
        assert_eq!(action.new_focused, Some(1));
        assert!(!action.select);
    }

    #[test]
    fn key_nav_up() {
        let entries = make_test_entries();
        let action = FileTreeView::handle_key_nav("up", Some(2), &entries);
        assert_eq!(action.new_focused, Some(1));
    }

    #[test]
    fn key_nav_down_clamps_at_end() {
        let entries = make_test_entries();
        let action = FileTreeView::handle_key_nav("down", Some(3), &entries);
        assert_eq!(action.new_focused, Some(3));
    }

    #[test]
    fn key_nav_up_clamps_at_start() {
        let entries = make_test_entries();
        let action = FileTreeView::handle_key_nav("up", Some(0), &entries);
        assert_eq!(action.new_focused, Some(0));
    }

    #[test]
    fn key_nav_home_end() {
        let entries = make_test_entries();
        let home = FileTreeView::handle_key_nav("home", Some(2), &entries);
        assert_eq!(home.new_focused, Some(0));
        let end = FileTreeView::handle_key_nav("end", Some(0), &entries);
        assert_eq!(end.new_focused, Some(3));
    }

    #[test]
    fn key_nav_left_collapses_expanded_folder() {
        let entries = make_test_entries();
        // Index 0 is an expanded folder
        let action = FileTreeView::handle_key_nav("left", Some(0), &entries);
        assert!(action.toggle_expand);
        assert_eq!(action.new_focused, Some(0));
    }

    #[test]
    fn key_nav_right_expands_collapsed_folder() {
        let entries = vec![FlatEntry {
            path: "src".into(),
            name: "src".into(),
            depth: 0,
            is_folder: true,
            is_expanded: false,
            custom_icon: None,
        }];
        let action = FileTreeView::handle_key_nav("right", Some(0), &entries);
        assert!(action.toggle_expand);
        assert_eq!(action.new_focused, Some(0));
    }

    #[test]
    fn key_nav_enter_selects_and_toggles_folder() {
        let entries = make_test_entries();
        let action = FileTreeView::handle_key_nav("enter", Some(0), &entries);
        assert!(action.select);
        assert!(action.toggle_expand); // folder
    }

    #[test]
    fn key_nav_enter_selects_file_without_toggle() {
        let entries = make_test_entries();
        let action = FileTreeView::handle_key_nav("enter", Some(1), &entries);
        assert!(action.select);
        assert!(!action.toggle_expand); // file
    }

    #[test]
    fn key_nav_left_on_file_moves_to_parent() {
        let entries = make_test_entries();
        // Index 1 is src/lib.rs at depth 1, parent is src at index 0
        let action = FileTreeView::handle_key_nav("left", Some(1), &entries);
        assert_eq!(action.new_focused, Some(0));
        assert!(!action.toggle_expand);
    }
}

#[cfg(test)]
mod drag_tests {
    use super::DraggedFilePath;
    use core::prelude::v1::test;

    #[test]
    fn dragged_file_path_constructs() {
        let d = DraggedFilePath {
            path: "src/main.rs".into(),
            name: "main.rs".into(),
            is_folder: false,
        };
        assert_eq!(d.path, "src/main.rs");
        assert_eq!(d.name, "main.rs");
        assert!(!d.is_folder);
    }

    #[test]
    fn dragged_file_path_eq_clone() {
        let d = DraggedFilePath {
            path: "a".into(),
            name: "a".into(),
            is_folder: true,
        };
        let d2 = d.clone();
        assert_eq!(d, d2);
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::FileTreeView;
    use crate::test_helpers::helpers::setup_test_window;
    use std::collections::HashSet;

    #[gpui::test]
    async fn draggable_flag_defaults_off(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| FileTreeView::new(cx));
        handle.update(cx, |tree, _cx| {
            assert!(!tree.is_draggable());
            tree.set_draggable(true);
            assert!(tree.is_draggable());
        });
    }

    // ─── Multi-select (finding #11) ────────────────────────────────────

    #[gpui::test]
    async fn select_replaces_when_single(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| FileTreeView::new(cx));
        handle.update(cx, |tree, cx| {
            tree.select("a", cx);
            tree.select("b", cx);
            assert_eq!(tree.selected_paths().len(), 1);
            assert!(tree.selected_paths().contains("b"));
        });
    }

    #[gpui::test]
    async fn toggle_select_builds_and_clears_set(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| FileTreeView::new(cx));
        handle.update(cx, |tree, cx| {
            tree.toggle_select("a", cx);
            tree.toggle_select("b", cx);
            assert_eq!(tree.selected_paths().len(), 2);
            tree.toggle_select("a", cx);
            assert_eq!(tree.selected_paths().len(), 1);
            assert!(tree.selected_paths().contains("b"));
        });
    }

    #[gpui::test]
    async fn toggle_select_ignores_empty_path(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| FileTreeView::new(cx));
        handle.update(cx, |tree, cx| {
            tree.toggle_select("", cx);
            assert!(tree.selected_paths().is_empty());
        });
    }

    #[gpui::test]
    async fn set_selected_paths_replaces_set(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| FileTreeView::new(cx));
        handle.update(cx, |tree, cx| {
            let mut set = HashSet::new();
            set.insert("a".into());
            set.insert("b".into());
            tree.set_selected_paths(set, cx);
            assert_eq!(tree.selected_paths().len(), 2);
        });
    }

    #[gpui::test]
    async fn multi_select_flag_defaults_off(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| FileTreeView::new(cx));
        handle.update(cx, |tree, _cx| {
            assert!(!tree.multi_select());
            tree.set_multi_select(true);
            assert!(tree.multi_select());
        });
    }
}
