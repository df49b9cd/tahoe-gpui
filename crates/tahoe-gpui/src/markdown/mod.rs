//! Streaming markdown renderer for GPUI.
//!
//! Incrementally parses markdown text via `pulldown-cmark` and renders
//! it as a GPUI element tree. Designed for streaming AI responses where
//! text arrives as deltas.
//!
//! Uses [`remend`] for auto-completing incomplete markdown syntax during
//! streaming, and provides word-level fade-in animation.

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
pub use selection::MarkdownSelection;
pub use renderer::{
    GenerativeLoadingState, MarkdownSecurity, StreamingMarkdown, render_block,
    render_block_at_depth,
};
pub use settings::StreamSettings;
