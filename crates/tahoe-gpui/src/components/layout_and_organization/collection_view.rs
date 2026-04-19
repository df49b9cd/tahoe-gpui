//! Collection view aligned with HIG Collections.
//!
//! Displays items in a flexible grid layout. Items can have uniform
//! or variable sizes and the grid adapts to the available width.
//!
//! # Sections
//!
//! HIG: `NSCollectionView` supports sections with headers and footers
//! (`NSCollectionViewSection`). [`CollectionSection`] models a single
//! section — an optional header element plus a row of items that share
//! the collection's layout. Use [`CollectionView::sections`] to render
//! grouped content.
//!
//! # Selection & keyboard
//!
//! Single selection + arrow-key navigation is supported via
//! [`CollectionView::on_select`] / [`CollectionView::selected_index`].
//! Arrow keys move the focused item by one column or one row.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/collections>

use std::rc::Rc;

use gpui::prelude::*;
use gpui::{AnyElement, App, ElementId, KeyDownEvent, Pixels, SharedString, Window, div, px};

use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::materials::apply_focus_ring;
use crate::foundations::theme::{ActiveTheme, TahoeTheme, TextStyle, TextStyledExt};

/// Grid layout style for a collection view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectionLayout {
    /// Fixed number of columns, items stretch to fill.
    Grid { columns: usize },
    /// Items have a fixed width, columns auto-calculated.
    Flow { item_width: Pixels },
}

impl CollectionLayout {
    fn columns(&self) -> usize {
        match self {
            CollectionLayout::Grid { columns } => (*columns).max(1),
            // Flow layouts have no inherent column count — caller drives
            // keyboard nav one-by-one.
            CollectionLayout::Flow { .. } => 1,
        }
    }
}

impl Default for CollectionLayout {
    fn default() -> Self {
        Self::Grid { columns: 3 }
    }
}

/// A single section within a [`CollectionView`].
///
/// Each section renders its header above a row of items that share the
/// collection's layout. Optional footer is reserved for HIG parity with
/// `NSCollectionView` but not rendered today.
pub struct CollectionSection {
    header: Option<AnyElement>,
    items: Vec<AnyElement>,
}

impl CollectionSection {
    pub fn new() -> Self {
        Self {
            header: None,
            items: Vec::new(),
        }
    }

    /// Render arbitrary content above the items row (e.g. a section title).
    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    /// Convenience: render a caption-style title string as the header.
    pub fn title(self, title: impl Into<SharedString>) -> Self {
        let title: SharedString = title.into();
        let header = SectionTitle { text: title };
        self.header(header)
    }

    pub fn item(mut self, item: impl IntoElement) -> Self {
        self.items.push(item.into_any_element());
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.items
            .extend(items.into_iter().map(|i| i.into_any_element()));
        self
    }
}

impl Default for CollectionSection {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(IntoElement)]
struct SectionTitle {
    text: SharedString,
}

impl RenderOnce for SectionTitle {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .w_full()
            .pb(theme.spacing_xs)
            .text_style(TextStyle::Caption2, theme)
            .text_color(theme.text_muted)
            .child(SharedString::from(self.text.to_uppercase()))
    }
}

type OnSelect = Option<Rc<dyn Fn(usize, &mut Window, &mut App)>>;

/// A grid/flow collection view per HIG.
///
/// Renders items in a responsive grid with consistent spacing, optional
/// sections, single selection, and arrow-key navigation.
#[derive(IntoElement)]
pub struct CollectionView {
    id: ElementId,
    layout: CollectionLayout,
    /// Spacing override. `None` resolves to `theme.spacing_sm_md` (12 pt)
    /// at render time so the grid tracks theme updates instead of baking
    /// in a value at construction.
    spacing: Option<Pixels>,
    items: Vec<AnyElement>,
    sections: Vec<CollectionSection>,
    labels: Vec<Option<SharedString>>,
    selected_index: Option<usize>,
    focused: bool,
    on_select: OnSelect,
}

impl CollectionView {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            layout: CollectionLayout::default(),
            spacing: None,
            items: Vec::new(),
            sections: Vec::new(),
            labels: Vec::new(),
            selected_index: None,
            focused: false,
            on_select: None,
        }
    }

    pub fn layout(mut self, layout: CollectionLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn columns(mut self, columns: usize) -> Self {
        self.layout = CollectionLayout::Grid {
            columns: columns.max(1),
        };
        self
    }

    pub fn spacing(mut self, spacing: Pixels) -> Self {
        self.spacing = Some(spacing);
        self
    }

    pub fn child(mut self, item: impl IntoElement) -> Self {
        self.items.push(item.into_any_element());
        self.labels.push(None);
        self
    }

    pub fn children(mut self, items: impl IntoIterator<Item = impl IntoElement>) -> Self {
        let extra: Vec<AnyElement> = items.into_iter().map(|i| i.into_any_element()).collect();
        let extra_len = extra.len();
        self.items.extend(extra);
        self.labels.extend(std::iter::repeat_n(None, extra_len));
        self
    }

    /// Append a labelled item. The label is exposed to VoiceOver and to
    /// on-screen debug selectors — use it to describe the tile's purpose
    /// (e.g. "Photo: Beach trip, June 2024").
    pub fn labeled_child(mut self, label: impl Into<SharedString>, item: impl IntoElement) -> Self {
        self.items.push(item.into_any_element());
        self.labels.push(Some(label.into()));
        self
    }

    /// Add a pre-built [`CollectionSection`] (header + items). Sections and
    /// top-level items may be mixed; sections render after items.
    pub fn section(mut self, section: CollectionSection) -> Self {
        self.sections.push(section);
        self
    }

    pub fn sections(mut self, sections: impl IntoIterator<Item = CollectionSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Current selected item index (counts top-level items only; sectioned
    /// layouts manage selection at the caller site).
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    /// Show a focus ring around the collection when keyboard-focused.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

fn items_row(
    items: Vec<AnyElement>,
    labels: &[Option<SharedString>],
    layout: CollectionLayout,
    spacing: Pixels,
    selected_index: Option<usize>,
    on_select: &OnSelect,
    theme: &TahoeTheme,
    id_offset: usize,
) -> gpui::Div {
    let mut grid = div().flex().flex_wrap().gap(spacing);
    for (i, item) in items.into_iter().enumerate() {
        let is_selected = selected_index == Some(i + id_offset);
        let label = labels.get(i).cloned().flatten();
        let ax = AccessibilityProps::new()
            .label(label.clone().unwrap_or_else(|| SharedString::from("item")))
            .role(AccessibilityRole::Button)
            .value(if is_selected {
                SharedString::from("selected")
            } else {
                SharedString::from("unselected")
            });

        let mut tile = div()
            .id(ElementId::NamedInteger(
                SharedString::from("collection-item"),
                (i + id_offset) as u64,
            ))
            .cursor_pointer()
            .rounded(theme.radius_md)
            .with_accessibility(&ax);

        if is_selected {
            tile = tile.bg(theme.hover);
        }

        tile = tile.hover(|s| s.bg(theme.hover));

        if let Some(cb) = on_select.clone() {
            let idx = i + id_offset;
            tile = tile.on_click(move |_event, window, cx| {
                cb(idx, window, cx);
            });
        }

        tile = tile.child(item);

        match layout {
            CollectionLayout::Grid { columns } => {
                let basis = gpui::relative(1.0 / columns.max(1) as f32);
                grid = grid.child(
                    div()
                        .flex_basis(basis)
                        .flex_grow()
                        .min_w(px(0.0))
                        .child(tile),
                );
            }
            CollectionLayout::Flow { item_width } => {
                grid = grid.child(div().w(item_width).flex_shrink_0().child(tile));
            }
        }
    }
    grid
}

impl RenderOnce for CollectionView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let spacing = self.spacing.unwrap_or(theme.spacing_sm_md);

        let total_items = self.items.len();
        let columns = self.layout.columns();
        let selected = self.selected_index;
        let select_cb = self.on_select.clone();

        let mut container = div().id(self.id).focusable().flex().flex_col().gap(spacing);

        // Top-level items (outside any section).
        if !self.items.is_empty() {
            container = container.child(items_row(
                self.items,
                &self.labels,
                self.layout,
                spacing,
                selected,
                &self.on_select,
                theme,
                0,
            ));
        }

        // Sections with optional headers.
        for section in self.sections {
            let mut section_el = div().flex().flex_col().gap(theme.spacing_xs);
            if let Some(header) = section.header {
                section_el = section_el.child(header);
            }
            if !section.items.is_empty() {
                let label_stub: Vec<Option<SharedString>> =
                    std::iter::repeat_n(None, section.items.len()).collect();
                section_el = section_el.child(items_row(
                    section.items,
                    &label_stub,
                    self.layout,
                    spacing,
                    None,
                    // Sectioned selection is caller-managed; no callback path
                    // into this wrapper.
                    &None,
                    theme,
                    0,
                ));
            }
            container = container.child(section_el);
        }

        // Keyboard navigation: arrow keys move by column / row, Home/End
        // jump to first / last, Enter/Space activate selection.
        if total_items > 0 {
            container = container.on_key_down(move |event: &KeyDownEvent, window, cx| {
                let Some(cb) = select_cb.clone() else {
                    return;
                };
                let key = event.keystroke.key.as_str();
                let current = selected.unwrap_or(0);
                let next: Option<usize> = match key {
                    "right" => Some((current + 1).min(total_items - 1)),
                    "left" => Some(current.saturating_sub(1)),
                    "down" => Some((current + columns).min(total_items - 1)),
                    "up" => Some(current.saturating_sub(columns)),
                    "home" => Some(0),
                    "end" => Some(total_items - 1),
                    "enter" | "space" => Some(current),
                    _ => None,
                };
                if let Some(idx) = next {
                    cx.stop_propagation();
                    cb(idx, window, cx);
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

    use gpui::{div, px};

    use super::{CollectionLayout, CollectionSection, CollectionView};

    #[test]
    fn collection_view_default_layout() {
        let view = CollectionView::new("test");
        assert!(matches!(view.layout, CollectionLayout::Grid { columns: 3 }));
    }

    #[test]
    fn collection_view_custom_columns() {
        let view = CollectionView::new("test").columns(4);
        assert!(matches!(view.layout, CollectionLayout::Grid { columns: 4 }));
    }

    #[test]
    fn collection_view_min_one_column() {
        let view = CollectionView::new("test").columns(0);
        assert!(matches!(view.layout, CollectionLayout::Grid { columns: 1 }));
    }

    #[test]
    fn collection_view_custom_spacing() {
        let view = CollectionView::new("test").spacing(px(24.0));
        assert_eq!(view.spacing.map(f32::from), Some(24.0));
    }

    #[test]
    fn collection_view_default_spacing_is_none() {
        let view = CollectionView::new("test");
        assert!(view.spacing.is_none());
    }

    #[test]
    fn collection_view_on_select_builder() {
        let view = CollectionView::new("test").on_select(|_idx, _w, _cx| {});
        assert!(view.on_select.is_some());
    }

    #[test]
    fn collection_view_selected_index_builder() {
        let view = CollectionView::new("test").selected_index(Some(2));
        assert_eq!(view.selected_index, Some(2));
    }

    #[test]
    fn collection_view_labeled_child() {
        let view = CollectionView::new("test").labeled_child("tile 1", div());
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.labels.len(), 1);
        assert_eq!(view.labels[0].as_ref().map(|s| s.as_ref()), Some("tile 1"));
    }

    #[test]
    fn collection_section_builder() {
        let section = CollectionSection::new()
            .title("Favorites")
            .item(div())
            .item(div());
        assert!(section.header.is_some());
        assert_eq!(section.items.len(), 2);
    }

    #[test]
    fn collection_view_section_wiring() {
        let view = CollectionView::new("test")
            .section(CollectionSection::new().title("Favorites").item(div()));
        assert_eq!(view.sections.len(), 1);
    }
}
