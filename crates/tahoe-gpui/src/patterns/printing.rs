//! Printing pattern (HIG Printing).
//!
//! GPUI exposes no direct print pipeline today. Apps that need to print
//! from a GPUI-built UI should delegate to the host platform layer
//! (`NSPrintOperation` on macOS) and position the print preview sheet
//! as a standard modal over the active window.
//!
//! HIG: offer print preview, page range, and paper-size controls in the
//! standard print dialog sequence; never skip the preview for
//! user-initiated prints, even when the output is a single page.
//!
//! # See also
//!
//! - [`crate::components::presentation::sheet::Sheet`] — modal host for
//!   a custom print preview rendered inside the GPUI element tree.
//! - [`crate::components::selection_and_input::picker::Picker`] — page
//!   size / orientation / duplex pickers.
//! - [`crate::components::selection_and_input::stepper::Stepper`] —
//!   copies and page-range steppers.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/printing>
