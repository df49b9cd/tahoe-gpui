//! Example: a Dashboard app with stat cards, charts, and a data table.
//!
//! Demonstrates SegmentedControl, Button, Avatar, plus simple custom
//! line and bar chart primitives. Mirrors the macOS 26 "Dashboard" screen
//! pattern from the Apple Tahoe UI Kit.

use gpui::prelude::*;
use gpui::{
    App, Bounds, FontWeight, Hsla, PathBuilder, Pixels, Pixels as GpuiPixels, SharedString, Window,
    WindowBackgroundAppearance, WindowBounds, WindowOptions, canvas, div, hsla, point, px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::content::avatar::Avatar;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::selection_and_input::segmented_control::{
    SegmentItem, SegmentedControl,
};
use tahoe_gpui::foundations::icons::{EmbeddedIconAssets, Icon, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

// ── Static data ──────────────────────────────────────────────────────────────

struct Stat {
    title: &'static str,
    value: &'static str,
    delta: &'static str,
}

const STATS: &[Stat] = &[
    Stat {
        title: "Title",
        value: "$45,678.90",
        delta: "+20% month over month",
    },
    Stat {
        title: "Title",
        value: "2,405",
        delta: "+33% month over month",
    },
    Stat {
        title: "Title",
        value: "10,353",
        delta: "-8% month over month",
    },
];

struct Person {
    name: &'static str,
    email: &'static str,
    color: (f32, f32, f32),
}

const PEOPLE: &[Person] = &[
    Person {
        name: "Helena",
        email: "email@figmasfakedomain.net",
        color: (0.95, 0.55, 0.50),
    },
    Person {
        name: "Oscar",
        email: "email@figmasfakedomain.net",
        color: (0.58, 0.55, 0.50),
    },
    Person {
        name: "Daniel",
        email: "email@figmasfakedomain.net",
        color: (0.12, 0.55, 0.55),
    },
    Person {
        name: "Daniel Jay Park",
        email: "email@figmasfakedomain.net",
        color: (0.40, 0.55, 0.55),
    },
    Person {
        name: "Mark Rojas",
        email: "email@figmasfakedomain.net",
        color: (0.30, 0.55, 0.50),
    },
];

struct Source {
    name: &'static str,
    sessions: &'static str,
    change: &'static str,
    positive: bool,
}

const SOURCES: &[Source] = &[
    Source {
        name: "website.net",
        sessions: "4321",
        change: "+84%",
        positive: true,
    },
    Source {
        name: "website.net",
        sessions: "4033",
        change: "-8%",
        positive: false,
    },
    Source {
        name: "website.net",
        sessions: "3128",
        change: "+2%",
        positive: true,
    },
    Source {
        name: "website.net",
        sessions: "2104",
        change: "+33%",
        positive: true,
    },
    Source {
        name: "website.net",
        sessions: "2003",
        change: "+30%",
        positive: true,
    },
    Source {
        name: "website.net",
        sessions: "1894",
        change: "+15%",
        positive: true,
    },
    Source {
        name: "website.net",
        sessions: "405",
        change: "-12%",
        positive: false,
    },
];

// Line chart Y values for the trend line (29 days)
const LINE_DATA: [f32; 29] = [
    0.18, 0.22, 0.20, 0.28, 0.30, 0.27, 0.32, 0.36, 0.40, 0.38, 0.42, 0.45, 0.50, 0.52, 0.49, 0.55,
    0.58, 0.62, 0.66, 0.70, 0.68, 0.74, 0.80, 0.83, 0.82, 0.88, 0.92, 0.95, 0.97,
];

// Bar chart Y values for the monthly bars (12 months)
const BAR_DATA: [f32; 12] = [
    0.42, 0.55, 0.50, 0.58, 0.74, 0.92, 0.78, 0.86, 0.65, 0.70, 0.55, 0.18,
];

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

// ── App state ────────────────────────────────────────────────────────────────

struct DashboardApp {
    selected_tab: usize,
}

impl DashboardApp {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self { selected_tab: 0 }
    }
}

#[derive(Clone, Copy)]
struct CardTokens {
    text: Hsla,
    text_muted: Hsla,
    border: Hsla,
    surface: Hsla,
    spacing_lg: Pixels,
    radius_lg: Pixels,
}

impl CardTokens {
    fn from(theme: &TahoeTheme) -> Self {
        Self {
            text: theme.text,
            text_muted: theme.text_muted,
            border: theme.border,
            surface: theme.surface,
            spacing_lg: theme.spacing_lg,
            radius_lg: theme.radius_lg,
        }
    }
}

fn card(t: CardTokens) -> gpui::Div {
    div()
        .bg(t.surface)
        .border_1()
        .border_color(t.border)
        .rounded(t.radius_lg)
}

fn stat_card(stat: &Stat, t: CardTokens) -> impl IntoElement + use<> {
    card(t)
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(t.spacing_lg)
        .child(
            div()
                .text_size(px(13.0))
                .text_color(t.text_muted)
                .font_weight(FontWeight::MEDIUM)
                .child(SharedString::from(stat.title)),
        )
        .child(
            div()
                .text_size(px(28.0))
                .font_weight(FontWeight::BOLD)
                .text_color(t.text)
                .child(SharedString::from(stat.value)),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(t.text_muted)
                .child(SharedString::from(stat.delta)),
        )
}

/// Line chart canvas. Reads its actual painted width/height from `bounds` so
/// the chart fills whatever flex container it lives in — passing a fixed
/// `width` here only sizes the GPUI element; the data points always span the
/// real bounds.
fn line_chart(height: Pixels, line_color: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _state, window, _cx| {
            let w_f = f32::from(bounds.size.width);
            let h_f = f32::from(bounds.size.height);
            let mut pb = PathBuilder::stroke(px(2.0));
            let n = LINE_DATA.len() as f32;
            for (i, v) in LINE_DATA.iter().enumerate() {
                let x = bounds.origin.x + GpuiPixels::from(w_f * (i as f32 / (n - 1.0)));
                let y = bounds.origin.y + GpuiPixels::from(h_f * (1.0 - v));
                if i == 0 {
                    pb.move_to(point(x, y));
                } else {
                    pb.line_to(point(x, y));
                }
            }
            if let Ok(path) = pb.build() {
                window.paint_path(path, line_color);
            }
        },
    )
    .w_full()
    .h(height)
}

/// Bar chart canvas. Reads its actual painted width/height from `bounds` for
/// the same reason as `line_chart`.
fn bar_chart(height: Pixels, bar_color: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _state, window, _cx| {
            let w_f = f32::from(bounds.size.width);
            let h_f = f32::from(bounds.size.height);
            let n = BAR_DATA.len() as f32;
            let bar_w_total = w_f / n;
            let bar_w = bar_w_total * 0.65;
            let pad = (bar_w_total - bar_w) / 2.0;
            for (i, v) in BAR_DATA.iter().enumerate() {
                let bar_h = h_f * v;
                let x = bounds.origin.x + GpuiPixels::from(bar_w_total * i as f32 + pad);
                let y = bounds.origin.y + GpuiPixels::from(h_f - bar_h);
                let mut pb = PathBuilder::fill();
                pb.move_to(point(x, y));
                pb.line_to(point(x + GpuiPixels::from(bar_w), y));
                pb.line_to(point(
                    x + GpuiPixels::from(bar_w),
                    y + GpuiPixels::from(bar_h),
                ));
                pb.line_to(point(x, y + GpuiPixels::from(bar_h)));
                pb.close();
                if let Ok(path) = pb.build() {
                    window.paint_path(path, bar_color);
                }
            }
        },
    )
    .w_full()
    .h(height)
}

impl Render for DashboardApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let card_tokens = CardTokens::from(&theme);
        let theme = &theme;
        let selected_tab = self.selected_tab;

        // ── Header ───────────────────────────────────────────────────────
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .px(theme.spacing_lg)
            .pt(theme.spacing_lg)
            .pb(theme.spacing_md)
            .child(
                div()
                    .text_style_emphasized(TextStyle::Title2, theme)
                    .text_color(theme.text)
                    .child("Dashboard app"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .child(
                        Button::new("more")
                            .label("...")
                            .variant(ButtonVariant::Outline)
                            .size(ButtonSize::Small),
                    )
                    .child(
                        Button::new("share")
                            .label("Share")
                            .variant(ButtonVariant::Filled)
                            .size(ButtonSize::Small),
                    )
                    .child(Avatar::new("U").size(px(28.0))),
            );

        // ── Tab + search row ─────────────────────────────────────────────
        let tab_row = div()
            .flex()
            .items_center()
            .justify_between()
            .px(theme.spacing_lg)
            .pb(theme.spacing_md)
            .child(
                SegmentedControl::new("tabs")
                    .items(vec![
                        SegmentItem::new("Tab"),
                        SegmentItem::new("Tab"),
                        SegmentItem::new("Tab"),
                    ])
                    .selected(selected_tab)
                    .on_change({
                        let handle = cx.entity().downgrade();
                        move |idx, _window, cx| {
                            if let Some(this) = handle.upgrade() {
                                this.update(cx, |this, cx| {
                                    this.selected_tab = idx;
                                    cx.notify();
                                });
                            }
                        }
                    }),
            )
            .child(
                div()
                    .w(px(280.0))
                    .h(px(32.0))
                    .px(theme.spacing_md)
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
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
                            .child("Search..."),
                    ),
            );

        // ── Stats row (3 cards) ──────────────────────────────────────────
        let stats_row = div()
            .px(theme.spacing_lg)
            .flex()
            .gap(theme.spacing_md)
            .children(
                STATS
                    .iter()
                    .map(|s| div().flex_1().child(stat_card(s, card_tokens))),
            );

        // ── Charts row (line chart + people list) ────────────────────────
        let line_card = card(card_tokens)
            .flex()
            .flex_col()
            .p(theme.spacing_lg)
            .gap(theme.spacing_md)
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.text_muted)
                    .font_weight(FontWeight::MEDIUM)
                    .child("Title"),
            )
            .child(line_chart(px(160.0), theme.text));

        let people_card = card(card_tokens)
            .flex()
            .flex_col()
            .p(theme.spacing_lg)
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.text_muted)
                    .font_weight(FontWeight::MEDIUM)
                    .pb(theme.spacing_xs)
                    .child("Title"),
            )
            .children(PEOPLE.iter().map(|p| {
                let bg = hsla(p.color.0, p.color.1, p.color.2, 1.0);
                // Use char-based extraction so multi-byte UTF-8 names (e.g.
                // "Søren") don't panic the way `&name[0..1]` would.
                let initial: String = p
                    .name
                    .chars()
                    .next()
                    .map(|c| c.to_string())
                    .unwrap_or_default();
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_sm)
                    .py(px(2.0))
                    .child(Avatar::new(initial).bg(bg).size(px(24.0)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme.text)
                                    .child(SharedString::from(p.name)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(theme.text_muted)
                                    .child(SharedString::from(p.email)),
                            ),
                    )
            }));

        let charts_row = div()
            .px(theme.spacing_lg)
            .pt(theme.spacing_md)
            .flex()
            .gap(theme.spacing_md)
            .child(div().flex_1().child(line_card))
            .child(div().w(px(360.0)).child(people_card));

        // ── Data row (sources table + bar chart) ─────────────────────────
        let sources_card = card(card_tokens)
            .flex()
            .flex_col()
            .p(theme.spacing_lg)
            .gap(theme.spacing_sm)
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.text_muted)
                    .font_weight(FontWeight::MEDIUM)
                    .child("Title"),
            )
            // Header row
            .child(
                div()
                    .flex()
                    .gap(theme.spacing_md)
                    .pt(theme.spacing_xs)
                    .pb(theme.spacing_xs)
                    .border_b_1()
                    .border_color(theme.border)
                    .text_size(px(11.0))
                    .text_color(theme.text_muted)
                    .child(div().flex_1().child("Source"))
                    .child(div().w(px(70.0)).child("Sessions"))
                    .child(div().w(px(50.0)).child("Change")),
            )
            .children(SOURCES.iter().map(|s| {
                let change_color = if s.positive {
                    theme.success
                } else {
                    theme.error
                };
                div()
                    .flex()
                    .gap(theme.spacing_md)
                    .py(px(4.0))
                    .text_size(px(12.0))
                    .text_color(theme.text)
                    .child(div().flex_1().child(SharedString::from(s.name)))
                    .child(div().w(px(70.0)).child(SharedString::from(s.sessions)))
                    .child(
                        div()
                            .w(px(50.0))
                            .text_color(change_color)
                            .child(SharedString::from(s.change)),
                    )
            }));

        let bar_card = card(card_tokens)
            .flex()
            .flex_col()
            .p(theme.spacing_lg)
            .gap(theme.spacing_md)
            .child(
                div()
                    .text_size(px(13.0))
                    .text_color(theme.text_muted)
                    .font_weight(FontWeight::MEDIUM)
                    .child("Title"),
            )
            .child(bar_chart(px(180.0), theme.text))
            // Month labels: each cell is 1/12 of the width with a center-
            // aligned label, so each label sits directly below its bar. A
            // plain `justify_between` row would put the first label at 0%
            // and the last at 100%, drifting off the bars in between.
            .child(
                div()
                    .flex()
                    .text_size(px(10.0))
                    .text_color(theme.text_muted)
                    .children(MONTHS.iter().map(|m| {
                        div()
                            .flex_1()
                            .flex()
                            .justify_center()
                            .child(SharedString::from(*m))
                    })),
            );

        let data_row = div()
            .px(theme.spacing_lg)
            .pt(theme.spacing_md)
            .pb(theme.spacing_lg)
            .flex()
            .gap(theme.spacing_md)
            .child(div().w(px(420.0)).child(sources_card))
            .child(div().flex_1().child(bar_card));

        // ── Root layout ──────────────────────────────────────────────────
        div()
            .id("dashboard-scroll")
            .size_full()
            .bg(theme.background)
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .child(header)
            .child(tab_row)
            .child(stats_row)
            .child(charts_row)
            .child(data_row)
    }
}

fn main() {
    application()
        .with_assets(EmbeddedIconAssets)
        .run(|cx: &mut App| {
            let theme = TahoeTheme::liquid_glass_light();
            cx.set_global(theme);
            cx.bind_keys(tahoe_gpui::all_keybindings());

            // Window size matches the other Tahoe demos (1440 x 960). The
            // Figma reference frame for Dashboard is taller (1440 x 1364);
            // we use 960 here so all four demo windows open at the same
            // size and the dashboard scrolls vertically inside it.
            let bounds = Bounds::centered(None, size(px(1440.0), px(960.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: WindowBackgroundAppearance::Blurred,
                    ..Default::default()
                },
                |_, cx| cx.new(DashboardApp::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
