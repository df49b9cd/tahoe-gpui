//! Example: a Music app with sidebar navigation and grid playlist content.
//!
//! Demonstrates Sidebar, SegmentedControl, Button, and grid layouts.
//! Mirrors the macOS 26 "Music" screen pattern from the Apple Tahoe UI Kit.

use gpui::prelude::*;
use gpui::{
    App, Bounds, ElementId, SharedString, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, hsla, px, size,
};
use gpui_platform::application;

use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::sidebar::{Sidebar, SidebarItem};
use tahoe_gpui::components::selection_and_input::segmented_control::{
    SegmentItem, SegmentedControl,
};
use tahoe_gpui::foundations::icons::{EmbeddedIconAssets, IconName};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

// ── Static data ──────────────────────────────────────────────────────────────

struct NavItem {
    label: &'static str,
    icon: IconName,
}

const DISCOVER_ITEMS: &[NavItem] = &[
    NavItem {
        label: "Home",
        icon: IconName::Folder,
    },
    NavItem {
        label: "Browse",
        icon: IconName::Search,
    },
    NavItem {
        label: "Radio",
        icon: IconName::Globe,
    },
];

const LIBRARY_ITEMS: &[NavItem] = &[
    NavItem {
        label: "Playlists",
        icon: IconName::ListTodo,
    },
    NavItem {
        label: "Songs",
        icon: IconName::File,
    },
    NavItem {
        label: "Personalized picks",
        icon: IconName::Sparkle,
    },
];

struct Playlist {
    title: &'static str,
    accent: (f32, f32, f32),
}

const PLAYLISTS: &[Playlist] = &[
    Playlist {
        title: "Playlist 1",
        accent: (0.05, 0.55, 0.45),
    }, // warm peach
    Playlist {
        title: "Playlist 2",
        accent: (0.0, 0.0, 0.30),
    }, // dark
    Playlist {
        title: "Playlist 3",
        accent: (0.35, 0.55, 0.45),
    }, // green
    Playlist {
        title: "Playlist 4",
        accent: (0.62, 0.55, 0.30),
    }, // night blue
];

struct Artist {
    name: &'static str,
    genre: &'static str,
    accent: (f32, f32, f32),
}

const ARTISTS: &[Artist] = &[
    Artist {
        name: "Artist Name",
        genre: "R&B",
        accent: (0.0, 0.45, 0.45),
    },
    Artist {
        name: "Artist Name",
        genre: "Indie pop",
        accent: (0.62, 0.55, 0.55),
    },
    Artist {
        name: "Artist Name",
        genre: "Hip hop",
        accent: (0.85, 0.30, 0.55),
    },
    Artist {
        name: "Artist Name",
        genre: "Electronic",
        accent: (0.55, 0.20, 0.45),
    },
    Artist {
        name: "Artist Name",
        genre: "R&B",
        accent: (0.10, 0.50, 0.50),
    },
    Artist {
        name: "Artist Name",
        genre: "Rock",
        accent: (0.0, 0.65, 0.45),
    },
];

// ── App state ────────────────────────────────────────────────────────────────

struct MusicApp {
    selected_nav: usize, // 0..=2 = discover items, 3..=5 = library items
    selected_tab: usize,
}

impl MusicApp {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            selected_nav: 0,
            selected_tab: 0,
        }
    }
}

fn nav_row(item: &NavItem, idx: usize, selected: bool, cx: &mut Context<MusicApp>) -> SidebarItem {
    SidebarItem::new(
        ElementId::NamedInteger("nav".into(), idx as u64),
        SharedString::from(item.label),
    )
    .icon(item.icon)
    .selected(selected)
    .on_click(cx.listener(move |this, _, _, cx| {
        this.selected_nav = idx;
        cx.notify();
    }))
}

fn section_header(label: &'static str, theme: &TahoeTheme) -> impl IntoElement + use<> {
    div()
        .px(theme.spacing_md)
        .pt(theme.spacing_md)
        .pb(theme.spacing_xs)
        .text_style_emphasized(TextStyle::Headline, theme)
        .text_color(theme.text)
        .child(label)
}

fn playlist_card(playlist: &Playlist, theme: &TahoeTheme) -> impl IntoElement + use<> {
    let accent = hsla(playlist.accent.0, playlist.accent.1, playlist.accent.2, 1.0);
    div()
        .flex()
        .flex_col()
        .gap(theme.spacing_sm)
        .child(
            // Album art square (placeholder colored block)
            div()
                .size(px(168.0))
                .rounded(theme.radius_lg)
                .bg(accent)
                .flex()
                .items_end()
                .p(theme.spacing_md)
                .child(
                    div()
                        .text_style_emphasized(TextStyle::Title2, theme)
                        .text_color(hsla(0.0, 0.0, 1.0, 1.0))
                        .child(SharedString::from(playlist.title)),
                ),
        )
        .child(
            div()
                .text_style_emphasized(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(SharedString::from(playlist.title)),
        )
        .child(
            div()
                .text_style(TextStyle::Footnote, theme)
                .text_color(theme.text_muted)
                .child("Description of playlist"),
        )
}

fn artist_card(artist: &Artist, theme: &TahoeTheme) -> impl IntoElement + use<> {
    let accent = hsla(artist.accent.0, artist.accent.1, artist.accent.2, 1.0);
    div()
        .flex()
        .flex_col()
        .gap(theme.spacing_xs)
        .child(div().size(px(120.0)).rounded(theme.radius_lg).bg(accent))
        .child(
            div()
                .text_style_emphasized(TextStyle::Body, theme)
                .text_color(theme.text)
                .child(SharedString::from(artist.name)),
        )
        .child(
            div()
                .text_style(TextStyle::Footnote, theme)
                .text_color(theme.text_muted)
                .child(SharedString::from(artist.genre)),
        )
}

impl Render for MusicApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<MusicApp>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>().clone();
        let selected_nav = self.selected_nav;
        let selected_tab = self.selected_tab;
        let theme = &theme;

        // ── Sidebar ──────────────────────────────────────────────────────
        let mut sidebar_content = div().flex().flex_col().size_full();

        // App title
        sidebar_content = sidebar_content.child(
            div()
                .px(theme.spacing_md)
                .pt(theme.spacing_md)
                .pb(theme.spacing_sm)
                .text_style_emphasized(TextStyle::Title3, theme)
                .text_color(theme.text)
                .child("Music app"),
        );

        // Discover section
        sidebar_content = sidebar_content.child(section_header("Discover", theme));
        for (i, item) in DISCOVER_ITEMS.iter().enumerate() {
            sidebar_content = sidebar_content.child(nav_row(item, i, i == selected_nav, cx));
        }

        // Library section
        sidebar_content = sidebar_content.child(section_header("Library", theme));
        for (i, item) in LIBRARY_ITEMS.iter().enumerate() {
            let idx = i + 3;
            sidebar_content = sidebar_content.child(nav_row(item, idx, idx == selected_nav, cx));
        }

        let sidebar = Sidebar::new("sidebar")
            .width(px(240.0))
            .child(sidebar_content);

        // ── Main content ─────────────────────────────────────────────────
        // Top toolbar: tabs + Call to action
        let toolbar = div()
            .flex()
            .items_center()
            .justify_between()
            .px(theme.spacing_lg)
            .py(theme.spacing_md)
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
                Button::new("call-to-action")
                    .label("Call to action")
                    .variant(ButtonVariant::Filled)
                    .size(ButtonSize::Regular),
            );

        // Featured playlists section
        let playlists_section = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_md)
            .px(theme.spacing_lg)
            .pt(theme.spacing_md)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_style_emphasized(TextStyle::Title2, theme)
                            .text_color(theme.text)
                            .child("Title"),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child("Subheading"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(theme.spacing_md)
                    .children(PLAYLISTS.iter().map(|p| playlist_card(p, theme))),
            );

        // Artists section
        let artists_section = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_md)
            .px(theme.spacing_lg)
            .pt(theme.spacing_xl)
            .pb(theme.spacing_lg)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_style_emphasized(TextStyle::Title2, theme)
                            .text_color(theme.text)
                            .child("Title"),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child("Subheading"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(theme.spacing_md)
                    .children(ARTISTS.iter().map(|a| artist_card(a, theme))),
            );

        let main = div()
            .id("main-scroll")
            .flex_1()
            .flex()
            .flex_col()
            .bg(theme.background)
            .overflow_y_scroll()
            .child(toolbar)
            .child(playlists_section)
            .child(artists_section);

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
                |_, cx| cx.new(MusicApp::new),
            )
            .unwrap();
            cx.activate(true);
        });
}
