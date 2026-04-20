//! Ratings and reviews pattern aligned with HIG.
//!
//! HIG: collect ratings at a natural moment of success (after the user
//! completes a task), never on first launch. Keep the prompt lightweight
//! — a single star row and an optional comment field. Respect
//! `SKStoreReviewController` throttling on iOS; on macOS, throttle your
//! own in-app prompts so the user can't be asked twice in the same
//! session.
//!
//! # See also
//!
#![cfg_attr(
    target_os = "macos",
    doc = "- [`crate::components::status::rating_indicator::RatingIndicator`] — 5-star display/input widget; pairs with [`crate::foundations::icons::IconName::Star`] / [`crate::foundations::icons::IconName::StarFill`] / [`crate::foundations::icons::IconName::StarLeadingHalfFilled`]."
)]
#![cfg_attr(
    not(target_os = "macos"),
    doc = "- `RatingIndicator` (macOS only) — 5-star display/input widget; pairs with [`crate::foundations::icons::IconName::Star`] / [`crate::foundations::icons::IconName::StarFill`] / [`crate::foundations::icons::IconName::StarLeadingHalfFilled`]."
)]
//! - [`crate::components::selection_and_input::text_field::TextField`]
//!   — optional free-form review entry.
//! - [`crate::components::menus_and_actions::button::Button`] — Submit,
//!   Cancel actions; use `ButtonVariant::Ghost` for "Maybe later".
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/ratings-and-reviews>
