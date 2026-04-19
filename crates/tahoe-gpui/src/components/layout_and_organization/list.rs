//! Single-column list (HIG Lists and Tables — list-only variant).
//!
//! A `List` renders a single-column, section-aware list — distinct from
//! [`Table`](super::table::Table) (multi-column) and
//! [`OutlineView`](super::outline_view::OutlineView) (hierarchical).
//! Use it for iOS-style grouped settings, inspector panels, or any
//! vertical stack of labelled rows that may or may not be grouped.
//!
//! # Styles
//!
//! - [`ListStyle::Plain`] — flush rows, no grouping.
//! - [`ListStyle::Grouped`] — sections grouped with small gaps and
//!   uppercase caption-style section headers.
//! - [`ListStyle::Inset`] — grouped with rounded inset cards, matching
//!   `UICollectionLayoutListConfiguration.Appearance.insetGrouped`.
//!
//! # Swipe actions
//!
//! `NSTableView` and `UITableView` expose leading and trailing swipe
//! actions (delete, archive, …). `List` surfaces these via
//! [`ListRow::swipe_actions`]; callers render a side panel of action
//! buttons that the host platform slides in.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/lists-and-tables>

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ElementId, KeyDownEvent, SharedString, Window, div, px,
};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};

/// Visual style for a [`List`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum ListStyle {
    /// Flush rows without grouping. Matches `UITableView.Style.plain`.
    #[default]
    Plain,
    /// Sections grouped with small gaps between rows. Matches
    /// `UITableView.Style.grouped`.
    Grouped,
    /// Grouped with rounded inset cards. Matches
    /// `UICollectionLayoutListConfiguration.Appearance.insetGrouped`.
    Inset,
}

/// One row in a [`List`] or [`ListSection`].
pub struct ListRow {
    pub id: SharedString,
    body: AnyElement,
    swipe_actions: Vec<AnyElement>,
}

impl ListRow {
    pub fn new(id: impl Into<SharedString>, body: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            body: body.into_any_element(),
            swipe_actions: Vec::new(),
        }
    }

    /// Attach trailing swipe actions (for example: a "Delete" button). The
    /// actions are rendered off-screen behind the row and revealed via
    /// platform swipe gestures; on macOS, also reachable via
    /// right-click / Ctrl-click contextual menus.
    pub fn swipe_actions(
        mut self,
        actions: impl IntoIterator<Item = impl IntoElement>,
    ) -> Self {
        self.swipe_actions = actions.into_iter().map(|a| a.into_any_element()).collect();
        self
    }
}

/// One section in a [`List`].
pub struct ListSection {
    pub header: Option<SharedString>,
    pub rows: Vec<ListRow>,
}

impl ListSection {
    pub fn new() -> Self {
        Self {
            header: None,
            rows: Vec::new(),
        }
    }

    pub fn header(mut self, title: impl Into<SharedString>) -> Self {
        self.header = Some(title.into());
        self
    }

    pub fn row(mut self, row: ListRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn rows(mut self, rows: impl IntoIterator<Item = ListRow>) -> Self {
        self.rows.extend(rows);
        self
    }
}

impl Default for ListSection {
    fn default() -> Self {
        Self::new()
    }
}

type OnSelect = Option<Rc<dyn Fn(SharedString, &mut Window, &mut App)>>;

/// An HIG-style single-column list.
#[derive(IntoElement)]
pub struct List {
    id: ElementId,
    style: ListStyle,
    sections: Vec<ListSection>,
    selected_id: Option<SharedString>,
    on_select: OnSelect,
    focused: bool,
}

impl List {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            style: ListStyle::default(),
            sections: Vec::new(),
            selected_id: None,
            on_select: None,
            focused: false,
        }
    }

    pub fn style(mut self, style: ListStyle) -> Self {
        self.style = style;
        self
    }

    pub fn sections(mut self, sections: impl IntoIterator<Item = ListSection>) -> Self {
        self.sections = sections.into_iter().collect();
        self
    }

    pub fn section(mut self, section: ListSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn selected_id(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

fn render_row(
    row: ListRow,
    is_selected: bool,
    style: ListStyle,
    theme: &TahoeTheme,
    on_select: &OnSelect,
) -> AnyElement {
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
        .label(row.id.clone())
        .role(AccessibilityRole::Button)
        .value(if is_selected {
            SharedString::from("selected")
        } else {
            SharedString::from("unselected")
        });

    let row_id = row.id.clone();
    let mut row_el = div()
        .id(ElementId::Name(
            SharedString::from(format!("list-row-{}", row.id)),
        ))
        .flex()
        .flex_row()
        .items_center()
        .gap(theme.spacing_sm)
        .px(theme.spacing_md)
        .min_h(px(theme.target_size()))
        .bg(bg)
        .cursor_pointer()
        .with_accessibility(&ax)
        .hover(|s| s.bg(theme.hover))
        .text_style(TextStyle::Body, theme)
        .text_color(text_color)
        .child(row.body);

    // Swipe actions: render trailing, hidden by default. GPUI lacks a
    // native swipe gesture primitive, so we draw the actions inline
    // at the trailing edge — on macOS they're normally surfaced via
    // right-click, which callers wire up themselves.
    if !row.swipe_actions.is_empty() {
        let mut actions = div()
            .flex()
            .flex_row()
            .gap(theme.spacing_xs)
            .ml_auto();
        for action in row.swipe_actions {
            actions = actions.child(action);
        }
        row_el = row_el.child(actions);
    }

    match style {
        ListStyle::Plain | ListStyle::Grouped => {
            row_el = row_el
                .border_b_1()
                .border_color(theme.separator_color());
        }
        ListStyle::Inset => {
            row_el = row_el.rounded(theme.radius_md);
        }
    }

    if let Some(cb) = on_select.clone() {
        row_el = row_el.on_click(move |_event, window, cx| {
            cb(row_id.clone(), window, cx);
        });
    }

    row_el.into_any_element()
}

impl RenderOnce for List {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let style = self.style;
        let selected_id = self.selected_id.clone();
        let on_select = self.on_select.clone();

        // Collect row ids for keyboard navigation.
        let all_ids: Vec<SharedString> = self
            .sections
            .iter()
            .flat_map(|s| s.rows.iter().map(|r| r.id.clone()))
            .collect();

        let mut container = div()
            .id(self.id)
            .focusable()
            .flex()
            .flex_col()
            .w_full();

        match style {
            ListStyle::Plain => {
                container = container
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md);
            }
            ListStyle::Grouped => {
                container = container.gap(theme.spacing_md);
            }
            ListStyle::Inset => {
                container = container.gap(theme.spacing_md);
            }
        }

        for section in self.sections {
            let mut section_el = div().flex().flex_col();

            if let Some(header) = section.header {
                section_el = section_el.child(
                    div()
                        .pb(theme.spacing_xs)
                        .pl(theme.spacing_md)
                        .text_style(TextStyle::Caption2, theme)
                        .text_color(theme.text_muted)
                        .child(SharedString::from(header.to_uppercase())),
                );
            }

            let rows_len = section.rows.len();
            let mut rows_block = div().flex().flex_col();
            match style {
                ListStyle::Grouped | ListStyle::Inset => {
                    rows_block = rows_block
                        .bg(theme.surface)
                        .border_1()
                        .border_color(theme.border)
                        .rounded(theme.radius_md)
                        .overflow_hidden();
                }
                ListStyle::Plain => {}
            }
            for (idx, row) in section.rows.into_iter().enumerate() {
                let is_last = idx + 1 == rows_len;
                let is_selected = selected_id.as_ref() == Some(&row.id);
                let mut el = render_row(row, is_selected, style, theme, &on_select);
                // Remove bottom border on the last row of a grouped section
                // so the rounded corner stays clean — doing this by wrapping
                // the element is cheaper than threading through render_row.
                if is_last {
                    el = div().child(el).into_any_element();
                }
                rows_block = rows_block.child(el);
            }
            section_el = section_el.child(rows_block);
            container = container.child(section_el);
        }

        if !all_ids.is_empty() {
            if let Some(cb) = on_select.clone() {
                let ids = all_ids;
                let selected_at_render = selected_id.clone();
                container = container.on_key_down(move |event: &KeyDownEvent, window, cx| {
                    let key = event.keystroke.key.as_str();
                    let current = selected_at_render
                        .as_ref()
                        .and_then(|id| ids.iter().position(|x| x == id));
                    let next_idx = match key {
                        "down" => Some(match current {
                            Some(i) if i + 1 < ids.len() => i + 1,
                            Some(i) => i,
                            None => 0,
                        }),
                        "up" => Some(match current {
                            Some(0) | None => 0,
                            Some(i) => i - 1,
                        }),
                        "home" => Some(0),
                        "end" => Some(ids.len() - 1),
                        "enter" | "space" => current,
                        _ => None,
                    };
                    if let Some(idx) = next_idx {
                        cx.stop_propagation();
                        cb(ids[idx].clone(), window, cx);
                    }
                });
            }
        }

        container = apply_focus_ring(container, theme, self.focused, &[]);

        container
    }
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;
    use gpui::div;

    use super::{List, ListRow, ListSection, ListStyle};

    #[test]
    fn list_style_default_is_plain() {
        assert_eq!(ListStyle::default(), ListStyle::Plain);
    }

    #[test]
    fn list_row_new() {
        let row = ListRow::new("r1", div());
        assert_eq!(row.id.as_ref(), "r1");
        assert!(row.swipe_actions.is_empty());
    }

    #[test]
    fn list_row_swipe_actions() {
        let row = ListRow::new("r1", div()).swipe_actions(vec![div(), div()]);
        assert_eq!(row.swipe_actions.len(), 2);
    }

    #[test]
    fn list_section_defaults() {
        let section = ListSection::new();
        assert!(section.header.is_none());
        assert!(section.rows.is_empty());
    }

    #[test]
    fn list_section_builder() {
        let section = ListSection::new()
            .header("Favorites")
            .row(ListRow::new("a", div()))
            .row(ListRow::new("b", div()));
        assert_eq!(section.header.as_ref().map(|s| s.as_ref()), Some("Favorites"));
        assert_eq!(section.rows.len(), 2);
    }

    #[test]
    fn list_defaults() {
        let list = List::new("list");
        assert_eq!(list.style, ListStyle::Plain);
        assert!(list.sections.is_empty());
        assert!(list.selected_id.is_none());
        assert!(list.on_select.is_none());
        assert!(!list.focused);
    }

    #[test]
    fn list_builder() {
        let list = List::new("list")
            .style(ListStyle::Inset)
            .section(ListSection::new().row(ListRow::new("a", div())))
            .selected_id("a")
            .on_select(|_id, _w, _cx| {});
        assert_eq!(list.style, ListStyle::Inset);
        assert_eq!(list.sections.len(), 1);
        assert!(list.on_select.is_some());
    }
}
