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

/// A built-in handler, either plain or taking pre-computed code-block ranges.
enum BuiltInHandler<'a> {
    /// Handler that does not need code-block ranges.
    Plain(Box<dyn Fn(&str) -> Cow<'_, str> + 'a>),
    /// Handler that takes pre-computed `CodeBlockRanges` to avoid redundant O(n) scans.
    ///
    /// The returned `Cow` borrows from the input `&str` (first parameter), not
    /// from the ranges — spelled out via HRTB because two `&` parameters defeat
    /// lifetime elision.
    WithRanges(Box<dyn for<'b> Fn(&'b str, &ranges::CodeBlockRanges) -> Cow<'b, str> + 'a>),
}

/// A handler entry in the priority-sorted pipeline.
enum HandlerEntry<'a> {
    BuiltIn {
        handler: BuiltInHandler<'a>,
        priority: i32,
        early_return: bool,
        /// `true` if the handler may rewrite bytes in the middle of the string,
        /// invalidating any pre-computed `CodeBlockRanges`.
        mutates_mid_text: bool,
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
            handler: BuiltInHandler::Plain(Box::new(single_tilde::handle)),
            priority: priority::SINGLE_TILDE,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.comparison_operators {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::Plain(Box::new(comparison_operators::handle)),
            priority: priority::COMPARISON_OPERATORS,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.html_tags {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(html_tags::handle_with_ranges)),
            priority: priority::HTML_TAGS,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.setext_headings {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::Plain(Box::new(setext_heading::handle)),
            priority: priority::SETEXT_HEADINGS,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.links || options.images {
        let link_mode = options.link_mode;
        let links_enabled = options.links;
        let images_enabled = options.images;
        // `LinkMode::TextOnly` rewrites mid-text (drops `[` from the opening
        // bracket), shifting byte offsets; `Protocol` either appends at the end
        // or triggers early return on the sentinel marker.
        let mutates_mid_text = link_mode == options::LinkMode::TextOnly;
        let early_return = link_mode == options::LinkMode::Protocol;
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(move |text, r| {
                link_image::handle_with_ranges(text, link_mode, links_enabled, images_enabled, r)
            })),
            priority: priority::LINKS,
            early_return,
            mutates_mid_text,
        });
    }
    if options.bold_italic {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(emphasis::handle_bold_italic_with_ranges)),
            priority: priority::BOLD_ITALIC,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.bold {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(emphasis::handle_bold_with_ranges)),
            priority: priority::BOLD,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.italic {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(
                emphasis::handle_double_underscore_with_ranges,
            )),
            priority: priority::ITALIC_DOUBLE_UNDERSCORE,
            early_return: false,
            mutates_mid_text: false,
        });
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(
                emphasis::handle_italic_asterisk_with_ranges,
            )),
            priority: priority::ITALIC_SINGLE_ASTERISK,
            early_return: false,
            mutates_mid_text: false,
        });
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(
                emphasis::handle_italic_underscore_with_ranges,
            )),
            priority: priority::ITALIC_SINGLE_UNDERSCORE,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.inline_code {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::Plain(Box::new(inline_code::handle)),
            priority: priority::INLINE_CODE,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.strikethrough {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(strikethrough::handle_with_ranges)),
            priority: priority::STRIKETHROUGH,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.katex {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(katex::handle_block_with_ranges)),
            priority: priority::KATEX,
            early_return: false,
            mutates_mid_text: false,
        });
    }
    if options.inline_katex {
        entries.push(HandlerEntry::BuiltIn {
            handler: BuiltInHandler::WithRanges(Box::new(katex::handle_inline_with_ranges)),
            priority: priority::INLINE_KATEX,
            early_return: false,
            mutates_mid_text: false,
        });
    }

    // Add custom handlers.
    for handler in &options.handlers {
        entries.push(HandlerEntry::Custom(handler.as_ref()));
    }

    // Sort by priority (stable sort preserves insertion order for equal priorities).
    entries.sort_by_key(|e| e.priority());

    // Share a single `CodeBlockRanges` across all range-using handlers, built
    // lazily on first use and invalidated after any handler that may rewrite
    // bytes in the middle of the string (custom handlers are opaque mutators).
    let mut shared_ranges: Option<ranges::CodeBlockRanges> = None;

    for entry in &entries {
        match entry {
            HandlerEntry::BuiltIn {
                handler,
                early_return,
                mutates_mid_text,
                ..
            } => {
                let before_ptr = result.as_ref().as_ptr();
                match handler {
                    BuiltInHandler::Plain(f) => {
                        result = apply_with(result, |text| f(text));
                    }
                    BuiltInHandler::WithRanges(f) => {
                        if shared_ranges.is_none() {
                            shared_ranges = Some(ranges::CodeBlockRanges::new(&result));
                        }
                        let r = shared_ranges.as_ref().expect("ranges just initialized");
                        result = apply_with(result, |text| f(text, r));
                    }
                }
                if *early_return && result.ends_with(INCOMPLETE_LINK_MARKER) {
                    return result;
                }
                if *mutates_mid_text && !std::ptr::eq(result.as_ref().as_ptr(), before_ptr) {
                    shared_ranges = None;
                }
            }
            HandlerEntry::Custom(h) => {
                let before_ptr = result.as_ref().as_ptr();
                result = apply_with(result, |text| h.handle(text));
                if !std::ptr::eq(result.as_ref().as_ptr(), before_ptr) {
                    shared_ranges = None;
                }
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

    // Compute CodeBlockRanges once for all handlers that need code-block detection,
    // converting ~15 O(n) scans per streaming delta into 1 O(n) scan + O(log n) queries.
    //
    // Must be `mut` because `link_image` in `LinkMode::TextOnly` can remove a `[`
    // byte from the middle of the string, shifting every subsequent byte offset.
    // Stale ranges would then mis-report code regions to downstream handlers.
    let needs_ranges = options.html_tags
        || options.links
        || options.images
        || options.bold_italic
        || options.bold
        || options.italic
        || options.strikethrough
        || options.katex
        || options.inline_katex;
    let mut ranges = needs_ranges.then(|| ranges::CodeBlockRanges::new(&result));

    if options.html_tags
        && let Some(ref r) = ranges
    {
        result = apply_with(result, |text| html_tags::handle_with_ranges(text, r));
    }
    if options.setext_headings {
        result = apply(result, setext_heading::handle);
    }
    if (options.links || options.images)
        && let Some(ref r_guard) = ranges
    {
        let link_mode = options.link_mode;
        let links_enabled = options.links;
        let images_enabled = options.images;
        let before_ptr = result.as_ref().as_ptr();
        result = apply_with(result, move |text| {
            link_image::handle_with_ranges(text, link_mode, links_enabled, images_enabled, r_guard)
        });
        if result.ends_with(INCOMPLETE_LINK_MARKER) {
            return result;
        }
        // Mid-text mutation (e.g. TextOnly mode removing `[`) shifts every
        // later byte; rebuild ranges so downstream handlers see correct offsets.
        if !std::ptr::eq(result.as_ref().as_ptr(), before_ptr) {
            ranges = Some(ranges::CodeBlockRanges::new(&result));
        }
    }
    if let Some(ref r) = ranges {
        if options.bold_italic {
            result = apply_with(result, |text| {
                emphasis::handle_bold_italic_with_ranges(text, r)
            });
        }
        if options.bold {
            result = apply_with(result, |text| emphasis::handle_bold_with_ranges(text, r));
        }
        if options.italic {
            result = apply_with(result, |text| {
                emphasis::handle_double_underscore_with_ranges(text, r)
            });
            result = apply_with(result, |text| {
                emphasis::handle_italic_asterisk_with_ranges(text, r)
            });
            result = apply_with(result, |text| {
                emphasis::handle_italic_underscore_with_ranges(text, r)
            });
        }
        if options.inline_code {
            result = apply(result, inline_code::handle);
        }
        if options.strikethrough {
            result = apply_with(result, |text| strikethrough::handle_with_ranges(text, r));
        }
        if options.katex {
            result = apply_with(result, |text| katex::handle_block_with_ranges(text, r));
        }
        if options.inline_katex {
            result = apply_with(result, |text| katex::handle_inline_with_ranges(text, r));
        }
    } else if options.inline_code {
        // Only inline_code can reach here: when `needs_ranges` is false, every
        // other option in this block is also disabled (they gate `needs_ranges`).
        result = apply(result, inline_code::handle);
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
