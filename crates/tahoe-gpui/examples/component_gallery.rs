//! Single-binary primitive gallery for the tahoe-gpui crate.
//!
//! All primitive demos live inside one container app so we only need to
//! grant computer-use permission once. Demos register themselves with
//! the gallery's sidebar; clicking a sidebar entry switches the right-
//! hand pane. New primitive demos slot in by adding a `Demo::new(...)`
//! to the list in `DEMOS`.
//!
//! Naming, layout, and visual treatment are all aimed at matching the
//! macOS 26 (Community) UI Kit pages component-by-component.

mod gallery {
    pub mod action_sheets;
    pub mod activity_indicators;
    pub mod activity_rings;
    pub mod alerts;
    pub mod avatars;
    pub mod badges;
    pub mod boxes;
    pub mod button_groups;
    pub mod buttons;
    pub mod collections;
    pub mod color_wells;
    pub mod colors;
    pub mod combo_boxes;
    pub mod context_menus;
    pub mod copy_buttons;
    pub mod date_pickers;
    pub mod dialogs;
    pub mod digit_entries;
    pub mod disclosure_controls;
    pub mod disclosure_groups;
    pub mod flex_headers;
    pub mod forms;
    pub mod gauges;
    pub mod hover_cards;
    pub mod image_wells;
    pub mod labels;
    pub mod liquid_glass;
    pub mod lists_and_tables;
    pub mod materials;
    pub mod menu_bar_and_dock;
    pub mod menus;
    pub mod modals;
    pub mod navigation_bars;
    pub mod notifications;
    pub mod outline_views;
    pub mod page_controls;
    pub mod panels;
    pub mod path_controls;
    pub mod pickers;
    pub mod pointers;
    pub mod popovers;
    pub mod popup_buttons;
    pub mod progress_indicators;
    pub mod pulldown_buttons;
    #[cfg(target_os = "macos")]
    pub mod rating_indicators;
    pub mod scroll_views;
    pub mod scrollbar;
    pub mod search_fields;
    pub mod segmented_controls;
    pub mod separators;
    pub mod sheets;
    pub mod shimmers;
    pub mod sidebars;
    pub mod sliders_and_dials;
    pub mod split_views;
    pub mod steppers;
    pub mod tab_bars;
    pub mod text_fields;
    pub mod text_views;
    pub mod time_pickers;
    pub mod toggles;
    pub mod token_fields;
    pub mod toolbars_and_titlebars;
    pub mod tooltips;
    pub mod typography;
    pub mod welcome;
    pub mod windows;
}

use std::collections::HashSet;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Bounds, ElementId, Entity, FocusHandle, Hsla, SharedString, Window,
    WindowBounds, WindowOptions, div, px, size,
};

use gpui_platform::application;
use tahoe_gpui::components::menus_and_actions::context_menu::{
    ContextMenu, ContextMenuEntry, ContextMenuItem, ContextMenuItemStyle,
};
use tahoe_gpui::components::presentation::hover_card::HoverCard;
use tahoe_gpui::components::selection_and_input::date_picker::SimpleDate;
use tahoe_gpui::foundations::accessibility::AccessibilityMode;

use tahoe_gpui::components::layout_and_organization::split_view::SplitView;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::sidebar::{Sidebar, SidebarItem};
use tahoe_gpui::components::navigation_and_search::token_field::{TokenField, TokenItem};
use tahoe_gpui::components::selection_and_input::slider::Slider;
use tahoe_gpui::components::selection_and_input::text_field::TextField;
use tahoe_gpui::foundations::icons::EmbeddedIconAssets;
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

// ── Demo registry ────────────────────────────────────────────────────────────

/// One entry in the gallery sidebar.
struct Demo {
    /// Section label (alphabetical key in the macOS 26 UI Kit).
    label: &'static str,
    /// Render the demo body for this primitive.
    render: fn(&mut ComponentGallery, &mut Window, &mut Context<ComponentGallery>) -> AnyElement,
}

const DEMOS: &[Demo] = &[
    Demo {
        label: "Welcome",
        render: gallery::welcome::render,
    },
    // ── Foundations ────────────────────────────────────────────────
    Demo {
        label: "Colors",
        render: gallery::colors::render,
    },
    Demo {
        label: "Liquid Glass",
        render: gallery::liquid_glass::render,
    },
    Demo {
        label: "Materials",
        render: gallery::materials::render,
    },
    Demo {
        label: "Typography",
        render: gallery::typography::render,
    },
    // ── Components (alphabetical) ──────────────────────────────────
    Demo {
        label: "Action Sheets",
        render: gallery::action_sheets::render,
    },
    Demo {
        label: "Activity Indicators",
        render: gallery::activity_indicators::render,
    },
    Demo {
        label: "Activity Rings",
        render: gallery::activity_rings::render,
    },
    Demo {
        label: "Alerts",
        render: gallery::alerts::render,
    },
    Demo {
        label: "Avatars",
        render: gallery::avatars::render,
    },
    Demo {
        label: "Badges",
        render: gallery::badges::render,
    },
    Demo {
        label: "Boxes",
        render: gallery::boxes::render,
    },
    Demo {
        label: "Button Groups",
        render: gallery::button_groups::render,
    },
    Demo {
        label: "Buttons",
        render: gallery::buttons::render,
    },
    Demo {
        label: "Collections",
        render: gallery::collections::render,
    },
    Demo {
        label: "Color Wells",
        render: gallery::color_wells::render,
    },
    Demo {
        label: "Combo Boxes",
        render: gallery::combo_boxes::render,
    },
    Demo {
        label: "Context Menus",
        render: gallery::context_menus::render,
    },
    Demo {
        label: "Copy Buttons",
        render: gallery::copy_buttons::render,
    },
    Demo {
        label: "Date Pickers",
        render: gallery::date_pickers::render,
    },
    Demo {
        label: "Dialogs",
        render: gallery::dialogs::render,
    },
    Demo {
        label: "Digit Entries",
        render: gallery::digit_entries::render,
    },
    Demo {
        label: "Disclosure Controls",
        render: gallery::disclosure_controls::render,
    },
    Demo {
        label: "Disclosure Groups",
        render: gallery::disclosure_groups::render,
    },
    Demo {
        label: "Flex Headers",
        render: gallery::flex_headers::render,
    },
    Demo {
        label: "Forms",
        render: gallery::forms::render,
    },
    Demo {
        label: "Gauges",
        render: gallery::gauges::render,
    },
    Demo {
        label: "Hover Cards",
        render: gallery::hover_cards::render,
    },
    Demo {
        label: "Image Wells",
        render: gallery::image_wells::render,
    },
    Demo {
        label: "Labels",
        render: gallery::labels::render,
    },
    Demo {
        label: "Lists and Tables",
        render: gallery::lists_and_tables::render,
    },
    Demo {
        label: "Menu Bar and Dock",
        render: gallery::menu_bar_and_dock::render,
    },
    Demo {
        label: "Menus",
        render: gallery::menus::render,
    },
    Demo {
        label: "Modals",
        render: gallery::modals::render,
    },
    Demo {
        label: "Navigation Bars",
        render: gallery::navigation_bars::render,
    },
    Demo {
        label: "Notifications",
        render: gallery::notifications::render,
    },
    Demo {
        label: "Outline Views",
        render: gallery::outline_views::render,
    },
    Demo {
        label: "Page Controls",
        render: gallery::page_controls::render,
    },
    Demo {
        label: "Panels",
        render: gallery::panels::render,
    },
    Demo {
        label: "Path Controls",
        render: gallery::path_controls::render,
    },
    Demo {
        label: "Pickers",
        render: gallery::pickers::render,
    },
    Demo {
        label: "Pointers",
        render: gallery::pointers::render,
    },
    Demo {
        label: "Pop-up Buttons",
        render: gallery::popup_buttons::render,
    },
    Demo {
        label: "Popovers",
        render: gallery::popovers::render,
    },
    Demo {
        label: "Progress Indicators",
        render: gallery::progress_indicators::render,
    },
    Demo {
        label: "Pull-down Buttons",
        render: gallery::pulldown_buttons::render,
    },
    #[cfg(target_os = "macos")]
    Demo {
        label: "Rating Indicators",
        render: gallery::rating_indicators::render,
    },
    Demo {
        label: "Scroll Views",
        render: gallery::scroll_views::render,
    },
    Demo {
        label: "Scrollbar",
        render: gallery::scrollbar::render,
    },
    Demo {
        label: "Search Fields",
        render: gallery::search_fields::render,
    },
    Demo {
        label: "Segmented Controls",
        render: gallery::segmented_controls::render,
    },
    Demo {
        label: "Separators",
        render: gallery::separators::render,
    },
    Demo {
        label: "Sheets",
        render: gallery::sheets::render,
    },
    Demo {
        label: "Shimmers",
        render: gallery::shimmers::render,
    },
    Demo {
        label: "Sidebars",
        render: gallery::sidebars::render,
    },
    Demo {
        label: "Sliders and Dials",
        render: gallery::sliders_and_dials::render,
    },
    Demo {
        label: "Split Views",
        render: gallery::split_views::render,
    },
    Demo {
        label: "Steppers",
        render: gallery::steppers::render,
    },
    Demo {
        label: "Tab Bars",
        render: gallery::tab_bars::render,
    },
    Demo {
        label: "Text Fields",
        render: gallery::text_fields::render,
    },
    Demo {
        label: "Text Views",
        render: gallery::text_views::render,
    },
    Demo {
        label: "Time Pickers",
        render: gallery::time_pickers::render,
    },
    Demo {
        label: "Toggles",
        render: gallery::toggles::render,
    },
    Demo {
        label: "Token Fields",
        render: gallery::token_fields::render,
    },
    Demo {
        label: "Toolbars and Titlebars",
        render: gallery::toolbars_and_titlebars::render,
    },
    Demo {
        label: "Tooltips",
        render: gallery::tooltips::render,
    },
    Demo {
        label: "Windows",
        render: gallery::windows::render,
    },
];

// ── App state ────────────────────────────────────────────────────────────────

pub struct ComponentGallery {
    pub selected_demo: usize,
    /// Whether the gallery is currently in dark mode.
    pub dark_mode: bool,
    /// State shared with demos (e.g. which alert is open).
    pub alerts_state: gallery::alerts::AlertsState,
    /// Entity-based input controls created once at gallery construction so we
    /// can render them inside stateless demo functions.
    pub slider_a: Entity<Slider>,
    pub slider_b: Entity<Slider>,
    pub text_input_empty: Entity<TextField>,
    pub text_input_filled: Entity<TextField>,
    pub split_view: Entity<SplitView>,

    // ── Interactive state for gallery pages ──────────────────────────────
    pub toggle_on: bool,
    pub stepper_value: f64,
    pub picker_selected: Option<SharedString>,
    pub picker_open: bool,
    pub date_picker_date: Option<SimpleDate>,
    pub date_picker_open: bool,
    pub time_hour: u8,
    pub time_minute: u8,
    pub time_picker_open: bool,
    pub combo_value: SharedString,
    pub combo_open: bool,
    pub color_well_color: Hsla,
    pub color_well_open: bool,
    pub segmented_index: usize,
    pub search_value: SharedString,
    pub search_focus: FocusHandle,
    pub tab_active: SharedString,
    pub digit_interactive: Entity<tahoe_gpui::components::selection_and_input::DigitEntry>,
    pub digit_static: Entity<tahoe_gpui::components::selection_and_input::DigitEntry>,
    pub digit_last_value: SharedString,
    pub form_notifications: bool,
    pub form_sounds: bool,
    pub form_theme: SharedString,
    pub pulldown_open: bool,
    pub popup_open: bool,
    pub popup_selected: SharedString,
    pub pulldown2_open: bool,
    pub popup2_open: bool,
    pub popup2_selected: SharedString,
    pub popover_open: Option<usize>,
    pub outline_expanded: HashSet<String>,
    pub rating_value: f32,
    pub table_selected: Option<usize>,
    pub disclosure_open: bool,
    pub date_viewing_year: i32,
    pub date_viewing_month: u8,
    pub context_menu: Entity<tahoe_gpui::components::menus_and_actions::context_menu::ContextMenu>,
    pub context_menu_status: SharedString,
    pub hover_card: Entity<tahoe_gpui::components::presentation::hover_card::HoverCard>,
    pub page_current: usize,
    pub token_field: Entity<TokenField>,
    /// Open-state booleans for the overlay-style demo pages so the live
    /// `Sheet`, `Modal`, `Dialog`, and `Panel` components can be exercised
    /// inside the gallery rather than approximated with static mockups
    /// (issue #156 F-05/F-14). The `Menus` page reuses the existing
    /// `context_menu` Entity, so its open state lives there.
    pub sheet_open: bool,
    pub modal_open: bool,
    pub dialog_open: bool,
    pub panel_open: bool,
}

impl ComponentGallery {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            selected_demo: 0,
            dark_mode: false,
            alerts_state: gallery::alerts::AlertsState::default(),
            slider_a: cx.new(Slider::new),
            slider_b: cx.new(|cx| {
                let mut s = Slider::new(cx);
                s.set_value(0.65, cx);
                s
            }),
            text_input_empty: cx.new(|cx| {
                let mut input = TextField::new(cx);
                input.set_placeholder("Enter your name");
                input
            }),
            text_input_filled: cx.new(|cx| TextField::new_with_text(cx, "Søren Magnus Olesen")),
            split_view: cx.new(|cx| {
                let mut sv = SplitView::new(cx);
                sv.set_primary(
                    |_window, _cx| {
                        div()
                            .p(px(16.0))
                            .child(div().child("Primary pane"))
                            .into_any_element()
                    },
                    cx,
                );
                sv.set_secondary(
                    |_window, _cx| {
                        div()
                            .p(px(16.0))
                            .child(div().child("Secondary pane"))
                            .into_any_element()
                    },
                    cx,
                );
                sv
            }),
            // Interactive state for gallery pages
            toggle_on: false,
            stepper_value: 5.0,
            picker_selected: Some(SharedString::from("cherry")),
            picker_open: false,
            date_picker_date: Some(SimpleDate::new(2026, 4, 12)),
            date_picker_open: false,
            time_hour: 9,
            time_minute: 30,
            time_picker_open: false,
            combo_value: SharedString::default(),
            combo_open: false,
            color_well_color: Hsla {
                h: 0.0,
                s: 0.85,
                l: 0.55,
                a: 1.0,
            },
            color_well_open: false,
            segmented_index: 0,
            search_value: SharedString::default(),
            search_focus: cx.focus_handle(),
            tab_active: SharedString::from("general"),
            digit_interactive: cx.new(|cx| {
                let mut entry = tahoe_gpui::components::selection_and_input::DigitEntry::new(cx);
                entry.set_id("de-interactive");
                entry
            }),
            digit_static: cx.new(|cx| {
                let mut entry = tahoe_gpui::components::selection_and_input::DigitEntry::new(cx);
                entry.set_id("de-4");
                entry.set_length(4);
                entry.set_text("1234", cx);
                entry
            }),
            digit_last_value: SharedString::default(),
            form_notifications: true,
            form_sounds: false,
            form_theme: SharedString::from("auto"),
            pulldown_open: false,
            popup_open: false,
            popup_selected: SharedString::from("dark"),
            pulldown2_open: false,
            popup2_open: false,
            popup2_selected: SharedString::from("dark"),
            popover_open: None,
            outline_expanded: {
                let mut set = HashSet::new();
                set.insert("src".to_string());
                set.insert("components".to_string());
                set
            },
            rating_value: 3.0,
            table_selected: Some(2),
            disclosure_open: true,
            date_viewing_year: 2026,
            date_viewing_month: 4,
            context_menu: cx.new(|cx| {
                let mut menu = ContextMenu::new(cx);
                menu.set_items(vec![
                    ContextMenuEntry::Item(ContextMenuItem::new("Cut").shortcut("\u{2318}X")),
                    ContextMenuEntry::Item(ContextMenuItem::new("Copy").shortcut("\u{2318}C")),
                    ContextMenuEntry::Item(ContextMenuItem::new("Paste").shortcut("\u{2318}V")),
                    ContextMenuEntry::Separator,
                    ContextMenuEntry::Item(
                        ContextMenuItem::new("Delete").style(ContextMenuItemStyle::Destructive),
                    ),
                ]);
                menu
            }),
            context_menu_status: SharedString::from("Right-click the target area above"),
            hover_card: cx.new(|_cx| HoverCard::new("gallery-hover-card")),
            page_current: 0,
            sheet_open: false,
            modal_open: false,
            dialog_open: false,
            panel_open: false,
            token_field: {
                let entity = cx.new(|cx| {
                    let mut tf = TokenField::new(cx);
                    tf.set_tokens(
                        vec![
                            TokenItem::new("rust", "Rust"),
                            TokenItem::new("swift", "Swift"),
                            TokenItem::new("ts", "TypeScript"),
                            TokenItem::fixed("required", "Required"),
                        ],
                        cx,
                    );
                    tf.set_suggestions(
                        vec!["Go".into(), "Kotlin".into(), "Python".into(), "Zig".into()],
                        cx,
                    );
                    tf
                });
                let weak = entity.downgrade();
                entity.update(cx, |tf, _cx| {
                    // The TokenField fires `on_add` / `on_remove` from
                    // *inside* its own `cx.update(...)` cycle, so we cannot
                    // mutate the same entity directly — that triggers
                    // GPUI's "already being updated" panic. `cx.defer(...)`
                    // schedules the mutation for after the current frame's
                    // entity-update lock is released.
                    let weak_for_add = weak.clone();
                    tf.set_on_add(move |label, _window, cx| {
                        let weak = weak_for_add.clone();
                        let label = label.to_string();
                        cx.defer(move |cx| {
                            if let Some(this) = weak.upgrade() {
                                this.update(cx, |tf, cx| {
                                    let id = label.to_lowercase().replace(' ', "-");
                                    tf.add_token(label, id, cx);
                                });
                            }
                        });
                    });
                    let weak_for_remove = weak.clone();
                    tf.set_on_remove(move |id, _window, cx| {
                        let weak = weak_for_remove.clone();
                        let id = id.to_string();
                        cx.defer(move |cx| {
                            if let Some(this) = weak.upgrade() {
                                this.update(cx, |tf, cx| tf.remove_token(&id, cx));
                            }
                        });
                    });
                });
                entity
            },
        }
    }

    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.dark_mode = !self.dark_mode;
        // Per HIG: Liquid Glass is always on in macOS Tahoe (26).
        // Toggle switches between dark and light glass variants while
        // preserving the accessibility-mode flags the user has set.
        let prior_mode = cx.global::<TahoeTheme>().accessibility_mode;
        let mut theme = if self.dark_mode {
            TahoeTheme::liquid_glass()
        } else {
            TahoeTheme::liquid_glass_light()
        };
        theme.accessibility_mode = prior_mode;
        // `apply` updates the global and calls `cx.refresh_windows()`, which
        // repaints every window. No additional `cx.notify()` needed.
        theme.apply(cx);
    }

    /// Flip a single [`AccessibilityMode`] flag on the active theme.
    ///
    /// HIG Accessibility (macOS 26 Tahoe): the four user-facing toggles
    /// (Reduce Transparency, Increase Contrast, Reduce Motion, Bold Text)
    /// each map onto an `AccessibilityMode` bit. Surfacing them in the
    /// gallery makes `effective_duration()`, `glass_or_surface()`, and the
    /// high-contrast border path observable to adopters.
    fn toggle_accessibility(&mut self, flag: AccessibilityMode, cx: &mut Context<Self>) {
        let mut theme = cx.global::<TahoeTheme>().clone();
        theme.accessibility_mode = theme.accessibility_mode.toggled(flag);
        theme.apply(cx);
    }
}

#[derive(Clone, Copy)]
pub struct GalleryTokens {
    pub text: Hsla,
    pub text_muted: Hsla,
    pub border: Hsla,
    pub hover: Hsla,
    pub spacing_xs: gpui::Pixels,
    pub spacing_sm: gpui::Pixels,
    pub spacing_md: gpui::Pixels,
    pub spacing_lg: gpui::Pixels,
    pub spacing_xl: gpui::Pixels,
    pub radius_md: gpui::Pixels,
}

impl GalleryTokens {
    pub fn from(theme: &TahoeTheme) -> Self {
        Self {
            text: theme.text,
            text_muted: theme.text_muted,
            border: theme.border,
            hover: theme.hover,
            spacing_xs: theme.spacing_xs,
            spacing_sm: theme.spacing_sm,
            spacing_md: theme.spacing_md,
            spacing_lg: theme.spacing_lg,
            spacing_xl: theme.spacing_xl,
            radius_md: theme.radius_md,
        }
    }
}

impl Render for ComponentGallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let tokens = GalleryTokens::from(&theme);
        let theme_ref = &theme;
        let selected = self.selected_demo;

        // ── Sidebar header (title + theme + accessibility toggles) ────
        //
        // Per HIG Accessibility (macOS 26): Reduce Transparency,
        // Increase Contrast, Reduce Motion, and Bold Text are user-facing
        // modes that adjust the rendered chrome. Surface them here so all
        // 60+ demo pages can be inspected with each flag enabled — that
        // makes `effective_duration()`, `glass.accessible_bg()`, and the
        // high-contrast border path observable in one place.
        let dark_mode = self.dark_mode;
        let access_mode = theme_ref.accessibility_mode;

        let access_chip =
            |id: &'static str, label: &'static str, flag: AccessibilityMode, active: bool| {
                Button::new(id)
                    .label(label)
                    .variant(if active {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .size(ButtonSize::Small)
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_accessibility(flag, cx);
                    }))
            };

        let header = div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .gap(tokens.spacing_sm)
            .px(tokens.spacing_md)
            .pt(tokens.spacing_md)
            .pb(tokens.spacing_sm)
            .border_b_1()
            .border_color(tokens.border)
            .child(
                div()
                    .text_style_emphasized(TextStyle::Title3, theme_ref)
                    .text_color(tokens.text)
                    .child("Primitives"),
            )
            .child(
                Button::new("toggle-theme")
                    .label(if dark_mode {
                        "\u{2600} Toggle to Light"
                    } else {
                        "\u{263E} Toggle to Dark"
                    })
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Small)
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.toggle_theme(cx);
                    })),
            )
            .child(
                div()
                    .pt(tokens.spacing_xs)
                    .text_style(TextStyle::Caption2, theme_ref)
                    .text_color(tokens.text_muted)
                    .child("Accessibility"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(tokens.spacing_xs)
                    .child(access_chip(
                        "ax-reduce-transparency",
                        "Reduce Transparency",
                        AccessibilityMode::REDUCE_TRANSPARENCY,
                        access_mode.reduce_transparency(),
                    ))
                    .child(access_chip(
                        "ax-increase-contrast",
                        "Increase Contrast",
                        AccessibilityMode::INCREASE_CONTRAST,
                        access_mode.increase_contrast(),
                    ))
                    .child(access_chip(
                        "ax-reduce-motion",
                        "Reduce Motion",
                        AccessibilityMode::REDUCE_MOTION,
                        access_mode.reduce_motion(),
                    ))
                    .child(access_chip(
                        "ax-bold-text",
                        "Bold Text",
                        AccessibilityMode::BOLD_TEXT,
                        access_mode.bold_text(),
                    )),
            );

        // ── Scrollable demo list ────────────────────────────────────────
        //
        // Uses the SidebarItem primitive so every row gets Enter/Space
        // activation and a 44pt minimum touch target.
        let mut demo_list = div()
            .id("demo-list-scroll")
            .flex()
            .flex_col()
            .flex_1()
            .pt(tokens.spacing_xs)
            .overflow_y_scroll();

        for (idx, demo) in DEMOS.iter().enumerate() {
            let is_selected = idx == selected;
            demo_list = demo_list.child(
                SidebarItem::new(
                    ElementId::NamedInteger("demo".into(), idx as u64),
                    SharedString::from(demo.label),
                )
                .selected(is_selected)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected_demo = idx;
                    cx.notify();
                })),
            );
        }

        let sidebar_content = div()
            .flex()
            .flex_col()
            .size_full()
            .child(header)
            .child(demo_list);

        // HIG Sidebars (macOS 26 Tahoe, June 9 2025): "Extend content
        // beneath the sidebar using `backgroundExtensionEffect()`." GPUI
        // lacks a render-to-texture primitive for the SwiftUI effect, so
        // pass the theme's window root fill as a static extension colour
        // — the glass sidebar floats above it and the content fill reads
        // as a continuation of the main pane.
        let sidebar = Sidebar::new("primitive-gallery-sidebar")
            .width(px(220.0))
            .background_extension(theme_ref.glass.root_bg)
            .child(sidebar_content);

        // ── Right pane: render the selected demo ────────────────────────
        let render_fn = DEMOS[selected].render;
        let body = render_fn(self, window, cx);

        // Use glass root_bg (semi-transparent) when glass theme is active,
        // so the macOS window blur shows through. Otherwise use opaque background.
        let main_bg = theme_ref.glass.root_bg;

        let main = div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(main_bg)
            .overflow_hidden()
            .child(
                div()
                    .id("gallery-content")
                    .size_full()
                    .overflow_y_scroll()
                    .child(body),
            );

        div()
            .size_full()
            .flex()
            .flex_row()
            .child(sidebar)
            .child(main)
    }
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            // Per HIG: Liquid Glass is always on in macOS Tahoe (26).
            // Default to light glass theme with window-level blur enabled.
            let theme = TahoeTheme::liquid_glass_light();
            let window_bg = theme.glass.window_background;
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            // Per HIG #windows (June 9, 2025): "adapt fluidly to
            // different sizes." Open at 1280×840, but expose a 900×640
            // minimum so demos stay legible when users resize the window
            // for split-screen review.
            let bounds = Bounds::centered(None, size(px(1280.0), px(840.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: window_bg,
                    window_min_size: Some(size(px(900.0), px(640.0))),
                    ..Default::default()
                },
                |_, cx| cx.new(ComponentGallery::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
