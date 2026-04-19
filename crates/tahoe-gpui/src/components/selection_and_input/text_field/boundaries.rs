//! Text boundary calculation utilities for cursor movement.
//!
//! Grapheme-cluster-aware via `unicode-segmentation` (UAX #29). A grapheme
//! cluster is what a human reads as "one character" — e.g. `👨‍👩‍👧` is three
//! codepoints joined by ZWJ but one grapheme; CJK composites like `が` can be
//! two codepoints (`か` + combining mark). Char-by-char navigation lands
//! cursors inside these clusters, which then break IME marking and selection.

use unicode_segmentation::UnicodeSegmentation;

use super::TextField;

impl TextField {
    /// Find the previous grapheme cluster boundary.
    pub(super) fn previous_boundary(&self, offset: usize) -> usize {
        self.content[..offset]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Find the next grapheme cluster boundary.
    pub(super) fn next_boundary(&self, offset: usize) -> usize {
        if offset >= self.content.len() {
            return self.content.len();
        }
        offset
            + self.content[offset..]
                .graphemes(true)
                .next()
                .map_or(0, str::len)
    }

    /// Find the previous word boundary.
    pub(super) fn previous_word_boundary(&self, offset: usize) -> usize {
        if offset == 0 {
            return 0;
        }
        let s = &self.content[..offset];
        let mut chars = s.char_indices().rev();
        // Skip whitespace backward
        let mut last_boundary = offset;
        for (idx, ch) in &mut chars {
            if !ch.is_whitespace() {
                last_boundary = idx;
                break;
            }
            last_boundary = idx;
        }
        // Skip word chars backward
        for (idx, ch) in chars {
            if ch.is_whitespace() {
                return idx + ch.len_utf8();
            }
            last_boundary = idx;
        }
        last_boundary.min(offset)
    }

    /// Find the next word boundary.
    pub(super) fn next_word_boundary(&self, offset: usize) -> usize {
        let s = &self.content[offset..];
        let len = self.content.len();
        if offset >= len {
            return len;
        }
        let mut chars = s.char_indices();
        // Skip word chars forward
        for (_idx, ch) in &mut chars {
            if ch.is_whitespace() {
                // Now skip whitespace forward
                for (idx2, ch2) in chars {
                    if !ch2.is_whitespace() {
                        return offset + idx2;
                    }
                }
                return len;
            }
        }
        len
    }
}

#[cfg(test)]
mod tests {
    //! Grapheme-cluster contract tests.
    //!
    //! These exercise the `unicode-segmentation` boundary logic directly so
    //! we can cover emoji ZWJ sequences, variation selectors, and CJK
    //! composites without constructing a full `TextField` (which requires a
    //! GPUI `Context`).
    use core::prelude::v1::test;
    use unicode_segmentation::UnicodeSegmentation;

    fn prev_boundary(content: &str, offset: usize) -> usize {
        content[..offset]
            .grapheme_indices(true)
            .next_back()
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn next_boundary(content: &str, offset: usize) -> usize {
        if offset >= content.len() {
            return content.len();
        }
        offset + content[offset..].graphemes(true).next().map_or(0, str::len)
    }

    #[test]
    fn prev_boundary_handles_ascii() {
        assert_eq!(prev_boundary("abc", 3), 2);
        assert_eq!(prev_boundary("abc", 1), 0);
    }

    #[test]
    fn prev_boundary_never_lands_mid_multibyte_char() {
        // "é" (U+00E9) is 2 bytes in UTF-8. Char-by-char previous would
        // return byte 1 (inside the char); grapheme-aware returns 0.
        let s = "é";
        assert_eq!(s.len(), 2);
        assert_eq!(prev_boundary(s, 2), 0);
    }

    #[test]
    fn prev_boundary_treats_zwj_emoji_as_one_cluster() {
        // "👨‍👩‍👧" = Man (4) + ZWJ (3) + Woman (4) + ZWJ (3) + Girl (4) = 18 bytes,
        // 5 codepoints, 1 grapheme cluster.
        let family = "👨\u{200D}👩\u{200D}👧";
        assert_eq!(family.chars().count(), 5);
        assert_eq!(family.graphemes(true).count(), 1);
        // Cursor just past the cluster should hop all the way back to 0.
        assert_eq!(prev_boundary(family, family.len()), 0);
    }

    #[test]
    fn next_boundary_advances_past_full_zwj_sequence() {
        let text = "a👨\u{200D}👩\u{200D}👧b";
        // Position right after "a" (byte 1). Next boundary should land
        // right after the entire emoji family — before "b".
        let after_family = next_boundary(text, 1);
        assert_eq!(&text[after_family..], "b");
    }

    #[test]
    fn next_boundary_handles_cjk_composite() {
        // Half-width katakana + combining voiced mark forms one grapheme.
        let s = "\u{FF76}\u{FF9E}"; // KA + ゛ = GA
        assert_eq!(s.chars().count(), 2);
        assert_eq!(s.graphemes(true).count(), 1);
        assert_eq!(next_boundary(s, 0), s.len());
    }

    #[test]
    fn backspace_pops_grapheme_not_byte() {
        // Documents the contract ComboBox backspace relies on.
        let mut s = String::from("a🇺🇸");
        if let Some((idx, _)) = s.grapheme_indices(true).next_back() {
            s.truncate(idx);
        }
        assert_eq!(s, "a");
    }
}
