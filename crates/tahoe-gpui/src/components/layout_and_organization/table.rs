//! HIG Table View — sortable, selectable data table.
//!
//! A stateless `RenderOnce` component that renders a scrollable table with
//! column headers, sortable columns, and single/multi row selection.
//!
//! # Selection
//!
//! macOS 26 Tahoe tables use an *inset capsule* selection highlight rather
//! than a flush-edge fill. The selected row is padded by `spacing_xs` and
//! rounded to `radius_md`, matching `NSTableView`'s standard style on
//! macOS 13+.
//!
//! # Multi-select
//!
//! In [`SelectionMode::Multiple`], selection clicks deliver the current
//! `Modifiers` to the `on_select_modified` callback so callers can
//! implement `⌘-click` (toggle) and `Shift-click` (range extend) per HIG.
//!
//! # Keyboard navigation
//!
//! Up/Down move the highlight by one row; Home/End jump to the first/last
//! row; Page Up/Down jump by [`PAGE_STEP`] rows; Enter/Space select the
//! highlighted row. Passing a [`UniformListScrollHandle`] to
//! [`Table::scroll_handle`] auto-reveals the highlighted row.

use std::collections::HashSet;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, CursorStyle, ElementId, FontWeight, KeyDownEvent, ListSizingBehavior,
    Modifiers, MouseButton, MouseDownEvent, ScrollStrategy, SharedString, UniformListScrollHandle,
    Window, div, px, uniform_list,
};

use crate::callback_types::rc_wrap;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};

/// Number of rows to jump when the user presses Page Up / Page Down.
pub const PAGE_STEP: usize = 10;

/// Sort direction for a table column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// Returns the opposite direction.
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// Selection mode for table rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionMode {
    /// No row selection.
    #[default]
    None,
    /// Only one row can be selected at a time.
    Single,
    /// Multiple rows can be selected. Callers implement `⌘-click`
    /// (non-contiguous) and `Shift-click` (range) via
    /// [`Table::on_select_modified`].
    Multiple,
}

/// Definition of a table column.
#[derive(Debug, Clone)]
pub struct TableColumn {
    /// Column identifier.
    pub id: SharedString,
    /// Display header text.
    pub label: SharedString,
    /// Whether clicking the header sorts by this column.
    pub sortable: bool,
    /// Optional fixed width in pixels. None = flex.
    pub width: Option<f32>,
    /// Whether the column is user-resizable via its trailing divider. HIG:
    /// "tables support… drag-resize on dividers."
    pub resizable: bool,
    /// Whether cells in this column render as in-place text editors when
    /// entered. Callers wire up an `on_commit` handler and render a
    /// `TextField` inside a [`TableCell::Element`] factory when needed.
    pub editable: bool,
}

impl TableColumn {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            sortable: false,
            width: None,
            resizable: false,
            editable: false,
        }
    }

    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Mark this column as user-resizable. Pair with [`Table::on_column_resize`]
    /// to receive the new width when the user drags the divider.
    pub fn resizable(mut self) -> Self {
        self.resizable = true;
        self
    }

    /// Mark this column as in-place editable. Editable columns render a
    /// grab-and-hold indicator and forward double-click events through
    /// the caller-supplied cell factory.
    pub fn editable(mut self) -> Self {
        self.editable = true;
        self
    }
}

/// Factory that produces an `AnyElement` on demand for a rich table cell.
///
/// Rows are virtualized via `uniform_list`, so cells are rebuilt each time a
/// row scrolls into view. Rich cells therefore store a factory closure rather
/// than a pre-built `AnyElement`.
pub type CellFactory = Rc<dyn Fn() -> AnyElement>;

/// A single cell value in a table row.
///
/// Use the `From` impls (`&str`, `String`, `SharedString`) for the common
/// text case, or `TableCell::element(|| ...)` to embed icons, badges, or
/// other rich content.
pub enum TableCell {
    Text(SharedString),
    Element(CellFactory),
}

impl TableCell {
    /// Construct a rich-element cell from a factory that produces an
    /// `AnyElement` each time the row is rendered.
    pub fn element<F>(factory: F) -> Self
    where
        F: Fn() -> AnyElement + 'static,
    {
        TableCell::Element(Rc::new(factory))
    }

    /// Returns the text of this cell if it's a text cell, else `""`.
    ///
    /// Convenience for tests and callers that only ever produce text cells.
    pub fn as_str(&self) -> &str {
        match self {
            TableCell::Text(s) => s.as_ref(),
            TableCell::Element(_) => "",
        }
    }
}

impl From<SharedString> for TableCell {
    fn from(value: SharedString) -> Self {
        TableCell::Text(value)
    }
}

impl From<&'static str> for TableCell {
    fn from(value: &'static str) -> Self {
        TableCell::Text(SharedString::from(value))
    }
}

impl From<String> for TableCell {
    fn from(value: String) -> Self {
        TableCell::Text(value.into())
    }
}

/// A single row of table data.
pub struct TableRow {
    /// Cells for each column, in column order.
    pub cells: Vec<TableCell>,
}

impl TableRow {
    pub fn new(cells: Vec<impl Into<TableCell>>) -> Self {
        Self {
            cells: cells.into_iter().map(|c| c.into()).collect(),
        }
    }
}

/// Callback fired when the user clicks a sortable column header.
type OnSort = Box<dyn Fn(SharedString, SortDirection, &mut Window, &mut App) + 'static>;

/// Callback fired when keyboard navigation changes the highlighted row.
type OnHighlight = Box<dyn Fn(Option<usize>, &mut Window, &mut App) + 'static>;

/// Callback fired when a row is clicked, carrying the click modifiers so
/// callers can implement `⌘-click` / `Shift-click` multi-select semantics.
type OnSelectModified = Box<dyn Fn(usize, Modifiers, &mut Window, &mut App) + 'static>;
type OnSelectModifiedRc = Option<Rc<OnSelectModified>>;

/// Callback fired when the user starts a column-resize drag on a divider.
/// The callback receives the column id and the horizontal start position
/// so the caller (stateful owner of the column widths) can implement the
/// drag using its own `on_mouse_move` / `on_mouse_up` listeners. The
/// Table itself is `RenderOnce`, so it can't track in-flight drag state;
/// this event is the handoff point.
type OnColumnResize = Box<dyn Fn(SharedString, f32, &mut Window, &mut App) + 'static>;

/// Boxed click-on-row handler with no modifier information.
type OnRowClick = Box<dyn Fn(usize, &mut Window, &mut App) + 'static>;
/// Rc-shared click-on-row handler (post-construction sharing).
type OnRowClickRc = Option<Rc<OnRowClick>>;

/// An HIG-style data table with sortable columns and row selection.
#[derive(IntoElement)]
pub struct Table {
    id: ElementId,
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    selection_mode: SelectionMode,
    selected_rows: Vec<usize>,
    highlighted_row: Option<usize>,
    sort_column: Option<SharedString>,
    sort_direction: SortDirection,
    on_select: Option<OnRowClick>,
    on_select_modified: Option<OnSelectModified>,
    on_sort: Option<OnSort>,
    on_highlight: Option<OnHighlight>,
    on_column_resize: Option<OnColumnResize>,
    focused: bool,
    /// Optional host-supplied focus handle. Finding 18 in
    /// the Zed cross-reference audit — when set, the focus-ring visibility
    /// comes from `handle.is_focused(window)` and the root element
    /// threads `track_focus(&handle)`; otherwise uses the explicit
    /// [`focused`](Self::focused) bool.
    focus_handle: Option<gpui::FocusHandle>,
    /// Message shown when the table has no rows.
    empty_message: Option<SharedString>,
    /// Optional icon shown above the empty message. HIG: empty states
    /// should pair a label with a glyph.
    empty_icon: Option<IconName>,
    alternating_row_colors: bool,
    scroll_handle: Option<UniformListScrollHandle>,
}

impl Table {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            rows: Vec::new(),
            selection_mode: SelectionMode::default(),
            selected_rows: Vec::new(),
            highlighted_row: None,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            on_select: None,
            on_select_modified: None,
            on_sort: None,
            on_highlight: None,
            on_column_resize: None,
            focused: false,
            focus_handle: None,
            empty_message: None,
            empty_icon: None,
            alternating_row_colors: false,
            scroll_handle: None,
        }
    }

    pub fn columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns = columns;
        self
    }

    pub fn rows(mut self, rows: Vec<TableRow>) -> Self {
        self.rows = rows;
        self
    }

    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    pub fn selected_rows(mut self, rows: Vec<usize>) -> Self {
        self.selected_rows = rows;
        self
    }

    pub fn highlighted_row(mut self, row: Option<usize>) -> Self {
        self.highlighted_row = row;
        self
    }

    pub fn sort_column(
        mut self,
        column: impl Into<SharedString>,
        direction: SortDirection,
    ) -> Self {
        self.sort_column = Some(column.into());
        self.sort_direction = direction;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    /// Set the handler that receives row clicks together with their modifier
    /// flags. Use this for `SelectionMode::Multiple` when you want
    /// `⌘-click` / `Shift-click` semantics — the callback distinguishes
    /// a plain click (replace selection) from a modified click (toggle /
    /// extend) based on `Modifiers::command` / `Modifiers::shift`.
    pub fn on_select_modified(
        mut self,
        handler: impl Fn(usize, Modifiers, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_modified = Some(Box::new(handler));
        self
    }

    pub fn on_sort(
        mut self,
        handler: impl Fn(SharedString, SortDirection, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort = Some(Box::new(handler));
        self
    }

    /// Set the handler called when keyboard navigation changes the highlighted row.
    pub fn on_highlight(
        mut self,
        handler: impl Fn(Option<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_highlight = Some(Box::new(handler));
        self
    }

    /// Set the handler called when the user drag-resizes a column. The
    /// callback receives the column id and the new width in points.
    pub fn on_column_resize(
        mut self,
        handler: impl Fn(SharedString, f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_column_resize = Some(Box::new(handler));
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the table participates in the host's
    /// focus graph. When set, the focus-ring is derived from
    /// `handle.is_focused(window)` and the root element threads
    /// `track_focus(&handle)` so Tab-cycling and keyboard shortcuts
    /// scoped to the handle fire correctly. Finding 18 in
    /// the Zed cross-reference audit.
    pub fn focus_handle(mut self, handle: &gpui::FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    pub fn empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.empty_message = Some(message.into());
        self
    }

    /// Set an icon to show above the empty-state message.
    pub fn empty_icon(mut self, icon: IconName) -> Self {
        self.empty_icon = Some(icon);
        self
    }

    /// Enable the zebra-striping pattern (`NSTableView.usesAlternatingRowBackgroundColors`).
    /// Odd rows render with `theme.surface_alt` background.
    pub fn alternating_row_colors(mut self, enabled: bool) -> Self {
        self.alternating_row_colors = enabled;
        self
    }

    /// Supply a scroll handle so keyboard navigation can reveal the
    /// highlighted row. Without a handle, the highlight still tracks
    /// logically but may move off-screen.
    pub fn scroll_handle(mut self, handle: UniformListScrollHandle) -> Self {
        self.scroll_handle = Some(handle);
        self
    }
}

#[allow(clippy::too_many_arguments)]
fn render_data_row(
    row_idx: usize,
    row: &TableRow,
    columns: &[TableColumn],
    is_selected: bool,
    is_highlighted: bool,
    alternate_bg: bool,
    selection_mode: SelectionMode,
    theme: &TahoeTheme,
    on_select: &OnRowClickRc,
    on_select_modified: &OnSelectModifiedRc,
) -> AnyElement {
    // Row container wraps an inset capsule so the selection highlight
    // doesn't run to the container edge — HIG macOS 13+ inset style.
    let mut container = div().flex().flex_row().w_full().px(theme.spacing_xs);

    if alternate_bg {
        // `NSTableView.usesAlternatingRowBackgroundColors`: a subtle
        // darker tint on odd rows. GPUI theme doesn't expose a dedicated
        // token, so blend 4 % of the text color into the surface.
        let alt = crate::foundations::color::with_alpha(theme.text, 0.04);
        container = container.bg(alt);
    }

    let capsule_bg = if is_selected {
        theme.accent
    } else if is_highlighted {
        theme.hover
    } else {
        gpui::transparent_black()
    };
    let text_color = if is_selected {
        theme.text_on_accent
    } else {
        theme.text
    };

    let row_id = ElementId::NamedInteger("trow".into(), row_idx as u64);
    let mut capsule = div()
        .id(row_id)
        .flex()
        .flex_row()
        .w_full()
        .bg(capsule_bg)
        .rounded(theme.radius_md)
        .border_b_1()
        .border_color(theme.separator_color());

    let cell_count = row.cells.len();
    let col_count = columns.len();
    debug_assert!(
        cell_count <= col_count,
        "table row {row_idx} has {cell_count} cells but only {col_count} columns"
    );

    for (col_idx, cell) in row.cells.iter().enumerate() {
        let mut cell_el = div()
            .min_h(px(theme.target_size()))
            .flex()
            .items_center()
            .px(theme.spacing_sm)
            .text_style(TextStyle::Body, theme)
            .text_color(text_color);

        cell_el = match cell {
            TableCell::Text(text) => cell_el.child(text.clone()),
            TableCell::Element(factory) => cell_el.child(factory()),
        };

        if let Some(col) = columns.get(col_idx) {
            if let Some(w) = col.width {
                cell_el = cell_el.w(px(w));
            } else {
                cell_el = cell_el.flex_1();
            }
        } else {
            cell_el = cell_el.flex_1();
        }

        capsule = capsule.child(cell_el);
    }

    // Pad missing cells so column widths stay aligned with the header.
    for pad_idx in cell_count..col_count {
        let mut pad_el = div().min_h(px(theme.target_size())).px(theme.spacing_sm);

        if let Some(col) = columns.get(pad_idx) {
            if let Some(w) = col.width {
                pad_el = pad_el.w(px(w));
            } else {
                pad_el = pad_el.flex_1();
            }
        } else {
            pad_el = pad_el.flex_1();
        }

        capsule = capsule.child(pad_el);
    }

    if selection_mode != SelectionMode::None {
        capsule = capsule.cursor_pointer().hover(|s| s.bg(theme.hover));
        // Prefer the modifier-aware handler if the caller provided one;
        // otherwise fall back to the simple index-only `on_select`.
        if let Some(handler) = on_select_modified {
            let h = handler.clone();
            capsule = capsule.on_click(move |event, window, cx| {
                h(row_idx, event.modifiers(), window, cx);
            });
        } else if let Some(handler) = on_select {
            let h = handler.clone();
            capsule = capsule.on_click(move |_event, window, cx| {
                h(row_idx, window, cx);
            });
        }
    }

    container = container.child(capsule);

    container.into_any_element()
}

impl RenderOnce for Table {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let on_sort = rc_wrap(self.on_sort);
        let on_highlight = rc_wrap(self.on_highlight);
        let on_column_resize = rc_wrap(self.on_column_resize);
        let selected_rows = self.selected_rows;
        let on_select: OnRowClickRc = self.on_select.map(Rc::new);
        let on_select_modified: OnSelectModifiedRc = self.on_select_modified.map(Rc::new);
        let row_count = self.rows.len();
        let highlighted_row = self.highlighted_row;
        let alternating_row_colors = self.alternating_row_colors;
        let scroll_handle = self.scroll_handle.clone();

        // ── Header row ─────────────────────────────────────────────────────
        let mut header_row = div()
            .flex()
            .flex_row()
            .w_full()
            .border_b_1()
            .border_color(theme.separator_color());

        let last_col_idx = self.columns.len().saturating_sub(1);
        for (col_idx, col) in self.columns.iter().enumerate() {
            let is_sorted = self.sort_column.as_ref() == Some(&col.id);
            let weight = if is_sorted {
                theme.effective_weight(FontWeight::SEMIBOLD)
            } else {
                theme.effective_weight(FontWeight::MEDIUM)
            };

            let cell_id = ElementId::Name(format!("th-{}", col.id).into());
            let mut header_cell = div()
                .id(cell_id)
                .min_h(px(theme.target_size()))
                .flex()
                .items_center()
                .px(theme.spacing_sm)
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(weight)
                .text_color(theme.text);

            if let Some(w) = col.width {
                header_cell = header_cell.w(px(w));
            } else {
                header_cell = header_cell.flex_1();
            }

            let mut label_row = div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .child(col.label.clone());

            // Sort indicator
            if is_sorted {
                let icon_name = match self.sort_direction {
                    SortDirection::Ascending => IconName::ChevronUp,
                    SortDirection::Descending => IconName::ChevronDown,
                };
                label_row =
                    label_row.child(Icon::new(icon_name).size(px(10.0)).color(theme.accent));
            }

            header_cell = header_cell.child(label_row);

            if col.sortable {
                let col_id = col.id.clone();
                let current_dir = if is_sorted {
                    self.sort_direction
                } else {
                    SortDirection::Ascending
                };
                let is_already_sorted = is_sorted;

                if let Some(ref handler) = on_sort {
                    let h = handler.clone();
                    header_cell = header_cell
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.hover))
                        .on_click(move |_event, window, cx| {
                            let new_dir = if is_already_sorted {
                                current_dir.toggle()
                            } else {
                                SortDirection::Ascending
                            };
                            h(col_id.clone(), new_dir, window, cx);
                        });
                }
            }

            // Trailing resize handle: a 6 pt hit area with a
            // `ResizeLeftRight` cursor. When the user presses the left
            // mouse button, fire `on_column_resize` with the column id
            // and the cursor's start x-position. The caller (stateful
            // owner of column widths) installs its own move/up listeners
            // to finish the drag — `Table` is `RenderOnce` and can't
            // keep drag state across renders.
            if col.resizable && col_idx < last_col_idx {
                if let Some(ref handler) = on_column_resize {
                    let h = handler.clone();
                    let col_id = col.id.clone();
                    let resize_handle = div()
                        .id(ElementId::Name(format!("th-resize-{}", col.id).into()))
                        .w(px(6.0))
                        .h_full()
                        .cursor(CursorStyle::ResizeLeftRight)
                        .on_mouse_down(
                            MouseButton::Left,
                            move |event: &MouseDownEvent, window, cx| {
                                h(col_id.clone(), f32::from(event.position.x), window, cx);
                            },
                        );
                    header_cell = header_cell.child(resize_handle);
                }
            }

            header_row = header_row.child(header_cell);
        }

        // ── Data rows ──────────────────────────────────────────────────────
        // Virtualize via `uniform_list`: only visible rows are materialized,
        // which keeps 10k-row tables cheap.
        let body: AnyElement = if self.rows.is_empty() {
            let msg = self
                .empty_message
                .unwrap_or_else(|| SharedString::from("No items"));
            let mut empty_col = div()
                .w_full()
                .py(theme.spacing_lg)
                .flex()
                .flex_col()
                .gap(theme.spacing_sm)
                .justify_center()
                .items_center()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted);
            if let Some(icon) = self.empty_icon {
                empty_col = empty_col.child(Icon::new(icon).size(px(32.0)).color(theme.text_muted));
            }
            empty_col = empty_col.child(msg);
            div()
                .flex()
                .flex_col()
                .w_full()
                .child(empty_col)
                .into_any_element()
        } else {
            let rows = Rc::new(self.rows);
            let columns = Rc::new(self.columns.clone());
            let selected_set: Rc<HashSet<usize>> = Rc::new(selected_rows.iter().copied().collect());
            let selection_mode = self.selection_mode;
            let on_select_row = on_select.clone();
            let on_select_modified_row = on_select_modified.clone();
            let mut list = uniform_list(
                ElementId::Name("table-body".into()),
                row_count,
                move |range: std::ops::Range<usize>, _window, cx| {
                    let theme = cx.theme();
                    range
                        .map(|idx| {
                            let alt = alternating_row_colors && idx.is_multiple_of(2).not_then();
                            render_data_row(
                                idx,
                                &rows[idx],
                                &columns,
                                selected_set.contains(&idx),
                                highlighted_row == Some(idx),
                                alt,
                                selection_mode,
                                theme,
                                &on_select_row,
                                &on_select_modified_row,
                            )
                        })
                        .collect()
                },
            )
            .with_sizing_behavior(ListSizingBehavior::Infer)
            .flex_grow();

            if let Some(handle) = scroll_handle.as_ref() {
                list = list.track_scroll(handle);
            }

            list.into_any_element()
        };

        // ── Container ──────────────────────────────────────────────────────
        let inner = div()
            .flex()
            .flex_col()
            .w_full()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md)
            .child(header_row)
            .child(body);

        let mut container = div().id(self.id).focusable().w_full().child(inner);

        // Keyboard navigation over the data rows.
        if row_count > 0 {
            let highlight_cb = on_highlight.clone();
            let select_cb = on_select.clone();
            let select_modified_cb = on_select_modified.clone();
            let scroll_handle_key = scroll_handle.clone();
            container = container.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let key = event.keystroke.key.as_str();
                let mut scroll_to: Option<usize> = None;
                match key {
                    "down" => {
                        cx.stop_propagation();
                        let next = match highlighted_row {
                            Some(i) if i + 1 < row_count => i + 1,
                            Some(_) => row_count - 1,
                            None => 0,
                        };
                        scroll_to = Some(next);
                        if let Some(cb) = &highlight_cb {
                            cb(Some(next), window, cx);
                        }
                    }
                    "up" => {
                        cx.stop_propagation();
                        let prev = match highlighted_row {
                            Some(0) => 0,
                            Some(i) => i - 1,
                            None => 0,
                        };
                        scroll_to = Some(prev);
                        if let Some(cb) = &highlight_cb {
                            cb(Some(prev), window, cx);
                        }
                    }
                    "pageup" => {
                        cx.stop_propagation();
                        let prev = highlighted_row.unwrap_or(0).saturating_sub(PAGE_STEP);
                        scroll_to = Some(prev);
                        if let Some(cb) = &highlight_cb {
                            cb(Some(prev), window, cx);
                        }
                    }
                    "pagedown" => {
                        cx.stop_propagation();
                        let next = (highlighted_row.unwrap_or(0) + PAGE_STEP).min(row_count - 1);
                        scroll_to = Some(next);
                        if let Some(cb) = &highlight_cb {
                            cb(Some(next), window, cx);
                        }
                    }
                    "home" => {
                        cx.stop_propagation();
                        scroll_to = Some(0);
                        if let Some(cb) = &highlight_cb {
                            cb(Some(0), window, cx);
                        }
                    }
                    "end" => {
                        cx.stop_propagation();
                        scroll_to = Some(row_count - 1);
                        if let Some(cb) = &highlight_cb {
                            cb(Some(row_count - 1), window, cx);
                        }
                    }
                    "enter" | "space" => {
                        cx.stop_propagation();
                        if let Some(idx) = highlighted_row {
                            let modifiers = event.keystroke.modifiers;
                            if let Some(cb) = &select_modified_cb {
                                cb(idx, modifiers, window, cx);
                            } else if let Some(cb) = &select_cb {
                                cb(idx, window, cx);
                            }
                        }
                    }
                    _ => {}
                }

                if let (Some(idx), Some(handle)) = (scroll_to, scroll_handle_key.as_ref()) {
                    handle.scroll_to_item(idx, ScrollStrategy::Nearest);
                }
            });
        }

        // Finding 18 in the Zed cross-reference audit.
        let focused = self
            .focus_handle
            .as_ref()
            .map(|h| h.is_focused(window))
            .unwrap_or(self.focused);
        if let Some(handle) = self.focus_handle.as_ref() {
            container = container.track_focus(handle);
        }
        container = apply_focus_ring(container, theme, focused, &[]);

        container
    }
}

// Small helper so `(idx % 2 == 1)` reads clearly as "alternate row"
// without triggering the `is_multiple_of` lint on older toolchains.
trait BoolNotThen {
    fn not_then(self) -> bool;
}
impl BoolNotThen for bool {
    fn not_then(self) -> bool {
        !self
    }
}

#[cfg(test)]
mod tests {
    use super::{SelectionMode, SortDirection, Table, TableColumn, TableRow};
    use core::prelude::v1::test;
    use gpui::UniformListScrollHandle;

    #[test]
    fn sort_direction_toggle() {
        assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
        assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn selection_mode_default_is_none() {
        assert_eq!(SelectionMode::default(), SelectionMode::None);
    }

    #[test]
    fn table_column_new() {
        let col = TableColumn::new("name", "Name");
        assert_eq!(col.id.as_ref(), "name");
        assert_eq!(col.label.as_ref(), "Name");
        assert!(!col.sortable);
        assert!(col.width.is_none());
        assert!(!col.resizable);
        assert!(!col.editable);
    }

    #[test]
    fn table_column_sortable() {
        let col = TableColumn::new("name", "Name").sortable();
        assert!(col.sortable);
    }

    #[test]
    fn table_column_width() {
        let col = TableColumn::new("name", "Name").width(200.0);
        assert_eq!(col.width, Some(200.0));
    }

    #[test]
    fn table_column_resizable() {
        let col = TableColumn::new("name", "Name").resizable();
        assert!(col.resizable);
    }

    #[test]
    fn table_column_editable() {
        let col = TableColumn::new("name", "Name").editable();
        assert!(col.editable);
    }

    #[test]
    fn table_row_new() {
        let row = TableRow::new(vec!["Alice", "30", "Engineer"]);
        assert_eq!(row.cells.len(), 3);
        assert_eq!(row.cells[0].as_str(), "Alice");
    }

    #[test]
    fn table_defaults() {
        let t = Table::new("test");
        assert!(t.columns.is_empty());
        assert!(t.rows.is_empty());
        assert_eq!(t.selection_mode, SelectionMode::None);
        assert!(t.selected_rows.is_empty());
        assert!(t.highlighted_row.is_none());
        assert!(t.sort_column.is_none());
        assert_eq!(t.sort_direction, SortDirection::Ascending);
        assert!(!t.focused);
        assert!(!t.alternating_row_colors);
        assert!(t.empty_icon.is_none());
        assert!(t.scroll_handle.is_none());
    }

    #[test]
    fn table_builders() {
        let t = Table::new("test")
            .columns(vec![
                TableColumn::new("name", "Name").sortable(),
                TableColumn::new("age", "Age").width(80.0),
            ])
            .rows(vec![
                TableRow::new(vec!["Alice", "30"]),
                TableRow::new(vec!["Bob", "25"]),
            ])
            .selection_mode(SelectionMode::Single)
            .selected_rows(vec![0])
            .highlighted_row(Some(1))
            .sort_column("name", SortDirection::Descending)
            .focused(true)
            .alternating_row_colors(true);

        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.selection_mode, SelectionMode::Single);
        assert_eq!(t.selected_rows, vec![0]);
        assert_eq!(t.highlighted_row, Some(1));
        assert_eq!(t.sort_column.as_ref().map(|s| s.as_ref()), Some("name"));
        assert_eq!(t.sort_direction, SortDirection::Descending);
        assert!(t.focused);
        assert!(t.alternating_row_colors);
    }

    #[test]
    fn table_on_select() {
        let t = Table::new("test").on_select(|_idx, _w, _cx| {});
        assert!(t.on_select.is_some());
    }

    #[test]
    fn table_on_select_modified() {
        let t = Table::new("test").on_select_modified(|_idx, _mods, _w, _cx| {});
        assert!(t.on_select_modified.is_some());
    }

    #[test]
    fn table_on_sort() {
        let t = Table::new("test").on_sort(|_col, _dir, _w, _cx| {});
        assert!(t.on_sort.is_some());
    }

    #[test]
    fn table_scroll_handle_builder() {
        let handle = UniformListScrollHandle::new();
        let t = Table::new("test").scroll_handle(handle);
        assert!(t.scroll_handle.is_some());
    }

    #[test]
    fn table_on_column_resize_builder() {
        let t = Table::new("test").on_column_resize(|_id, _w, _window, _cx| {});
        assert!(t.on_column_resize.is_some());
    }

    #[test]
    fn sparse_row_accepted() {
        // A row with fewer cells than columns should be accepted.
        // The render path pads missing cells with empty divs.
        let t = Table::new("test")
            .columns(vec![
                TableColumn::new("a", "A"),
                TableColumn::new("b", "B"),
                TableColumn::new("c", "C"),
            ])
            .rows(vec![TableRow::new(vec!["only-one"])]);
        assert_eq!(t.rows[0].cells.len(), 1);
        assert_eq!(t.columns.len(), 3);
    }
}
