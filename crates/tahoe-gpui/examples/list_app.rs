//! Example: a Task list app with sidebar nav and an active-issues table.
//!
//! Demonstrates Sidebar, SearchField, Button, Avatar, Badge, and table-style
//! row layouts. Mirrors the macOS 26 "List" screen pattern from the Apple
//! Tahoe UI Kit.

use gpui::prelude::*;
use gpui::{
    App, Bounds, ElementId, FontWeight, Hsla, Pixels, SharedString, Window,
    WindowBackgroundAppearance, WindowBounds, WindowOptions, div, hsla, px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::content::avatar::Avatar;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::sidebar::{Sidebar, SidebarItem};
use tahoe_gpui::foundations::icons::{EmbeddedIconAssets, Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

// ── Static data ──────────────────────────────────────────────────────────────

// Sidebar menu labels. Selection state is owned by `ListApp::selected_menu`,
// not the static data, so clicking a row updates the highlight.
const MENU_LABELS: &[&str] = &[
    "Active issues",
    "Menu item",
    "Menu item",
    "Menu item",
    "Menu item",
];

#[derive(Clone, Copy)]
enum Priority {
    High,
    Medium,
    Low,
}

impl Priority {
    fn label(&self) -> &'static str {
        match self {
            Priority::High => "High",
            Priority::Medium => "Medium",
            Priority::Low => "Low",
        }
    }

    /// Map priority severity onto the theme's semantic feedback colors so
    /// the demo follows light/dark mode and brand-color overrides instead of
    /// being pinned to hardcoded HSL values.
    fn color(&self, theme: &TahoeTheme) -> Hsla {
        match self {
            Priority::High => theme.error,
            Priority::Medium => theme.warning,
            Priority::Low => theme.info,
        }
    }
}

struct Issue {
    id: &'static str,
    title: &'static str,
    project: &'static str,
    priority: Priority,
    date: &'static str,
    owner_initials: &'static str,
    owner_color: (f32, f32, f32),
}

const ISSUES: &[Issue] = &[
    Issue {
        id: "FIG-123",
        title: "Task 1",
        project: "Project 1",
        priority: Priority::High,
        date: "Dec 5",
        owner_initials: "AL",
        owner_color: (0.05, 0.55, 0.55),
    },
    Issue {
        id: "FIG-122",
        title: "Task 2",
        project: "Acme GTM",
        priority: Priority::Low,
        date: "Dec 5",
        owner_initials: "BM",
        owner_color: (0.55, 0.55, 0.55),
    },
    Issue {
        id: "FIG-121",
        title: "Write blog post for demo day",
        project: "Acme GTM",
        priority: Priority::High,
        date: "Dec 5",
        owner_initials: "CN",
        owner_color: (0.10, 0.55, 0.55),
    },
    Issue {
        id: "FIG-120",
        title: "Publish blog page",
        project: "Website launch",
        priority: Priority::Low,
        date: "Dec 5",
        owner_initials: "DO",
        owner_color: (0.62, 0.55, 0.55),
    },
    Issue {
        id: "FIG-119",
        title: "Add gradients to design system",
        project: "Design backlog",
        priority: Priority::Medium,
        date: "Dec 5",
        owner_initials: "EP",
        owner_color: (0.85, 0.55, 0.55),
    },
    Issue {
        id: "FIG-118",
        title: "Responsive behavior doesn't work on Android",
        project: "Bug fixes",
        priority: Priority::Medium,
        date: "Dec 5",
        owner_initials: "FQ",
        owner_color: (0.30, 0.55, 0.55),
    },
    Issue {
        id: "FIG-117",
        title: "Confirmation states not rendering properly",
        project: "Bug fixes",
        priority: Priority::Medium,
        date: "Dec 5",
        owner_initials: "GR",
        owner_color: (0.45, 0.55, 0.55),
    },
    Issue {
        id: "FIG-116",
        title: "Text wrapping is awkward on older iPhones",
        project: "Bug fixes",
        priority: Priority::Low,
        date: "Dec 5",
        owner_initials: "HS",
        owner_color: (0.25, 0.50, 0.55),
    },
    Issue {
        id: "FIG-115",
        title: "Revise copy on About page",
        project: "Website launch",
        priority: Priority::Low,
        date: "Dec 5",
        owner_initials: "IT",
        owner_color: (0.12, 0.55, 0.55),
    },
    Issue {
        id: "FIG-114",
        title: "Publish HackerNews post",
        project: "Acme GTM",
        priority: Priority::Low,
        date: "Dec 5",
        owner_initials: "JU",
        owner_color: (0.50, 0.55, 0.55),
    },
    Issue {
        id: "FIG-113",
        title: "Review image licensing for header section images",
        project: "Website launch",
        priority: Priority::High,
        date: "Dec 5",
        owner_initials: "KV",
        owner_color: (0.85, 0.45, 0.55),
    },
    Issue {
        id: "FIG-112",
        title: "Accessibility focused state for input fields",
        project: "Design backlog",
        priority: Priority::High,
        date: "Dec 5",
        owner_initials: "LW",
        owner_color: (0.70, 0.55, 0.55),
    },
];

// ── App state ────────────────────────────────────────────────────────────────

struct ListApp {
    selected_menu: usize,
}

impl ListApp {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self { selected_menu: 0 }
    }
}

#[derive(Clone, Copy)]
struct RowTokens {
    text: Hsla,
    text_muted: Hsla,
    border: Hsla,
    spacing_sm: Pixels,
    spacing_md: Pixels,
    radius_sm: Pixels,
}

impl RowTokens {
    fn from(theme: &TahoeTheme) -> Self {
        Self {
            text: theme.text,
            text_muted: theme.text_muted,
            border: theme.border,
            spacing_sm: theme.spacing_sm,
            spacing_md: theme.spacing_md,
            radius_sm: theme.radius_sm,
        }
    }
}

fn priority_pill(p: Priority, t: RowTokens, theme: &TahoeTheme) -> impl IntoElement + use<> {
    let color = p.color(theme);
    div()
        .px(t.spacing_sm)
        .py(px(2.0))
        .rounded(t.radius_sm)
        .border_1()
        .border_color(t.border)
        .text_size(px(11.0))
        .text_color(color)
        .font_weight(FontWeight::MEDIUM)
        .child(p.label())
}

fn project_pill(name: &'static str, t: RowTokens) -> impl IntoElement + use<> {
    div()
        .px(t.spacing_sm)
        .py(px(2.0))
        .rounded(t.radius_sm)
        .border_1()
        .border_color(t.border)
        .text_size(px(11.0))
        .text_color(t.text)
        .child(name)
}

fn issue_row(issue: &Issue, t: RowTokens, theme: &TahoeTheme) -> impl IntoElement + use<> {
    let owner_color = hsla(
        issue.owner_color.0,
        issue.owner_color.1,
        issue.owner_color.2,
        1.0,
    );
    div()
        .flex()
        .items_center()
        .gap(t.spacing_md)
        .px(t.spacing_md)
        .py(t.spacing_sm)
        .border_b_1()
        .border_color(t.border)
        // Task ID column (~80px)
        .child(
            div()
                .w(px(80.0))
                .text_size(px(12.0))
                .text_color(t.text_muted)
                .font_weight(FontWeight::MEDIUM)
                .child(SharedString::from(issue.id)),
        )
        // Title column (flex 1)
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .truncate()
                .text_size(px(13.0))
                .text_color(t.text)
                .child(SharedString::from(issue.title)),
        )
        // Project pill (~140px)
        .child(div().w(px(140.0)).child(project_pill(issue.project, t)))
        // Priority pill (~90px)
        .child(
            div()
                .w(px(90.0))
                .child(priority_pill(issue.priority, t, theme)),
        )
        // Date (~70px)
        .child(
            div()
                .w(px(70.0))
                .text_size(px(12.0))
                .text_color(t.text_muted)
                .child(SharedString::from(issue.date)),
        )
        // Owner avatar (~36px)
        .child(
            Avatar::new(issue.owner_initials)
                .bg(owner_color)
                .size(px(28.0)),
        )
        // ... menu (right)
        .child(
            div()
                .text_size(px(14.0))
                .text_color(t.text_muted)
                .child("..."),
        )
}

impl Render for ListApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let row_tokens = RowTokens::from(&theme);
        let theme = &theme;
        let selected_menu = self.selected_menu;

        // ── Sidebar ──────────────────────────────────────────────────────
        let mut sidebar_content = div().flex().flex_col().size_full();

        // App title
        sidebar_content = sidebar_content.child(
            div()
                .px(theme.spacing_md)
                .pt(theme.spacing_md)
                .pb(theme.spacing_md)
                .text_style_emphasized(TextStyle::Title2, theme)
                .text_color(theme.text)
                .child("Task app"),
        );

        // Menu items
        for (idx, label) in MENU_LABELS.iter().enumerate() {
            sidebar_content = sidebar_content.child(
                SidebarItem::new(
                    ElementId::NamedInteger("menu".into(), idx as u64),
                    SharedString::from(*label),
                )
                .selected(idx == selected_menu)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.selected_menu = idx;
                    cx.notify();
                })),
            );
        }

        let sidebar = Sidebar::new("sidebar")
            .width(px(240.0))
            .child(sidebar_content);

        // ── Main content ─────────────────────────────────────────────────
        // Page title
        let page_title = div()
            .px(theme.spacing_lg)
            .pt(theme.spacing_lg)
            .pb(theme.spacing_md)
            .text_style_emphasized(TextStyle::Title2, theme)
            .text_color(theme.text)
            .child("Active issues");

        // Toolbar
        let toolbar = div()
            .px(theme.spacing_lg)
            .pb(theme.spacing_md)
            .flex()
            .items_center()
            .gap(theme.spacing_md)
            // Search field (placeholder)
            .child(
                div()
                    .flex_1()
                    .h(px(32.0))
                    .px(theme.spacing_md)
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .bg(theme.background)
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md)
                    .child(
                        Icon::new(IconName::Search)
                            .size(px(14.0))
                            .color(theme.text_muted),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .text_color(theme.text_muted)
                            .child("Search tickets..."),
                    ),
            )
            // Filter button
            .child(
                Button::new("filter")
                    .label("Filter")
                    .icon(Icon::new(IconName::ListTodo).size(px(14.0)))
                    .variant(ButtonVariant::Outline)
                    .size(ButtonSize::Sm),
            )
            // View toggles (list/grid/calendar)
            .child(
                div()
                    .flex()
                    .gap(px(2.0))
                    .border_1()
                    .border_color(theme.border)
                    .rounded(theme.radius_md)
                    .p(px(2.0))
                    .child(
                        div()
                            .px(theme.spacing_sm)
                            .py(px(4.0))
                            .rounded(theme.radius_sm)
                            .bg(theme.hover)
                            .child(Icon::new(IconName::ListTodo).size(px(14.0))),
                    )
                    .child(
                        div()
                            .px(theme.spacing_sm)
                            .py(px(4.0))
                            .child(Icon::new(IconName::File).size(px(14.0))),
                    )
                    .child(
                        div()
                            .px(theme.spacing_sm)
                            .py(px(4.0))
                            .child(Icon::new(IconName::Square).size(px(14.0))),
                    ),
            );

        // Table header
        let table_header = div()
            .flex()
            .items_center()
            .gap(theme.spacing_md)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .mx(theme.spacing_lg)
            .border_b_1()
            .border_color(theme.border)
            .text_size(px(12.0))
            .text_color(theme.text_muted)
            .font_weight(FontWeight::MEDIUM)
            .child(div().w(px(80.0)).child("Task"))
            .child(div().flex_1().child("Title"))
            .child(div().w(px(140.0)).child("Project"))
            .child(div().w(px(90.0)).child("Priority"))
            .child(div().w(px(70.0)).child("Date"))
            .child(div().w(px(28.0)).child("Owner"))
            .child(div().w(px(20.0)));

        // Table rows
        let mut table_rows = div()
            .id("issues-list")
            .mx(theme.spacing_lg)
            .flex_1()
            .flex()
            .flex_col()
            .overflow_y_scroll();
        for issue in ISSUES {
            table_rows = table_rows.child(issue_row(issue, row_tokens, theme));
        }

        let main = div()
            .flex_1()
            .flex()
            .flex_col()
            .bg(theme.background)
            .child(page_title)
            .child(toolbar)
            .child(table_header)
            .child(table_rows);

        // ── Root layout ──────────────────────────────────────────────────
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
            let theme = TahoeTheme::liquid_glass_light();
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            // Match Figma frame size (1440 x 960)
            let bounds = Bounds::centered(None, size(px(1440.0), px(960.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| cx.new(ListApp::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
