//! Tests for code block components.

use super::{
    CodeBlockActions, CodeBlockContainer, CodeBlockContent, CodeBlockFilename, CodeBlockHeader,
    CodeBlockLanguageSelector, CodeBlockTitle, CodeBlockView, LanguageVariant,
};
use core::prelude::v1::test;
use gpui::{ParentElement, div};

// -- CodeBlockContent tests -----------------------------------------------

#[test]
fn code_block_content_defaults() {
    let content = CodeBlockContent::new("let x = 1;");
    assert!(content.language.is_none());
    assert!(!content.show_line_numbers);
    assert_eq!(content.code, "let x = 1;");
}

#[test]
fn code_block_content_with_language() {
    let content = CodeBlockContent::new("x").language(Some("rust".into()));
    assert_eq!(content.language.as_deref(), Some("rust"));
}

#[test]
fn code_block_content_with_line_numbers() {
    let content = CodeBlockContent::new("x").show_line_numbers(true);
    assert!(content.show_line_numbers);
}

// -- CodeBlockContainer tests ---------------------------------------------

#[test]
fn code_block_container_starts_empty() {
    let container = CodeBlockContainer::new();
    assert!(container.children.is_empty());
}

// -- CodeBlockHeader tests ------------------------------------------------

#[test]
fn code_block_header_starts_empty() {
    let header = CodeBlockHeader::new();
    assert!(header.children_left.is_empty());
    assert!(header.children_right.is_empty());
}

// -- CodeBlockTitle tests -------------------------------------------------

#[test]
fn code_block_title_starts_empty() {
    let title = CodeBlockTitle::new();
    assert!(title.children.is_empty());
}

// -- CodeBlockFilename tests ----------------------------------------------

#[test]
fn code_block_filename_stores_name() {
    let filename = CodeBlockFilename::new("main.rs");
    assert_eq!(filename.name.as_ref(), "main.rs");
}

// -- CodeBlockActions tests -----------------------------------------------

#[test]
fn code_block_actions_starts_empty() {
    let actions = CodeBlockActions::new();
    assert!(actions.children.is_empty());
}

// -- CodeBlockLanguageSelector tests --------------------------------------

#[test]
fn language_selector_defaults() {
    let variants = vec![LanguageVariant {
        label: "Rust".into(),
        language: "rust".into(),
        code: "let x = 1;".into(),
    }];
    let selector = CodeBlockLanguageSelector::new("sel", variants);
    assert_eq!(selector.active_index, 0);
    assert!(!selector.is_open);
    assert!(selector.on_change.is_none());
    assert!(selector.on_toggle.is_none());
}

#[test]
fn language_selector_active_index() {
    let variants = vec![
        LanguageVariant {
            label: "Rust".into(),
            language: "rust".into(),
            code: "x".into(),
        },
        LanguageVariant {
            label: "Python".into(),
            language: "python".into(),
            code: "y".into(),
        },
    ];
    let selector = CodeBlockLanguageSelector::new("sel", variants).active(1);
    assert_eq!(selector.active_index, 1);
}

#[test]
fn language_selector_empty_variants() {
    let selector = CodeBlockLanguageSelector::new("sel", vec![]);
    assert!(selector.variants.is_empty());
    assert_eq!(selector.active_index, 0);
    assert!(!selector.is_open);
}

// -- CodeBlockView tests --------------------------------------------------

#[test]
fn code_block_builder_defaults() {
    let view = CodeBlockView::new("let x = 1;");
    assert!(view.language.is_none());
    assert!(view.filename.is_none());
    assert!(!view.show_line_numbers);
    assert!(view.show_header);
    assert!(view.max_lines.is_none());
    assert!(!view.expanded);
    assert!(view.language_variants.is_empty());
    assert_eq!(view.active_variant_index, 0);
    assert!(view.on_variant_change.is_none());
    assert!(view.custom_header.is_none());
    assert!(view.custom_footer.is_none());
}

#[test]
fn code_block_from_parts_defaults() {
    let view = CodeBlockView::from_parts("fn main() {}");
    assert!(view.custom_header.is_none());
    assert!(view.custom_footer.is_none());
    assert_eq!(view.code, "fn main() {}");
}

#[test]
fn code_block_builder_chain() {
    let view = CodeBlockView::new("fn main() {}")
        .language(Some("rust".into()))
        .filename("main.rs")
        .show_line_numbers(true)
        .show_header(false)
        .max_lines(10);
    assert_eq!(view.language.as_deref(), Some("rust"));
    assert_eq!(view.filename.as_ref().map(|s| s.as_ref()), Some("main.rs"));
    assert!(view.show_line_numbers);
    assert!(!view.show_header);
    assert_eq!(view.max_lines, Some(10));
}

#[test]
fn language_variant_resolves_code() {
    let variants = vec![
        LanguageVariant {
            label: "Rust".into(),
            language: "rust".into(),
            code: "let x = 1;".into(),
        },
        LanguageVariant {
            label: "Python".into(),
            language: "python".into(),
            code: "x = 1".into(),
        },
    ];
    let view = CodeBlockView::new("")
        .language_variants(variants.clone())
        .active_variant_index(1);
    assert_eq!(view.active_variant_index, 1);
    assert_eq!(view.language_variants.len(), 2);
}

#[test]
fn active_variant_clamped() {
    let variants = vec![LanguageVariant {
        label: "Rust".into(),
        language: "rust".into(),
        code: "let x = 1;".into(),
    }];
    let view = CodeBlockView::new("")
        .language_variants(variants)
        .active_variant_index(99);
    // When rendered, active_variant_index is clamped to len-1
    assert_eq!(view.active_variant_index, 99);
    // The clamping happens at render time, not at build time
}

#[test]
fn from_parts_with_custom_header() {
    let view = CodeBlockView::from_parts("code").header(CodeBlockHeader::new());
    assert!(view.custom_header.is_some());
}

#[test]
fn from_parts_with_custom_footer() {
    let view = CodeBlockView::from_parts("code").footer(div().child("custom footer"));
    assert!(view.custom_footer.is_some());
}

#[test]
fn expanded_builder() {
    let view = CodeBlockView::new("code").expanded(true);
    assert!(view.expanded);
}

#[test]
fn on_variant_change_builder() {
    let view = CodeBlockView::new("code").on_variant_change(|_idx, _window, _cx| {});
    assert!(view.on_variant_change.is_some());
}
