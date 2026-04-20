//! Status display components (HIG: Components > Status).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/components/status>

pub mod activity_indicator;
pub mod activity_ring;
pub mod gauge;
pub mod progress_indicator;
pub mod rating_indicator;
pub mod shimmer;

pub use activity_indicator::{ActivityIndicator, ActivityIndicatorSize, ActivityIndicatorStyle};
pub use activity_ring::{
    ACTIVITY_RING_EXERCISE, ACTIVITY_RING_MOVE, ACTIVITY_RING_STAND, ActivityRing, ActivityRingSet,
};
pub use gauge::{Gauge, GaugeDirection, GaugeStyle};
pub use progress_indicator::{ProgressIndicator, ProgressIndicatorSize, ProgressIndicatorValue};
pub use rating_indicator::RatingIndicator;
pub use shimmer::{Shimmer, ShimmerEasing, SweepDirection, TextShimmer, TextShimmerState};
