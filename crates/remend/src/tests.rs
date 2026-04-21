use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use proptest::prelude::*;

use super::{
    LinkMode, RemendHandler, RemendOptions, has_incomplete_code_fence, is_inside_code_block, remend,
};

fn opts() -> RemendOptions {
    RemendOptions::default()
}

fn r(text: &str) -> Cow<'_, str> {
    remend(text, &opts())
}

// ===========================================================================
// Basic input
// ===========================================================================

#[test]
fn empty_string() {
    assert!(matches!(r(""), Cow::Borrowed(_)));
}

#[test]
fn plain_text() {
    assert_eq!(r("hello world").as_ref(), "hello world");
}

#[test]
fn strips_trailing_single_space() {
    assert_eq!(r("hello ").as_ref(), "hello");
}

#[test]
fn preserves_double_trailing_space() {
    assert_eq!(r("hello  ").as_ref(), "hello  ");
}

// ===========================================================================
// Bold
// ===========================================================================

#[test]
fn bold_incomplete() {
    assert_eq!(r("Text with **bold").as_ref(), "Text with **bold**");
}

#[test]
fn bold_incomplete_at_start() {
    assert_eq!(r("**incomplete").as_ref(), "**incomplete**");
}

#[test]
fn bold_complete() {
    assert_eq!(
        r("Text with **bold text**").as_ref(),
        "Text with **bold text**"
    );
}

#[test]
fn bold_multiple_complete() {
    assert_eq!(
        r("**bold1** and **bold2**").as_ref(),
        "**bold1** and **bold2**"
    );
}

#[test]
fn bold_odd_markers() {
    assert_eq!(
        r("**first** and **second").as_ref(),
        "**first** and **second**"
    );
}

#[test]
fn bold_partial_boundary() {
    assert_eq!(
        r("Here is some **bold tex").as_ref(),
        "Here is some **bold tex**"
    );
}

#[test]
fn bold_half_close_simple() {
    assert_eq!(r("**xxx*").as_ref(), "**xxx**");
}

#[test]
fn bold_half_close_phrase() {
    assert_eq!(r("**bold text*").as_ref(), "**bold text**");
}

#[test]
fn bold_half_close_sentence() {
    assert_eq!(r("Text with **bold*").as_ref(), "Text with **bold**");
}

#[test]
fn bold_half_close_full() {
    assert_eq!(r("This is **bold text*").as_ref(), "This is **bold text**");
}

// ===========================================================================
// Italic (double underscores)
// ===========================================================================

#[test]
fn italic_double_underscore_incomplete() {
    assert_eq!(r("Text with __italic").as_ref(), "Text with __italic__");
}

#[test]
fn italic_double_underscore_at_start() {
    assert_eq!(r("__incomplete").as_ref(), "__incomplete__");
}

#[test]
fn italic_double_underscore_complete() {
    assert_eq!(
        r("Text with __italic text__").as_ref(),
        "Text with __italic text__"
    );
}

#[test]
fn italic_double_underscore_odd() {
    assert_eq!(
        r("__first__ and __second").as_ref(),
        "__first__ and __second__"
    );
}

#[test]
fn italic_double_underscore_half_close() {
    assert_eq!(r("__xxx_").as_ref(), "__xxx__");
}

#[test]
fn italic_double_underscore_half_close_phrase() {
    assert_eq!(r("__bold text_").as_ref(), "__bold text__");
}

// ===========================================================================
// Italic (single asterisk)
// ===========================================================================

#[test]
fn italic_asterisk_incomplete() {
    assert_eq!(r("Text with *italic").as_ref(), "Text with *italic*");
}

#[test]
fn italic_asterisk_at_start() {
    assert_eq!(r("*incomplete").as_ref(), "*incomplete*");
}

#[test]
fn italic_asterisk_complete() {
    assert_eq!(
        r("Text with *italic text*").as_ref(),
        "Text with *italic text*"
    );
}

#[test]
fn italic_asterisk_with_bold() {
    assert_eq!(r("**bold** and *italic").as_ref(), "**bold** and *italic*");
}

#[test]
fn italic_asterisk_word_internal_digits() {
    assert_eq!(r("234234*123").as_ref(), "234234*123");
}

#[test]
fn italic_asterisk_word_internal_letters() {
    assert_eq!(r("hello*world").as_ref(), "hello*world");
}

#[test]
fn italic_asterisk_word_internal_mixed() {
    assert_eq!(r("test*123*test").as_ref(), "test*123*test");
}

#[test]
fn italic_asterisk_with_var_names() {
    assert_eq!(
        r("*italic with some*var*name inside").as_ref(),
        "*italic with some*var*name inside*"
    );
}

#[test]
fn italic_asterisk_complete_word() {
    assert_eq!(r("*word* and more text").as_ref(), "*word* and more text");
}

// ===========================================================================
// Italic (single underscore)
// ===========================================================================

#[test]
fn italic_underscore_incomplete() {
    assert_eq!(r("Text with _italic").as_ref(), "Text with _italic_");
}

#[test]
fn italic_underscore_at_start() {
    assert_eq!(r("_incomplete").as_ref(), "_incomplete_");
}

#[test]
fn italic_underscore_complete() {
    assert_eq!(
        r("Text with _italic text_").as_ref(),
        "Text with _italic text_"
    );
}

#[test]
fn italic_underscore_with_bold() {
    assert_eq!(r("__bold__ and _italic").as_ref(), "__bold__ and _italic_");
}

#[test]
fn italic_underscore_word_internal_cafe() {
    assert_eq!(r("café_price").as_ref(), "café_price");
}

#[test]
fn italic_underscore_word_internal_naive() {
    assert_eq!(r("naïve_approach").as_ref(), "naïve_approach");
}

#[test]
fn italic_underscore_word_internal_variable() {
    assert_eq!(r("some_variable_name").as_ref(), "some_variable_name");
}

#[test]
fn italic_underscore_word_internal_digits() {
    assert_eq!(r("test_123_value").as_ref(), "test_123_value");
}

#[test]
fn italic_underscore_with_var_names() {
    assert_eq!(
        r("_italic with some_var_name inside").as_ref(),
        "_italic with some_var_name inside_"
    );
}

#[test]
fn italic_underscore_trailing_newline() {
    assert_eq!(r("Text with _italic\n").as_ref(), "Text with _italic_\n");
}

#[test]
fn italic_underscore_trailing_double_newline() {
    assert_eq!(r("_incomplete\n\n").as_ref(), "_incomplete_\n\n");
}

// ===========================================================================
// Bold-italic
// ===========================================================================

#[test]
fn bold_italic_incomplete() {
    assert_eq!(
        r("Text with ***bold-italic").as_ref(),
        "Text with ***bold-italic***"
    );
}

#[test]
fn bold_italic_at_start() {
    assert_eq!(r("***incomplete").as_ref(), "***incomplete***");
}

#[test]
fn bold_italic_complete() {
    assert_eq!(
        r("Text with ***bold and italic text***").as_ref(),
        "Text with ***bold and italic text***"
    );
}

#[test]
fn bold_italic_multiple_complete() {
    assert_eq!(
        r("***first*** and ***second***").as_ref(),
        "***first*** and ***second***"
    );
}

#[test]
fn bold_italic_odd() {
    assert_eq!(
        r("***first*** and ***second").as_ref(),
        "***first*** and ***second***"
    );
}

#[test]
fn bold_italic_four_asterisks_text() {
    assert_eq!(r("****").as_ref(), "****");
}

#[test]
fn bold_italic_five_asterisks() {
    assert_eq!(r("*****").as_ref(), "*****");
}

#[test]
fn bold_italic_trailing_asterisks_unchanged() {
    assert_eq!(r("text ***").as_ref(), "text ***");
    assert_eq!(r("text ****").as_ref(), "text ****");
    assert_eq!(r("text *****").as_ref(), "text *****");
}

#[test]
fn bold_italic_overlapping_302() {
    // Overlapping bold + italic: already balanced.
    assert_eq!(
        r("Combined **bold and *italic*** text").as_ref(),
        "Combined **bold and *italic*** text"
    );
}

#[test]
fn bold_italic_overlapping_already_complete() {
    assert_eq!(
        r("**bold and *italic*** more text").as_ref(),
        "**bold and *italic*** more text"
    );
}

// ===========================================================================
// Inline code
// ===========================================================================

#[test]
fn inline_code_incomplete() {
    assert_eq!(r("`code").as_ref(), "`code`");
}

#[test]
fn inline_code_complete() {
    assert_eq!(r("`code`").as_ref(), "`code`");
}

#[test]
fn inline_code_empty() {
    assert_eq!(r("`").as_ref(), "`");
}

// ===========================================================================
// Links
// ===========================================================================

#[test]
fn link_incomplete_url() {
    assert_eq!(
        r("[Click here](http://exam").as_ref(),
        "[Click here](streamdown:incomplete-link)"
    );
}

#[test]
fn link_incomplete_text() {
    assert_eq!(
        r("[Click here").as_ref(),
        "[Click here](streamdown:incomplete-link)"
    );
}

#[test]
fn link_complete() {
    assert_eq!(
        r("[text](http://example.com)").as_ref(),
        "[text](http://example.com)"
    );
}

#[test]
fn link_multiple_complete() {
    assert_eq!(
        r("[link1](url1) and [link2](url2)").as_ref(),
        "[link1](url1) and [link2](url2)"
    );
}

#[test]
fn link_nested_brackets_incomplete_url() {
    assert_eq!(
        r("[outer [nested] text](incomplete").as_ref(),
        "[outer [nested] text](streamdown:incomplete-link)"
    );
}

#[test]
fn link_nested_brackets_complete() {
    assert_eq!(
        r("[link with [brackets] inside](https://example.com)").as_ref(),
        "[link with [brackets] inside](https://example.com)"
    );
}

#[test]
fn link_partial_boundary() {
    assert_eq!(
        r("Check out [this lin").as_ref(),
        "Check out [this lin](streamdown:incomplete-link)"
    );
}

#[test]
fn link_partial_url_boundary() {
    assert_eq!(
        r("Visit [our site](https://exa").as_ref(),
        "Visit [our site](streamdown:incomplete-link)"
    );
}

#[test]
fn link_no_matching_bracket() {
    assert_eq!(
        r("Text [outer [inner").as_ref(),
        "Text [outer [inner](streamdown:incomplete-link)"
    );
}

// ===========================================================================
// Images
// ===========================================================================

#[test]
fn image_incomplete_removed() {
    assert_eq!(r("text ![alt](http://").as_ref(), "text");
}

#[test]
fn image_incomplete_text_removed() {
    assert_eq!(r("text ![alt").as_ref(), "text");
}

#[test]
fn image_partial_removed() {
    assert_eq!(r("![partial").as_ref(), "");
}

#[test]
fn image_complete_unchanged() {
    assert_eq!(
        r("Text with ![alt text](image.png)").as_ref(),
        "Text with ![alt text](image.png)"
    );
}

#[test]
fn image_nested_brackets_removed() {
    assert_eq!(r("Text ![outer [inner]").as_ref(), "Text");
}

#[test]
fn image_url_with_underscores_unchanged() {
    let text = "textContent ![image](https://img.example.com/path_name.png)";
    assert_eq!(r(text).as_ref(), text);
}

// ===========================================================================
// Strikethrough
// ===========================================================================

#[test]
fn strikethrough_incomplete() {
    assert_eq!(r("Text with ~~strike").as_ref(), "Text with ~~strike~~");
}

#[test]
fn strikethrough_at_start() {
    assert_eq!(r("~~incomplete").as_ref(), "~~incomplete~~");
}

#[test]
fn strikethrough_complete() {
    assert_eq!(
        r("~~strikethrough text~~").as_ref(),
        "~~strikethrough text~~"
    );
}

#[test]
fn strikethrough_multiple_complete() {
    assert_eq!(
        r("~~strike1~~ and ~~strike2~~").as_ref(),
        "~~strike1~~ and ~~strike2~~"
    );
}

#[test]
fn strikethrough_odd() {
    assert_eq!(
        r("~~first~~ and ~~second").as_ref(),
        "~~first~~ and ~~second~~"
    );
}

#[test]
fn strikethrough_half_close() {
    assert_eq!(r("~~xxx~").as_ref(), "~~xxx~~");
}

#[test]
fn strikethrough_half_close_phrase() {
    assert_eq!(r("~~strike text~").as_ref(), "~~strike text~~");
}

// ===========================================================================
// KaTeX (block)
// ===========================================================================

#[test]
fn katex_block_incomplete() {
    assert_eq!(r("$$x + y").as_ref(), "$$x + y$$");
}

#[test]
fn katex_block_at_start() {
    assert_eq!(r("$$incomplete").as_ref(), "$$incomplete$$");
}

#[test]
fn katex_block_complete() {
    assert_eq!(r("$$E = mc^2$$").as_ref(), "$$E = mc^2$$");
}

#[test]
fn katex_block_multiple() {
    assert_eq!(
        r("$$formula1$$ and $$formula2$$").as_ref(),
        "$$formula1$$ and $$formula2$$"
    );
}

#[test]
fn katex_block_odd() {
    assert_eq!(
        r("$$first$$ and $$second").as_ref(),
        "$$first$$ and $$second$$"
    );
}

#[test]
fn katex_block_half_dollar() {
    assert_eq!(r("$$formula$").as_ref(), "$$formula$$");
}

#[test]
fn katex_block_multiline() {
    assert_eq!(r("$$\nx = 1\ny = 2").as_ref(), "$$\nx = 1\ny = 2\n$$");
}

// ===========================================================================
// KaTeX (inline — opt-in)
// ===========================================================================

#[test]
fn katex_inline_default_no_completion() {
    // Inline KaTeX is disabled by default.
    assert_eq!(r("Text with $formula").as_ref(), "Text with $formula");
    assert_eq!(r("$incomplete").as_ref(), "$incomplete");
}

#[test]
fn katex_inline_enabled_completes() {
    let opts = RemendOptions::default().inline_katex(true);
    assert_eq!(
        remend("Text with $formula", &opts).as_ref(),
        "Text with $formula$"
    );
    assert_eq!(remend("$incomplete", &opts).as_ref(), "$incomplete$");
}

#[test]
fn katex_inline_enabled_complete_unchanged() {
    let opts = RemendOptions::default().inline_katex(true);
    assert_eq!(
        remend("$x^2 + y^2 = z^2$", &opts).as_ref(),
        "$x^2 + y^2 = z^2$"
    );
}

#[test]
fn katex_inline_enabled_odd() {
    let opts = RemendOptions::default().inline_katex(true);
    assert_eq!(
        remend("$first$ and $second", &opts).as_ref(),
        "$first$ and $second$"
    );
}

#[test]
fn katex_inline_enabled_escaped() {
    let opts = RemendOptions::default().inline_katex(true);
    assert_eq!(remend("Price is \\$100", &opts).as_ref(), "Price is \\$100");
}

#[test]
fn katex_math_with_underscores_unchanged() {
    assert_eq!(r("$$x_1 + y_2 = z_3$$").as_ref(), "$$x_1 + y_2 = z_3$$");
}

#[test]
fn katex_dollar_in_inline_code() {
    assert_eq!(
        r("Streamdown uses double dollar signs (`$$`) to delimit mathematical expressions.")
            .as_ref(),
        "Streamdown uses double dollar signs (`$$`) to delimit mathematical expressions."
    );
}

#[test]
fn katex_asterisks_in_math() {
    assert_eq!(r("$$\\mathbf{w}^{*}$$").as_ref(), "$$\\mathbf{w}^{*}$$");
}

// ===========================================================================
// Setext headings
// ===========================================================================

#[test]
fn setext_heading_dash() {
    assert_eq!(r("Heading\n-").as_ref(), "Heading\n-\u{200B}");
}

#[test]
fn setext_heading_double_dash() {
    assert_eq!(r("Heading\n--").as_ref(), "Heading\n--\u{200B}");
}

#[test]
fn setext_heading_equals() {
    assert_eq!(r("Heading\n=").as_ref(), "Heading\n=\u{200B}");
}

#[test]
fn setext_heading_triple_dash_unchanged() {
    // Three dashes is a valid horizontal rule.
    assert_eq!(r("Heading\n---").as_ref(), "Heading\n---");
}

#[test]
fn setext_heading_four_space_indent_is_code_block() {
    assert_eq!(r("Head\n    -").as_ref(), "Head\n    -");
}

#[test]
fn setext_heading_three_space_indent_fires() {
    assert_eq!(r("Head\n   -").as_ref(), "Head\n   -\u{200B}");
}

#[test]
fn setext_heading_tab_indent_is_code_block() {
    assert_eq!(r("Head\n\t-").as_ref(), "Head\n\t-");
}

#[test]
fn setext_heading_blank_line_between_unchanged() {
    assert_eq!(r("a\n\n-").as_ref(), "a\n\n-");
}

// ===========================================================================
// HTML tags
// ===========================================================================

#[test]
fn html_tag_incomplete_opening() {
    assert_eq!(r("Hello <div").as_ref(), "Hello");
}

#[test]
fn html_tag_incomplete_closing() {
    assert_eq!(r("Hello </div").as_ref(), "Hello");
}

#[test]
fn html_tag_incomplete_custom() {
    assert_eq!(r("Hello <custom").as_ref(), "Hello");
}

#[test]
fn html_tag_incomplete_at_start() {
    assert_eq!(r("<div").as_ref(), "");
}

#[test]
fn html_tag_complete_unchanged() {
    assert_eq!(r("Hello <div>").as_ref(), "Hello <div>");
}

#[test]
fn html_tag_complete_pair_unchanged() {
    assert_eq!(r("<div>content</div>").as_ref(), "<div>content</div>");
}

#[test]
fn html_tag_less_than_sign() {
    assert_eq!(r("3 < 5").as_ref(), "3 < 5");
}

#[test]
fn html_tag_partial_attributes() {
    assert_eq!(r("Hello <div class=\"foo").as_ref(), "Hello");
}

#[test]
fn html_tag_inside_code_block() {
    assert_eq!(r("```\n<div\n```").as_ref(), "```\n<div\n```");
}

// ===========================================================================
// Single tilde
// ===========================================================================

#[test]
fn single_tilde_between_words() {
    assert_eq!(r("20~25").as_ref(), "20\\~25");
}

#[test]
fn single_tilde_double_unchanged() {
    assert_eq!(r("~~strike~~").as_ref(), "~~strike~~");
}

#[test]
fn single_tilde_at_boundary() {
    assert_eq!(r("~start").as_ref(), "~start");
    assert_eq!(r("end~").as_ref(), "end~");
}

// ===========================================================================
// Comparison operators
// ===========================================================================

#[test]
fn comparison_in_list() {
    assert_eq!(r("- > 25").as_ref(), "- \\> 25");
}

#[test]
fn comparison_gte_in_list() {
    assert_eq!(r("- >= 25").as_ref(), "- \\>= 25");
}

#[test]
fn comparison_ordered_list() {
    assert_eq!(r("1. > 25").as_ref(), "1. \\> 25");
}

#[test]
fn comparison_not_blockquote() {
    // Not followed by digit — not a comparison.
    assert_eq!(r("- > text").as_ref(), "- > text");
}

#[test]
fn comparison_not_in_list() {
    assert_eq!(r("> 25").as_ref(), "> 25");
}

// ===========================================================================
// Options disabled
// ===========================================================================

#[test]
fn bold_disabled() {
    let opts = RemendOptions::default().bold(false);
    assert_eq!(remend("**bold text", &opts).as_ref(), "**bold text");
}

#[test]
fn links_disabled() {
    let opts = RemendOptions::default().links(false).images(false);
    assert_eq!(
        remend("[Click here](http://exam", &opts).as_ref(),
        "[Click here](http://exam"
    );
}

#[test]
fn all_disabled() {
    let opts = RemendOptions::default()
        .bold(false)
        .italic(false)
        .bold_italic(false)
        .inline_code(false)
        .strikethrough(false)
        .links(false)
        .images(false)
        .katex(false)
        .setext_headings(false)
        .html_tags(false)
        .single_tilde(false)
        .comparison_operators(false);
    assert_eq!(
        remend("**bold *italic `code [link", &opts).as_ref(),
        "**bold *italic `code [link"
    );
}

// ===========================================================================
// Cow efficiency
// ===========================================================================

#[test]
fn cow_borrowed_for_complete_markdown() {
    let text = "Hello **bold** and *italic* and `code` done.";
    assert!(matches!(r(text), Cow::Borrowed(_)));
}

#[test]
fn cow_borrowed_for_plain_text() {
    assert!(matches!(r("just plain text"), Cow::Borrowed(_)));
}

// ===========================================================================
// Code blocks
// ===========================================================================

#[test]
fn code_block_content_untouched() {
    let text = "```\n**bold\n*italic\n~~strike\n```";
    assert_eq!(r(text).as_ref(), text);
}

#[test]
fn code_block_python_underscores() {
    let text = "```python\ndef __init__(self):\n    pass\n```";
    assert_eq!(r(text).as_ref(), text);
}

#[test]
fn code_block_brackets_not_links() {
    let text = "```javascript\nconst arr = [1, 2, 3];\nconsole.log(arr[0]);\n```";
    assert_eq!(r(text).as_ref(), text);
}

#[test]
fn code_block_mermaid_star_syntax() {
    let text = "```mermaid\nstateDiagram-v2\n    [*] --> Idle\n    Idle --> Loading\n```";
    assert_eq!(r(text).as_ref(), text);
}

#[test]
fn incomplete_bold_after_code_block() {
    let text = "```css\ncode here\n```\n\n**incomplete bold";
    assert_eq!(
        r(text).as_ref(),
        "```css\ncode here\n```\n\n**incomplete bold**"
    );
}

#[test]
fn incomplete_italic_after_code_block() {
    let text = "```mermaid\nstateDiagram-v2\n    [*] --> Idle\n```\n\nHere is *incomplete italic";
    assert_eq!(
        r(text).as_ref(),
        "```mermaid\nstateDiagram-v2\n    [*] --> Idle\n```\n\nHere is *incomplete italic*"
    );
}

// ===========================================================================
// Issue #50: mid-line fence runs must not open code blocks
// ===========================================================================

// Per CommonMark §4.5, a fenced code block opens only when 3+ backticks (or
// tildes) appear at the start of a line with ≤3 leading spaces. A mid-line
// run is literal text and must leave downstream emphasis counters untouched.

#[test]
fn mid_line_backtick_run_does_not_open_fence_for_italic() {
    assert_eq!(r("hello ```\n*italic").as_ref(), "hello ```\n*italic*");
}

#[test]
fn mid_line_tilde_run_does_not_open_fence_for_bold() {
    // Disable strikethrough so the test isolates the fence-vs-prose decision:
    // the `~~~` must NOT open a fenced code block, so `**bold` gets closed.
    let opts = RemendOptions::default().strikethrough(false);
    assert_eq!(
        remend("text ~~~ more\n**bold", &opts).as_ref(),
        "text ~~~ more\n**bold**"
    );
}

#[test]
fn indented_fence_up_to_three_spaces_still_opens() {
    let text = "   ```\n**bold";
    // Leading 3 spaces is a valid fence per CommonMark §4.5; bold stays inside
    // the unclosed block and is NOT completed.
    assert_eq!(r(text).as_ref(), text);
}

#[test]
fn four_space_indent_is_not_a_fence_so_bold_is_completed() {
    // 4 leading spaces = indented code block, not a fenced one. The `**bold`
    // on the next line is prose and gets a closing `**`.
    assert_eq!(r("    ```\n**bold").as_ref(), "    ```\n**bold**");
}

#[test]
fn mid_line_fence_inside_same_line_as_emphasis() {
    // Mid-line ``` between two asterisks: the markers should complete.
    assert_eq!(r("a ``` *italic").as_ref(), "a ``` *italic*");
}

// ===========================================================================
// Mixed formatting
// ===========================================================================

#[test]
fn mixed_all_complete() {
    let text = "**bold** and *italic* and `code` and ~~strike~~";
    assert_eq!(r(text).as_ref(), text);
}

#[test]
fn mixed_bold_and_italic_incomplete() {
    assert_eq!(r("**bold and *italic").as_ref(), "**bold and *italic*");
}

#[test]
fn mixed_italic_with_bold() {
    assert_eq!(r("*italic with **bold").as_ref(), "*italic with **bold***");
}

#[test]
fn mixed_bold_with_code() {
    assert_eq!(r("**bold with `code").as_ref(), "**bold with `code**`");
}

#[test]
fn mixed_strikethrough_with_bold() {
    assert_eq!(
        r("~~strike with **bold").as_ref(),
        "~~strike with **bold**~~"
    );
}

#[test]
fn mixed_underscore_inside_bold() {
    assert_eq!(r("**_text").as_ref(), "**_text_**");
}

#[test]
fn mixed_underscore_italic_before_bold() {
    assert_eq!(r("_italic and **bold").as_ref(), "_italic and **bold**_");
}

#[test]
fn mixed_link_priority() {
    // Link handler has early return — further handlers don't run.
    assert_eq!(
        r("Text with [link and **bold").as_ref(),
        "Text with [link and **bold](streamdown:incomplete-link)"
    );
}

#[test]
fn mixed_bold_italic_complete() {
    assert_eq!(
        r("**bold with *italic* inside**").as_ref(),
        "**bold with *italic* inside**"
    );
}

#[test]
fn mixed_complex_complete() {
    let text = "# Heading\n\n**Bold text** with *italic* and `code`.\n\n- List item\n- Another item with ~~strike~~";
    assert_eq!(r(text).as_ref(), text);
}

#[test]
fn mixed_dollar_inside_bold() {
    assert_eq!(r("**bold with $x^2").as_ref(), "**bold with $x^2**");
}

// ===========================================================================
// Lists
// ===========================================================================

#[test]
fn list_asterisk_unchanged() {
    assert_eq!(
        r("* Item 1\n* Item 2\n* Item 3").as_ref(),
        "* Item 1\n* Item 2\n* Item 3"
    );
}

#[test]
fn list_single_item() {
    assert_eq!(r("* Single item").as_ref(), "* Single item");
}

#[test]
fn list_nested_unchanged() {
    assert_eq!(
        r("* Parent item\n  * Nested item 1\n  * Nested item 2").as_ref(),
        "* Parent item\n  * Nested item 1\n  * Nested item 2"
    );
}

#[test]
fn list_with_complete_italic() {
    assert_eq!(
        r("* Item with *italic* text\n* Another item").as_ref(),
        "* Item with *italic* text\n* Another item"
    );
}

#[test]
fn list_dash_with_bold() {
    assert_eq!(
        r("- Item 1\n- Item 2 with **bol").as_ref(),
        "- Item 1\n- Item 2 with **bol**"
    );
}

#[test]
fn list_emphasis_only_markers() {
    assert_eq!(r("- __").as_ref(), "- __");
    assert_eq!(r("- **").as_ref(), "- **");
    assert_eq!(r("- ***").as_ref(), "- ***");
    assert_eq!(r("- *").as_ref(), "- *");
    assert_eq!(r("- _").as_ref(), "- _");
    assert_eq!(r("- ~~").as_ref(), "- ~~");
}

#[test]
fn list_emphasis_with_text() {
    assert_eq!(r("- ** text after").as_ref(), "- ** text after**");
}

// ===========================================================================
// Horizontal rules
// ===========================================================================

#[test]
fn horizontal_rule_dashes() {
    assert_eq!(r("---").as_ref(), "---");
    assert_eq!(r("----").as_ref(), "----");
}

#[test]
fn horizontal_rule_asterisks() {
    assert_eq!(r("***").as_ref(), "***");
    assert_eq!(r("****").as_ref(), "****");
}

#[test]
fn horizontal_rule_underscores() {
    assert_eq!(r("___").as_ref(), "___");
    assert_eq!(r("____").as_ref(), "____");
}

#[test]
fn horizontal_rule_spaced() {
    assert_eq!(r("- - -").as_ref(), "- - -");
    assert_eq!(r("* * *").as_ref(), "* * *");
}

#[test]
fn horizontal_rule_after_text() {
    assert_eq!(r("Some text\n\n---").as_ref(), "Some text\n\n---");
}

#[test]
fn horizontal_rule_between_sections() {
    assert_eq!(
        r("Section 1\n\n---\n\nSection 2").as_ref(),
        "Section 1\n\n---\n\nSection 2"
    );
}

#[test]
fn partial_rules_streaming() {
    assert_eq!(r("--").as_ref(), "--");
    assert_eq!(r("**").as_ref(), "**");
    assert_eq!(r("__").as_ref(), "__");
}

// ===========================================================================
// Edge cases
// ===========================================================================

#[test]
fn standalone_markers_unchanged() {
    assert_eq!(r("**").as_ref(), "**");
    assert_eq!(r("__").as_ref(), "__");
    assert_eq!(r("***").as_ref(), "***");
    assert_eq!(r("*").as_ref(), "*");
    assert_eq!(r("_").as_ref(), "_");
    assert_eq!(r("~~").as_ref(), "~~");
    assert_eq!(r("`").as_ref(), "`");
}

#[test]
fn standalone_markers_with_space() {
    assert_eq!(r("** __").as_ref(), "** __");
    assert_eq!(r("* _ ~~ `").as_ref(), "* _ ~~ `");
}

#[test]
fn unicode_in_bold() {
    assert_eq!(r("**émoji 🎉").as_ref(), "**émoji 🎉**");
}

#[test]
fn unicode_in_code() {
    assert_eq!(r("`código").as_ref(), "`código`");
}

#[test]
fn html_entities_in_bold() {
    assert_eq!(r("**&lt;tag&gt;").as_ref(), "**&lt;tag&gt;**");
}

#[test]
fn whitespace_flanked_asterisks() {
    assert_eq!(r("5 * 0").as_ref(), "5 * 0");
    assert_eq!(r("x * y").as_ref(), "x * y");
    assert_eq!(r("2 * 3 * 4").as_ref(), "2 * 3 * 4");
}

#[test]
fn whitespace_asterisk_with_italic() {
    assert_eq!(r("5 * 0 and *italic").as_ref(), "5 * 0 and *italic*");
}

#[test]
fn escaped_asterisk() {
    assert_eq!(
        r("Text with \\* escaped asterisk").as_ref(),
        "Text with \\* escaped asterisk"
    );
}

#[test]
fn very_long_text() {
    let long = "a".repeat(10_000);
    let text = format!("{long} **bold");
    assert_eq!(remend(&text, &opts()).as_ref(), format!("{long} **bold**"));
}

#[test]
fn markdown_at_end_unchanged() {
    assert_eq!(r("text**").as_ref(), "text**");
    assert_eq!(r("text*").as_ref(), "text*");
    assert_eq!(r("`text`").as_ref(), "`text`");
    assert_eq!(r("text~~").as_ref(), "text~~");
}

#[test]
fn whitespace_before_incomplete() {
    assert_eq!(r("text **bold").as_ref(), "text **bold**");
    assert_eq!(r("text\n**bold").as_ref(), "text\n**bold**");
    assert_eq!(r("text\t`code").as_ref(), "text\t`code`");
}

// ===========================================================================
// Link text-only mode
// ===========================================================================

#[test]
fn link_text_only_mode() {
    let opts = RemendOptions::default().link_mode(LinkMode::TextOnly);
    assert_eq!(
        remend("Text with [incomplete link", &opts).as_ref(),
        "Text with incomplete link"
    );
}

#[test]
fn link_text_only_incomplete_url() {
    let opts = RemendOptions::default().link_mode(LinkMode::TextOnly);
    assert_eq!(
        remend("Visit [our site](https://exa", &opts).as_ref(),
        "Visit our site"
    );
}

#[test]
fn link_text_only_complete_unchanged() {
    let opts = RemendOptions::default().link_mode(LinkMode::TextOnly);
    assert_eq!(
        remend("[text](http://example.com)", &opts).as_ref(),
        "[text](http://example.com)"
    );
}

#[test]
fn link_text_only_image_removed() {
    let opts = RemendOptions::default().link_mode(LinkMode::TextOnly);
    assert_eq!(remend("Text ![incomplete image", &opts).as_ref(), "Text");
}

#[test]
fn link_text_only_rebuilds_ranges_after_bracket_strip() {
    // Regression: TextOnly mode strips the `[` byte mid-text, which shifts every
    // subsequent byte and invalidates pre-computed CodeBlockRanges. The italic
    // handler must see fresh ranges so it correctly classifies `*` as outside
    // the inline-code span and closes the emphasis.
    let opts = RemendOptions::default().link_mode(LinkMode::TextOnly);
    assert_eq!(remend("[abc`def`*xyz", &opts).as_ref(), "abc`def`*xyz*");
}

// ===========================================================================
// Streaming simulation
// ===========================================================================

#[test]
fn streaming_nested_formatting() {
    // Bold outer, italic inner — only inner closes (outer stays open).
    assert_eq!(
        r("This is **bold with *ital").as_ref(),
        "This is **bold with *ital*"
    );
}

#[test]
fn streaming_heading_with_emphasis() {
    assert_eq!(
        r("# Main Title\n## Subtitle with **emph").as_ref(),
        "# Main Title\n## Subtitle with **emph**"
    );
}

#[test]
fn streaming_blockquote_with_bold() {
    assert_eq!(r("> Quote with **bold").as_ref(), "> Quote with **bold**");
}

#[test]
fn streaming_table_with_bold() {
    assert_eq!(
        r("| Col1 | Col2 |\n|------|------|\n| **dat").as_ref(),
        "| Col1 | Col2 |\n|------|------|\n| **dat**"
    );
}

#[test]
fn streaming_crlf_between_bracket_and_url() {
    // Stream-chunk boundary splitting `](` from the URL with CRLF in between
    // must not mis-complete a URL whose `)` is on the following line.
    assert!(matches!(
        r("[text](\r\nhttp://example.com)"),
        Cow::Borrowed(_)
    ));
}

// ===========================================================================
// Bug fix: KaTeX inside fenced code blocks
// ===========================================================================

#[test]
fn katex_dollar_pairs_inside_fenced_code() {
    // $$ inside ``` should not be treated as math delimiters.
    assert_eq!(r("```\n$$x + y\n```").as_ref(), "```\n$$x + y\n```");
}

#[test]
fn katex_escaped_dollar_pairs() {
    // Escaped \$$ should not trigger math completion.
    assert_eq!(r("\\$$100").as_ref(), "\\$$100");
}

#[test]
fn inline_katex_inside_fenced_code() {
    let opts = RemendOptions::default().inline_katex(true);
    assert_eq!(
        remend("```\n$x + y\n```", &opts).as_ref(),
        "```\n$x + y\n```"
    );
}

// ===========================================================================
// Bug fix: text-only link mode forward scanning
// ===========================================================================

#[test]
fn text_only_link_with_preceding_complete_link() {
    let opts = RemendOptions::default().link_mode(LinkMode::TextOnly);
    // The first link is complete; only the second bracket should be stripped.
    assert_eq!(
        remend("[done](http://ok) and [incomplete", &opts).as_ref(),
        "[done](http://ok) and incomplete"
    );
}

#[test]
fn text_only_nested_brackets() {
    let opts = RemendOptions::default().link_mode(LinkMode::TextOnly);
    // Both [outer and [inner are incomplete — all stripped in one pass for idempotency.
    assert_eq!(
        remend("Text [outer [inner", &opts).as_ref(),
        "Text outer inner"
    );
}

// ===========================================================================
// Custom handler support
// ===========================================================================

#[test]
fn custom_handler_runs() {
    struct UpperHandler;
    impl RemendHandler for UpperHandler {
        fn handle<'a>(&self, text: &'a str) -> Cow<'a, str> {
            if text.contains("UPPER") {
                Cow::Borrowed(text)
            } else {
                Cow::Owned(text.to_uppercase())
            }
        }
        fn name(&self) -> &str {
            "upper"
        }
        fn priority(&self) -> i32 {
            200 // runs after all built-ins
        }
    }

    let opts = RemendOptions::default().handler(Box::new(UpperHandler));
    let result = remend("hello **world", &opts);
    // Built-in bold handler closes **, then custom handler uppercases.
    assert_eq!(result.as_ref(), "HELLO **WORLD**");
}

#[test]
fn custom_handler_priority_before_builtin() {
    struct PrependHandler;
    impl RemendHandler for PrependHandler {
        fn handle<'a>(&self, text: &'a str) -> Cow<'a, str> {
            if text.starts_with("PREFIX: ") {
                Cow::Borrowed(text)
            } else {
                Cow::Owned(format!("PREFIX: {}", text))
            }
        }
        fn name(&self) -> &str {
            "prepend"
        }
        fn priority(&self) -> i32 {
            -1 // runs before all built-ins
        }
    }

    let opts = RemendOptions::default()
        .bold(false) // disable bold so we can test just the prepend
        .handler(Box::new(PrependHandler));
    let result = remend("hello", &opts);
    assert_eq!(result.as_ref(), "PREFIX: hello");
}

// ===========================================================================
// Property-based tests (fuzz invariants)
// ===========================================================================

/// Biases toward characters remend actively inspects — markdown punctuation,
/// KaTeX/HTML/table delimiters, URL punctuation, CR/LF, and plain prose.
/// Capped at 80 chars so shrunk counterexamples stay readable.
fn markdown_soup() -> impl Strategy<Value = String> {
    prop::string::string_regex(r#"[ \n\r\t*_`~\[\]()<>{}|!#$\\/:'"a-zA-Z0-9.,-]{0,80}"#).unwrap()
}

/// Fence-rich generator: mixes prose, newlines, leading-space indents, and
/// backtick/tilde runs of assorted lengths so line-start vs mid-line fence
/// decisions get exercised. Used by the cross-scanner agreement proptest.
///
/// Tildes are given equal weight to backticks so the proptest regularly
/// exercises tilde fences, not just the more common backtick case.
fn fence_soup() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            2 => prop::string::string_regex(r"[a-z ]{0,6}").unwrap(),
            2 => Just("\n".into()),
            1 => Just("```".into()),
            1 => Just("````".into()),
            1 => Just("~~~".into()),
            1 => Just("~~~~".into()),
            1 => Just("   ".into()),
            1 => Just("    ".into()),
        ],
        0..20,
    )
    .prop_map(|parts: Vec<String>| parts.concat())
}

/// Tilde-only fence generator — the mid-line `~~~` case must hold just as
/// strictly as the backtick case, so give it a dedicated proptest.
fn tilde_fence_soup() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            2 => prop::string::string_regex(r"[a-z ]{0,6}").unwrap(),
            2 => Just("\n".into()),
            1 => Just("~~~".into()),
            1 => Just("~~~~".into()),
            1 => Just("   ".into()),
            1 => Just("    ".into()),
        ],
        0..20,
    )
    .prop_map(|parts: Vec<String>| parts.concat())
}

#[derive(Debug, Clone)]
struct OptionFlags {
    bold: bool,
    italic: bool,
    bold_italic: bool,
    inline_code: bool,
    strikethrough: bool,
    links: bool,
    images: bool,
    katex: bool,
    inline_katex: bool,
    setext_headings: bool,
    html_tags: bool,
    single_tilde: bool,
    comparison_operators: bool,
    link_mode: LinkMode,
}

impl OptionFlags {
    fn to_options(&self) -> RemendOptions {
        RemendOptions::default()
            .bold(self.bold)
            .italic(self.italic)
            .bold_italic(self.bold_italic)
            .inline_code(self.inline_code)
            .strikethrough(self.strikethrough)
            .links(self.links)
            .images(self.images)
            .katex(self.katex)
            .inline_katex(self.inline_katex)
            .setext_headings(self.setext_headings)
            .html_tags(self.html_tags)
            .single_tilde(self.single_tilde)
            .comparison_operators(self.comparison_operators)
            .link_mode(self.link_mode)
    }
}

fn arbitrary_options() -> impl Strategy<Value = OptionFlags> {
    // Nested tuples: proptest's Strategy impl caps at 10-tuples.
    (
        (
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
        ),
        (
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            prop_oneof![Just(LinkMode::Protocol), Just(LinkMode::TextOnly)],
        ),
    )
        .prop_map(
            |(
                (bold, italic, bold_italic, inline_code, strikethrough, links, images),
                (
                    katex,
                    inline_katex,
                    setext_headings,
                    html_tags,
                    single_tilde,
                    comparison_operators,
                    link_mode,
                ),
            )| OptionFlags {
                bold,
                italic,
                bold_italic,
                inline_code,
                strikethrough,
                links,
                images,
                katex,
                inline_katex,
                setext_headings,
                html_tags,
                single_tilde,
                comparison_operators,
                link_mode,
            },
        )
}

/// Records handler executions so the ordering property can assert the
/// pipeline respects priority.
#[derive(Clone)]
struct Recorder {
    tag: char,
    pri: i32,
    log: Arc<Mutex<String>>,
}

impl RemendHandler for Recorder {
    fn handle<'a>(&self, text: &'a str) -> Cow<'a, str> {
        self.log.lock().unwrap().push(self.tag);
        Cow::Borrowed(text)
    }
    fn name(&self) -> &str {
        "recorder"
    }
    fn priority(&self) -> i32 {
        self.pri
    }
}

/// Direct tripwire for issue #144 (idempotency violation on `"*0\t"`).
#[test]
fn idempotency_regression_0144() {
    let opts = RemendOptions::default();
    let once = remend("*0\t", &opts).into_owned();
    let twice = remend(&once, &opts).into_owned();
    assert_eq!(twice, once);
}

/// Pipeline-level idempotency tripwires for each proptest regression seed.
/// These exercise the full remend() pipeline (not individual handlers).
#[test]
fn idempotency_seeds_pipeline() {
    fn only(f: impl FnOnce(&mut RemendOptions)) -> RemendOptions {
        let mut o = RemendOptions {
            bold: false,
            italic: false,
            bold_italic: false,
            inline_code: false,
            strikethrough: false,
            links: false,
            images: false,
            katex: false,
            inline_katex: false,
            setext_headings: false,
            html_tags: false,
            single_tilde: false,
            comparison_operators: false,
            link_mode: LinkMode::Protocol,
            handlers: Vec::new(),
        };
        f(&mut o);
        o
    }

    let seeds: &[(&str, RemendOptions)] = &[
        (
            "_$",
            only(|o| {
                o.italic = true;
                o.inline_katex = true;
            }),
        ),
        (
            "[[",
            only(|o| {
                o.links = true;
            }),
        ),
        (
            "`\\",
            only(|o| {
                o.inline_code = true;
            }),
        ),
        (
            "*\\",
            only(|o| {
                o.italic = true;
            }),
        ),
        (
            "_*>0",
            only(|o| {
                o.italic = true;
            }),
        ),
        (
            "``[ [",
            only(|o| {
                o.links = true;
                o.link_mode = LinkMode::TextOnly;
            }),
        ),
        (
            "$**",
            only(|o| {
                o.bold = true;
                o.inline_katex = true;
            }),
        ),
    ];

    for (input, opts) in seeds {
        let once = remend(input, opts).into_owned();
        let twice = remend(&once, opts).into_owned();
        assert_eq!(
            twice, once,
            "idempotency violated for seed {input:?} with opts {opts:?}"
        );
    }
}

/// Deterministic coverage for known-risky prefix + trailing-backslash combos.
#[test]
fn idempotency_trailing_backslash_combos() {
    let inputs = ["**\\", "~~\\", "$*\\", "$$\\", "***\\"];
    let opts = RemendOptions::default();
    for input in inputs {
        let once = remend(input, &opts).into_owned();
        let twice = remend(&once, &opts).into_owned();
        assert_eq!(
            twice, once,
            "idempotency violated for trailing-backslash input {input:?}"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 128, ..ProptestConfig::default() })]

    #[test]
    fn fuzz_never_panics_on_arbitrary_utf8(chars in prop::collection::vec(any::<char>(), 0..256)) {
        let s: String = chars.iter().collect();
        let _ = remend(&s, &RemendOptions::default());
    }

    #[test]
    fn fuzz_never_panics_on_prefixes(chars in prop::collection::vec(any::<char>(), 0..128)) {
        // Streaming hits every prefix as tokens arrive; iterate all boundaries
        // per input rather than a single random cut.
        let s: String = chars.iter().collect();
        let opts = RemendOptions::default();
        let boundaries = s
            .char_indices()
            .map(|(i, _)| i)
            .chain(std::iter::once(s.len()));
        for cut in boundaries {
            let _ = remend(&s[..cut], &opts);
        }
    }

    // Idempotency stress test across every option combination. Collapsed from
    // four near-duplicates; the direct regression test above is the canonical
    // tripwire.
    #[test]
    fn fuzz_idempotent_all_option_combinations(
        s in markdown_soup(),
        flags in arbitrary_options(),
    ) {
        let once = remend(&s, &flags.to_options()).into_owned();
        let twice = remend(&once, &flags.to_options()).into_owned();
        prop_assert_eq!(twice, once);
    }

    #[test]
    fn fuzz_incomplete_autolinks_never_panic(
        prefix in markdown_soup(),
        url in r"<https?://[a-zA-Z0-9./:?#&=~%\-_]{0,40}",
        suffix in markdown_soup(),
    ) {
        let s = format!("{prefix}{url}{suffix}");
        let _ = remend(&s, &RemendOptions::default());
    }

    #[test]
    fn fuzz_reference_style_links_never_panic(
        s in r"\[[a-zA-Z0-9 ]{0,20}\](\[[a-zA-Z0-9]{0,10}\])?(\n\[[a-zA-Z0-9]{0,10}\]: https?://[a-zA-Z0-9./\-]{0,30})?",
    ) {
        let _ = remend(&s, &RemendOptions::default());
    }

    #[test]
    fn fuzz_incomplete_block_katex_never_panic(
        prefix in markdown_soup(),
        math in r"\$\$?[a-zA-Z0-9 ^_{}\\]{0,40}",
        suffix in markdown_soup(),
    ) {
        let s = format!("{prefix}{math}{suffix}");
        let _ = remend(&s, &RemendOptions::default());
    }

    #[test]
    fn fuzz_incomplete_inline_katex_never_panic(
        prefix in markdown_soup(),
        math in r"\$[a-zA-Z0-9 ^_{}\\]{0,40}",
        suffix in markdown_soup(),
    ) {
        let s = format!("{prefix}{math}{suffix}");
        let _ = remend(&s, &RemendOptions::default().inline_katex(true));
    }

    #[test]
    fn fuzz_handler_order_matches_priority_sort(
        specs in prop::collection::vec((0u8..26, -10i32..=200), 1..=5),
    ) {
        let log = Arc::new(Mutex::new(String::new()));

        // Disable every built-in so only the custom recorders run.
        let mut opts = RemendOptions::default()
            .bold(false)
            .italic(false)
            .bold_italic(false)
            .inline_code(false)
            .strikethrough(false)
            .links(false)
            .images(false)
            .katex(false)
            .inline_katex(false)
            .setext_headings(false)
            .html_tags(false)
            .single_tilde(false)
            .comparison_operators(false);

        for (idx, pri) in &specs {
            opts.handlers.push(Box::new(Recorder {
                tag: (b'a' + idx) as char,
                pri: *pri,
                log: log.clone(),
            }));
        }

        let _ = remend("x", &opts);

        let actual = log.lock().unwrap().clone();

        // Stable sort: equal priorities preserve insertion order — mirrors
        // `sort_by_key` in the pipeline at lib.rs.
        let mut expected_indices: Vec<usize> = (0..specs.len()).collect();
        expected_indices.sort_by_key(|&i| specs[i].1);
        let expected: String = expected_indices
            .into_iter()
            .map(|i| (b'a' + specs[i].0) as char)
            .collect();

        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn fuzz_custom_handlers_respect_priority_among_builtins(
        priorities in prop::collection::vec(-10i32..=200, 2..=4),
    ) {
        // Keep every built-in enabled so recorders interleave with real
        // handlers; assert recorder-relative order still matches priority sort
        // (stable sort preserves insertion order on ties).
        let log = Arc::new(Mutex::new(String::new()));
        let mut opts = RemendOptions::default();
        for (i, &pri) in priorities.iter().enumerate() {
            opts.handlers.push(Box::new(Recorder {
                tag: (b'a' + i as u8) as char,
                pri,
                log: log.clone(),
            }));
        }

        let _ = remend("plain text", &opts);

        let actual = log.lock().unwrap().clone();

        let mut expected_indices: Vec<usize> = (0..priorities.len()).collect();
        expected_indices.sort_by_key(|&i| priorities[i]);
        let expected: String = expected_indices
            .into_iter()
            .map(|i| (b'a' + i as u8) as char)
            .collect();

        prop_assert_eq!(actual, expected);
    }

    #[test]
    fn fuzz_trailing_single_space_stripped(s in markdown_soup()) {
        // Force exactly one trailing space (not two, which would be a line break).
        let trimmed = s.trim_end_matches(' ');
        let input = format!("{trimmed} ");
        let result = remend(&input, &RemendOptions::default()).into_owned();
        prop_assert!(
            !result.ends_with(' '),
            "single trailing space should be stripped; got {result:?}",
        );
    }

    // Issue #50 regression guard: `has_incomplete_code_fence` and
    // `is_inside_code_block` both drive fence detection and must agree on
    // fence openness, or streaming emphasis gets silently swallowed.
    // Reject any input containing a non-fence backtick run (length 1 or 2) so
    // the inline-code branch of `is_inside_code_block` stays out of the
    // invariant; any divergence is then purely fence-state divergence.
    #[test]
    fn fuzz_fence_scanners_agree_without_single_backticks(
        s in fence_soup().prop_filter(
            "all backtick runs must have length >= 3",
            |s| {
                let bytes = s.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'`' {
                        let mut run = 0;
                        while i + run < bytes.len() && bytes[i + run] == b'`' {
                            run += 1;
                        }
                        if run < 3 {
                            return false;
                        }
                        i += run;
                    } else {
                        i += 1;
                    }
                }
                true
            },
        ),
    ) {
        // Any backtick that remains is part of a 3+ run (fence-only).
        let has_open = has_incomplete_code_fence(&s);
        let ends_inside = is_inside_code_block(&s, s.len());
        prop_assert_eq!(
            has_open,
            ends_inside,
            "fence-open divergence on {:?}",
            s,
        );
    }

    // Same property but over tilde-only inputs — tildes never participate in
    // inline code, so no filter is needed.
    #[test]
    fn fuzz_tilde_fence_scanners_agree(s in tilde_fence_soup()) {
        // Self-guard: the invariant assumes no backtick characters; if a
        // future refactor adds backticks to the generator, this catches it.
        prop_assume!(!s.as_bytes().contains(&b'`'));
        let has_open = has_incomplete_code_fence(&s);
        let ends_inside = is_inside_code_block(&s, s.len());
        prop_assert_eq!(
            has_open,
            ends_inside,
            "tilde fence-open divergence on {:?}",
            s,
        );
    }

    #[test]
    fn fuzz_output_length_bounded(s in markdown_soup(), flags in arbitrary_options()) {
        // No handler should expand input by more than a handful of closing
        // markers; 64 bytes of headroom catches unbounded-growth regressions
        // without depending on #144.
        let opts = flags.to_options();
        let result = remend(&s, &opts);
        prop_assert!(
            result.len() <= s.len() + 64,
            "output grew by more than 64 bytes: input len={}, output len={}, input={s:?}",
            s.len(),
            result.len(),
        );
    }
}
