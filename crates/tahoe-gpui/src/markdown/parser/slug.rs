//! GitHub-compatible slug generator for heading anchor IDs.

/// Produce a URL-fragment slug from a heading's plain text.
///
/// - Lowercases Unicode letters (e.g. `Héllo` → `héllo`).
/// - Keeps alphanumerics, `_`, and `-`.
/// - Replaces whitespace with `-`.
/// - Drops punctuation and other symbols.
/// - Collapses consecutive `-` and trims leading/trailing `-`.
/// - Empty result when the input yields no retained characters; callers
///   typically map this to `None` (no anchor).
pub(super) fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        if c.is_alphanumeric() {
            for lower in c.to_lowercase() {
                out.push(lower);
            }
        } else if c == '_' || c == '-' {
            out.push(c);
        } else if c.is_whitespace() {
            out.push('-');
        }
    }

    let mut collapsed = String::with_capacity(out.len());
    let mut prev_dash = false;
    for c in out.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push('-');
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }

    let trimmed = collapsed.trim_matches('-');
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::slugify;
    use core::prelude::v1::test;

    #[test]
    fn ascii_words() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn strips_punctuation() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
    }

    #[test]
    fn preserves_unicode_letters() {
        assert_eq!(slugify("Héllo Wörld"), "héllo-wörld");
    }

    #[test]
    fn collapses_consecutive_dashes() {
        assert_eq!(slugify("a---b"), "a-b");
    }

    #[test]
    fn trims_edges() {
        assert_eq!(slugify("  x  "), "x");
        assert_eq!(slugify("--x--"), "x");
    }

    #[test]
    fn all_punctuation_is_empty() {
        assert_eq!(slugify("!!!"), "");
        assert_eq!(slugify("— — —"), "");
    }

    #[test]
    fn preserves_underscore() {
        assert_eq!(slugify("foo_bar"), "foo_bar");
    }

    #[test]
    fn mixed_case_and_numbers() {
        assert_eq!(slugify("Section 2: Overview"), "section-2-overview");
    }

    #[test]
    fn cjk_letters_survive() {
        // `is_alphanumeric` returns true for CJK ideographs; they're
        // retained verbatim since they have no case-folding.
        assert_eq!(slugify("中文 标题"), "中文-标题");
    }

    #[test]
    fn emoji_is_dropped() {
        // Most emoji are `Other_Symbol`, not alphanumeric, so they fall
        // out of the slug. Surrounding words keep their spacing.
        assert_eq!(slugify("Hello 🎉 World"), "hello-world");
    }

    #[test]
    fn numeric_leading_preserved() {
        assert_eq!(slugify("1. Overview"), "1-overview");
    }

    #[test]
    fn combining_diacritic_is_dropped() {
        // Decomposed form: base letter + combining mark. The combining
        // mark (`\u{0301}`) is not alphanumeric and gets dropped, so the
        // result diverges from the NFC form. Documented so callers that
        // feed externally-normalized text know what to expect.
        let decomposed = "cafe\u{0301}";
        assert_eq!(slugify(decomposed), "cafe");
    }

    #[test]
    fn tab_is_treated_as_whitespace() {
        assert_eq!(slugify("foo\tbar"), "foo-bar");
    }

    #[test]
    fn very_long_input_does_not_panic() {
        // Pathological input should not allocate beyond O(n) or panic.
        let long = "a".repeat(10_000);
        let out = slugify(&long);
        assert_eq!(out.len(), 10_000);
    }

    // Property tests pin three invariants that any future rewrite of
    // `slugify` must preserve: idempotence (a slug slugifies to itself),
    // no edge dashes, no consecutive dashes.
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn slug_is_idempotent(s in ".{0,80}") {
            let once = slugify(&s);
            let twice = slugify(&once);
            prop_assert_eq!(once, twice);
        }

        #[test]
        fn slug_has_no_edge_dashes(s in ".{0,80}") {
            let out = slugify(&s);
            prop_assert!(!out.starts_with('-'), "slug {:?} starts with '-'", out);
            prop_assert!(!out.ends_with('-'), "slug {:?} ends with '-'", out);
        }

        #[test]
        fn slug_has_no_consecutive_dashes(s in ".{0,80}") {
            let out = slugify(&s);
            prop_assert!(!out.contains("--"), "slug {:?} contains '--'", out);
        }
    }
}
