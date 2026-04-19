#![doc(
    html_logo_url = "https://raw.githubusercontent.com/df49b9cd/tahoe-gpui/main/branding/tahoe-gpui-logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/df49b9cd/tahoe-gpui/main/branding/tahoe-gpui-logo.png"
)]
//! Streaming markdown preprocessor that auto-completes incomplete syntax.
//!
//! A Rust port of Vercel's [remend](https://github.com/vercel/streamdown/tree/main/packages/remend).
//! Runs on raw markdown strings **before** the pulldown-cmark parser, detecting
//! and closing unterminated formatting markers so content renders correctly
//! during token-by-token streaming.

mod options;
mod ranges;
mod utils;

mod comparison_operators;
mod emphasis;
mod html_tags;
mod inline_code;
mod katex;
mod link_image;
mod setext_heading;
mod single_tilde;
mod strikethrough;

mod detect_direction;
mod incomplete_code;
mod preprocess;

pub use options::{LinkMode, RemendHandler, RemendOptions, priority};
pub use ranges::CodeBlockRanges;

// Re-export public items from internal modules.
pub use detect_direction::{TextDirection, detect_text_direction};
pub use incomplete_code::{has_incomplete_code_fence, has_table};
pub use preprocess::{
    normalize_html_indentation, preprocess_custom_tags, preprocess_literal_tag_content,
};

// Re-export utility functions for use by custom handlers.
// These four are the most commonly needed when implementing `RemendHandler`:
// code block detection, link/image URL detection, math block detection, and
// word character classification.
pub use utils::{
    is_inside_code_block, is_within_link_or_image_url, is_within_math_block, is_word_char,
};

use std::borrow::Cow;

const INCOMPLETE_LINK_MARKER: &str = "](streamdown:incomplete-link)";

/// A handler entry in the priority-sorted pipeline.
enum HandlerEntry<'a> {
    /// A built-in handler (function pointer).
    BuiltIn {
        handler: Box<dyn Fn(&str) -> Cow<'_, str> + 'a>,
        priority: i32,
        early_return: bool,
    },
    /// A custom handler (trait object).
    Custom(&'a dyn RemendHandler),
}

impl HandlerEntry<'_> {
    fn priority(&self) -> i32 {
        match self {
            HandlerEntry::BuiltIn { priority, .. } => *priority,
            HandlerEntry::Custom(h) => h.priority(),
        }
    }
}

/// Preprocesses streaming markdown text, auto-completing any incomplete syntax.
///
/// Returns `Cow::Borrowed` when no changes are needed (zero-allocation fast path).
pub fn remend<'a>(text: &'a str, options: &RemendOptions) -> Cow<'a, str> {
    if text.is_empty() {
        return Cow::Borrowed(text);
    }

    // Strip trailing single space (preserve double space for line breaks).
    let mut result: Cow<'a, str> = if text.ends_with(' ') && !text.ends_with("  ") {
        Cow::Borrowed(&text[..text.len() - 1])
    } else {
        Cow::Borrowed(text)
    };

    // If no custom handlers, use the fast fixed-order pipeline.
    if options.handlers.is_empty() {
        return run_builtin_pipeline(result, options);
    }

    // Build and sort handler entries by priority.
    let mut entries: Vec<HandlerEntry<'_>> = Vec::new();

    if options.single_tilde {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(single_tilde::handle),
            priority: priority::SINGLE_TILDE,
            early_return: false,
        });
    }
    if options.comparison_operators {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(comparison_operators::handle),
            priority: priority::COMPARISON_OPERATORS,
            early_return: false,
        });
    }
    if options.html_tags {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(html_tags::handle),
            priority: priority::HTML_TAGS,
            early_return: false,
        });
    }
    if options.setext_headings {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(setext_heading::handle),
            priority: priority::SETEXT_HEADINGS,
            early_return: false,
        });
    }
    if options.links || options.images {
        let link_mode = options.link_mode;
        let links_enabled = options.links;
        let images_enabled = options.images;
        let early_return = link_mode == options::LinkMode::Protocol;
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(move |text| {
                link_image::handle(text, link_mode, links_enabled, images_enabled)
            }),
            priority: priority::LINKS,
            early_return,
        });
    }
    if options.bold_italic {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(emphasis::handle_bold_italic),
            priority: priority::BOLD_ITALIC,
            early_return: false,
        });
    }
    if options.bold {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(emphasis::handle_bold),
            priority: priority::BOLD,
            early_return: false,
        });
    }
    if options.italic {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(emphasis::handle_double_underscore),
            priority: priority::ITALIC_DOUBLE_UNDERSCORE,
            early_return: false,
        });
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(emphasis::handle_italic_asterisk),
            priority: priority::ITALIC_SINGLE_ASTERISK,
            early_return: false,
        });
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(emphasis::handle_italic_underscore),
            priority: priority::ITALIC_SINGLE_UNDERSCORE,
            early_return: false,
        });
    }
    if options.inline_code {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(inline_code::handle),
            priority: priority::INLINE_CODE,
            early_return: false,
        });
    }
    if options.strikethrough {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(strikethrough::handle),
            priority: priority::STRIKETHROUGH,
            early_return: false,
        });
    }
    if options.katex {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(katex::handle_block),
            priority: priority::KATEX,
            early_return: false,
        });
    }
    if options.inline_katex {
        entries.push(HandlerEntry::BuiltIn {
            handler: Box::new(katex::handle_inline),
            priority: priority::INLINE_KATEX,
            early_return: false,
        });
    }

    // Add custom handlers.
    for handler in &options.handlers {
        entries.push(HandlerEntry::Custom(handler.as_ref()));
    }

    // Sort by priority (stable sort preserves insertion order for equal priorities).
    entries.sort_by_key(|e| e.priority());

    // Execute in priority order.
    for entry in &entries {
        match entry {
            HandlerEntry::BuiltIn {
                handler,
                early_return,
                ..
            } => {
                result = apply_with(result, |text| handler(text));
                if *early_return && result.ends_with(INCOMPLETE_LINK_MARKER) {
                    return result;
                }
            }
            HandlerEntry::Custom(h) => {
                result = apply_with(result, |text| h.handle(text));
            }
        }
    }

    result
}

/// Fast path: fixed-order pipeline with no dynamic dispatch (used when no custom handlers).
fn run_builtin_pipeline<'a>(mut result: Cow<'a, str>, options: &RemendOptions) -> Cow<'a, str> {
    if options.single_tilde {
        result = apply(result, single_tilde::handle);
    }
    if options.comparison_operators {
        result = apply(result, comparison_operators::handle);
    }
    if options.html_tags {
        result = apply(result, html_tags::handle);
    }
    if options.setext_headings {
        result = apply(result, setext_heading::handle);
    }
    if options.links || options.images {
        let link_mode = options.link_mode;
        let links_enabled = options.links;
        let images_enabled = options.images;
        result = apply_with(result, move |text| {
            link_image::handle(text, link_mode, links_enabled, images_enabled)
        });
        if result.ends_with(INCOMPLETE_LINK_MARKER) {
            return result;
        }
    }
    // Compute CodeBlockRanges once for all emphasis handlers (avoids 5x redundant O(n) scans).
    if options.bold_italic || options.bold || options.italic {
        let ranges = ranges::CodeBlockRanges::new(&result);
        if options.bold_italic {
            result = apply_with(result, |text| {
                emphasis::handle_bold_italic_with_ranges(text, &ranges)
            });
        }
        if options.bold {
            result = apply_with(result, |text| {
                emphasis::handle_bold_with_ranges(text, &ranges)
            });
        }
        if options.italic {
            result = apply_with(result, |text| {
                emphasis::handle_double_underscore_with_ranges(text, &ranges)
            });
            result = apply_with(result, |text| {
                emphasis::handle_italic_asterisk_with_ranges(text, &ranges)
            });
            result = apply_with(result, |text| {
                emphasis::handle_italic_underscore_with_ranges(text, &ranges)
            });
        }
    }
    if options.inline_code {
        result = apply(result, inline_code::handle);
    }
    if options.strikethrough {
        result = apply(result, strikethrough::handle);
    }
    if options.katex {
        result = apply(result, katex::handle_block);
    }
    if options.inline_katex {
        result = apply(result, katex::handle_inline);
    }

    result
}

/// Applies a handler to a `Cow<str>`, threading ownership efficiently.
fn apply<'a>(input: Cow<'a, str>, handler: fn(&str) -> Cow<'_, str>) -> Cow<'a, str> {
    apply_with(input, handler)
}

/// Applies a closure handler to a `Cow<str>`, threading ownership efficiently.
fn apply_with<'a>(input: Cow<'a, str>, handler: impl FnOnce(&str) -> Cow<'_, str>) -> Cow<'a, str> {
    match handler(&input) {
        Cow::Borrowed(b) if std::ptr::eq(b, input.as_ref() as &str) => {
            // Handler returned its input unchanged — preserve the original Cow.
            input
        }
        Cow::Borrowed(b) => {
            // Handler returned a borrowed sub-slice (e.g. trimmed) — must own it.
            Cow::Owned(b.to_owned())
        }
        Cow::Owned(s) => Cow::Owned(s),
    }
}

#[cfg(test)]
mod tests;
