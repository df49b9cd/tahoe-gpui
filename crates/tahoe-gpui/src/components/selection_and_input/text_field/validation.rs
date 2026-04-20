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
    /// Soft warning — shows a caution-tinted border and a warning icon
    /// paired with the supplied message below the field. Use when the
    /// value is acceptable but outside a recommended range (password
    /// strength, near-limit constraints, deprecated format).
    Warning(SharedString),
}
