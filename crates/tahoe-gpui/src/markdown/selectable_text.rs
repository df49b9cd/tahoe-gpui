//! Re-export shim for the selectable-text element.
//!
//! The element's canonical home is
//! [`crate::components::content::selectable_text`]; this module
//! preserves the legacy `markdown::selectable_text::SelectableText`
//! path used by the streaming-markdown and terminal render paths.

pub use crate::components::content::selectable_text::{
    AnchorClickHandler, SelectableText, SelectionCoordinator, fragment_of,
};
