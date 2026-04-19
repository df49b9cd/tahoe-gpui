//! Example: macOS window layout variants.
//!
//! Demonstrates five HIG window chrome patterns using Sidebar,
//! Toolbar, NavigationBarIOS, Disclosure, and SegmentedControl.
//!
//! Variants: Titlebar, Toolbar, Monobar, Toolbar (No Nav), Utility Panel.

use gpui::prelude::*;
use gpui::{
    App, Bounds, ElementId, SharedString, Window, WindowBounds, WindowOptions, div, px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::layout_and_organization::disclosure::Disclosure;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::sidebar::{Sidebar, SidebarItem};
use tahoe_gpui::components::navigation_and_search::toolbar::Toolbar;
use tahoe_gpui::components::selection_and_input::segmented_control::{
    SegmentItem, SegmentedControl,
};
use tahoe_gpui::foundations::icons::{EmbeddedIconAssets, Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

// u2500u2500 Folder data u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

struct FolderSection {
    header: &'static str,
    folders: &'static [&'static str],
}

const SECTIONS: &[FolderSection] = &[
    FolderSection {
        header: "Section Header",
        folders: &["Folder 1", "Folder 2", "Folder 3", "Folder 4", "Folder 5"],
    },
    FolderSection {
        header: "Section Header",
        folders: &["Folder 6", "Folder 7"],
    },
];

// u2500u2500 Window variant enum u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

#[derive(Debug, Clone, Copy, PartialEq)]
enum WindowVariant {
    Titlebar,
    ToolbarFull,
    Monobar,
    ToolbarNoNav,
    UtilityPanel,
}

impl WindowVariant {
    fn label(&self) -> &'static str {
        match self {
            Self::Titlebar => "Titlebar",
            Self::ToolbarFull => "Toolbar",
            Self::Monobar => "Monobar",
            Self::ToolbarNoNav => "No Nav",
            Self::UtilityPanel => "Utility",
        }
    }

    const ALL: [WindowVariant; 5] = [
        Self::Titlebar,
        Self::ToolbarFull,
        Self::Monobar,
        Self::ToolbarNoNav,
        Self::UtilityPanel,
    ];
}

// u2500u2500 App state u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500

struct WindowLayouts {
    variant_index: usize,
    selected_folder: usize,
    section_expanded: Vec<bool>,
}

impl WindowLayouts {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            variant_index: 1,   // Start on Toolbar
            selected_folder: 1, // Folder 2 selected
            section_expanded: vec![true; SECTIONS.len()],
        }
    }

    fn current_variant(&self) -> WindowVariant {
        WindowVariant::ALL[self.variant_index]
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

    // u2500u2500 Shared sidebar content u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500
    fn render_sidebar(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        let theme = cx.global::<TahoeTheme>();
        let mut content = div().flex().flex_col().size_full();
        let mut global_idx: usize = 0;

        for (section_idx, section) in SECTIONS.iter().enumerate() {
            let is_expanded = self.section_expanded[section_idx];

            // Section header with disclosure
            content = content.child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(theme.spacing_md)
                    .pt(if section_idx > 0 {
                        theme.spacing_md
                    } else {
                        theme.spacing_sm
                    })
                    .pb(theme.spacing_xs)
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child(section.header),
                    )
                    .child({
                        let entity = cx.entity().downgrade();
                        Disclosure::new(ElementId::NamedInteger(
                            "section-disc".into(),
                            section_idx as u64,
                        ))
                        .expanded(is_expanded)
                        .on_toggle(move |_expanded, _window, cx| {
                            if let Some(this) = entity.upgrade() {
                                this.update(cx, |this, cx| {
                                    this.section_expanded[section_idx] =
                                        !this.section_expanded[section_idx];
                                    cx.notify();
                                });
                            }
                        })
                    }),
            );

            if is_expanded {
                for folder in section.folders {
                    let idx = global_idx;
                    content = content.child(
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

        Sidebar::new("sidebar").child(content).into_any_element()
    }

    // u2500u2500 Toolbar action icons (shared across toolbar variants) u2500u2500u2500u2500u2500u2500u2500u2500u2500
    fn toolbar_actions() -> Vec<Button> {
        vec![
            Button::new("act-archive")
                .icon(Icon::new(IconName::Folder))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
            Button::new("act-trash")
                .icon(Icon::new(IconName::Trash))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
            Button::new("act-clipboard")
                .icon(Icon::new(IconName::Copy))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
            Button::new("act-tag")
                .icon(Icon::new(IconName::Bookmark))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
            Button::new("act-compose")
                .icon(Icon::new(IconName::Pencil))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm),
        ]
    }

    // u2500u2500 Render each variant's top chrome u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500u2500
    fn render_chrome(&self, variant: WindowVariant, theme: &TahoeTheme) -> gpui::AnyElement {
        let folder_name = self.selected_folder_name();

        match variant {
            // Titlebar: simple centered title
            WindowVariant::Titlebar => div()
                .flex()
                .items_center()
                .justify_center()
                .px(theme.spacing_md)
                .py(theme.spacing_sm)
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .text_style(TextStyle::Headline, theme)
                        .text_color(theme.text)
                        .child("Window Title"),
                )
                .into_any_element(),

            // Toolbar: sidebar toggle, back/forward nav, title, action icons, search
            WindowVariant::ToolbarFull => {
                let mut tb = Toolbar::new("toolbar")
                    .leading(
                        Button::new("toggle-sidebar")
                            .icon(Icon::new(IconName::DevSidebar))
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm),
                    )
                    .leading(
                        div()
                            .flex()
                            .items_center()
                            .gap(theme.spacing_xs)
                            .child(
                                Button::new("nav-back")
                                    .icon(Icon::new(IconName::ChevronLeft))
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSm),
                            )
                            .child(
                                Button::new("nav-forward")
                                    .icon(Icon::new(IconName::ChevronRight))
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSm),
                            ),
                    )
                    .title(SharedString::from(folder_name));

                for btn in Self::toolbar_actions() {
                    tb = tb.trailing(btn);
                }
                tb = tb.trailing(
                    Button::new("act-search")
                        .icon(Icon::new(IconName::Search))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm),
                );

                tb.into_any_element()
            }

            // Monobar: sidebar toggle, loading indicator, forward arrow, title, actions, search text
            WindowVariant::Monobar => {
                let mut tb = Toolbar::new("monobar")
                    .leading(
                        Button::new("toggle-sidebar")
                            .icon(Icon::new(IconName::DevSidebar))
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm),
                    )
                    .leading(
                        div()
                            .flex()
                            .items_center()
                            .gap(theme.spacing_xs)
                            .child(
                                Icon::new(IconName::Loader)
                                    .size(px(16.0))
                                    .color(theme.text_muted),
                            )
                            .child(
                                Button::new("nav-forward")
                                    .icon(Icon::new(IconName::ChevronRight))
                                    .variant(ButtonVariant::Ghost)
                                    .size(ButtonSize::IconSm),
                            ),
                    )
                    .title(SharedString::from(folder_name));

                for btn in Self::toolbar_actions() {
                    tb = tb.trailing(btn);
                }
                // Search as text label instead of icon
                tb = tb.trailing(
                    div()
                        .flex()
                        .items_center()
                        .gap(theme.spacing_xs)
                        .child(
                            Icon::new(IconName::Search)
                                .size(px(14.0))
                                .color(theme.text_muted),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child("Search"),
                        ),
                );

                tb.into_any_element()
            }

            // Toolbar (No Nav): sidebar toggle, title, action icons, search
            WindowVariant::ToolbarNoNav => {
                let mut tb = Toolbar::new("toolbar-nonav")
                    .leading(
                        Button::new("toggle-sidebar")
                            .icon(Icon::new(IconName::DevSidebar))
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm),
                    )
                    .title(SharedString::from(folder_name));

                for btn in Self::toolbar_actions() {
                    tb = tb.trailing(btn);
                }
                tb = tb.trailing(
                    Button::new("act-search")
                        .icon(Icon::new(IconName::Search))
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm),
                );

                tb.into_any_element()
            }

            // Utility Panel: title + icon row
            WindowVariant::UtilityPanel => div()
                .flex()
                .flex_col()
                .items_center()
                .px(theme.spacing_md)
                .py(theme.spacing_sm)
                .border_b_1()
                .border_color(theme.border)
                .child(
                    div()
                        .text_style(TextStyle::Headline, theme)
                        .text_color(theme.text)
                        .pb(theme.spacing_xs)
                        .child("Window Title"),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(theme.spacing_md)
                        .child(
                            Button::new("u-palette")
                                .icon(Icon::new(IconName::DevPalette))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm),
                        )
                        .child(
                            Button::new("u-settings")
                                .icon(Icon::new(IconName::Settings))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm),
                        )
                        .child(
                            Button::new("u-grid")
                                .icon(Icon::new(IconName::DevSplitView))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm),
                        )
                        .child(
                            Button::new("u-image")
                                .icon(Icon::new(IconName::Image))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm),
                        )
                        .child(
                            Button::new("u-edit")
                                .icon(Icon::new(IconName::Pencil))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::IconSm),
                        ),
                )
                .into_any_element(),
        }
    }
}

impl Render for WindowLayouts {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>();
        let variant = self.current_variant();
        let is_utility = variant == WindowVariant::UtilityPanel;

        // Variant switcher at the very top
        let switcher = div()
            .flex()
            .justify_center()
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .bg(theme.surface)
            .border_b_1()
            .border_color(theme.border)
            .child(
                SegmentedControl::new("variant-picker")
                    .items(
                        WindowVariant::ALL
                            .iter()
                            .map(|v| SegmentItem::new(v.label()))
                            .collect(),
                    )
                    .selected(self.variant_index)
                    .on_change({
                        let entity = cx.entity().downgrade();
                        move |idx, _window, cx| {
                            if let Some(this) = entity.upgrade() {
                                this.update(cx, |this, cx| {
                                    this.variant_index = idx;
                                    cx.notify();
                                });
                            }
                        }
                    }),
            );

        // For Utility Panel variant, no sidebar
        if is_utility {
            let chrome = self.render_chrome(variant, theme);
            let main_content = div().flex_1().bg(theme.background);
            return div()
                .size_full()
                .flex()
                .flex_col()
                .bg(theme.background)
                .child(switcher)
                .child(chrome)
                .child(main_content)
                .into_any_element();
        }

        // Capture values from theme before render_sidebar borrows cx mutably.
        let chrome = self.render_chrome(variant, theme);
        let bg = theme.background;
        let sidebar = self.render_sidebar(cx);
        let main_content = div().flex_1().bg(bg);

        let right_area = div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(bg)
            .child(chrome)
            .child(main_content);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(bg)
            .child(switcher)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .child(sidebar)
                    .child(right_area),
            )
            .into_any_element()
    }
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            let theme = TahoeTheme::liquid_glass_light();
            let window_bg = theme.glass.window_background;
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            let bounds = Bounds::centered(None, size(px(1100.0), px(700.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: window_bg,
                    ..Default::default()
                },
                |_, cx| cx.new(WindowLayouts::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
