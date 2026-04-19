//! Writing foundation aligned with HIG Writing.
//!
//! Writing guidance covers voice and tone — not visual text styling.
//! The HIG recommends writing that is concise, clear, and action-
//! oriented; every label is an opportunity to reduce cognitive load.
//! Specific anti-patterns called out by the HIG:
//!
//! - Don't use "please" or "sorry" in system-facing copy.
//! - Prefer verbs ("Save", "Send") over nouns ("Saving", "Submission").
//! - Write dates as `Jan 12, 2026` not `01/12/26` unless dense data
//!   tables require the compact form.
//! - Avoid jargon; spell out acronyms on first use.
//!
//! This module does not ship a separate "tone enum" — copy decisions
//! belong in the app content layer, not in a UI library. Provided here
//! as a documentation anchor so HIG audits have a place to land.
//!
//! # See also
//!
//! - [`crate::foundations::inclusion`] — inclusive-language guidance
//!   that pairs with writing tone.
//! - [`crate::foundations::typography`] — visual typography tokens
//!   (distinct from writing-tone guidance).
//! - [`crate::patterns::feedback`] — feedback copy conventions
//!   (`Success`, `Warning`, `Error` labels).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/writing>
//!
//! Tracked by `docs/hig/foundations.md:2531`.
