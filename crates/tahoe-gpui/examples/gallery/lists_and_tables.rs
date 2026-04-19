//! Lists and Tables demo.

use gpui::prelude::*;
use gpui::{AnyElement, Context, Window, div};

use tahoe_gpui::components::layout_and_organization::table::{
    SelectionMode, Table, TableColumn, TableRow,
};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

use crate::ComponentGallery;

pub fn render(
    state: &mut ComponentGallery,
    _window: &mut Window,
    cx: &mut Context<ComponentGallery>,
) -> AnyElement {
    let theme = cx.global::<TahoeTheme>().clone();
    let theme = &theme;
    let entity = cx.entity().clone();
    let table_selected = state.table_selected;

    let columns = vec![
        TableColumn::new("name", "Name").width(180.0).sortable(),
        TableColumn::new("kind", "Kind").width(100.0).sortable(),
        TableColumn::new("size", "Size").width(80.0),
        TableColumn::new("modified", "Modified"),
    ];

    let rows = vec![
        TableRow::new(vec!["Documents", "Folder", "\u{2014}", "Yesterday"]),
        TableRow::new(vec!["Photos", "Folder", "\u{2014}", "2 days ago"]),
        TableRow::new(vec!["report.pdf", "PDF", "1.2 MB", "Mar 12"]),
        TableRow::new(vec!["budget.xlsx", "Spreadsheet", "84 KB", "Mar 10"]),
        TableRow::new(vec!["notes.md", "Markdown", "3 KB", "Mar 8"]),
        TableRow::new(vec!["screenshot.png", "Image", "2.1 MB", "Mar 7"]),
    ];

    let selected_rows = table_selected.map(|i| vec![i]).unwrap_or_default();

    let status_text = match table_selected {
        Some(i) => format!("Selected row: {}", i),
        None => "No row selected".to_string(),
    };

    div()
        .id("lists-tables-pane")
        .p(theme.spacing_xl)
        .flex()
        .flex_col()
        .gap(theme.spacing_md)
        .child(
            div()
                .text_style_emphasized(TextStyle::LargeTitle, theme)
                .text_color(theme.text)
                .child("Lists and Tables"),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text_muted)
                .child("Tables present sortable, selectable rows of structured data."),
        )
        .child(
            div()
                .text_style(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(status_text),
        )
        .child(div().h(theme.spacing_sm))
        .child(
            div()
                .border_1()
                .border_color(theme.border)
                .rounded(theme.radius_lg)
                .overflow_hidden()
                .child(
                    Table::new("demo-table")
                        .columns(columns)
                        .rows(rows)
                        .selection_mode(SelectionMode::Single)
                        .selected_rows(selected_rows)
                        .on_select(move |row_idx, _window, cx| {
                            entity.update(cx, |this, cx| {
                                this.table_selected = Some(row_idx);
                                cx.notify();
                            });
                        }),
                ),
        )
        .into_any_element()
}
