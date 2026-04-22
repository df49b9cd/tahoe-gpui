//! Example: Liquid Glass interactive messenger.
//!
//! A chat app shell demonstrating glass morphism on interactive components:
//! TabBar, Modal, Popover, Tooltip, HoverCard, Toggle, DisclosureGroup, ProgressIndicator.
//! All surfaces use translucent glass styling with macOS window blur.

use gpui::prelude::*;
use gpui::{
    AnyElement, App, Bounds, Entity, FontWeight, SharedString, Window, WindowBounds, WindowOptions,
    div, hsla, px, size,
};
use gpui_platform::application;
use tahoe_gpui::components::content::badge::{Badge, BadgeVariant};
use tahoe_gpui::components::layout_and_organization::disclosure_group::DisclosureGroup;
use tahoe_gpui::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use tahoe_gpui::components::navigation_and_search::tab_bar::{TabBar, TabItem};
use tahoe_gpui::components::presentation::hover_card::HoverCard;
use tahoe_gpui::components::presentation::modal::Modal;
use tahoe_gpui::components::presentation::popover::{Popover, PopoverPlacement};
use tahoe_gpui::components::selection_and_input::toggle::Toggle;
use tahoe_gpui::components::status::progress_indicator::ProgressIndicator;
use tahoe_gpui::foundations::GlassSurfaceScope;
use tahoe_gpui::foundations::icons::{Icon, IconName};
use tahoe_gpui::foundations::materials::GlassTintColor;
use tahoe_gpui::foundations::materials::{glass_surface, tinted_glass_surface};
use tahoe_gpui::foundations::theme::{GlassSize, TahoeTheme, TextStyle, TextStyledExt};

// ─── Static Data ─────────────────────────────────────────────────────────────

struct Contact {
    name: &'static str,
    initials: &'static str,
    status: &'static str,
    last_msg: &'static str,
    bio: &'static str,
    color: (f32, f32, f32), // h, s, l for avatar circle
    unread: u32,
}

const CONTACTS: &[Contact] = &[
    Contact {
        name: "Alice Chen",
        initials: "AC",
        status: "Online",
        last_msg: "Can you explain async/await?",
        bio: "Senior engineer at Oxide. Rust evangelist and async runtime enthusiast.",
        color: (0.55, 0.65, 0.55),
        unread: 2,
    },
    Contact {
        name: "Bob Miller",
        initials: "BM",
        status: "Away",
        last_msg: "The PR looks good, merging now",
        bio: "Staff engineer. Loves type systems, category theory, and strong coffee.",
        color: (0.08, 0.70, 0.55),
        unread: 0,
    },
    Contact {
        name: "Carol Park",
        initials: "CP",
        status: "Online",
        last_msg: "Let me check the benchmarks",
        bio: "Performance engineering lead. If it's not measured, it doesn't exist.",
        color: (0.75, 0.55, 0.55),
        unread: 0,
    },
    Contact {
        name: "Dave Kim",
        initials: "DK",
        status: "Offline",
        last_msg: "Shipped the new glass theme!",
        bio: "Design engineer bridging pixels and metal. GPUI contributor.",
        color: (0.35, 0.60, 0.55),
        unread: 1,
    },
];

struct DemoMessage {
    role: &'static str, // "user" or "assistant"
    content: &'static str,
}

fn conversations(idx: usize) -> &'static [DemoMessage] {
    match idx {
        0 => &[
            DemoMessage {
                role: "user",
                content: "Can you explain how async/await works in Rust?",
            },
            DemoMessage {
                role: "assistant",
                content: "In Rust, async/await is built on top of Futures. When you mark a function as `async`, it returns a Future that represents a value that will be available later. The `await` keyword suspends execution until the Future resolves. Unlike JavaScript, Rust futures are lazy  -- they don't do anything until polled by an executor like Tokio.",
            },
            DemoMessage {
                role: "user",
                content: "What about pinning? I keep running into Pin<Box<dyn Future>> issues.",
            },
            DemoMessage {
                role: "assistant",
                content: "Pinning guarantees that a value won't move in memory. This matters for self-referential futures  -- when an async block borrows from itself across an await point. `Pin<Box<dyn Future>>` heap-allocates the future and pins it. For most cases, you can use `Box::pin()` or `tokio::pin!()` to handle this ergonomically.",
            },
        ],
        1 => &[
            DemoMessage {
                role: "user",
                content: "Hey Bob, the type inference PR is ready for review.",
            },
            DemoMessage {
                role: "assistant",
                content: "Took a look  -- the bidirectional type inference changes look solid. One thing: the coercion sites in match arms might need explicit annotation for the edge case where both arms return impl Trait with different concrete types.",
            },
            DemoMessage {
                role: "user",
                content: "Good catch. I'll add a test case for that.",
            },
        ],
        2 => &[
            DemoMessage {
                role: "user",
                content: "Carol, the render loop is taking 4ms per frame. Any ideas?",
            },
            DemoMessage {
                role: "assistant",
                content: "4ms is tight but workable for 60fps. First thing I'd check: are you allocating in the hot path? Vec::push in a per-frame loop is a classic culprit. Pre-allocate with `with_capacity()`. Also profile with `cargo flamegraph`  -- I bet layout computation is the bottleneck, not painting.",
            },
            DemoMessage {
                role: "user",
                content: "You were right, it was layout. Caching the flex results brought it down to 1.2ms.",
            },
            DemoMessage {
                role: "assistant",
                content: "That's a 3x improvement. Ship it. Make sure to add a benchmark so we catch regressions early.",
            },
        ],
        _ => &[
            DemoMessage {
                role: "user",
                content: "The liquid glass theme is looking amazing, Dave!",
            },
            DemoMessage {
                role: "assistant",
                content: "Thanks! The key insight was using NSVisualEffectView for the window blur, then layering semi-transparent surfaces on top. The specular highlight on the top edge of each card really sells the glass illusion. Next up: tinted variants for semantic colors.",
            },
            DemoMessage {
                role: "user",
                content: "The tinted variants would be great for status badges and message bubbles.",
            },
        ],
    }
}

// ─── App State ────────────────────────────────────────────────────────────────

struct GlassMessenger {
    active_tab: SharedString,
    selected_contact: usize,
    show_new_chat_modal: bool,
    show_attach_popover: bool,
    streaming_enabled: bool,
    sound_enabled: bool,
    model_section_open: bool,
    hover_cards: Vec<Entity<HoverCard>>,
}

impl GlassMessenger {
    fn new(cx: &mut Context<Self>) -> Self {
        let hover_cards: Vec<Entity<HoverCard>> = CONTACTS
            .iter()
            .enumerate()
            .map(|(i, contact)| {
                cx.new(|cx| {
                    let mut hc = HoverCard::new(format!("hc-{i}"), cx);
                    let (h, s, l) = contact.color;
                    hc.set_trigger(
                        move |_cx| {
                            div()
                                .size(px(32.0))
                                .rounded(px(16.0))
                                .bg(hsla(h, s, l, 1.0))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_size(px(11.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(hsla(0.0, 0.0, 1.0, 0.9))
                                .child(CONTACTS[i].initials.to_string())
                                .into_any_element()
                        },
                        cx,
                    );
                    hc.set_content(
                        move |cx| {
                            let theme = cx.global::<TahoeTheme>();
                            let c = &CONTACTS[i];
                            let badge_variant = match c.status {
                                "Online" => BadgeVariant::Success,
                                "Away" => BadgeVariant::Warning,
                                _ => BadgeVariant::Muted,
                            };
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .p(px(12.0))
                                .min_w(px(200.0))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .text_style(TextStyle::Body, theme)
                                                .font_weight(FontWeight::MEDIUM)
                                                .child(c.name.to_string()),
                                        )
                                        .child(Badge::new(c.status).variant(badge_variant)),
                                )
                                .child(
                                    div()
                                        .text_style(TextStyle::Subheadline, theme)
                                        .text_color(theme.text_muted)
                                        .child(c.bio.to_string()),
                                )
                                .into_any_element()
                        },
                        cx,
                    );
                    hc
                })
            })
            .collect();

        Self {
            active_tab: "chat".into(),
            selected_contact: 0,
            show_new_chat_modal: false,
            show_attach_popover: false,
            streaming_enabled: true,
            sound_enabled: false,
            model_section_open: true,
            hover_cards,
        }
    }

    // ─── Header ──────────────────────────────────────────────────────────────

    fn render_header(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<TahoeTheme>();
        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .px(px(20.0))
            .py(px(12.0));

        header = glass_surface(header, theme, GlassSize::Small)
            .border_b_1()
            .border_color(gpui::hsla(0.0, 0.0, 1.0, 0.08));

        header
            .child(
                div()
                    .text_size(px(18.0))
                    .font_weight(FontWeight::LIGHT)
                    .text_color(theme.text)
                    .child("Glass Messenger"),
            )
            .child(
                div()
                    .flex()
                    .gap(px(6.0))
                    .child(Badge::new("Claude 4").variant(BadgeVariant::Info))
                    .child(Badge::new("Online").variant(BadgeVariant::Success)),
            )
            .into_any_element()
    }

    // ─── Sidebar ─────────────────────────────────────────────────────────────

    fn render_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<TahoeTheme>();
        let entity = cx.entity().clone();
        let mut sidebar = div()
            .flex()
            .flex_col()
            .w(px(250.0))
            .p(px(10.0))
            .gap(px(6.0));

        sidebar = glass_surface(sidebar, theme, GlassSize::Small)
            .border_r_1()
            .border_color(gpui::hsla(0.0, 0.0, 1.0, 0.08));

        // Section label
        sidebar = sidebar.child(
            div()
                .px(px(8.0))
                .py(px(4.0))
                .text_style(TextStyle::Caption1, theme)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child("Conversations"),
        );

        // Contact rows with hover cards
        for (i, contact) in CONTACTS.iter().enumerate() {
            let is_selected = self.selected_contact == i;
            let entity_clone = entity.clone();

            let row = div()
                .id(SharedString::from(format!("contact-{i}")))
                .flex()
                .items_center()
                .gap(px(8.0))
                .px(px(8.0))
                .py(px(6.0))
                .rounded(theme.glass.radius(GlassSize::Small))
                .cursor_pointer()
                .when(is_selected, |el| el.bg(theme.glass.hover_bg))
                .hover(|el| el.bg(theme.glass.hover_bg))
                .on_click(move |_, _, cx| {
                    entity_clone.update(cx, |this, cx| {
                        this.selected_contact = i;
                        cx.notify();
                    });
                })
                .child(self.hover_cards[i].clone())
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .overflow_x_hidden()
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .font_weight(FontWeight::MEDIUM)
                                .child(contact.name.to_string()),
                        )
                        .child(
                            div()
                                .text_style(TextStyle::Caption1, theme)
                                .text_color(theme.text_muted)
                                .overflow_x_hidden()
                                .child(contact.last_msg.to_string()),
                        ),
                )
                .when(contact.unread > 0, |el| {
                    el.child(Badge::new(contact.unread.to_string()).variant(BadgeVariant::Info))
                });

            sidebar = sidebar.child(row);
        }

        // Spacer + New Chat button
        let new_chat_entity = entity.clone();
        sidebar = sidebar.child(div().flex_1()).child(
            div().p(px(8.0)).child(
                Button::new("new-chat-btn")
                    .label("New Chat")
                    .icon(Icon::new(IconName::Plus))
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, _, cx| {
                        new_chat_entity.update(cx, |this, cx| {
                            this.show_new_chat_modal = true;
                            cx.notify();
                        });
                    }),
            ),
        );

        sidebar.into_any_element()
    }

    // ─── Chat Tab Body ───────────────────────────────────────────────────────

    fn render_chat(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<TahoeTheme>();
        let messages = conversations(self.selected_contact);
        let contact = &CONTACTS[self.selected_contact];

        let mut msg_list = div()
            .id("msg-scroll")
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .p(px(16.0))
            .overflow_y_scroll();

        for (i, msg) in messages.iter().enumerate() {
            let is_user = msg.role == "user";
            let mut bubble = div().flex().flex_col().gap(px(4.0)).p(px(12.0));

            if is_user {
                bubble = tinted_glass_surface(
                    bubble,
                    theme,
                    theme.glass.tints.get(GlassTintColor::Blue),
                    GlassSize::Small,
                );
            } else {
                bubble = glass_surface(bubble, theme, GlassSize::Small);
            }

            // Role label
            let sender = if is_user { "You" } else { contact.name };
            bubble = bubble.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(theme.text_muted)
                    .child(sender.to_string()),
            );

            // Content
            bubble = bubble.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .child(msg.content.to_string()),
            );

            // Action row for assistant messages
            if !is_user {
                bubble = bubble.child(
                    div()
                        .flex()
                        .gap(px(4.0))
                        .pt(px(4.0))
                        .child(
                            Button::new(SharedString::from(format!("copy-btn-{i}")))
                                .icon(Icon::new(IconName::Copy))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Icon)
                                .tooltip("Copy to clipboard"),
                        )
                        .child(
                            Button::new(SharedString::from(format!("up-btn-{i}")))
                                .icon(Icon::new(IconName::ThumbsUp))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Icon)
                                .tooltip("Helpful"),
                        )
                        .child(
                            Button::new(SharedString::from(format!("down-btn-{i}")))
                                .icon(Icon::new(IconName::ThumbsDown))
                                .variant(ButtonVariant::Ghost)
                                .size(ButtonSize::Icon)
                                .tooltip("Not helpful"),
                        ),
                );
            }

            // Align user messages right, assistant left
            let wrapper = div()
                .flex()
                .when(is_user, |el| el.justify_end())
                .child(div().max_w(px(550.0)).child(bubble));

            msg_list = msg_list.child(wrapper);
        }

        // Prompt bar
        let prompt_bar = self.render_prompt_bar(cx);

        div()
            .flex_1()
            .flex()
            .flex_col()
            .child(msg_list)
            .child(prompt_bar)
            .into_any_element()
    }

    // ─── Prompt Bar ──────────────────────────────────────────────────────────

    fn render_prompt_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<TahoeTheme>();
        let entity = cx.entity().clone();
        let entity_dismiss = cx.entity().clone();
        let show_popover = self.show_attach_popover;

        let attach_btn = Button::new("attach-btn")
            .icon(Icon::new(IconName::Paperclip))
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Icon)
            .tooltip("Attach")
            .on_click({
                let entity = entity.clone();
                move |_, _, cx| {
                    entity.update(cx, |this, cx| {
                        this.show_attach_popover = !this.show_attach_popover;
                        cx.notify();
                    });
                }
            });

        let popover_content = div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .p(px(4.0))
            .min_w(px(140.0))
            .child(
                Button::new("att-image")
                    .label("Image")
                    .icon(Icon::new(IconName::Image))
                    .variant(ButtonVariant::Ghost),
            )
            .child(
                Button::new("att-file")
                    .label("File")
                    .icon(Icon::new(IconName::File))
                    .variant(ButtonVariant::Ghost),
            )
            .child(
                Button::new("att-code")
                    .label("Code")
                    .icon(Icon::new(IconName::Code))
                    .variant(ButtonVariant::Ghost),
            )
            .child(
                Button::new("att-link")
                    .label("Link")
                    .icon(Icon::new(IconName::Link))
                    .variant(ButtonVariant::Ghost),
            );

        let attach_popover = Popover::new("attach-popover", attach_btn, popover_content)
            .open(show_popover)
            .placement(PopoverPlacement::AboveLeft)
            .on_dismiss(move |_, cx| {
                entity_dismiss.update(cx, |this, cx| {
                    this.show_attach_popover = false;
                    cx.notify();
                });
            });

        // Fake input area
        let input = div()
            .flex_1()
            .px(px(12.0))
            .py(px(8.0))
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text_muted)
            .child("Type a message...");

        let send_btn = Button::new("send-btn")
            .icon(Icon::new(IconName::Send))
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Icon)
            .tooltip("Send message");

        let mut bar = div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .m(px(12.0))
            .px(px(8.0))
            .py(px(4.0));

        bar = glass_surface(bar, theme, GlassSize::Small);

        bar.child(attach_popover)
            .child(input)
            .child(send_btn)
            .into_any_element()
    }

    // ─── Settings Tab Body ───────────────────────────────────────────────────

    fn render_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<TahoeTheme>();
        let entity_stream = cx.entity().clone();
        let entity_sound = cx.entity().clone();
        let entity_collapse = cx.entity().clone();

        let model_body = div()
            .flex()
            .flex_wrap()
            .gap(px(6.0))
            .child(Badge::new("Claude 4").variant(BadgeVariant::Info))
            .child(Badge::new("GPT-4o").variant(BadgeVariant::Muted))
            .child(Badge::new("Gemini 2").variant(BadgeVariant::Muted))
            .child(Badge::new("Llama 4").variant(BadgeVariant::Muted));

        let model_section = DisclosureGroup::new(
            "model-section",
            div()
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(FontWeight::MEDIUM)
                .child("Model Selection"),
            model_body,
        )
        .open(self.model_section_open)
        .on_toggle(move |open, _, cx| {
            entity_collapse.update(cx, |this, cx| {
                this.model_section_open = open;
                cx.notify();
            });
        });

        let streaming_row = div()
            .flex()
            .items_center()
            .justify_between()
            .py(px(6.0))
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .child("Enable streaming"),
            )
            .child(
                Toggle::new("streaming-switch")
                    .checked(self.streaming_enabled)
                    .on_change(move |val, _, cx| {
                        entity_stream.update(cx, |this, cx| {
                            this.streaming_enabled = val;
                            cx.notify();
                        });
                    }),
            );

        let sound_row = div()
            .flex()
            .items_center()
            .justify_between()
            .py(px(6.0))
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .child("Sound notifications"),
            )
            .child(
                Toggle::new("sound-switch")
                    .checked(self.sound_enabled)
                    .on_change(move |val, _, cx| {
                        entity_sound.update(cx, |this, cx| {
                            this.sound_enabled = val;
                            cx.notify();
                        });
                    }),
            );

        let usage_section = div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .pt(px(8.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .font_weight(FontWeight::MEDIUM)
                            .child("Token Usage"),
                    )
                    .child(
                        div()
                            .text_style(TextStyle::Caption1, theme)
                            .text_color(theme.text_muted)
                            .child("1,247 / 4,096"),
                    ),
            )
            .child(ProgressIndicator::new(0.3));

        let mut card = div().flex().flex_col().gap(px(8.0)).p(px(16.0)).m(px(16.0));
        card = glass_surface(card, theme, GlassSize::Large);

        card.child(model_section)
            .child(streaming_row)
            .child(sound_row)
            .child(usage_section)
            .into_any_element()
    }

    // ─── New Chat Modal ──────────────────────────────────────────────────────

    fn render_modal(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.global::<TahoeTheme>();
        let entity_close = cx.entity().clone();
        let entity_close2 = cx.entity().clone();

        let starters = [
            "Explain a Rust concept",
            "Review my code",
            "Help me debug an issue",
        ];

        let mut content = div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .p(px(20.0))
            .child(
                div()
                    .text_size(px(16.0))
                    .font_weight(FontWeight::MEDIUM)
                    .child("Start a Conversation"),
            )
            .child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child("Choose a conversation starter:"),
            );

        for (i, starter) in starters.iter().enumerate() {
            content = content.child(
                Button::new(SharedString::from(format!("starter-{i}")))
                    .label(*starter)
                    .variant(ButtonVariant::Ghost),
            );
        }

        content = content.child(
            div().flex().justify_end().pt(px(6.0)).child(
                Button::new("cancel-modal")
                    .label("Cancel")
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, _, cx| {
                        entity_close2.update(cx, |this, cx| {
                            this.show_new_chat_modal = false;
                            cx.notify();
                        });
                    }),
            ),
        );

        Modal::new("new-chat-modal", content)
            .open(self.show_new_chat_modal)
            .width(px(380.0))
            .on_dismiss(move |_, cx| {
                entity_close.update(cx, |this, cx| {
                    this.show_new_chat_modal = false;
                    cx.notify();
                });
            })
            .into_any_element()
    }
}

// ─── Render ──────────────────────────────────────────────────────────────────

impl Render for GlassMessenger {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Extract theme values we need before borrowing cx mutably.
        let theme = cx.global::<TahoeTheme>();
        let root_bg = theme.glass.root_bg;
        let text_color = theme.text;

        // Build all sub-sections (each fetches theme internally).
        let active_tab = self.active_tab.clone();
        let entity_tab = cx.entity().clone();

        let chat_body = self.render_chat(cx);
        let settings_body = self.render_settings(cx);
        let header = self.render_header(cx);
        let sidebar = self.render_sidebar(cx);
        let modal = self.render_modal(cx);

        let tabs = TabBar::new("main-tabs")
            .items(vec![
                TabItem::new("chat", div().child("Chat"), chat_body),
                TabItem::new("settings", div().child("Settings"), settings_body),
            ])
            .active(active_tab)
            .on_change(move |tab_id, _, cx| {
                entity_tab.update(cx, |this, cx| {
                    this.active_tab = tab_id;
                    cx.notify();
                });
            });

        let main_area = div().flex_1().flex().flex_col().child(tabs);

        // The entire messenger UI sits on Liquid Glass chrome — wrap the
        // root so every descendant Icon auto-resolves to glass coloring.
        GlassSurfaceScope::new(
            div()
                .size_full()
                .flex()
                .flex_col()
                .bg(root_bg)
                .text_color(text_color)
                .child(header)
                .child(div().flex_1().flex().child(sidebar).child(main_area))
                .child(modal),
        )
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() {
    application().run(|cx: &mut App| {
        let theme = TahoeTheme::liquid_glass();
        let window_bg = theme.glass.window_background;
        cx.set_global(theme);
        cx.bind_keys(tahoe_gpui::all_keybindings());

        let bounds = Bounds::centered(None, size(px(1100.0), px(750.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: window_bg,
                ..Default::default()
            },
            |_, cx| cx.new(GlassMessenger::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
