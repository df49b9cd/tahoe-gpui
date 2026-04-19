//! Workouts pattern (HIG Workouts).
//!
//! Platform: **watchOS primary** — this pattern targets Activity /
//! Workout apps on Apple Watch (HealthKit-backed session, always-on
//! display, digital-crown dial). Not applicable to GPUI on macOS;
//! documented for taxonomy completeness so HIG audits can grep every
//! pattern entry.
//!
//! For cross-platform Apple apps that embed this crate on macOS while
//! shipping a companion watchOS workout experience, no GPUI helpers are
//! needed here — watchOS has its own HealthKit + WorkoutKit SDKs.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/workouts>
