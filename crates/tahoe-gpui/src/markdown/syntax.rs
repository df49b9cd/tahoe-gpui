//! Tree-sitter based syntax highlighting for code blocks.

use crate::foundations::theme::SyntaxColors;
use gpui::{FontWeight, HighlightStyle, Hsla, SharedString};
use std::cell::RefCell;
use std::sync::OnceLock;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

thread_local! {
    static HIGHLIGHTER: RefCell<Highlighter> = RefCell::new(Highlighter::new());
}

/// Recognized highlight names that map to theme colors.
/// Order matters — index into this array becomes the Highlight.0 value.
pub const HIGHLIGHT_NAMES: &[&str] = &[
    "keyword",
    "string",
    "comment",
    "function",
    "type",
    "variable",
    "number",
    "operator",
    "punctuation",
    "constant",
    "attribute",
    "tag",
    "function.builtin",
    "type.builtin",
    "variable.builtin",
    "string.special",
    "constant.builtin",
    "property",
    "label",
    "constructor",
    "embedded",
];

/// Maps a highlight index to a theme color.
pub fn highlight_color(index: usize, syntax: &SyntaxColors) -> Option<Hsla> {
    match HIGHLIGHT_NAMES.get(index)? {
        &"keyword" => Some(syntax.keyword),
        &"string" | &"string.special" => Some(syntax.string),
        &"comment" => Some(syntax.comment),
        &"function" | &"function.builtin" => Some(syntax.function),
        &"type" | &"type.builtin" | &"constructor" => Some(syntax.r#type),
        &"variable" | &"variable.builtin" | &"property" => Some(syntax.variable),
        &"number" => Some(syntax.number),
        &"operator" => Some(syntax.operator),
        &"punctuation" => Some(syntax.punctuation),
        &"constant" | &"constant.builtin" => Some(syntax.constant),
        &"attribute" => Some(syntax.attribute),
        &"tag" | &"label" => Some(syntax.tag),
        _ => None,
    }
}

/// A span of highlighted text.
pub struct HighlightedSpan {
    pub text: String,
    pub highlight_index: Option<usize>,
}

/// Highlight source code with the given language name.
/// Returns spans of text with optional highlight indices (mapping to HIGHLIGHT_NAMES).
/// Falls back to a single unhighlighted span if the language is unknown.
pub fn highlight_code(code: &str, language: &str) -> Vec<HighlightedSpan> {
    let config = match get_language_config(language) {
        Some(c) => c,
        None => {
            return vec![HighlightedSpan {
                text: code.to_string(),
                highlight_index: None,
            }];
        }
    };

    HIGHLIGHTER.with_borrow_mut(|highlighter| {
        let events = match highlighter.highlight(config, code.as_bytes(), None, |_| None) {
            Ok(events) => events,
            Err(_) => {
                return vec![HighlightedSpan {
                    text: code.to_string(),
                    highlight_index: None,
                }];
            }
        };

        let mut spans = Vec::new();
        let mut current_highlight: Option<usize> = None;

        for event in events {
            match event {
                Ok(HighlightEvent::Source { start, end }) => {
                    if start < end && end <= code.len() {
                        spans.push(HighlightedSpan {
                            text: code[start..end].to_string(),
                            highlight_index: current_highlight,
                        });
                    }
                }
                Ok(HighlightEvent::HighlightStart(highlight)) => {
                    current_highlight = Some(highlight.0);
                }
                Ok(HighlightEvent::HighlightEnd) => {
                    current_highlight = None;
                }
                Err(_) => break,
            }
        }

        if spans.is_empty() {
            spans.push(HighlightedSpan {
                text: code.to_string(),
                highlight_index: None,
            });
        }

        spans
    })
}

/// Build GPUI highlight styles from code spans + syntax colors.
/// Returns (full_text, highlight_ranges) suitable for StyledText.
pub fn build_styled_highlights(
    code: &str,
    language: &str,
    syntax: &SyntaxColors,
) -> (SharedString, Vec<(std::ops::Range<usize>, HighlightStyle)>) {
    let spans = highlight_code(code, language);
    let mut full_text = String::new();
    let mut highlights = Vec::new();

    for span in &spans {
        let start = full_text.len();
        full_text.push_str(&span.text);
        let end = full_text.len();

        if let Some(idx) = span.highlight_index
            && let Some(color) = highlight_color(idx, syntax)
        {
            let style = HighlightStyle {
                color: Some(color),
                font_weight: if matches!(HIGHLIGHT_NAMES.get(idx), Some(&"keyword")) {
                    Some(FontWeight::SEMIBOLD)
                } else {
                    None
                },
                ..Default::default()
            };
            highlights.push((start..end, style));
        }
    }

    (SharedString::from(full_text), highlights)
}

// ─── Language configurations (lazily initialized) ────────────────────────────

macro_rules! define_language {
    ($fn_name:ident, $lang_fn:expr, $name:expr, $highlights:expr) => {
        fn $fn_name() -> &'static HighlightConfiguration {
            static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
            CONFIG.get_or_init(|| {
                let language = $lang_fn;
                let mut config = HighlightConfiguration::new(
                    language.into(),
                    $name,
                    $highlights,
                    "", // injection query
                    "", // locals query
                )
                .expect(concat!("Failed to create highlight config for ", $name));
                config.configure(HIGHLIGHT_NAMES);
                config
            })
        }
    };
}

define_language!(
    rust_config,
    tree_sitter_rust::LANGUAGE,
    "rust",
    tree_sitter_rust::HIGHLIGHTS_QUERY
);

define_language!(
    python_config,
    tree_sitter_python::LANGUAGE,
    "python",
    tree_sitter_python::HIGHLIGHTS_QUERY
);

define_language!(
    javascript_config,
    tree_sitter_javascript::LANGUAGE,
    "javascript",
    tree_sitter_javascript::HIGHLIGHT_QUERY
);

define_language!(
    typescript_config,
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
    "typescript",
    tree_sitter_typescript::HIGHLIGHTS_QUERY
);

define_language!(
    tsx_config,
    tree_sitter_typescript::LANGUAGE_TSX,
    "tsx",
    tree_sitter_typescript::HIGHLIGHTS_QUERY
);

define_language!(
    json_config,
    tree_sitter_json::LANGUAGE,
    "json",
    tree_sitter_json::HIGHLIGHTS_QUERY
);

fn toml_config() -> &'static HighlightConfiguration {
    static CONFIG: OnceLock<HighlightConfiguration> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let language = tree_sitter_toml_ng::LANGUAGE;
        let mut config = HighlightConfiguration::new(
            language.into(),
            "toml",
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
            "",
            "",
        )
        .expect("Failed to create highlight config for toml");
        config.configure(HIGHLIGHT_NAMES);
        config
    })
}

define_language!(
    bash_config,
    tree_sitter_bash::LANGUAGE,
    "bash",
    tree_sitter_bash::HIGHLIGHT_QUERY
);

define_language!(
    go_config,
    tree_sitter_go::LANGUAGE,
    "go",
    tree_sitter_go::HIGHLIGHTS_QUERY
);

define_language!(
    c_config,
    tree_sitter_c::LANGUAGE,
    "c",
    tree_sitter_c::HIGHLIGHT_QUERY
);

define_language!(
    cpp_config,
    tree_sitter_cpp::LANGUAGE,
    "cpp",
    tree_sitter_cpp::HIGHLIGHT_QUERY
);

define_language!(
    css_config,
    tree_sitter_css::LANGUAGE,
    "css",
    tree_sitter_css::HIGHLIGHTS_QUERY
);

define_language!(
    html_config,
    tree_sitter_html::LANGUAGE,
    "html",
    tree_sitter_html::HIGHLIGHTS_QUERY
);

/// Look up a language highlight configuration by name.
fn get_language_config(name: &str) -> Option<&'static HighlightConfiguration> {
    match name.to_lowercase().as_str() {
        "rust" | "rs" => Some(rust_config()),
        "python" | "py" => Some(python_config()),
        "javascript" | "js" | "jsx" => Some(javascript_config()),
        "typescript" | "ts" => Some(typescript_config()),
        "tsx" => Some(tsx_config()),
        "json" | "jsonc" => Some(json_config()),
        "toml" => Some(toml_config()),
        "bash" | "sh" | "shell" | "zsh" => Some(bash_config()),
        "go" | "golang" => Some(go_config()),
        "c" => Some(c_config()),
        "cpp" | "c++" | "cxx" | "cc" => Some(cpp_config()),
        "css" => Some(css_config()),
        "html" | "htm" => Some(html_config()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{build_styled_highlights, highlight_code, highlight_color};
    use crate::foundations::theme::TahoeTheme;
    use core::prelude::v1::test;

    #[test]
    fn highlight_rust_fn() {
        let spans = highlight_code("fn main() {}", "rust");
        assert!(
            spans.len() > 1,
            "expected multiple spans, got {}",
            spans.len()
        );
        // "fn" should be highlighted as keyword (index 0)
        assert!(
            spans
                .iter()
                .any(|s| s.text == "fn" && s.highlight_index == Some(0))
        );
    }

    #[test]
    fn highlight_python_def() {
        let spans = highlight_code("def hello():\n    pass", "python");
        assert!(spans.len() > 1);
        assert!(spans.iter().any(|s| s.highlight_index.is_some()));
    }

    #[test]
    fn highlight_javascript_var() {
        let spans = highlight_code("const x = 42;", "javascript");
        assert!(spans.len() > 1);
    }

    #[test]
    fn highlight_json() {
        let spans = highlight_code(r#"{"key": "value"}"#, "json");
        assert!(spans.len() > 1);
    }

    #[test]
    fn highlight_unknown_language() {
        let spans = highlight_code("hello world", "unknown_lang");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "hello world");
        assert!(spans[0].highlight_index.is_none());
    }

    #[test]
    fn highlight_empty_code() {
        let spans = highlight_code("", "rust");
        assert_eq!(spans.len(), 1);
        assert!(spans[0].highlight_index.is_none());
    }

    #[test]
    fn highlight_go_func() {
        let spans = highlight_code("func main() {}", "go");
        assert!(spans.len() > 1);
    }

    #[test]
    fn highlight_c_include() {
        let spans = highlight_code("#include <stdio.h>\nint main() {}", "c");
        assert!(spans.len() > 1);
    }

    #[test]
    fn highlight_cpp_class() {
        let spans = highlight_code("class Foo { public: int x; };", "cpp");
        assert!(spans.len() > 1);
    }

    #[test]
    fn highlight_css_rule() {
        let spans = highlight_code("body { color: red; }", "css");
        assert!(spans.len() > 1);
    }

    #[test]
    fn highlight_html_tag() {
        let spans = highlight_code("<div class=\"test\">hello</div>", "html");
        assert!(spans.len() > 1);
    }

    #[test]
    fn language_aliases() {
        for alias in &[
            "rs", "py", "js", "ts", "sh", "zsh", "jsx", "tsx", "shell", "jsonc", "golang", "c++",
            "cxx", "cc", "htm",
        ] {
            let spans = highlight_code("x", alias);
            // Should not fall back to unknown (single unhighlighted span may still happen for "x")
            // but the config should exist
            assert!(!spans.is_empty(), "no spans for alias {}", alias);
        }
    }

    #[test]
    fn build_styled_highlights_preserves_text() {
        let theme = TahoeTheme::dark();
        let code = "fn main() {}";
        let (text, _highlights) = build_styled_highlights(code, "rust", &theme.syntax);
        assert_eq!(text.as_ref(), code);
    }

    #[test]
    fn build_styled_highlights_has_colors() {
        let theme = TahoeTheme::dark();
        let (_, highlights) = build_styled_highlights("fn main() {}", "rust", &theme.syntax);
        assert!(!highlights.is_empty(), "expected some highlight ranges");
    }

    #[test]
    fn highlight_color_keyword() {
        let theme = TahoeTheme::dark();
        let color = highlight_color(0, &theme.syntax); // index 0 = "keyword"
        assert!(color.is_some());
        assert_eq!(color.unwrap(), theme.syntax.keyword);
    }

    #[test]
    fn highlight_color_string() {
        let theme = TahoeTheme::dark();
        let color = highlight_color(1, &theme.syntax); // index 1 = "string"
        assert!(color.is_some());
        assert_eq!(color.unwrap(), theme.syntax.string);
    }

    #[test]
    fn highlight_color_out_of_bounds() {
        let theme = TahoeTheme::dark();
        let color = highlight_color(999, &theme.syntax);
        assert!(color.is_none());
    }
}
