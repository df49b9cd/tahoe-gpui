//! Searching pattern aligned with HIG.
//!
//! HIG: place search where the user expects it (toolbar on macOS,
//! top-navigation on iOS). Show results inline as the user types —
//! never gate results behind a Submit. Provide a clear affordance
//! (the `xmark.circle.fill` glyph) to clear the query quickly.
//! Preserve recent queries across sessions when privacy permits.
//!
//! # See also
//!
//! - [`crate::components::navigation_and_search::search_field::SearchField`]
//!   — the canonical search input with leading magnifier glyph and
//!   trailing clear button.
//! - [`crate::foundations::icons::IconName::Search`] /
//!   [`crate::foundations::icons::IconName::XmarkCircleFill`] — the
//!   search leading icon and canonical clear-affordance.
//! - `crate::components::menus_and_actions::command_palette` —
//!   keyboard-first command palette (⌘K) for power-user search flows (planned).
//! - [`crate::components::navigation_and_search::path_control`] — when
//!   the user's current location in a hierarchy should anchor the
//!   search scope.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/searching>
