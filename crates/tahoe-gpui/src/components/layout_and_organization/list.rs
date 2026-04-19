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
//! - [`ListStyle::Sidebar`] — Liquid Glass source-list variant intended
//!   for sidebar/navigator panes. Selection tints with `theme.accent`
//!   and keyboard-focused rows draw a 2pt accent outline.
//! - [`ListStyle::Bordered`] — hairline-bordered container with
//!   alternating-row zebra striping, matching AppKit's
//!   `NSTableView.Style.bordered`.
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
use gpui::{AnyElement, App, ElementId, KeyDownEvent, SharedString, Window, div, px};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::color::with_alpha;
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, GlassSize, TahoeTheme, TextStyle, TextStyledExt};

/// Visual style for a [`List`].
///
/// Marked `#[non_exhaustive]` so additional HIG variants can be added in
/// future minor releases without breaking downstream `match` statements.
#[non_exhaustive]
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
    /// Source-list variant with a Liquid Glass (`GlassSize::Medium`)
    /// background. Selected rows tint with `theme.accent`; the
    /// keyboard-focused row draws a 2pt accent outline via the standard
    /// focus-ring token. Intended for navigator / sidebar panes.
    Sidebar,
    /// 1pt `theme.separator` border around the list, alternating-row
    /// zebra stripes using `theme.surface_muted()`, `theme.radius_md`
    /// corners. Matches AppKit's `NSTableView.Style.bordered`.
    Bordered,
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
    pub fn swipe_actions(mut self, actions: impl IntoIterator<Item = impl IntoElement>) -> Self {
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
    is_focused: bool,
    row_index: usize,
    style: ListStyle,
    theme: &TahoeTheme,
    on_select: &OnSelect,
) -> AnyElement {
    // Selection colour: the Sidebar style tints with `theme.accent` at
    // ~15% alpha (matching the HIG source-list soft-tint selection), so
    // the row label keeps the surface's primary text colour. All other
    // styles fall back to the solid accent fill with on-accent text.
    let selection_is_tinted = matches!(style, ListStyle::Sidebar);
    let text_color = if is_selected && !selection_is_tinted {
        theme.text_on_accent
    } else {
        theme.text
    };
    let bg = if is_selected {
        if selection_is_tinted {
            with_alpha(theme.accent, 0.15)
        } else {
            theme.accent
        }
    } else if matches!(style, ListStyle::Bordered) && !row_index.is_multiple_of(2) {
        // Bordered style: zebra-stripe every *odd* row with
        // `surface_muted`, matching NSTableView's alternating-row
        // convention (row 0 on the list surface, row 1 tinted, …).
        theme.surface_muted()
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
        .id(ElementId::Name(SharedString::from(format!(
            "list-row-{}",
            row.id
        ))))
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
        let mut actions = div().flex().flex_row().gap(theme.spacing_xs).ml_auto();
        for action in row.swipe_actions {
            actions = actions.child(action);
        }
        row_el = row_el.child(actions);
    }

    match style {
        ListStyle::Plain | ListStyle::Grouped => {
            row_el = row_el.border_b_1().border_color(theme.separator_color());
        }
        ListStyle::Inset | ListStyle::Sidebar => {
            row_el = row_el.rounded(theme.radius_md);
        }
        ListStyle::Bordered => {
            row_el = row_el.border_b_1().border_color(theme.separator_color());
        }
    }

    // Sidebar style: the *keyboard-focused* row (distinct from the
    // mouse-selected one) gets a 2pt accent outline via the theme's
    // focus-ring shadow stack — mirrors HIG's keyboard-navigation tell
    // on AppKit source lists.
    if matches!(style, ListStyle::Sidebar) {
        row_el = apply_focus_ring(row_el, theme, is_focused, &[]);
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

        let mut container = div().id(self.id).focusable().flex().flex_col().w_full();

        match style {
            ListStyle::Plain => {
                container = container
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md);
            }
            ListStyle::Grouped | ListStyle::Inset => {
                container = container.gap(theme.spacing_md);
            }
            ListStyle::Sidebar => {
                // Liquid Glass source-list surface (`GlassSize::Medium`)
                // — matches the HIG sidebar material. Applying the glass
                // tokens inline (rather than via `glass_surface`) keeps
                // the container's `Stateful<Div>` type without requiring
                // a `Div -> Div` adapter.
                let glass_bg = theme
                    .glass
                    .accessible_bg(GlassSize::Medium, theme.accessibility_mode);
                container = container
                    .bg(glass_bg)
                    .rounded(theme.glass.radius(GlassSize::Medium))
                    .shadow(theme.glass.shadows(GlassSize::Medium).to_vec());
            }
            ListStyle::Bordered => {
                // 1pt `separator` hairline + rounded corners. Background
                // is the list surface so zebra-striped rows sit on it.
                container = container
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.separator_color())
                    .rounded(theme.radius_md)
                    .overflow_hidden();
            }
        }

        // Focus tracking: when the list itself is focused, the
        // Sidebar style promotes the currently-selected row to the
        // keyboard-focused row so it picks up the 2pt accent outline.
        // Other styles ignore per-row focus.
        let focused_id = if self.focused {
            selected_id.clone()
        } else {
            None
        };
        let mut global_row_index: usize = 0;

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
                ListStyle::Plain | ListStyle::Sidebar | ListStyle::Bordered => {}
            }
            for (idx, row) in section.rows.into_iter().enumerate() {
                let is_last = idx + 1 == rows_len;
                let is_selected = selected_id.as_ref() == Some(&row.id);
                let is_focused = focused_id.as_ref() == Some(&row.id);
                let mut el = render_row(
                    row,
                    is_selected,
                    is_focused,
                    global_row_index,
                    style,
                    theme,
                    &on_select,
                );
                global_row_index += 1;
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

        if !all_ids.is_empty()
            && let Some(cb) = on_select.clone()
        {
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
        assert_eq!(
            section.header.as_ref().map(|s| s.as_ref()),
            Some("Favorites")
        );
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

    #[test]
    fn list_style_sidebar_and_bordered_are_distinct() {
        // Sidebar and Bordered are additive HIG variants; make sure
        // equality + Copy semantics still hold so patterns keyed off
        // `ListStyle` don't accidentally coalesce the two.
        assert_ne!(ListStyle::Sidebar, ListStyle::Bordered);
        assert_ne!(ListStyle::Sidebar, ListStyle::default());
        assert_ne!(ListStyle::Bordered, ListStyle::default());
        let copy = ListStyle::Sidebar;
        assert_eq!(copy, ListStyle::Sidebar);
    }

    #[test]
    fn list_accepts_sidebar_and_bordered_styles() {
        let l = List::new("l").style(ListStyle::Sidebar);
        assert_eq!(l.style, ListStyle::Sidebar);
        let l = List::new("l").style(ListStyle::Bordered);
        assert_eq!(l.style, ListStyle::Bordered);
    }
}
