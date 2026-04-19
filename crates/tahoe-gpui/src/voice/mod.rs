//! Voice-related components (behind `voice` feature flag).
//!
//! Audio playback, speech input, transcription display, persona, and device selectors.
//!
//! # Audit notes (issue #148)
//!
//! The HIG Voice audit for macOS 26 Tahoe (`df49b9cd/ai-sdk-rust#148`)
//! surfaced a set of findings and open questions that are answered below
//! so consumers can understand the crate's contract without re-reading the
//! issue.
//!
//! ## Open question answers
//!
//! 1. **Permission prompt timing** — The host app owns the
//!    `AVCaptureDevice.requestAccess(for: .audio, …)` lifecycle. Pass the
//!    observed status into [`SpeechInputView::set_permission`] /
//!    [`MicSelectorView::set_permission`] so the view can render the
//!    correct state; wire [`SpeechInputView::set_on_request_permission`]
//!    to invoke `requestAccess` from the button-activation handler.
//! 2. **Reduce Motion API** — Read via
//!    `TahoeTheme::accessibility_mode.reduce_motion()`. The theme is the
//!    single source of truth; consumers refresh it from
//!    `NSAccessibilityReduceMotionEnabled` at window focus.
//! 3. **Live-region equivalent** — GPUI does not yet expose an AX API.
//!    Components mark streaming indicators with
//!    [`foundations::accessibility::AccessibilityProps`] via
//!    [`foundations::accessibility::AccessibleExt::with_accessibility`];
//!    the label becomes a live-region announcement once GPUI lands the
//!    upstream AX tree.
//! 4. **Hypothesis tokens** — [`TranscriptionSegment::is_hypothesis`]
//!    marks unstable streaming tokens; the view renders them muted and
//!    italic, and does not participate in click-to-seek. Consumers set
//!    this from the AI SDK streaming transcription API (hypothesis vs.
//!    final tokens).
//! 5. **Liquid Glass HUD** — The floating voice pill (mic/voice selectors
//!    and future persona HUDs) uses `glass_surface` as an inline surface,
//!    not a floating `NSWindow`. Consumers that need window-level
//!    overlays compose this view inside a borderless window themselves.
//! 6. **Now Playing** — [`AudioPlayerView`] delegates playback entirely
//!    to the host; it does not set `MPNowPlayingInfoCenter` itself. Use
//!    [`AudioPlayerView::set_on_interruption_began`] /
//!    [`AudioPlayerView::set_on_interruption_ended`] to forward
//!    `AVAudioSessionInterruptionNotification` back into the UI.
//!
//! ## TODOs filed from the audit
//!
//! * **F17 (Siri):** investigate `INStartCallIntent` (or a custom
//!   `StartVoiceRecording` intent under the `Create` category) so users
//!   can trigger recording via Shortcuts without opening the app. Blocked
//!   on the permission-state machine being stable (F2 complete).
//! * **F18 (Siri vocabulary):** when the Siri intent lands, register the
//!   [`VoiceOption::localized_name`] and `siri_vocabulary` fields so
//!   "Hey Siri, use Echo voice" works.
//! * **F20 (Scottish flag):** no stable Unicode ZWJ sequence for Scotland
//!   renders across macOS's bundled fonts. Re-evaluate when Apple adds
//!   support for `🏴󠁧󠁢󠁳󠁣󠁴󠁿`.

use gpui::SharedString;

mod audio_capture;
pub mod audio_player;
pub mod mic_selector;
pub mod persona;
pub mod speech_input;
pub mod transcription;
pub mod voice_selector;

/// An audio input device.
#[derive(Clone, Debug)]
pub struct AudioDevice {
    pub id: String,
    pub name: SharedString,
}

pub use audio_capture::{
    AudioCapture, AudioCaptureError, CapturedAudio, PermissionHint, StreamErrorCallback,
    default_input_device_name, enumerate_input_devices,
};
pub use audio_player::{AudioPlayerView, AudioSource};
pub use mic_selector::{MicPermission, MicSelectorState, MicSelectorView};
pub use persona::{Persona, PersonaOrb, PersonaOrbState, PersonaState, PersonaVariant};
pub use speech_input::{SpeechInputState, SpeechInputView};
pub use transcription::{SegmentState, TranscriptionSegment, TranscriptionView};
pub use voice_selector::{
    VoiceAccent, VoiceGender, VoiceOption, VoicePreviewState, VoiceSelectorVariant,
    VoiceSelectorView,
};
