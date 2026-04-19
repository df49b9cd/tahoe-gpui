//! Audio player display component (visual only, no actual audio playback).
//!
//! Provides a composable audio player UI with play/pause, seek controls,
//! interactive time slider, volume/mute, and time display. Consumers wire
//! up actual audio playback via event callbacks.
//!
//! # HIG alignment (issue #148)
//!
//! * F9 — All interactive controls (play/pause, skip ±10 s, mute, seek
//!   slider, volume slider) carry an explicit `accessibility_label` so
//!   VoiceOver announces them even though this view is visual-only.
//! * F12 — The Playing audio HIG mandates:
//!   * Now Playing metadata via `MPNowPlayingInfoCenter`.
//!   * An audio-session category (`Playback`) registered with
//!     `AVAudioSession` so audio continues in background.
//!   * Graceful interruption handling (phone call, other apps).
//!
//!   This view is display-only by design; consumers own the playback
//!   engine. The `on_interruption_began` / `on_interruption_ended` callbacks
//!   let the host UI react to OS-level interruptions the consumer receives
//!   via `AVAudioSessionInterruptionNotification`. The view does not call
//!   `MPNowPlayingInfoCenter` itself — consumers must set the Now Playing
//!   metadata from their playback engine.

use gpui::prelude::*;
use gpui::{App, ElementId, Entity, FontWeight, SharedString, WeakEntity, Window, div, px};

use crate::callback_types::{OnF32Change, OnMutCallback, OnToggle};
use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::components::menus_and_actions::button_group::ButtonGroup;
use crate::components::selection_and_input::slider::Slider;
use crate::components::status::progress_indicator::ProgressIndicator;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;

/// Audio source type for the player.
#[derive(Debug, Clone)]
pub enum AudioSource {
    /// Remote audio file URL.
    Url(SharedString),
    /// Base64-encoded audio data with MIME type.
    Base64 {
        data: SharedString,
        mime_type: SharedString,
    },
}

/// Default seek offset in seconds for forward/backward seek buttons.
pub(crate) const DEFAULT_SEEK_OFFSET: f32 = 10.0;

/// A display-only audio player with play/pause, seek, volume, and time controls.
///
/// All controls are visual only. Wire up actual audio playback via the
/// `set_on_*` callback methods.
pub struct AudioPlayerView {
    element_id: ElementId,
    is_playing: bool,
    progress: f32,
    duration_secs: f32,
    current_secs: f32,
    title: Option<SharedString>,

    // Volume
    volume: f32,
    is_muted: bool,

    // Seek
    seek_offset_secs: f32,

    // Source
    source: Option<AudioSource>,

    // UI config
    disabled: bool,
    show_seek_buttons: bool,
    show_volume: bool,
    show_time_range: bool,

    // Child slider entities
    seek_slider: Entity<Slider>,
    volume_slider: Entity<Slider>,

    // Callbacks
    on_play: OnMutCallback,
    on_pause: OnMutCallback,
    on_seek: OnF32Change,
    on_volume_change: OnF32Change,
    on_mute_toggle: OnToggle,
    on_seek_forward: OnF32Change,
    on_seek_backward: OnF32Change,
    /// Invoked when the host's `AVAudioSession` reports an interruption
    /// begin (phone call, Siri, another app taking exclusive audio). The
    /// view does not itself observe `AVAudioSessionInterruptionNotification` —
    /// consumers forward the notification here so the UI can show a paused
    /// indicator or disable controls until the interruption resolves
    /// (issue #148 F12).
    on_interruption_began: OnMutCallback,
    /// Invoked when `AVAudioSession` reports that the interruption ended.
    /// The view-side handler typically restores playback state.
    on_interruption_ended: OnMutCallback,
}

impl AudioPlayerView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let weak_self: WeakEntity<Self> = cx.entity().downgrade();

        // Create seek slider
        let seek_slider = cx.new(|cx| {
            let mut s = Slider::new(cx);
            s.set_height(px(4.0));
            s.set_thumb_size(px(12.0));
            // Issue #148 F9: label so VoiceOver announces this as the
            // playback position control rather than a generic slider.
            s.set_accessibility_label("Playback position");
            s
        });

        // Wire seek slider -> parent
        let handle = weak_self.clone();
        seek_slider.update(cx, |slider, _cx| {
            slider.set_on_change(move |value, window, cx| {
                if let Some(this) = handle.upgrade() {
                    this.update(cx, |this, cx| {
                        this.progress = value;
                        this.current_secs = this.duration_secs * value;
                        if let Some(on_seek) = &this.on_seek {
                            on_seek(this.current_secs, window, cx);
                        }
                        cx.notify();
                    });
                }
            });
        });

        // Create volume slider
        let volume_slider = cx.new(|cx| {
            let mut s = Slider::new(cx);
            s.set_value(1.0, cx);
            s.set_height(px(4.0));
            s.set_thumb_size(px(10.0));
            // Issue #148 F9.
            s.set_accessibility_label("Volume");
            s
        });

        // Wire volume slider -> parent
        let handle = weak_self;
        volume_slider.update(cx, |slider, _cx| {
            slider.set_on_change(move |value, window, cx| {
                if let Some(this) = handle.upgrade() {
                    this.update(cx, |this, cx| {
                        this.volume = value;
                        if let Some(on_volume_change) = &this.on_volume_change {
                            on_volume_change(value, window, cx);
                        }
                        cx.notify();
                    });
                }
            });
        });

        Self {
            element_id: next_element_id("audio-player"),
            is_playing: false,
            progress: 0.0,
            duration_secs: 0.0,
            current_secs: 0.0,
            title: None,
            volume: 1.0,
            is_muted: false,
            seek_offset_secs: DEFAULT_SEEK_OFFSET,
            source: None,
            disabled: false,
            show_seek_buttons: true,
            show_volume: true,
            show_time_range: true,
            seek_slider,
            volume_slider,
            on_play: None,
            on_pause: None,
            on_seek: None,
            on_volume_change: None,
            on_mute_toggle: None,
            on_seek_forward: None,
            on_seek_backward: None,
            on_interruption_began: None,
            on_interruption_ended: None,
        }
    }

    // ── State setters ──

    pub fn set_playing(&mut self, playing: bool, cx: &mut Context<Self>) {
        self.is_playing = playing;
        cx.notify();
    }

    pub fn set_progress(&mut self, progress: f32, cx: &mut Context<Self>) {
        self.progress = progress.clamp(0.0, 1.0);
        self.current_secs = self.duration_secs * self.progress;
        self.seek_slider
            .update(cx, |s, cx| s.set_value(self.progress, cx));
        cx.notify();
    }

    pub fn set_duration(&mut self, secs: f32, cx: &mut Context<Self>) {
        self.duration_secs = secs;
        cx.notify();
    }

    pub fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.title = Some(title.into());
        cx.notify();
    }

    pub fn toggle_playback(&mut self, cx: &mut Context<Self>) {
        self.is_playing = !self.is_playing;
        cx.notify();
    }

    pub fn set_volume(&mut self, volume: f32, cx: &mut Context<Self>) {
        self.volume = volume.clamp(0.0, 1.0);
        self.volume_slider
            .update(cx, |s, cx| s.set_value(self.volume, cx));
        cx.notify();
    }

    pub fn set_muted(&mut self, muted: bool, cx: &mut Context<Self>) {
        self.is_muted = muted;
        cx.notify();
    }

    pub fn set_source(&mut self, source: AudioSource, cx: &mut Context<Self>) {
        self.source = Some(source);
        cx.notify();
    }

    pub fn set_seek_offset(&mut self, secs: f32, cx: &mut Context<Self>) {
        self.seek_offset_secs = secs;
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    // ── State getters ──

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn current_time(&self) -> f32 {
        self.current_secs
    }

    pub fn duration(&self) -> f32 {
        self.duration_secs
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn is_muted(&self) -> bool {
        self.is_muted
    }

    // ── UI config ──

    pub fn set_show_seek_buttons(&mut self, show: bool) {
        self.show_seek_buttons = show;
    }

    pub fn set_show_volume(&mut self, show: bool) {
        self.show_volume = show;
    }

    pub fn set_show_time_range(&mut self, show: bool) {
        self.show_time_range = show;
    }

    // ── Callbacks ──

    pub fn set_on_play(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_play = Some(Box::new(handler));
    }

    pub fn set_on_pause(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_pause = Some(Box::new(handler));
    }

    pub fn set_on_seek(&mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) {
        self.on_seek = Some(Box::new(handler));
    }

    pub fn set_on_volume_change(&mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) {
        self.on_volume_change = Some(Box::new(handler));
    }

    pub fn set_on_mute_toggle(&mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) {
        self.on_mute_toggle = Some(Box::new(handler));
    }

    pub fn set_on_seek_forward(&mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) {
        self.on_seek_forward = Some(Box::new(handler));
    }

    pub fn set_on_seek_backward(&mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) {
        self.on_seek_backward = Some(Box::new(handler));
    }

    /// Register a handler fired by the consumer when the underlying
    /// `AVAudioSession` reports an interruption begin. Issue #148 F12.
    ///
    /// The consumer observes `AVAudioSessionInterruptionNotification` (or
    /// the platform equivalent) and forwards a call to
    /// [`Self::notify_interruption_began`] so the UI can reflect the state.
    pub fn set_on_interruption_began(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_interruption_began = Some(Box::new(handler));
    }

    /// Register a handler fired by the consumer when the interruption
    /// ends. Typically resumes playback and restores Now Playing metadata.
    pub fn set_on_interruption_ended(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_interruption_ended = Some(Box::new(handler));
    }

    /// Called by the consumer when an interruption begins. The view
    /// forces `is_playing=false` and fires any registered handler.
    pub fn notify_interruption_began(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.is_playing = false;
        if let Some(ref handler) = self.on_interruption_began {
            handler(window, &mut *cx);
        }
        cx.notify();
    }

    /// Called by the consumer when an interruption ends. Does not
    /// auto-resume playback — the consumer decides based on the
    /// `shouldResume` option from `AVAudioSession`.
    pub fn notify_interruption_ended(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ref handler) = self.on_interruption_ended {
            handler(window, &mut *cx);
        }
        cx.notify();
    }

    // ── Helpers ──

    /// Format seconds as MM:SS.
    fn format_time(secs: f32) -> SharedString {
        let total = secs as u64;
        let m = total / 60;
        let s = total % 60;
        SharedString::from(format!("{m:02}:{s:02}"))
    }

    fn handle_play_pause(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.is_playing = !self.is_playing;
        if self.is_playing {
            if let Some(on_play) = &self.on_play {
                on_play(window, cx);
            }
        } else if let Some(on_pause) = &self.on_pause {
            on_pause(window, cx);
        }
        cx.notify();
    }

    fn handle_seek_backward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.current_secs = (self.current_secs - self.seek_offset_secs).max(0.0);
        self.progress = if self.duration_secs > 0.0 {
            self.current_secs / self.duration_secs
        } else {
            0.0
        };
        self.seek_slider
            .update(cx, |s, cx| s.set_value(self.progress, cx));
        if let Some(on_seek_backward) = &self.on_seek_backward {
            on_seek_backward(self.seek_offset_secs, window, cx);
        }
        cx.notify();
    }

    fn handle_seek_forward(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.current_secs = (self.current_secs + self.seek_offset_secs).min(self.duration_secs);
        self.progress = if self.duration_secs > 0.0 {
            self.current_secs / self.duration_secs
        } else {
            0.0
        };
        self.seek_slider
            .update(cx, |s, cx| s.set_value(self.progress, cx));
        if let Some(on_seek_forward) = &self.on_seek_forward {
            on_seek_forward(self.seek_offset_secs, window, cx);
        }
        cx.notify();
    }

    fn handle_mute_toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        self.is_muted = !self.is_muted;
        if let Some(on_mute_toggle) = &self.on_mute_toggle {
            on_mute_toggle(self.is_muted, window, cx);
        }
        cx.notify();
    }
}

impl Render for AudioPlayerView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let disabled = self.disabled;

        let play_icon = if self.is_playing {
            Icon::new(IconName::Pause).size(theme.icon_size_inline)
        } else {
            Icon::new(IconName::Play).size(theme.icon_size_inline)
        };

        let current = Self::format_time(self.current_secs);
        let duration = Self::format_time(self.duration_secs);

        let mut container = div()
            .id(self.element_id.clone())
            .flex()
            .flex_col()
            .gap(theme.spacing_xs)
            .p(theme.spacing_sm)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border);

        // Title row (optional)
        if let Some(ref title) = self.title {
            container = container.child(
                div()
                    .text_style(TextStyle::Subheadline, theme)
                    .text_color(theme.text)
                    .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                    .child(title.clone()),
            );
        }

        // Controls row
        let mut controls = div().flex().items_center().gap(theme.spacing_sm);

        // Play controls in a ButtonGroup
        let mut play_group = ButtonGroup::new(ElementId::from(SharedString::from(format!(
            "{}-play-group",
            self.element_id
        ))));

        // Seek backward button (optional)
        if self.show_seek_buttons {
            let seek_back_label = SharedString::from(format!(
                "Seek {} seconds backward",
                self.seek_offset_secs as u32
            ));
            play_group = play_group.child(
                Button::new(ElementId::from(SharedString::from(format!(
                    "{}-seek-back",
                    self.element_id
                ))))
                .icon(Icon::new(IconName::SkipBack).size(px(12.0)))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(disabled)
                .accessibility_label(seek_back_label)
                .on_click(cx.listener(|this, _event, window, cx| {
                    this.handle_seek_backward(window, cx);
                })),
            );
        }

        // Play/pause button — issue #148 F9, label reflects current state
        // so VoiceOver reads "Play" when paused and "Pause" when playing.
        let play_label: SharedString = if self.is_playing {
            "Pause".into()
        } else {
            "Play".into()
        };
        play_group = play_group.child(
            Button::new(ElementId::from(SharedString::from(format!(
                "{}-play",
                self.element_id
            ))))
            .icon(play_icon)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::IconSm)
            .disabled(disabled)
            .accessibility_label(play_label)
            .on_click(cx.listener(|this, _event, window, cx| {
                this.handle_play_pause(window, cx);
            })),
        );

        // Seek forward button (optional)
        if self.show_seek_buttons {
            let seek_fwd_label = SharedString::from(format!(
                "Seek {} seconds forward",
                self.seek_offset_secs as u32
            ));
            play_group = play_group.child(
                Button::new(ElementId::from(SharedString::from(format!(
                    "{}-seek-fwd",
                    self.element_id
                ))))
                .icon(Icon::new(IconName::SkipForward).size(px(12.0)))
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(disabled)
                .accessibility_label(seek_fwd_label)
                .on_click(cx.listener(|this, _event, window, cx| {
                    this.handle_seek_forward(window, cx);
                })),
            );
        }

        controls = controls.child(play_group);

        // Current time
        controls = controls.child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .flex_shrink_0()
                .child(current),
        );

        // Seek slider or static progress bar. When disabled we render the
        // non-interactive `ProgressIndicator` instead of the live `Slider`
        // so there is no focus handle or mouse handler to reach — GPUI
        // 0.231 has no `pointer_events_none` primitive, so we emulate it
        // by swapping the interactive child out entirely.
        if self.show_time_range && !disabled {
            controls = controls.child(div().flex_1().child(self.seek_slider.clone()));
        } else {
            controls = controls.child(
                div()
                    .flex_1()
                    .when(disabled, |el| el.opacity(0.4))
                    .child(ProgressIndicator::new(self.progress).height(px(4.0))),
            );
        }

        // Duration
        controls = controls.child(
            div()
                .text_style(TextStyle::Caption1, theme)
                .text_color(theme.text_muted)
                .flex_shrink_0()
                .child(duration),
        );

        // Volume controls (optional)
        if self.show_volume {
            let mute_icon = if self.is_muted {
                Icon::new(IconName::VolumeX).size(px(12.0))
            } else {
                Icon::new(IconName::Volume2).size(px(12.0))
            };
            let mute_label: SharedString = if self.is_muted {
                "Unmute".into()
            } else {
                "Mute".into()
            };

            controls = controls.child(
                Button::new(ElementId::from(SharedString::from(format!(
                    "{}-mute",
                    self.element_id
                ))))
                .icon(mute_icon)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(disabled)
                .accessibility_label(mute_label)
                .on_click(cx.listener(|this, _event, window, cx| {
                    this.handle_mute_toggle(window, cx);
                })),
            );

            // Volume slider: same disabled-state substitution as the seek
            // slider — swap the interactive `Slider` for a non-interactive
            // `ProgressIndicator` so disabled really means no input.
            let volume_fill = if self.is_muted { 0.0 } else { self.volume };
            if disabled {
                controls = controls.child(
                    div()
                        .w(px(60.0))
                        .flex_shrink_0()
                        .opacity(0.4)
                        .child(ProgressIndicator::new(volume_fill).height(px(4.0))),
                );
            } else {
                controls = controls.child(
                    div()
                        .w(px(60.0))
                        .flex_shrink_0()
                        .child(self.volume_slider.clone()),
                );
            }
        }

        container = container.child(controls);

        container
    }
}

#[cfg(test)]
mod tests {
    use super::{AudioPlayerView, AudioSource};
    use core::prelude::v1::test;

    #[test]
    fn test_format_time() {
        assert_eq!(AudioPlayerView::format_time(0.0).as_ref(), "00:00");
        assert_eq!(AudioPlayerView::format_time(61.0).as_ref(), "01:01");
        assert_eq!(AudioPlayerView::format_time(3599.0).as_ref(), "59:59");
        assert_eq!(AudioPlayerView::format_time(3600.0).as_ref(), "60:00");
    }

    #[test]
    fn test_volume_clamps() {
        // Verify the clamping logic directly
        let v = 1.5_f32.clamp(0.0, 1.0);
        assert_eq!(v, 1.0);
        let v = (-0.5_f32).clamp(0.0, 1.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_progress_clamps() {
        let p = 1.5_f32.clamp(0.0, 1.0);
        assert_eq!(p, 1.0);
        let p = (-0.1_f32).clamp(0.0, 1.0);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn test_seek_forward_clamps() {
        let duration = 60.0_f32;
        let current = 55.0_f32;
        let offset = 10.0_f32;
        let result = (current + offset).min(duration);
        assert_eq!(result, 60.0);
    }

    #[test]
    fn test_seek_backward_clamps() {
        let current = 5.0_f32;
        let offset = 10.0_f32;
        let result = (current - offset).max(0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_audio_source_variants() {
        let url = AudioSource::Url("https://example.com/audio.mp3".into());
        assert!(matches!(url, AudioSource::Url(_)));

        let b64 = AudioSource::Base64 {
            data: "SGVsbG8=".into(),
            mime_type: "audio/mp3".into(),
        };
        assert!(matches!(b64, AudioSource::Base64 { .. }));
    }

    #[test]
    fn test_format_time_large() {
        assert_eq!(AudioPlayerView::format_time(3661.0).as_ref(), "61:01");
    }

    #[test]
    fn test_format_time_fractional() {
        // Truncates, does not round
        assert_eq!(AudioPlayerView::format_time(61.9).as_ref(), "01:01");
    }

    #[test]
    fn test_seek_offset_default() {
        assert_eq!(super::DEFAULT_SEEK_OFFSET, 10.0);
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::AudioPlayerView;
    use crate::test_helpers::helpers::setup_test_window;

    #[gpui::test]
    async fn seek_forward_clamps_to_duration(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| AudioPlayerView::new(cx));
        handle.update_in(cx, |player, window, cx| {
            player.set_duration(60.0, cx);
            player.set_progress(55.0 / 60.0, cx); // 55s in
            player.handle_seek_forward(window, cx);
            // 55 + 10 = 65, clamped to 60
            assert_eq!(player.current_time(), 60.0);
        });
    }

    #[gpui::test]
    async fn seek_backward_clamps_to_zero(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| AudioPlayerView::new(cx));
        handle.update_in(cx, |player, window, cx| {
            player.set_duration(60.0, cx);
            player.set_progress(5.0 / 60.0, cx); // 5s in
            player.handle_seek_backward(window, cx);
            // 5 - 10 = -5, clamped to 0
            assert_eq!(player.current_time(), 0.0);
        });
    }

    #[gpui::test]
    async fn seek_forward_with_zero_duration(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| AudioPlayerView::new(cx));
        handle.update_in(cx, |player, window, cx| {
            // duration defaults to 0
            player.handle_seek_forward(window, cx);
            assert_eq!(player.current_time(), 0.0);
        });
    }

    #[gpui::test]
    async fn disabled_blocks_play_pause(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| AudioPlayerView::new(cx));
        handle.update_in(cx, |player, window, cx| {
            player.set_disabled(true, cx);
            player.handle_play_pause(window, cx);
            assert!(!player.is_playing());
        });
    }

    #[gpui::test]
    async fn disabled_blocks_mute_toggle(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| AudioPlayerView::new(cx));
        handle.update_in(cx, |player, window, cx| {
            player.set_disabled(true, cx);
            player.handle_mute_toggle(window, cx);
            assert!(!player.is_muted());
        });
    }

    #[gpui::test]
    async fn disabled_defaults_to_false(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| AudioPlayerView::new(cx));
        handle.update_in(cx, |player, _window, _cx| {
            assert!(!player.disabled);
        });
    }
}
