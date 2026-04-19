//! Managing accounts pattern aligned with HIG.
//!
//! HIG: lean on Sign in with Apple / Passkeys where possible; keep
//! manual credential entry minimal and give the user a clear exit
//! (Cancel, Back). Never display the raw password in plain text — use
//! a secure text field with a toggle to reveal.
//!
//! # See also
//!
//! - [`crate::components::selection_and_input::text_field::TextField`]
//!   — username / email input. Configure `secure(true)` for passwords.
//! - [`crate::components::menus_and_actions::button::Button`] — primary
//!   `Sign In` action; `ButtonVariant::Primary` for the submit, `Ghost`
//!   for the Cancel.
//! - [`crate::components::presentation::alert::Alert`] — modal for
//!   destructive account actions (Sign Out, Delete Account).
//! - [`crate::components::content::avatar::Avatar`] — identity glyph
//!   for the signed-in user.
//! - [`crate::patterns::privacy`] — permission + privacy copy guidance
//!   for sign-up flows.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/managing-accounts>
