//! Example: a standalone macOS Tahoe Windows/Toolbar pattern.
//!
//! Mirrors the canonical "Windows/Toolbar" frame from the macOS 26
//! (Community) UI Kit by Apple Design Resources at 1512x982. Renders the
//! white window-with-traffic-lights pattern centered on a wallpaper-tinted
//! background, with the standard sidebar + toolbar chrome:
//!
//!   - Sidebar: folder list with section header + disclosure
//!   - Toolbar: sidebar toggle, back/forward, title, action icons, search
//!
//! Distinct from `window_layouts` which bundles five chrome variants behind
//! a switcher; this one is the canonical Toolbar window in isolation.

use gpui::prelude::*;
use gpui::{
    App, Bounds, ElementId, FontWeight, SharedString, Window, WindowBackgroundAppearance,
    WindowBounds, WindowOptions, div, hsla, px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::layout_and_organization::disclosure::Disclosure;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::sidebar::{Sidebar, SidebarItem};
use tahoe_gpui::components::navigation_and_search::toolbar::Toolbar;
use tahoe_gpui::foundations::icons::{EmbeddedIconAssets, Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

// ── Static data ──────────────────────────────────────────────────────────────

struct Section {
    header: Option<&'static str>,
    folders: &'static [&'static str],
}

const SECTIONS: &[Section] = &[
    Section {
        header: None,
        folders: &["Folder 1", "Folder 2", "Folder 3", "Folder 4", "Folder 5"],
    },
    Section {
        header: Some("Section Header"),
        folders: &["Folder 6", "Folder 7"],
    },
];

// ── App state ────────────────────────────────────────────────────────────────

struct ToolbarApp {
    selected_folder: usize,
    section_expanded: Vec<bool>,
}

impl ToolbarApp {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            // Folder 2 is the canonical default selection in the Figma reference.
            selected_folder: 1,
            section_expanded: vec![true; SECTIONS.len()],
        }
    }

    fn selected_folder_name(&self) -> &'static str {
        let mut idx = 0;
        for section in SECTIONS {
            for folder in section.folders {
                if idx == self.selected_folder {
                    return folder;
                }
                idx += 1;
            }
        }
        "Folder"
    }
}

impl Render for ToolbarApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let theme = &theme;
        let folder_name = self.selected_folder_name();

        // ── Sidebar ──────────────────────────────────────────────────────
        let mut sidebar_content = div().flex().flex_col().size_full().pt(theme.spacing_sm);
        let mut global_idx: usize = 0;

        for (section_idx, section) in SECTIONS.iter().enumerate() {
            let is_expanded = self.section_expanded[section_idx];

            // Section header (with disclosure) — only the second section has one
            // in the canonical Figma reference.
            if let Some(header) = section.header {
                sidebar_content = sidebar_content.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(theme.spacing_md)
                        .pt(theme.spacing_md)
                        .pb(theme.spacing_xs)
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child(header),
                        )
                        .child({
                            let entity = cx.entity().downgrade();
                            Disclosure::new(ElementId::NamedInteger(
                                "section-disc".into(),
                                section_idx as u64,
                            ))
                            .expanded(is_expanded)
                            .on_toggle(
                                move |_expanded, _window, cx| {
                                    if let Some(this) = entity.upgrade() {
                                        this.update(cx, |this, cx| {
                                            this.section_expanded[section_idx] =
                                                !this.section_expanded[section_idx];
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                        }),
                );
            }

            if is_expanded {
                for folder in section.folders {
                    let idx = global_idx;
                    sidebar_content = sidebar_content.child(
                        SidebarItem::new(
                            ElementId::NamedInteger("folder".into(), idx as u64),
                            SharedString::from(*folder),
                        )
                        .icon(IconName::Folder)
                        .selected(idx == self.selected_folder)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.selected_folder = idx;
                            cx.notify();
                        })),
                    );
                    global_idx += 1;
                }
            } else {
                global_idx += section.folders.len();
            }
        }

        let sidebar = Sidebar::new("toolbar-sidebar")
            .width(px(220.0))
            .child(sidebar_content);

        // ── Toolbar (canonical Apple Tahoe layout) ───────────────────────
        let toolbar = Toolbar::new("window-toolbar")
            // Leading: sidebar toggle, then back/forward
            .leading(
                Button::new("toggle-sidebar")
                    .icon(Icon::new(IconName::DevSidebar).size(px(14.0)))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm),
            )
            .leading(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.0))
                    .child(
                        Button::new("nav-back")
                            .icon(Icon::new(IconName::ChevronLeft).size(px(14.0)))
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm),
                    )
                    .child(
                        Button::new("nav-forward")
                            .icon(Icon::new(IconName::ChevronRight).size(px(14.0)))
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm),
                    ),
            )
            .title(SharedString::from(folder_name))
            // Trailing: action icons (folder, trash, archive, tag, pencil) + search
            .trailing(
                Button::new("act-new-folder")
                    .icon(Icon::new(IconName::FolderOpen).size(px(14.0)))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm),
            )
            .trailing(
                Button::new("act-trash")
                    .icon(Icon::new(IconName::Trash).size(px(14.0)))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm),
            )
            .trailing(
                Button::new("act-archive")
                    .icon(Icon::new(IconName::Package).size(px(14.0)))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm),
            )
            .trailing(
                Button::new("act-tag")
                    .icon(Icon::new(IconName::Bookmark).size(px(14.0)))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm),
            )
            .trailing(
                Button::new("act-edit")
                    .icon(Icon::new(IconName::Pencil).size(px(14.0)))
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm),
            )
            // Search field placeholder (capsule with icon + "Search")
            .trailing(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.0))
                    .h(px(28.0))
                    .px(px(10.0))
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md)
                    .child(
                        Icon::new(IconName::Search)
                            .size(px(13.0))
                            .color(theme.text_muted),
                    )
                    .child(
                        div()
                            .w(px(120.0))
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child("Search"),
                    ),
            );

        // ── Window content area (right of sidebar) ───────────────────────
        let content_area = div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(theme.background)
            .child(toolbar)
            .child(div().flex_1());

        // ── The actual application window (sidebar + content) ───────────
        // Centered inside an outer wallpaper region.
        let app_window = div()
            .w(px(900.0))
            .h(px(560.0))
            .rounded(px(12.0))
            .overflow_hidden()
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.0, 0.12))
            .bg(theme.background)
            .shadow(vec![gpui::BoxShadow {
                color: hsla(0.0, 0.0, 0.0, 0.18),
                offset: gpui::point(px(0.0), px(20.0)),
                blur_radius: px(40.0),
                spread_radius: px(0.0),
            }])
            .flex()
            .flex_row()
            .child(sidebar)
            .child(content_area);

        // ── Wallpaper region (Tahoe-blue tinted background) ──────────────
        // 1512×982 to match the Figma frame, with the window centered.
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(210.0 / 360.0, 0.55, 0.55, 1.0))
            .child(app_window)
            // Tiny status line for debugging which folder is selected
            .child(
                div()
                    .absolute()
                    .top(px(12.0))
                    .left(px(20.0))
                    .text_size(px(11.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(hsla(0.0, 0.0, 1.0, 0.85))
                    .child(SharedString::from(format!("Selected: {folder_name}"))),
            )
    }
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            let theme = TahoeTheme::liquid_glass_light();
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            // Match the Figma frame size (1512 x 982).
            let bounds = Bounds::centered(None, size(px(1512.0), px(982.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| cx.new(ToolbarApp::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
