//! Playing video pattern (HIG Playing video).
//!
//! GPUI currently exposes no AVFoundation-backed video element. Hosts
//! that need inline video today should render into a platform-specific
//! `AVPlayerLayer` / `WKWebView` outside the GPUI element tree and
//! position it via window coordinates reported by a GPUI placeholder
//! div.
//!
//! HIG: default to native transport controls (Play, Pause, scrub bar,
//! captions, picture-in-picture); never autoplay with audio on the
//! initial load; allow the user to pause, mute, and exit full-screen
//! at any point.
//!
//! # See also
//!
//! - [`crate::foundations::icons::IconName::Video`] — glyph for
//!   video-related affordances and placeholder tiles.
//! - [`crate::components::content::web_view`] — related web-content
//!   surface; video embedded in web pages uses the host `WKWebView`.
//! - [`crate::patterns::playing_audio`] — audio transport conventions
//!   that mirror the video transport set.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/playing-video>
