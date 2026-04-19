//! Speech input component — a button that toggles microphone recording.
//!
//! Captures audio via cpal, encodes to WAV, and fires callbacks for
//! transcription integration. Matches the web AI SDK Elements SpeechInput
//! component API (Firefox/Safari path: record + server-side transcription).
//!
//! # HIG alignment
//!
//! * `SpeechInputState::PermissionRequired` and `PermissionDenied`
//!   distinguish "never prompted" from "user denied" so the UI can render
//!   explanatory copy plus an "Open Privacy Settings" affordance.
//! * The listening pulse rings check
//!   `TahoeTheme::accessibility_mode.reduce_motion()` and fall back to a
//!   static red border indicator.
//! * The `Disabled` state renders the mic-slash glyph
//!   ([`IconName::MicOff`]) instead of sharing the idle microphone icon at
//!   reduced opacity.
//! * [`SpeechInputView::show_menu_bar_tip`] surfaces a first-use
//!   tooltip explaining the macOS 26 orange menu-bar dot.
//! * [`SpeechInputView::set_ai_disclosure`] renders an optional info
//!   affordance for transparency about AI processing.
//! * Elapsed recording time is rendered as `mm:ss` next to the
//!   button while `SpeechInputState::Listening`.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use gpui::prelude::*;
use gpui::{Animation, AnimationExt, App, ElementId, SharedString, Task, Window, div, px};

use crate::components::menus_and_actions::button::{Button, ButtonSize, ButtonVariant};
use crate::foundations::accessibility::{AccessibilityProps, AccessibilityRole, AccessibleExt};
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;

use super::audio_capture::{AudioCapture, AudioCaptureError, CapturedAudio, PermissionHint};
use crate::callback_types::OnStringChange;

/// Async handler that receives captured audio and returns an optional transcription.
///
/// The closure receives `&mut App` to extract any needed state before the async work
/// begins. The returned future must be `Send` — only move `Send`-safe data into it.
type AsyncAudioHandler = Box<
    dyn Fn(CapturedAudio, &mut App) -> Pin<Box<dyn Future<Output = Option<String>> + Send>>
        + 'static,
>;

/// Visual and operational state of the speech input component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechInputState {
    /// Ready to record. Shows microphone icon.
    Idle,
    /// Actively recording. Shows stop icon with pulsing animation.
    Listening,
    /// Recording stopped, audio is being processed. Shows spinner.
    Processing,
    /// Microphone permission has never been requested. The host should
    /// trigger `AVCaptureDevice.requestAccess(for: .audio, …)` when the
    /// user activates the button.
    PermissionRequired,
    /// Microphone access was denied or restricted. The UI surfaces
    /// explanatory copy and an "Open Privacy Settings" affordance.
    PermissionDenied,
    /// Microphone hardware is unavailable or the host app has intentionally
    /// disabled the feature.
    Disabled,
}

impl SpeechInputState {
    /// Returns true if the component is in a permission-blocked state that
    /// prevents recording from starting.
    pub fn is_permission_blocked(self) -> bool {
        matches!(self, Self::PermissionRequired | Self::PermissionDenied)
    }
}

/// A button that toggles microphone recording for speech-to-text input.
///
/// # Setter conventions
///
/// Property setters that affect rendering (e.g. `set_size`, `set_lang`) take
/// `&mut Context<Self>` and call `cx.notify()`. Callback setters (e.g.
/// `set_on_audio_recorded`) do not, since registering a callback alone does
/// not change the visual state.
///
/// # Usage
///
/// ```ignore
/// let speech = cx.new(|cx| {
///     let mut view = SpeechInputView::new(cx);
///     view.set_on_audio_recorded(|audio, _window, _cx| {
///         println!("Recorded {:.1}s of audio ({} bytes)",
///             audio.duration_secs, audio.data.len());
///     });
///     view
/// });
/// ```
#[allow(clippy::type_complexity)]
pub struct SpeechInputView {
    element_id: ElementId,
    state: SpeechInputState,
    /// Audio level from the capture device (0.0–1.0). Updated by the timer
    /// task during recording. Retained for planned audio-reactive ring animation.
    audio_level: f32,
    /// Elapsed recording time in seconds. Updated by the timer task during
    /// recording. Rendered as `mm:ss` while `Listening`.
    elapsed_secs: f32,
    lang: SharedString,
    size: ButtonSize,
    idle_variant: ButtonVariant,
    /// Host-supplied microphone authorization hint. See [`PermissionHint`]
    /// for the mapping to `AVCaptureDevice.authorizationStatus`.
    permission: PermissionHint,
    /// Optional transparency copy displayed next to the button. When set,
    /// renders an info glyph + tooltip disclosing how audio is processed.
    ai_disclosure: Option<SharedString>,
    /// Whether the first-use tooltip about the macOS orange menu-bar
    /// microphone indicator has been dismissed.
    menu_bar_tip_visible: bool,
    on_transcription_change: OnStringChange,
    on_audio_recorded: Option<Box<dyn Fn(CapturedAudio, &mut Window, &mut App) + 'static>>,
    on_audio_recorded_async: Option<AsyncAudioHandler>,
    on_request_permission: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_open_privacy_settings: Option<Box<dyn Fn(&mut Window, &mut App) + 'static>>,
    audio_capture: Option<AudioCapture>,
    timer_task: Option<Task<()>>,
    processing_task: Option<Task<()>>,
}

impl SpeechInputView {
    /// Default button size for new instances.
    pub const DEFAULT_SIZE: ButtonSize = ButtonSize::Icon;
    /// Default button variant for the idle/disabled states.
    pub const DEFAULT_IDLE_VARIANT: ButtonVariant = ButtonVariant::Primary;

    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            element_id: next_element_id("speech-input"),
            state: SpeechInputState::Idle,
            audio_level: 0.0,
            elapsed_secs: 0.0,
            lang: "en-US".into(),
            size: Self::DEFAULT_SIZE,
            idle_variant: Self::DEFAULT_IDLE_VARIANT,
            permission: PermissionHint::Unknown,
            ai_disclosure: None,
            menu_bar_tip_visible: false,
            on_transcription_change: None,
            on_audio_recorded: None,
            on_audio_recorded_async: None,
            on_request_permission: None,
            on_open_privacy_settings: None,
            audio_capture: None,
            timer_task: None,
            processing_task: None,
        }
    }

    /// Set the host-supplied microphone authorization hint. When the host
    /// observes `AVCaptureDevice.authorizationStatus` as `.notDetermined`
    /// or `.denied`, this lets the view render the matching explanatory
    /// state without relying on a failed capture attempt to discover it.
    pub fn set_permission(&mut self, permission: PermissionHint, cx: &mut Context<Self>) {
        self.permission = permission;
        match permission {
            PermissionHint::NotDetermined if self.state == SpeechInputState::Idle => {
                self.state = SpeechInputState::PermissionRequired;
            }
            PermissionHint::Denied if !matches!(self.state, SpeechInputState::Listening) => {
                self.stop_internal();
                self.state = SpeechInputState::PermissionDenied;
            }
            PermissionHint::Authorized
                if matches!(
                    self.state,
                    SpeechInputState::PermissionRequired | SpeechInputState::PermissionDenied
                ) =>
            {
                self.state = SpeechInputState::Idle;
            }
            _ => {}
        }
        cx.notify();
    }

    /// Returns the current permission hint.
    pub fn permission(&self) -> PermissionHint {
        self.permission
    }

    /// Set optional transparency copy disclosing how audio is used by AI.
    /// When set, a tooltip-bearing info glyph is rendered next to the button.
    pub fn set_ai_disclosure(
        &mut self,
        disclosure: Option<impl Into<SharedString>>,
        cx: &mut Context<Self>,
    ) {
        self.ai_disclosure = disclosure.map(Into::into);
        cx.notify();
    }

    /// Register a callback invoked when the button is activated while in
    /// [`SpeechInputState::PermissionRequired`]. The host should call
    /// `AVCaptureDevice.requestAccess(for: .audio, …)` from this handler
    /// and update the view via [`Self::set_permission`] on completion.
    pub fn set_on_request_permission(&mut self, handler: impl Fn(&mut Window, &mut App) + 'static) {
        self.on_request_permission = Some(Box::new(handler));
    }

    /// Register a callback for the "Open Privacy Settings" action shown
    /// while in [`SpeechInputState::PermissionDenied`]. Hosts open
    /// `x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone`
    /// via `NSWorkspace.shared.open(…)` from this handler.
    pub fn set_on_open_privacy_settings(
        &mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) {
        self.on_open_privacy_settings = Some(Box::new(handler));
    }

    /// Show or hide the first-use tooltip explaining the macOS 26 orange
    /// menu-bar microphone-in-use indicator. Hosts call
    /// `show_menu_bar_tip(true, …)` on the first record and persist the
    /// dismissal in `UserDefaults` when `on_menu_bar_tip_dismissed` fires.
    pub fn show_menu_bar_tip(&mut self, visible: bool, cx: &mut Context<Self>) {
        self.menu_bar_tip_visible = visible;
        cx.notify();
    }

    /// Returns whether the menu-bar orange-dot tip is currently visible.
    pub fn menu_bar_tip_visible(&self) -> bool {
        self.menu_bar_tip_visible
    }

    /// Returns the current elapsed recording time in seconds. Only
    /// meaningful while [`SpeechInputState::Listening`].
    pub fn elapsed_secs(&self) -> f32 {
        self.elapsed_secs
    }

    /// Returns the current state.
    pub fn state(&self) -> SpeechInputState {
        self.state
    }

    /// Set the language hint for transcription (e.g. `"en-US"`).
    pub fn set_lang(&mut self, lang: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.lang = lang.into();
        cx.notify();
    }

    /// Returns the configured language.
    pub fn lang(&self) -> &SharedString {
        &self.lang
    }

    /// Set a callback that fires when transcription text is available.
    ///
    /// The consumer should call this from their `on_audio_recorded` handler
    /// after receiving the transcribed text from a transcription service.
    pub fn set_on_transcription_change(
        &mut self,
        handler: impl Fn(String, &mut Window, &mut App) + 'static,
    ) {
        self.on_transcription_change = Some(Box::new(handler));
    }

    /// Set a callback that fires when recording stops with the captured audio.
    ///
    /// The receiver should send the `CapturedAudio` to a transcription service
    /// (e.g. via `TranscriptionModel::transcribe`).
    pub fn set_on_audio_recorded(
        &mut self,
        handler: impl Fn(CapturedAudio, &mut Window, &mut App) + 'static,
    ) {
        self.on_audio_recorded = Some(Box::new(handler));
    }

    /// Set an async callback for audio recording with automatic state transitions.
    ///
    /// When recording stops, the handler receives the captured audio and an `&mut App`,
    /// and returns a future that resolves to `Option<String>`. The component
    /// automatically transitions from `Processing` → `Idle` when the future resolves,
    /// and fires `on_transcription_change` if the result is `Some`. Returns `None`
    /// to skip the transcription callback (e.g. when the audio was too short).
    ///
    /// This is analogous to the web AI SDK Elements `onAudioRecorded` prop, but
    /// the Rust API may intentionally resolve to `None` instead of always producing
    /// a transcription string. Takes precedence over `set_on_audio_recorded` if both
    /// are set.
    ///
    /// # `Send` constraint
    ///
    /// The closure receives `&mut App` so you can read entity state or configuration
    /// before the async work begins. However, the returned `Future` must be `Send` —
    /// extract any needed data into local variables in the closure body and move only
    /// `Send`-safe values into `Box::pin(async move { ... })`. Do **not** capture
    /// GPUI entity handles or `App` references in the future itself.
    ///
    /// Unlike the synchronous `set_on_audio_recorded`, this callback does not receive
    /// `&mut Window` — window access is not available in the async setup context.
    pub fn set_on_audio_recorded_async(
        &mut self,
        handler: impl Fn(
            CapturedAudio,
            &mut App,
        ) -> Pin<Box<dyn Future<Output = Option<String>> + Send>>
        + 'static,
    ) {
        self.on_audio_recorded_async = Some(Box::new(handler));
    }

    /// Set the button size (default: `ButtonSize::Icon`).
    pub fn set_size(&mut self, size: ButtonSize, cx: &mut Context<Self>) {
        self.size = size;
        cx.notify();
    }

    /// Returns the configured button size.
    pub fn size(&self) -> ButtonSize {
        self.size
    }

    /// Set the button variant used in the idle state (default: `ButtonVariant::Primary`).
    pub fn set_idle_variant(&mut self, variant: ButtonVariant, cx: &mut Context<Self>) {
        self.idle_variant = variant;
        cx.notify();
    }

    /// Returns the configured idle variant.
    pub fn idle_variant(&self) -> ButtonVariant {
        self.idle_variant
    }

    /// Fire the transcription change callback (call from app code after transcription completes).
    pub fn notify_transcription(&self, text: String, window: &mut Window, cx: &mut App) {
        if let Some(ref handler) = self.on_transcription_change {
            handler(text, window, cx);
        }
    }

    /// Force the component into the disabled state.
    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        if disabled {
            self.stop_internal();
            self.state = SpeechInputState::Disabled;
        } else if matches!(
            self.state,
            SpeechInputState::Disabled
                | SpeechInputState::PermissionRequired
                | SpeechInputState::PermissionDenied
        ) {
            self.state = SpeechInputState::Idle;
        }
        cx.notify();
    }

    /// Transition back to idle (call after processing is complete).
    pub fn finish_processing(&mut self, cx: &mut Context<Self>) {
        if self.state == SpeechInputState::Processing {
            self.state = SpeechInputState::Idle;
            cx.notify();
        }
    }

    fn start_recording(&mut self, cx: &mut Context<Self>) {
        // Stream-time errors (USB unplugged, sample-format change) are
        // forwarded by [`AudioCapture`] through the optional `on_stream_error`
        // callback. The view does not wire an internal callback here
        // because marshalling a cpal thread error back into
        // `Context<Self>` requires a bounded channel the consumer should
        // own. Consumers that care call
        // `AudioCapture::start_with_permission` directly, or relay via
        // their own channel after observing state transitions.

        match AudioCapture::start_with_permission(self.permission, None) {
            Ok(capture) => {
                self.audio_capture = Some(capture);
                self.state = SpeechInputState::Listening;
                self.elapsed_secs = 0.0;
                self.audio_level = 0.0;
                self.start_timer(cx);
                cx.notify();
            }
            Err(err) => {
                self.state = match err {
                    AudioCaptureError::NotDetermined => SpeechInputState::PermissionRequired,
                    AudioCaptureError::PermissionDenied => SpeechInputState::PermissionDenied,
                    AudioCaptureError::NoDevice | AudioCaptureError::StreamError(_) => {
                        SpeechInputState::Disabled
                    }
                };
                cx.notify();
            }
        }
    }

    fn stop_recording(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.timer_task = None;

        if let Some(capture) = self.audio_capture.take() {
            let audio = capture.stop();
            self.state = SpeechInputState::Processing;
            self.audio_level = 0.0;
            cx.notify();

            // Async callback takes precedence: auto-transitions Processing → Idle.
            if let Some(ref handler) = self.on_audio_recorded_async {
                let future = handler(audio, &mut *cx);
                self.processing_task = Some(cx.spawn_in(window, async move |this, cx| {
                    let result = future.await;
                    let _ = this.update_in(cx, |this: &mut Self, window, cx| {
                        // Guard: only transition if still Processing (the component
                        // may have been disabled or stopped while the future ran).
                        if this.state == SpeechInputState::Processing {
                            this.state = SpeechInputState::Idle;
                        }
                        if let Some(text) = result {
                            this.notify_transcription(text, window, &mut *cx);
                        }
                        cx.notify();
                    });
                }));
            } else if let Some(ref handler) = self.on_audio_recorded {
                handler(audio, window, &mut *cx);
            }
        } else {
            self.state = SpeechInputState::Idle;
            cx.notify();
        }
    }

    fn stop_internal(&mut self) {
        self.timer_task = None;
        self.processing_task = None;
        self.audio_capture = None;
        self.audio_level = 0.0;
        self.elapsed_secs = 0.0;
    }

    fn start_timer(&mut self, cx: &mut Context<Self>) {
        // Tracks elapsed_secs and audio_level. The elapsed display is
        // rendered, so notify the UI once per whole-second transition
        // (avoiding 20 Hz redraws while still keeping the `mm:ss` text
        // accurate).
        self.timer_task = Some(cx.spawn(async |this, cx| {
            let interval = Duration::from_millis(50);
            loop {
                cx.background_executor().timer(interval).await;
                let Ok(()) = this.update(cx, |this: &mut Self, cx| {
                    if this.state != SpeechInputState::Listening {
                        this.timer_task = None;
                        return;
                    }
                    let prev_secs = this.elapsed_secs as u64;
                    this.elapsed_secs += 0.05;
                    if let Some(ref capture) = this.audio_capture {
                        this.audio_level = capture.current_level();
                    }
                    if (this.elapsed_secs as u64) != prev_secs {
                        cx.notify();
                    }
                }) else {
                    break;
                };
            }
        }));
    }

    /// Returns the icon for a given state.
    ///
    /// `Disabled` uses [`IconName::MicOff`] so disabled and idle states are
    /// distinguishable by shape, not just opacity.
    fn icon_for_state(state: SpeechInputState) -> IconName {
        match state {
            SpeechInputState::Idle => IconName::Mic,
            SpeechInputState::Listening => IconName::Square,
            SpeechInputState::Processing => IconName::Loader,
            SpeechInputState::PermissionRequired => IconName::Mic,
            SpeechInputState::PermissionDenied | SpeechInputState::Disabled => IconName::MicOff,
        }
    }

    /// Returns the VoiceOver label announcing the current state.
    fn accessibility_label_for_state(state: SpeechInputState) -> &'static str {
        match state {
            SpeechInputState::Idle => "Start recording",
            SpeechInputState::Listening => "Stop recording",
            SpeechInputState::Processing => "Transcribing",
            SpeechInputState::PermissionRequired => "Microphone, request access",
            SpeechInputState::PermissionDenied => "Microphone, access denied",
            SpeechInputState::Disabled => "Microphone, unavailable",
        }
    }

    /// Returns the button variant for a given state, using the configured idle variant.
    fn variant_for_state(state: SpeechInputState, idle_variant: ButtonVariant) -> ButtonVariant {
        match state {
            SpeechInputState::Idle
            | SpeechInputState::Disabled
            | SpeechInputState::PermissionRequired
            | SpeechInputState::PermissionDenied => idle_variant,
            SpeechInputState::Listening => ButtonVariant::Destructive,
            SpeechInputState::Processing => ButtonVariant::Outline,
        }
    }

    fn format_elapsed_mmss(secs: f32) -> SharedString {
        let total = secs as u64;
        let m = total / 60;
        let s = total % 60;
        SharedString::from(format!("{m:02}:{s:02}"))
    }
}

impl Render for SpeechInputView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let state = self.state;
        let listening = state == SpeechInputState::Listening;
        let processing = state == SpeechInputState::Processing;
        let disabled = state == SpeechInputState::Disabled;
        let reduce_motion = theme.accessibility_mode.reduce_motion();

        let icon_name = Self::icon_for_state(state);
        let btn_variant = Self::variant_for_state(state, self.idle_variant);

        let icon = Icon::new(icon_name).size(theme.icon_size_inline);

        let mut button = Button::new(ElementId::from(SharedString::from(format!(
            "{}-rec",
            self.element_id
        ))))
        .icon(icon)
        .variant(btn_variant)
        .size(self.size)
        .round(true)
        .accessibility_label(Self::accessibility_label_for_state(state));

        // Activation wiring depends on state:
        //   Idle → start recording
        //   Listening → stop recording
        //   PermissionRequired → trigger host's requestAccess
        //   PermissionDenied → open Privacy & Security settings
        //   Disabled / Processing → no-op
        if !disabled && !processing {
            button = button.on_click(cx.listener(|this, _event, window, cx| match this.state {
                SpeechInputState::Idle => this.start_recording(cx),
                SpeechInputState::Listening => this.stop_recording(window, cx),
                SpeechInputState::PermissionRequired => {
                    if let Some(ref handler) = this.on_request_permission {
                        handler(window, &mut *cx);
                    }
                }
                SpeechInputState::PermissionDenied => {
                    if let Some(ref handler) = this.on_open_privacy_settings {
                        handler(window, &mut *cx);
                    }
                }
                _ => {}
            }));
        }

        // Wrap button in a relative container so the pulse rings (or the
        // Reduce Motion static fallback) can be absolute-positioned.
        let mut btn_wrapper = div().relative().flex_shrink_0();

        if listening {
            if reduce_motion {
                // Reduce Motion substitute — a single solid red border stays
                // visible while recording so the active state is communicated
                // without oscillating motion. Matches the HIG guidance that
                // motion must not be the sole carrier of state.
                let static_id = ElementId::from(SharedString::from(format!(
                    "{}-ring-static",
                    self.element_id
                )));
                btn_wrapper = btn_wrapper.child(
                    div()
                        .id(static_id)
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .rounded_full()
                        .border_2()
                        .border_color(theme.error),
                );
            } else {
                // Animated pulse rings (3 concentric rings with staggered
                // animation). Each ring expands outward via negative margins
                // while fading, creating an expanding-ripple effect.
                let ring_color = theme.error.opacity(0.3);
                for i in 0..3 {
                    let delay = i as f32 * 0.25; // stagger: ring 0 starts first
                    let ring_id = ElementId::from(SharedString::from(format!(
                        "{}-ring-{}",
                        self.element_id, i
                    )));
                    // Maximum outward expansion in pixels per ring.
                    let max_expand = 4.0 + i as f32 * 3.0;
                    btn_wrapper = btn_wrapper.child(
                        div()
                            .id(ring_id.clone())
                            .absolute()
                            .top_0()
                            .left_0()
                            .size_full()
                            .rounded_full()
                            .border_2()
                            .border_color(ring_color)
                            .with_animation(
                                ring_id,
                                Animation::new(Duration::from_secs(2)).repeat(),
                                move |el, delta| {
                                    let t = (delta - delay).clamp(0.0, 1.0);
                                    let opacity = 0.6 * (1.0 - t);
                                    let expand = t * max_expand;
                                    el.opacity(opacity)
                                        .mt(px(-expand))
                                        .mb(px(-expand))
                                        .ml(px(-expand))
                                        .mr(px(-expand))
                                },
                            ),
                    );
                }
            }
        }

        btn_wrapper = btn_wrapper.child(button);

        let mut container = div()
            .id(self.element_id.clone())
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .child(btn_wrapper);

        // Render elapsed time next to the button while Listening so users —
        // including VoiceOver users reading the live text — can gauge how
        // long they've been recording.
        if listening {
            container = container.child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .with_accessibility(
                        &AccessibilityProps::new()
                            .role(AccessibilityRole::StaticText)
                            .label(SharedString::from(format!(
                                "Recording, {} seconds",
                                self.elapsed_secs as u64
                            ))),
                    )
                    .child(Self::format_elapsed_mmss(self.elapsed_secs)),
            );
        }

        // Inline explanatory copy for permission states. The button's
        // accessibility label already announces the state; this row gives
        // sighted users the same cue.
        match state {
            SpeechInputState::PermissionRequired => {
                container = container.child(
                    div()
                        .text_style(TextStyle::Caption1, theme)
                        .text_color(theme.text_muted)
                        .child("Microphone access is needed to record your voice."),
                );
            }
            SpeechInputState::PermissionDenied => {
                container = container.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(theme.spacing_xs)
                        .text_style(TextStyle::Caption1, theme)
                        .text_color(theme.text_muted)
                        .child("Microphone access is denied.")
                        .child(
                            div()
                                .text_color(theme.accent)
                                .cursor_pointer()
                                .child("Tap the microphone button to open Privacy Settings."),
                        ),
                );
            }
            _ => {}
        }

        // Optional AI transparency disclosure. Rendered as a subdued info
        // glyph + caption text so consumers can document how audio is
        // processed (on-device vs. server, provider).
        if let Some(ref disclosure) = self.ai_disclosure {
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(
                        Icon::new(IconName::AlertTriangle)
                            .size(px(10.0))
                            .color(theme.text_muted),
                    )
                    .child(disclosure.clone()),
            );
        }

        // First-use tooltip explaining the macOS 26 orange menu-bar
        // microphone indicator so first-time recorders don't mistake it for
        // an alert.
        if self.menu_bar_tip_visible {
            container = container.child(
                div()
                    .flex()
                    .items_center()
                    .gap(theme.spacing_xs)
                    .px(theme.spacing_sm)
                    .py(theme.spacing_xs)
                    .rounded(theme.radius_md)
                    .bg(theme.surface)
                    .border_1()
                    .border_color(theme.border)
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(
                        "While recording, macOS shows an orange dot in the menu bar \
                         to indicate microphone activity.",
                    ),
            );
        }

        if disabled {
            container = container.opacity(0.5).cursor_default();
        }

        container
    }
}

impl Drop for SpeechInputView {
    fn drop(&mut self) {
        // Ensure we release the audio stream on component teardown.
        self.stop_internal();
    }
}

#[cfg(test)]
mod tests {
    use super::{SpeechInputState, SpeechInputView};
    use crate::components::menus_and_actions::button::{ButtonSize, ButtonVariant};
    use crate::foundations::icons::IconName;
    use core::prelude::v1::test;

    #[test]
    fn format_elapsed_zero() {
        assert_eq!(SpeechInputView::format_elapsed_mmss(0.0).as_ref(), "00:00");
    }

    #[test]
    fn format_elapsed_seconds() {
        assert_eq!(SpeechInputView::format_elapsed_mmss(5.0).as_ref(), "00:05");
        assert_eq!(SpeechInputView::format_elapsed_mmss(59.9).as_ref(), "00:59");
    }

    #[test]
    fn format_elapsed_minutes() {
        assert_eq!(SpeechInputView::format_elapsed_mmss(60.0).as_ref(), "01:00");
        assert_eq!(
            SpeechInputView::format_elapsed_mmss(125.0).as_ref(),
            "02:05"
        );
    }

    #[test]
    fn state_enum_equality() {
        assert_eq!(SpeechInputState::Idle, SpeechInputState::Idle);
        assert_ne!(SpeechInputState::Idle, SpeechInputState::Listening);
    }

    #[test]
    fn default_size_is_icon() {
        assert_eq!(SpeechInputView::DEFAULT_SIZE, ButtonSize::Icon);
    }

    #[test]
    fn default_idle_variant_is_primary() {
        assert_eq!(
            SpeechInputView::DEFAULT_IDLE_VARIANT,
            ButtonVariant::Primary
        );
    }

    #[test]
    fn state_to_icon_mapping() {
        // Uses the real associated function from SpeechInputView.
        assert_eq!(
            SpeechInputView::icon_for_state(SpeechInputState::Idle),
            IconName::Mic
        );
        // Disabled uses the mic-slash glyph so it differs from Idle by
        // shape, not just opacity.
        assert_eq!(
            SpeechInputView::icon_for_state(SpeechInputState::Disabled),
            IconName::MicOff
        );
        assert_eq!(
            SpeechInputView::icon_for_state(SpeechInputState::PermissionDenied),
            IconName::MicOff
        );
        assert_eq!(
            SpeechInputView::icon_for_state(SpeechInputState::PermissionRequired),
            IconName::Mic
        );
        assert_eq!(
            SpeechInputView::icon_for_state(SpeechInputState::Listening),
            IconName::Square
        );
        assert_eq!(
            SpeechInputView::icon_for_state(SpeechInputState::Processing),
            IconName::Loader
        );
    }

    #[test]
    fn permission_blocked_states() {
        assert!(SpeechInputState::PermissionRequired.is_permission_blocked());
        assert!(SpeechInputState::PermissionDenied.is_permission_blocked());
        assert!(!SpeechInputState::Idle.is_permission_blocked());
        assert!(!SpeechInputState::Listening.is_permission_blocked());
        assert!(!SpeechInputState::Processing.is_permission_blocked());
        assert!(!SpeechInputState::Disabled.is_permission_blocked());
    }

    #[test]
    fn accessibility_label_for_states() {
        assert_eq!(
            SpeechInputView::accessibility_label_for_state(SpeechInputState::Idle),
            "Start recording"
        );
        assert_eq!(
            SpeechInputView::accessibility_label_for_state(SpeechInputState::Listening),
            "Stop recording"
        );
        assert_eq!(
            SpeechInputView::accessibility_label_for_state(SpeechInputState::PermissionDenied),
            "Microphone, access denied"
        );
        assert_eq!(
            SpeechInputView::accessibility_label_for_state(SpeechInputState::Disabled),
            "Microphone, unavailable"
        );
    }

    #[test]
    fn state_to_variant_mapping_primary() {
        let idle = ButtonVariant::Primary;
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Idle, idle),
            ButtonVariant::Primary
        );
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Listening, idle),
            ButtonVariant::Destructive
        );
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Processing, idle),
            ButtonVariant::Outline
        );
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Disabled, idle),
            ButtonVariant::Primary
        );
    }

    #[test]
    fn state_to_variant_mapping_custom_idle() {
        // Verify configurable idle variant propagates to Idle and Disabled states.
        let idle = ButtonVariant::Ghost;
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Idle, idle),
            ButtonVariant::Ghost
        );
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Disabled, idle),
            ButtonVariant::Ghost
        );
        // Listening and Processing remain fixed regardless of idle_variant.
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Listening, idle),
            ButtonVariant::Destructive
        );
        assert_eq!(
            SpeechInputView::variant_for_state(SpeechInputState::Processing, idle),
            ButtonVariant::Outline
        );
    }
}

#[cfg(test)]
mod gpui_tests {
    use super::{CapturedAudio, SpeechInputState, SpeechInputView};
    use crate::test_helpers::helpers::setup_test_window;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[gpui::test]
    async fn defaults_match_constants(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| SpeechInputView::new(cx));
        handle.update_in(cx, |view, _window, _cx| {
            assert_eq!(view.size(), SpeechInputView::DEFAULT_SIZE);
            assert_eq!(view.idle_variant(), SpeechInputView::DEFAULT_IDLE_VARIANT);
            assert_eq!(view.state(), SpeechInputState::Idle);
        });
    }

    /// Helper to create a minimal CapturedAudio for testing.
    fn test_audio() -> CapturedAudio {
        CapturedAudio {
            data: vec![],
            mime_type: "audio/wav",
            sample_rate: 16000,
            duration_secs: 1.0,
        }
    }

    #[gpui::test]
    async fn async_callback_transitions_and_fires_transcription(cx: &mut gpui::TestAppContext) {
        let transcription = Rc::new(RefCell::new(None::<String>));
        let transcription_clone = transcription.clone();

        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            let mut view = SpeechInputView::new(cx);
            view.set_on_audio_recorded_async(|_audio, _app| {
                Box::pin(async move { Some("hello world".to_string()) })
            });
            view.set_on_transcription_change(move |text, _window, _cx| {
                *transcription_clone.borrow_mut() = Some(text);
            });
            view
        });

        // Simulate: set state to Processing and spawn the async task manually.
        handle.update_in(cx, |view, window, cx| {
            let audio = test_audio();
            view.state = SpeechInputState::Processing;
            if let Some(ref handler) = view.on_audio_recorded_async {
                let future = handler(audio, &mut *cx);
                view.processing_task = Some(cx.spawn_in(window, async move |this, cx| {
                    let result = future.await;
                    let _ = this.update_in(cx, |this: &mut SpeechInputView, window, cx| {
                        if this.state == SpeechInputState::Processing {
                            this.state = SpeechInputState::Idle;
                        }
                        if let Some(text) = result {
                            this.notify_transcription(text, window, &mut *cx);
                        }
                        cx.notify();
                    });
                }));
            }
        });

        // Run pending async tasks.
        cx.run_until_parked();

        handle.update_in(cx, |view, _window, _cx| {
            assert_eq!(view.state(), SpeechInputState::Idle);
        });
        assert_eq!(*transcription.borrow(), Some("hello world".to_string()));
    }

    #[gpui::test]
    async fn async_callback_none_skips_transcription(cx: &mut gpui::TestAppContext) {
        let fired = Rc::new(Cell::new(false));
        let fired_clone = fired.clone();

        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            let mut view = SpeechInputView::new(cx);
            view.set_on_audio_recorded_async(|_audio, _app| Box::pin(async move { None }));
            view.set_on_transcription_change(move |_text, _window, _cx| {
                fired_clone.set(true);
            });
            view
        });

        handle.update_in(cx, |view, window, cx| {
            let audio = test_audio();
            view.state = SpeechInputState::Processing;
            if let Some(ref handler) = view.on_audio_recorded_async {
                let future = handler(audio, &mut *cx);
                view.processing_task = Some(cx.spawn_in(window, async move |this, cx| {
                    let result = future.await;
                    let _ = this.update_in(cx, |this: &mut SpeechInputView, window, cx| {
                        if this.state == SpeechInputState::Processing {
                            this.state = SpeechInputState::Idle;
                        }
                        if let Some(text) = result {
                            this.notify_transcription(text, window, &mut *cx);
                        }
                        cx.notify();
                    });
                }));
            }
        });

        cx.run_until_parked();

        handle.update_in(cx, |view, _window, _cx| {
            assert_eq!(view.state(), SpeechInputState::Idle);
        });
        // Transcription callback should NOT have fired.
        assert!(!fired.get());
    }

    #[gpui::test]
    async fn async_callback_takes_precedence_over_sync(cx: &mut gpui::TestAppContext) {
        let async_fired = Rc::new(Cell::new(false));
        let sync_fired = Rc::new(Cell::new(false));
        let async_clone = async_fired.clone();
        let sync_clone = sync_fired.clone();

        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            let mut view = SpeechInputView::new(cx);
            view.set_on_audio_recorded_async(move |_audio, _app| {
                async_clone.set(true);
                Box::pin(async move { None })
            });
            view.set_on_audio_recorded(move |_audio, _window, _cx| {
                sync_clone.set(true);
            });
            view
        });

        handle.update_in(cx, |view, window, cx| {
            let audio = test_audio();
            view.state = SpeechInputState::Processing;
            // Reproduce the precedence logic from stop_recording.
            if let Some(ref handler) = view.on_audio_recorded_async {
                let future = handler(audio, &mut *cx);
                view.processing_task = Some(cx.spawn_in(window, async move |this, cx| {
                    let result = future.await;
                    let _ = this.update_in(cx, |this: &mut SpeechInputView, window, cx| {
                        if this.state == SpeechInputState::Processing {
                            this.state = SpeechInputState::Idle;
                        }
                        if let Some(text) = result {
                            this.notify_transcription(text, window, &mut *cx);
                        }
                        cx.notify();
                    });
                }));
            } else if let Some(ref handler) = view.on_audio_recorded {
                handler(audio, window, &mut *cx);
            }
        });

        cx.run_until_parked();

        assert!(async_fired.get(), "async callback should have fired");
        assert!(!sync_fired.get(), "sync callback should NOT have fired");
    }

    #[gpui::test]
    async fn disable_during_processing_prevents_idle_transition(cx: &mut gpui::TestAppContext) {
        let (handle, cx) = setup_test_window(cx, |_window, cx| {
            let mut view = SpeechInputView::new(cx);
            view.set_on_audio_recorded_async(|_audio, _app| {
                Box::pin(async move { Some("late result".to_string()) })
            });
            view
        });

        handle.update_in(cx, |view, window, cx| {
            let audio = test_audio();
            view.state = SpeechInputState::Processing;
            if let Some(ref handler) = view.on_audio_recorded_async {
                let future = handler(audio, &mut *cx);
                view.processing_task = Some(cx.spawn_in(window, async move |this, cx| {
                    let result = future.await;
                    let _ = this.update_in(cx, |this: &mut SpeechInputView, window, cx| {
                        if this.state == SpeechInputState::Processing {
                            this.state = SpeechInputState::Idle;
                        }
                        if let Some(text) = result {
                            this.notify_transcription(text, window, &mut *cx);
                        }
                        cx.notify();
                    });
                }));
            }
            // Disable before the async task completes.
            view.set_disabled(true, cx);
        });

        cx.run_until_parked();

        handle.update_in(cx, |view, _window, _cx| {
            // Should remain Disabled, not transition to Idle.
            assert_eq!(view.state(), SpeechInputState::Disabled);
        });
    }
}
