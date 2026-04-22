//! Streaming markdown renderer for GPUI.
//!
//! Incrementally parses markdown text via `pulldown-cmark` and renders
//! it as a GPUI element tree. Designed for streaming AI responses where
//! text arrives as deltas.
//!
//! Uses [`mdstitch`] for auto-completing incomplete markdown syntax during
//! streaming, and provides word-level fade-in animation.
//!
//! # Heading anchors
//!
//! Headings carry GitHub-compatible slug anchors (see
//! [`parser::MarkdownBlock::Heading::anchor_id`]) and are rendered with an
//! element id of `{HEADING_ID_PREFIX}{slug}` so consumers can locate them in
//! their scroll container. Fragment links clicked inside the rendered
//! markdown invoke the handler installed via
//! [`StreamingMarkdown::with_anchor_click`]. See
//! [`StreamingMarkdown::with_anchor_click`] for the exact scroll contract and
//! accessibility / motion caveats.

pub mod animation;
pub mod caret;
pub mod code_block;
pub mod mermaid;
pub mod parser;
pub mod renderer;
pub mod selectable_text;
pub mod selection;
pub mod settings;
pub mod syntax;

pub use animation::{AnimationKind, AnimationState, Easing};
pub use caret::CaretKind;
pub use code_block::{
    CodeBlockActions, CodeBlockContainer, CodeBlockContent, CodeBlockFilename, CodeBlockHeader,
    CodeBlockLanguageSelector, CodeBlockTitle, CodeBlockView, LanguageVariant,
};
pub use parser::{IncrementalMarkdownParser, InlineContent, MarkdownBlock, TableAlignment};
pub use renderer::{
    GenerativeLoadingState, MarkdownSecurity, StreamingMarkdown, render_block,
    render_block_at_depth,
};
pub use selectable_text::AnchorClickHandler;
pub use selection::MarkdownSelection;
pub use settings::StreamSettings;

/// Element-id prefix emitted by the renderer for every addressable
/// heading. The full id is `{HEADING_ID_PREFIX}{slug}` where `slug` comes
/// from [`parser::MarkdownBlock::Heading::anchor_id`]. Consumers that
/// implement scroll-to-anchor should reference this constant rather than
/// hard-coding the literal so a future rename surfaces as a compile
/// error.
pub const HEADING_ID_PREFIX: &str = "md-heading-";

/// Build the [`gpui::ElementId`] the renderer attaches to an addressable
/// heading. Exposed so consumers implementing scroll-to-anchor can look
/// up the element bounds without duplicating the formatting.
///
/// The returned id is the **child** id; GPUI scopes it under the nearest
/// stateful ancestor. `StreamingMarkdown`'s root is itself stateful
/// (keyed by its entity id), so two markdown entities on the same
/// window never collide on this child id. Consumers whose lookup API
/// needs a full path should compose it with the scrollable container's
/// own id.
pub fn heading_element_id(slug: &str) -> gpui::ElementId {
    gpui::ElementId::Name(format!("{HEADING_ID_PREFIX}{slug}").into())
}
