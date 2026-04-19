//! Input validation types for TextField.

use gpui::SharedString;

/// Validation state for a `TextField`.
#[derive(Clone, Debug, Default)]
pub enum TextFieldValidation {
    /// No validation styling (default).
    #[default]
    None,
    /// Invalid input — shows error border and an error message below the field.
    Invalid(SharedString),
    /// Valid input — shows success border.
    Valid,
}
