//! Activity view component stub (HIG Activity views).
//!
//! HIG: <https://developer.apple.com/design/human-interface-guidelines/activity-views>
//!
//! The macOS equivalent is `NSSharingServicePicker`; the iOS equivalent is
//! `UIActivityViewController`. GPUI does not yet expose either AppKit or
//! UIKit bridge, so a full implementation must wait on upstream work.
//!
//! ## Current replacement
//!
//! Until GPUI lands a share-service bridge, callers should use
//! [`crate::components::menus_and_actions::share_button::ShareButton`],
//! which renders the HIG-canonical Share glyph (`square.and.arrow.up`)
//! and populates an in-app pull-down menu from a caller-supplied list of
//! [`ShareService`](crate::components::menus_and_actions::share_button::ShareService)
//! entries. It is a functional stand-in for the native share sheet.
//!
//! Tracked by `docs/hig/components/menus-and-actions.md:26`.
