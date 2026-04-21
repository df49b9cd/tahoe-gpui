//! Column view (HIG Column Views — macOS Finder / `NSBrowser`).
//!
//! A column view presents hierarchical data as a horizontal strip of
//! scrollable columns. Selecting an item in one column populates the
//! next column with its children; the rightmost column typically shows
//! leaf detail or an item preview.
//!
//! # Scope
//!
//! macOS-only. HIG: "a column view — also called a browser — lets
//! people view and navigate a data hierarchy using a series of vertical
//! columns." Not supported on iOS / iPadOS / tvOS / visionOS.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/column-views>

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ElementId, FocusHandle, KeyDownEvent, Pixels, SharedString, Window, div, px,
};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::materials::{apply_focus_ring, resolve_focused};
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};

/// Default per-column width in points. Matches `NSBrowser`'s
/// `columnsAutosaveName`-derived convention used by Finder.
pub const DEFAULT_COLUMN_WIDTH: f32 = 200.0;

/// One item in a column. Callers supply a stable id (used for selection
/// tracking) and a display label.
#[derive(Clone, Debug)]
pub struct ColumnItem {
    pub id: SharedString,
    pub label: SharedString,
    pub has_children: bool,
}

impl ColumnItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            has_children: false,
        }
    }

    pub fn with_children(mut self) -> Self {
        self.has_children = true;
        self
    }
}

/// A single column (list of items) within a [`ColumnView`].
pub struct Column {
    pub id: SharedString,
    pub items: Vec<ColumnItem>,
    pub selected_id: Option<SharedString>,
}

impl Column {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            selected_id: None,
        }
    }

    pub fn items(mut self, items: Vec<ColumnItem>) -> Self {
        self.items = items;
        self
    }

    pub fn selected_id(mut self, id: Option<SharedString>) -> Self {
        self.selected_id = id;
        self
    }
}

/// Callback fired when the user selects a row in a column. The callback
/// receives the column id (the caller-supplied column identifier) and
/// the selected item id.
type OnSelect = Option<Rc<dyn Fn(SharedString, SharedString, &mut Window, &mut App)>>;

/// A macOS Finder-style column browser.
///
/// Layout: a horizontal flex row of fixed-width scrollable columns. The
/// caller owns column state — populate the next column with children of
/// the selected item and re-render when selection changes.
///
/// # Example
///
/// ```ignore
/// ColumnView::new("browser")
///     .column_width(220.0)
///     .columns(vec![
///         Column::new("col0").items(root_items).selected_id(Some("docs".into())),
///         Column::new("col1").items(docs_items),
///     ])
///     .on_select(|col_id, item_id, _, cx| { /* navigate */ })
/// ```
#[derive(IntoElement)]
pub struct ColumnView {
    id: ElementId,
    columns: Vec<Column>,
    column_width: Pixels,
    on_select: OnSelect,
    focused: bool,
    /// Optional host-supplied focus handle. Precedence rules live on
    /// [`resolve_focused`](crate::foundations::materials::resolve_focused):
    /// when set, the focus-ring derives from `handle.is_focused(window)`
    /// and the root element threads `track_focus(&handle)`.
    focus_handle: Option<FocusHandle>,
}

impl ColumnView {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            column_width: px(DEFAULT_COLUMN_WIDTH),
            on_select: None,
            focused: false,
            focus_handle: None,
        }
    }

    pub fn columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    /// Per-column width. HIG: columns should be user-resizable; callers
    /// implementing resize ship the updated width back through their
    /// own column state.
    pub fn column_width(mut self, width: f32) -> Self {
        self.column_width = px(width);
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(SharedString, SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    /// Show a focus ring around the browser when keyboard-focused.
    /// Ignored when a [`focus_handle`](Self::focus_handle) is also attached
    /// — the handle's live `is_focused(window)` state wins.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Attach a [`FocusHandle`] so the column view participates in the
    /// host's focus graph. Takes precedence over [`focused`](Self::focused)
    /// per [`resolve_focused`].
    pub fn focus_handle(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }
}

fn render_column(
    column: &Column,
    column_width: Pixels,
    theme: &TahoeTheme,
    on_select: &OnSelect,
) -> AnyElement {
    let mut col_el = div()
        .id(ElementId::Name(SharedString::from(format!(
            "column-{}",
            column.id
        ))))
        .flex()
        .flex_col()
        .w(column_width)
        .h_full()
        .overflow_y_scroll()
        .bg(theme.surface)
        .border_r_1()
        .border_color(theme.separator_color());

    for item in &column.items {
        let is_selected = column.selected_id.as_ref() == Some(&item.id);
        let text_color = if is_selected {
            theme.text_on_accent
        } else {
            theme.text
        };
        let bg = if is_selected {
            theme.accent
        } else {
            gpui::transparent_black()
        };

        let ax = AccessibilityProps::new()
            .label(item.label.clone())
            .role(AccessibilityRole::Button)
            .value(if is_selected {
                SharedString::from("selected")
            } else {
                SharedString::from("unselected")
            });

        let item_id_for_click = item.id.clone();
        let column_id_for_click = column.id.clone();
        let mut row = div()
            .id(ElementId::Name(SharedString::from(format!(
                "column-{}-{}",
                column.id, item.id
            ))))
            .flex()
            .flex_row()
            .items_center()
            .gap(theme.spacing_xs)
            .px(theme.spacing_sm)
            .min_h(px(theme.target_size()))
            .bg(bg)
            .cursor_pointer()
            .with_accessibility(&ax)
            .hover(|s| s.bg(theme.hover))
            .text_style(TextStyle::Body, theme)
            .text_color(text_color)
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .truncate()
                    .child(item.label.clone()),
            );

        if item.has_children {
            row = row.child(
                div()
                    .text_color(text_color)
                    .child(SharedString::from("\u{203A}")),
            );
        }

        if let Some(cb) = on_select.clone() {
            row = row.on_click(move |_event, window, cx| {
                cb(
                    column_id_for_click.clone(),
                    item_id_for_click.clone(),
                    window,
                    cx,
                );
            });
        }

        col_el = col_el.child(row);
    }

    col_el.into_any_element()
}

impl RenderOnce for ColumnView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let column_width = self.column_width;
        let on_select = self.on_select.clone();
        let focused = resolve_focused(self.focus_handle.as_ref(), window, self.focused);

        let mut container = div()
            .id(self.id)
            .focusable()
            .flex()
            .flex_row()
            .w_full()
            .h_full()
            .overflow_x_scroll()
            .bg(theme.surface)
            .border_1()
            .border_color(theme.border)
            .rounded(theme.radius_md);

        for column in &self.columns {
            let el = render_column(column, column_width, theme, &on_select);
            container = container.child(el);
        }

        // Keyboard navigation: Up/Down scroll within the rightmost
        // column's selection. Real item-level keyboard handling is
        // delegated to the caller who owns the column state; we absorb
        // arrow keys here so focus doesn't escape the browser.
        container = container.on_key_down(|event: &KeyDownEvent, _window, cx| {
            if matches!(
                event.keystroke.key.as_str(),
                "up" | "down" | "left" | "right" | "home" | "end"
            ) {
                cx.stop_propagation();
            }
        });

        if let Some(handle) = self.focus_handle.as_ref() {
            container = container.track_focus(handle);
        }
        container = apply_focus_ring(container, theme, focused, &[]);

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::{Column, ColumnItem, ColumnView, DEFAULT_COLUMN_WIDTH};

    #[test]
    fn column_item_defaults() {
        let item = ColumnItem::new("foo", "Foo");
        assert_eq!(item.id.as_ref(), "foo");
        assert_eq!(item.label.as_ref(), "Foo");
        assert!(!item.has_children);
    }

    #[test]
    fn column_item_with_children() {
        let item = ColumnItem::new("dir", "Dir").with_children();
        assert!(item.has_children);
    }

    #[test]
    fn column_defaults_empty() {
        let col = Column::new("c0");
        assert!(col.items.is_empty());
        assert!(col.selected_id.is_none());
    }

    #[test]
    fn column_view_defaults() {
        let view = ColumnView::new("browser");
        assert!(view.columns.is_empty());
        assert_eq!(f32::from(view.column_width), DEFAULT_COLUMN_WIDTH);
        assert!(!view.focused);
        assert!(view.on_select.is_none());
        assert!(view.focus_handle.is_none());
    }

    #[test]
    fn column_view_builder_sets_columns() {
        let view = ColumnView::new("browser").columns(vec![
            Column::new("a").items(vec![ColumnItem::new("x", "X")]),
            Column::new("b"),
        ]);
        assert_eq!(view.columns.len(), 2);
        assert_eq!(view.columns[0].items.len(), 1);
    }

    #[test]
    fn column_view_on_select_is_some() {
        let view = ColumnView::new("browser").on_select(|_col, _item, _window, _cx| {});
        assert!(view.on_select.is_some());
    }
}
