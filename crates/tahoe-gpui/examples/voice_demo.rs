//! Example: voice components demo (requires `voice` feature).

#[cfg(not(feature = "voice"))]
fn main() {
    eprintln!("Run with: cargo run --example voice_demo --features voice");
}

#[cfg(feature = "voice")]
fn main() {
    use gpui::prelude::*;
    use gpui::{
        App, Bounds, Div, Entity, FontWeight, Window, WindowBackgroundAppearance, WindowBounds,
        WindowOptions, div, px, size,
    };
    use gpui_platform::application;
    use tahoe_gpui::foundations::theme::{TahoeTheme, TextStyle, TextStyledExt};
    use tahoe_gpui::voice::*;

    struct VoiceDemo {
        audio_player: Entity<AudioPlayerView>,
        speech_input: Entity<SpeechInputView>,
        voice_selector: Entity<VoiceSelectorView>,
        voice_selector_dialog: Entity<VoiceSelectorView>,
        transcription: Entity<TranscriptionView>,
        persona_orbs: Vec<Entity<PersonaOrbState>>,
    }

    impl VoiceDemo {
        fn new(cx: &mut Context<Self>) -> Self {
            let audio_player = cx.new(|cx| {
                let mut view = AudioPlayerView::new(cx);
                view.set_title("Sample Audio Track", cx);
                view.set_duration(185.0, cx); // 3:05
                view.set_progress(0.35, cx); // ~65s in
                view.set_source(
                    AudioSource::Url("https://example.com/sample.mp3".into()),
                    cx,
                );
                view.set_on_play(|_window, _cx| {
                    println!("[voice_demo] Play");
                });
                view.set_on_pause(|_window, _cx| {
                    println!("[voice_demo] Pause");
                });
                view.set_on_seek(|time, _window, _cx| {
                    println!("[voice_demo] Seek to {time:.1}s");
                });
                view.set_on_volume_change(|vol, _window, _cx| {
                    println!("[voice_demo] Volume: {vol:.2}");
                });
                view.set_on_mute_toggle(|muted, _window, _cx| {
                    println!("[voice_demo] Muted: {muted}");
                });
                view.set_on_seek_forward(|offset, _window, _cx| {
                    println!("[voice_demo] Seek forward {offset}s");
                });
                view.set_on_seek_backward(|offset, _window, _cx| {
                    println!("[voice_demo] Seek backward {offset}s");
                });
                view
            });

            let transcription = cx.new(|cx| {
                let mut view = TranscriptionView::new(cx);
                view.set_segments(
                    vec![
                        TranscriptionSegment::new("Hello, welcome to the demo.", 0.0, 2.5)
                            .with_speaker("Alice"),
                        TranscriptionSegment::new(
                            "Thanks! Let me show you the transcription component.",
                            2.5,
                            5.8,
                        )
                        .with_speaker("Bob"),
                        TranscriptionSegment::new("", 5.8, 6.0), // empty segment (should be filtered)
                        TranscriptionSegment::new(
                            "It highlights segments based on playback time.",
                            6.0,
                            9.0,
                        )
                        .with_speaker("Alice"),
                        TranscriptionSegment::new("And you can click to seek.", 9.0, 11.0)
                            .with_speaker("Bob"),
                        TranscriptionSegment::new("Future segments appear dimmed.", 11.0, 13.5)
                            .with_speaker("Alice"),
                    ],
                    cx,
                );
                // Set playback at 7 seconds to show active/past/future states
                view.set_current_time(7.0, cx);
                view.set_on_seek(|time, _window, _cx| {
                    println!("[voice_demo] Seek to {time:.1}s");
                });
                view
            });

            let speech_input = cx.new(|cx| {
                let mut view = SpeechInputView::new(cx);
                // Use the async callback: simulates a transcription service call.
                view.set_on_audio_recorded_async(|audio, _app| {
                    let duration = audio.duration_secs;
                    let size = audio.data.len();
                    let sample_rate = audio.sample_rate;
                    // Return a future that simulates async transcription.
                    // In production: call a transcription service (Whisper, etc.).
                    Box::pin(async move {
                        Some(format!(
                            "[simulated transcription] {:.1}s of audio ({} bytes, {} Hz)",
                            duration, size, sample_rate
                        ))
                    })
                });
                view.set_on_transcription_change(|text, _window, _cx| {
                    println!("[voice_demo] Transcription: {}", text);
                });
                view
            });

            let demo_voices = vec![
                VoiceOption::new("alloy", "Alloy")
                    .description("Warm and confident")
                    .gender(VoiceGender::Female)
                    .accent(VoiceAccent::American)
                    .age("Young adult")
                    .shortcut("Cmd+1")
                    .group("OpenAI"),
                VoiceOption::new("echo", "Echo")
                    .description("Clear and precise")
                    .gender(VoiceGender::Male)
                    .accent(VoiceAccent::American)
                    .age("Adult")
                    .shortcut("Cmd+2")
                    .group("OpenAI"),
                VoiceOption::new("nova", "Nova")
                    .description("Energetic and bright")
                    .gender(VoiceGender::Female)
                    .accent(VoiceAccent::American)
                    .group("OpenAI"),
                VoiceOption::new("shimmer", "Shimmer")
                    .description("Soothing and gentle")
                    .gender(VoiceGender::Female)
                    .accent(VoiceAccent::British)
                    .age("Adult")
                    .group("OpenAI"),
                VoiceOption::new("onyx", "Onyx")
                    .description("Deep and authoritative")
                    .gender(VoiceGender::Male)
                    .accent(VoiceAccent::British)
                    .age("Middle-aged")
                    .group("OpenAI"),
                VoiceOption::new("sky", "Sky")
                    .description("Fluid and expressive")
                    .gender(VoiceGender::Androgyne)
                    .accent(VoiceAccent::American)
                    .group("OpenAI"),
                VoiceOption::new("aria", "Aria")
                    .gender(VoiceGender::Female)
                    .accent(VoiceAccent::Australian)
                    .group("ElevenLabs"),
                VoiceOption::new("roger", "Roger")
                    .gender(VoiceGender::Male)
                    .accent(VoiceAccent::British)
                    .group("ElevenLabs"),
            ];

            let voice_selector = cx.new(|cx| {
                let mut view = VoiceSelectorView::new(cx);
                view.set_voices(demo_voices.clone(), cx);
                view.set_value(Some("alloy"), cx);
                view.set_on_select(|voice, _window, _cx| {
                    println!("[voice_demo] Selected voice: {} ({})", voice.name, voice.id);
                });
                view.set_on_preview(|voice, _window, _cx| {
                    println!(
                        "[voice_demo] Preview requested: {} ({})",
                        voice.name, voice.id
                    );
                });
                view
            });

            let voice_selector_dialog = cx.new(|cx| {
                let mut view = VoiceSelectorView::new(cx);
                view.set_voices(demo_voices, cx);
                view.set_variant(VoiceSelectorVariant::Dialog, cx);
                view.set_empty_message("No matching voices found", cx);
                view.set_on_select(|voice, _window, _cx| {
                    println!(
                        "[voice_demo] Dialog selected: {} ({})",
                        voice.name, voice.id
                    );
                });
                view
            });

            // Stateful PersonaOrbState entities — one per variant, cycling through states
            let variants = [
                PersonaVariant::Obsidian,
                PersonaVariant::Mana,
                PersonaVariant::Opal,
                PersonaVariant::Halo,
                PersonaVariant::Glint,
                PersonaVariant::Command,
            ];
            let states = [
                PersonaState::Idle,
                PersonaState::Listening,
                PersonaState::Thinking,
                PersonaState::Speaking,
                PersonaState::Asleep,
                PersonaState::Speaking,
            ];
            let persona_orbs: Vec<Entity<PersonaOrbState>> = variants
                .iter()
                .zip(states.iter())
                .map(|(variant, state)| {
                    let v = *variant;
                    let s = *state;
                    cx.new(|cx| {
                        let mut orb = PersonaOrbState::new_with(s, v);
                        orb.set_size(px(80.0), cx);
                        orb
                    })
                })
                .collect();

            Self {
                audio_player,
                speech_input,
                voice_selector,
                voice_selector_dialog,
                transcription,
                persona_orbs,
            }
        }
    }

    impl Render for VoiceDemo {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let theme = cx.global::<TahoeTheme>();

            div()
                .id("voice-demo-scroll")
                .size_full()
                .flex()
                .flex_col()
                .bg(theme.background)
                .p(px(24.0))
                .gap(px(24.0))
                .overflow_y_scroll()
                .child(
                    div()
                        .text_style(TextStyle::Title1, theme)
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme.text)
                        .child("AI Elements - Voice Demo"),
                )
                // PersonaOrb (standalone animated visual)
                .child(
                    section("PersonaOrb - Variants", theme).child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(16.0))
                            .child(PersonaOrb::new().variant(PersonaVariant::Obsidian).state(PersonaState::Speaking))
                            .child(PersonaOrb::new().variant(PersonaVariant::Mana).state(PersonaState::Speaking))
                            .child(PersonaOrb::new().variant(PersonaVariant::Opal).state(PersonaState::Speaking))
                            .child(PersonaOrb::new().variant(PersonaVariant::Halo).state(PersonaState::Speaking))
                            .child(PersonaOrb::new().variant(PersonaVariant::Glint).state(PersonaState::Speaking))
                            .child(PersonaOrb::new().variant(PersonaVariant::Command).state(PersonaState::Speaking)),
                    ),
                )
                .child(
                    section("PersonaOrb - States", theme).child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(16.0))
                            .child(PersonaOrb::new().state(PersonaState::Idle).variant(PersonaVariant::Mana))
                            .child(PersonaOrb::new().state(PersonaState::Listening).variant(PersonaVariant::Mana))
                            .child(PersonaOrb::new().state(PersonaState::Thinking).variant(PersonaVariant::Mana))
                            .child(PersonaOrb::new().state(PersonaState::Speaking).variant(PersonaVariant::Mana))
                            .child(PersonaOrb::new().state(PersonaState::Asleep).variant(PersonaVariant::Mana)),
                    ),
                )
                // PersonaOrbState (stateful, delta-driven animation)
                .child(
                    section("PersonaOrbState - Delta-driven Animation", theme).child({
                        let mut row = div()
                            .flex()
                            .flex_wrap()
                            .gap(px(16.0));
                        for orb in &self.persona_orbs {
                            row = row.child(orb.clone());
                        }
                        row
                    }),
                )
                .child(
                    section("Persona Cards", theme).child(
                        div()
                            .flex()
                            .gap(px(16.0))
                            .child(Persona::new("Aria", "A").state(PersonaState::Idle))
                            .child(
                                Persona::new("Nova", "N")
                                    .state(PersonaState::Speaking)
                                    .description("Warm and friendly"),
                            )
                            .child(Persona::new("Echo", "E").state(PersonaState::Listening)),
                    ),
                )
                // Speech Input (functional u2014 click to record)
                .child(
                    section("Speech Input", theme)
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child(
                                    "Click the mic button to record. Audio is captured via cpal and encoded as WAV.",
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .w(px(300.0))
                                .child(self.speech_input.clone()),
                        ),
                )
                // Audio Player (display only)
                .child(
                    section("Audio Player", theme)
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child("Audio Player with all controls (display-only, no actual playback)"),
                        )
                        .child(
                            div()
                                .w(px(400.0))
                                .child(self.audio_player.clone()),
                        ),
                )
                // Transcription
                .child(
                    section("Transcription", theme)
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child("Playback at 7.0s: past segments are muted, active is accented, future is dimmed. Click to seek."),
                        )
                        .child(
                            div()
                                .w(px(500.0))
                                .child(self.transcription.clone()),
                        ),
                )
                // Voice Selector (Dropdown)
                .child(
                    section("Voice Selector (Dropdown)", theme)
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child(
                                    "Click to open. Supports search, keyboard nav, grouping, metadata, and shortcuts.",
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(self.voice_selector.clone()),
                        ),
                )
                // Voice Selector (Dialog)
                .child(
                    section("Voice Selector (Dialog)", theme)
                        .child(
                            div()
                                .text_style(TextStyle::Subheadline, theme)
                                .text_color(theme.text_muted)
                                .child(
                                    "Full-screen modal variant with centered dialog and backdrop.",
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(self.voice_selector_dialog.clone()),
                        ),
                )
        }
    }

    fn section(title: &str, theme: &TahoeTheme) -> Div {
        div().flex().flex_col().gap(px(8.0)).child(
            div()
                .text_style(TextStyle::Subheadline, theme)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme.text_muted)
                .child(title.to_string()),
        )
    }

    application().run(|cx: &mut App| {
        cx.set_global(TahoeTheme::dark());
        let bounds = Bounds::centered(None, size(px(800.), px(700.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_background: WindowBackgroundAppearance::Blurred,
                ..Default::default()
            },
            |_, cx| cx.new(VoiceDemo::new),
        )
        .unwrap();
        cx.activate(true);
    });
}
