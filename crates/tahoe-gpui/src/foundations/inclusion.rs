//! Inclusion foundation aligned with HIG Inclusion.
//!
//! Inclusion covers representing diverse people, avoiding stereotypes,
//! using inclusive language, and picking imagery / illustrations that
//! welcome everyone. Distinct from [`crate::foundations::accessibility`]
//! (which covers motor, cognitive, visual, and auditory assistance) —
//! inclusion is about *content* fairness.
//!
//! This module does not ship a separate type system; inclusion
//! decisions belong in the app content layer. Provided here as a
//! documentation anchor so HIG audits have a place to land.
//!
//! # Guidance highlights
//!
//! - Use singular they for gender-neutral references.
//! - Avoid region-specific idioms in system copy.
//! - Provide `Avatar` initials for every contributor — not just those
//!   with profile images.
//! - Name examples with a mix of cultures and pronouns.
//! - Respect accessibility preferences even in demo content (use
//!   high-contrast imagery, caption videos).
//!
//! # See also
//!
//! - [`crate::foundations::accessibility::AccessibilityMode`] — the
//!   runtime accessibility toggles (Reduce Motion, Increase Contrast,
//!   Bold Text, Reduce Transparency, …).
//! - [`crate::foundations::typography::DynamicTypeSize`] — Dynamic Type
//!   support that larger-text users rely on.
//! - [`crate::foundations::writing`] — inclusive writing tone.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/inclusion>
//!
//! Tracked by `docs/hig/foundations.md:667`.
