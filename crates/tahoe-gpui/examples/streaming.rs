//! Example: simulated streaming with word-level animation.
//!
//! Demonstrates the AnimationState system by revealing words one-by-one
//! with fade-in animation and a blinking caret.

use std::time::{Duration, Instant};

use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};
use tahoe_gpui::markdown::animation::{AnimationKind, AnimationState, Easing};
use tahoe_gpui::markdown::caret::{self, CaretKind};
use gpui::prelude::*;
use gpui::{
    AnyElement, App, Bounds, Div, FontWeight, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, div, px, size,
};
use gpui_platform::application;

const SAMPLE_TEXT: &str = "Rust is a systems programming language focused on safety, \
    concurrency, and performance. It achieves memory safety without a garbage collector \
    through its ownership system and borrow checker.";

struct StreamingExample {
    animation: AnimationState,
    words: Vec<String>,
    next_word: usize,
    start_time: Instant,
    last_reveal: Instant,
    words_per_tick: usize,
}

impl StreamingExample {
    fn new() -> Self {
        let words: Vec<String> = SAMPLE_TEXT.split_whitespace().map(String::from).collect();
        let now = Instant::now();
        Self {
            animation: AnimationState::new(AnimationKind::FadeIn)
                .duration(Duration::from_millis(300))
                .stagger(Duration::from_millis(40))
                .easing(Easing::EaseOut),
            words,
            next_word: 0,
            start_time: now,
            last_reveal: now,
            words_per_tick: 2,
        }
    }

    fn tick(&mut self) {
        let now = Instant::now();
        // Simulate tokens arriving every 80ms
        if now.duration_since(self.last_reveal) >= Duration::from_millis(80)
            && self.next_word < self.words.len()
        {
            let count = self.words_per_tick.min(self.words.len() - self.next_word);
            self.animation.reveal_words(count, now);
            self.next_word += count;
            self.last_reveal = now;
        }
    }
}

impl Render for StreamingExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let now = Instant::now();

        // Simulate streaming
        self.tick();

        // Always request the next frame — the elapsed timer and caret
        // blink need continuous updates even after streaming completes.
        window.request_animation_frame();

        let theme = cx.global::<TahoeTheme>();
        let is_streaming = self.next_word < self.words.len();
        let elapsed = now.duration_since(self.start_time);

        // Build word elements with per-word opacity
        let word_elements: Vec<AnyElement> = self
            .words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                let opacity = self.animation.word_opacity(i, now);
                let separator = if i + 1 < self.words.len() { " " } else { "" };
                div()
                    .opacity(opacity)
                    .child(format!("{word}{separator}"))
                    .into_any_element()
            })
            .collect();

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
                    .child("Streaming Animation Demo"),
            )
            // Stats
            .child(
                div()
                    .flex()
                    .gap(px(16.0))
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text_muted)
                    .child(format!("Words: {}/{}", self.next_word, self.words.len()))
                    .child(format!("Elapsed: {:.1}s", elapsed.as_secs_f32()))
                    .child(format!(
                        "Watermark: {}",
                        self.animation.fully_revealed_watermark(now)
                    ))
                    .child(if is_streaming {
                        "Status: Streaming..."
                    } else if self.animation.is_animating(now) {
                        "Status: Finishing animation"
                    } else {
                        "Status: Complete"
                    }),
            )
            // Animated text
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .bg(theme.surface)
                    .rounded(theme.radius_lg)
                    .p(theme.spacing_md)
                    .text_style(TextStyle::Body, theme)
                    .text_color(theme.text)
                    .children(word_elements)
                    .when(is_streaming, |el: Div| {
                        el.child(caret::render_caret(
                            CaretKind::Block,
                            theme.accent,
                            now,
                            Duration::from_millis(530),
                            TextStyle::Body.attrs().leading,
                        ))
                    }),
            )
            // Easing visualization
            .child(
                div().flex().flex_col().gap(px(8.0)).child(
                    div()
                        .text_style(TextStyle::Subheadline, theme)
                        .text_color(theme.text_muted)
                        .child("Animation: FadeIn, 300ms duration, 40ms stagger, EaseOut"),
                ),
            )
    }
}

fn main() {
    application().run(|cx: &mut App| {
        cx.set_global(TahoeTheme::dark());

        let bounds = Bounds::centered(None, size(px(700.), px(500.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_, cx| cx.new(|_| StreamingExample::new()),
        )
        .unwrap();
        cx.activate(true);
    });
}
