//! Playing audio pattern aligned with HIG.
//!
//! HIG: show the audio state plainly (playing / paused, level meter),
//! give the user scrubbing / transport controls, and respect the system
//! mute and AirPlay routing. Match the transport glyph set — Play,
//! Pause, SkipBack, SkipForward — rather than inventing new ones.
//!
//! # See also
//!
//! - [`crate::voice`] (requires `voice` feature) — persona / microphone
//!   UI + realtime voice session primitives.
//! - [`crate::foundations::icons::IconName::Play`] /
//!   [`crate::foundations::icons::IconName::Pause`] /
//!   [`crate::foundations::icons::IconName::Mic`] /
//!   [`crate::foundations::icons::IconName::MicOff`] /
//!   [`crate::foundations::icons::IconName::Volume2`] /
//!   [`crate::foundations::icons::IconName::VolumeX`] — transport and
//!   level glyphs (SF Symbols).
//! - [`crate::components::selection_and_input::slider::Slider`] —
//!   volume and playback-position scrubber.
//! - [`crate::components::status::activity_indicator::ActivityIndicator`]
//!   — indeterminate buffering indicator.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/playing-audio>
