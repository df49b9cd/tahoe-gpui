//! Offering help pattern aligned with HIG.
//!
//! HIG: help is contextual, not destination content. Pair every non-
//! obvious affordance with an inline hint, a tooltip, or a first-run
//! coachmark — but never force the user to read documentation before
//! they can proceed. Use progressive disclosure: summary by default,
//! details on request.
//!
//! # See also
//!
//! - [`crate::components::presentation::tooltip::Tooltip`] — canonical
//!   contextual help over hover targets (400–500 ms delay per HIG).
//! - `crate::components::menus_and_actions::help_button` — the `?`
//!   button that opens context help (planned).
//! - [`crate::components::content::badge::Badge`] with
//!   `BadgeVariant::Info` — inline advisory annotations.
//! - [`crate::components::presentation::popover::Popover`] — longer
//!   help content anchored to an affordance (e.g. "What's this?").
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/offering-help>
