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
}
