//! Entering data pattern aligned with HIG.
//!
//! HIG: prefer structured pickers over free-form text whenever the input
//! domain is bounded. Free text should accept any reasonable variation
//! (case, whitespace) without forcing the user to match an exact format,
//! and should validate inline rather than on submit so mistakes surface
//! early.
//!
//! # See also
//!
//! - [`crate::components::selection_and_input::text_field::TextField`]
//!   — free-form text + inline validation.
//! - [`crate::components::selection_and_input::toggle::Toggle`] — binary
//!   on/off.
//! - [`crate::components::selection_and_input::picker::Picker`] — bounded
//!   enum input.
//! - [`crate::components::selection_and_input::stepper::Stepper`] —
//!   bounded integer input.
//! - [`crate::components::selection_and_input::slider::Slider`] —
//!   continuous bounded input.
//! - [`crate::components::selection_and_input::date_picker`] — dates and
//!   date ranges (macOS calendar widget).
//! - [`crate::components::selection_and_input::image_well`] — image drop
//!   zone with external-file drop support.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/entering-data>
