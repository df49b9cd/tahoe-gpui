//! Playing haptics pattern (HIG Playing haptics).
//!
//! Platform: **iOS / iPadOS / watchOS / visionOS primary.** Haptics
//! augment visible feedback with a physical sensation; GPUI on macOS
//! exposes no haptics API (the Magic Trackpad and Touch ID sensors are
//! the only vectors, and they require `NSHapticFeedbackPerformer` on
//! AppKit — outside GPUI's public surface today).
//!
//! For cross-platform haptic feedback in an Apple app that embeds this
//! crate, delegate to the host's platform layer; keep visual feedback
//! (colour change, animation) as the primary channel and treat haptics
//! as additive.
//!
//! # See also
//!
//! - [`crate::patterns::feedback`] — visual feedback primitives that
//!   should always accompany haptic feedback.
//! - [`crate::foundations::motion`] — motion tokens so visual feedback
//!   timing can be synchronised with host-dispatched haptics.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/playing-haptics>
