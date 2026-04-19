//! Example: Context window usage display with model pricing.
//!
//! Demonstrates the `ContextView` compound component and its sub-components.

use gpui::prelude::*;
use gpui::{
    App, Bounds, FontWeight, Window, WindowBackgroundAppearance, WindowBounds, WindowOptions, div,
    px, size,
};
use gpui_platform::application;
use tahoe_gpui::context::{ContextPill, ContextView, ModelId, Usage};
use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};

struct ContextExample;

impl Render for ContextExample {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<TahoeTheme>();

        let model = ModelId::parse("anthropic:claude-sonnet-4");
        let usage = Usage::new(50_000, 30_000, 5_000, 2_000);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .p(px(32.0))
            .gap(px(24.0))
            // Title
            .child(
                div()
                    .text_style(TextStyle::Title1, theme)
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme.text)
                    .child("Context Window Usage"),
            )
            // Collapsed pill (default hover-to-expand)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child("Compact pill (hover to expand):"),
                    )
                    .child(
                        ContextView::new()
                            .model_id(&model)
                            .max_tokens(200_000)
                            .used_tokens(87_000)
                            .usage(usage),
                    ),
            )
            // Expanded card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child("Detail card (expanded):"),
                    )
                    .child(
                        ContextView::new()
                            .model_id(&model)
                            .max_tokens(200_000)
                            .used_tokens(87_000)
                            .usage(usage)
                            .expanded(true),
                    ),
            )
            // Multiple models comparison
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child("Multiple models:"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(12.0))
                            .child(
                                ContextView::new()
                                    .model_id(&ModelId::parse("openai:gpt-4o"))
                                    .max_tokens(128_000)
                                    .used_tokens(32_000)
                                    .usage(Usage::new(25_000, 7_000, 0, 0)),
                            )
                            .child(
                                ContextView::new()
                                    .model_id(&ModelId::parse("anthropic:claude-opus-4"))
                                    .max_tokens(200_000)
                                    .used_tokens(150_000)
                                    .usage(Usage::new(100_000, 40_000, 0, 10_000)),
                            )
                            .child(
                                ContextView::new()
                                    .model_id(&ModelId::parse("google:gemini-2.5-pro"))
                                    .max_tokens(1_000_000)
                                    .used_tokens(250_000)
                                    .usage(Usage::new(200_000, 50_000, 0, 0)),
                            )
                            .child(
                                ContextView::new()
                                    .model_id(&ModelId::parse("ollama:llama3"))
                                    .max_tokens(8_192)
                                    .used_tokens(4_000)
                                    .usage(Usage::new(3_000, 1_000, 0, 0)),
                            ),
                    ),
            )
            // Sub-component composition demo
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_style(TextStyle::Subheadline, theme)
                            .text_color(theme.text_muted)
                            .child("Individual sub-components:"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(16.0))
                            .items_start()
                            .child(ContextPill::new(0.435))
                            .child(ContextPill::new(0.85))
                            .child(ContextPill::new(1.0)),
                    ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.set_global(TahoeTheme::dark());

        let bounds = Bounds::centered(None, size(px(700.), px(700.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_, cx| cx.new(|_| ContextExample),
        )
        .unwrap();
        cx.activate(true);
    });
}
