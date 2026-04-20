//! Text boundary calculation utilities for cursor movement.
//!
//! Grapheme-cluster-aware via `unicode-segmentation` (UAX #29). A grapheme
//! cluster is what a human reads as "one character" — e.g. `👨‍👩‍👧` is three
//! codepoints joined by ZWJ but one grapheme; CJK composites like `が` can be
//! two codepoints (`か` + combining mark). Char-by-char navigation lands
//! cursors inside these clusters, which then break IME marking and selection.

use unicode_segmentation::UnicodeSegmentation;

use super::TextField;

/// Walk `index` backward to the nearest valid UTF-8 char boundary within `s`.
/// If `index` is already on a char boundary, it's returned as-is.
fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    let mut i = index;
    while !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

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

    /// Clamp `offset` to `content.len()`, then snap backward to the previous
    /// grapheme boundary if it landed mid-grapheme (e.g. after an undo restored
    /// shorter content). Offsets already at a valid boundary are returned
    /// unchanged. Use for selection range *starts*.
    ///
    /// Safe to call with offsets that are not valid UTF-8 boundaries — the
    /// method snaps to the start of the enclosing char boundary.
    pub(super) fn clamp_to_grapheme(&self, offset: usize) -> usize {
        let clamped = offset.min(self.content.len());
        if clamped == 0 || clamped == self.content.len() {
            return clamped;
        }
        // Not on a UTF-8 char boundary → definitely mid-grapheme.
        // Snap to the start of the enclosing char.
        if !self.content.is_char_boundary(clamped) {
            return floor_char_boundary(&self.content, clamped);
        }
        let prev = self.previous_boundary(clamped);
        if self.next_boundary(prev) == clamped {
            clamped
        } else {
            prev
        }
    }

    /// Like `clamp_to_grapheme`, but snaps *forward* to the next grapheme
    /// boundary when mid-grapheme. Use for selection range *ends* to avoid
    /// shrinking the selection from the right edge.
    ///
    /// Safe to call with offsets that are not valid UTF-8 boundaries.
    pub(super) fn clamp_to_grapheme_forward(&self, offset: usize) -> usize {
        let clamped = offset.min(self.content.len());
        if clamped == 0 || clamped == self.content.len() {
            return clamped;
        }
        // Not on a UTF-8 char boundary → snap to the end of the char.
        if !self.content.is_char_boundary(clamped) {
            let start = floor_char_boundary(&self.content, clamped);
            return start
                + self.content[start..]
                    .chars()
                    .next()
                    .map_or(0, char::len_utf8);
        }
        let next = self.next_boundary(clamped);
        if self.previous_boundary(next) == clamped {
            clamped
        } else {
            next
        }
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

    fn floor_char_boundary(s: &str, index: usize) -> usize {
        if index >= s.len() {
            return s.len();
        }
        let mut i = index;
        while !s.is_char_boundary(i) {
            i -= 1;
        }
        i
    }

    fn clamp_backward(content: &str, offset: usize) -> usize {
        let clamped = offset.min(content.len());
        if clamped == 0 || clamped == content.len() {
            return clamped;
        }
        if !content.is_char_boundary(clamped) {
            return floor_char_boundary(content, clamped);
        }
        let prev = prev_boundary(content, clamped);
        if next_boundary(content, prev) == clamped {
            clamped
        } else {
            prev
        }
    }

    fn clamp_forward(content: &str, offset: usize) -> usize {
        let clamped = offset.min(content.len());
        if clamped == 0 || clamped == content.len() {
            return clamped;
        }
        if !content.is_char_boundary(clamped) {
            let start = floor_char_boundary(content, clamped);
            return start + content[start..].chars().next().map_or(0, char::len_utf8);
        }
        let next = next_boundary(content, clamped);
        if prev_boundary(content, next) == clamped {
            clamped
        } else {
            next
        }
    }

    #[test]
    fn clamp_backward_passthrough_at_valid_boundary() {
        // Byte 3 is end of "aé" — a valid grapheme boundary.
        assert_eq!(clamp_backward("aé", 3), 3);
        assert_eq!(clamp_backward("abc", 1), 1);
    }

    #[test]
    fn clamp_backward_snaps_mid_grapheme_back() {
        // "é" is 2 bytes. Byte 1 is mid-grapheme.
        assert_eq!(clamp_backward("é", 1), 0);
    }

    #[test]
    fn clamp_backward_overshoot_clamps_to_len() {
        // Offset 99 overshoots "ab" (len 2) → clamp to 2, which is end-of-string.
        assert_eq!(clamp_backward("ab", 99), 2);
    }

    #[test]
    fn clamp_backward_empty_string_returns_zero() {
        assert_eq!(clamp_backward("", 0), 0);
        assert_eq!(clamp_backward("", 5), 0);
    }

    #[test]
    fn clamp_forward_passthrough_at_valid_boundary() {
        assert_eq!(clamp_forward("aé", 3), 3);
        assert_eq!(clamp_forward("abc", 1), 1);
    }

    #[test]
    fn clamp_forward_snaps_mid_grapheme_ahead() {
        // "é" is 2 bytes. Byte 1 is mid-grapheme → snap forward to 2.
        assert_eq!(clamp_forward("é", 1), 2);
    }

    #[test]
    fn clamp_forward_overshoot_clamps_to_len() {
        assert_eq!(clamp_forward("ab", 99), 2);
    }

    #[test]
    fn clamp_forward_empty_string_returns_zero() {
        assert_eq!(clamp_forward("", 0), 0);
        assert_eq!(clamp_forward("", 5), 0);
    }

    #[test]
    fn clamp_backward_vs_forward_mid_grapheme() {
        // e + combining acute (U+0301) = "é" as two codepoints, one grapheme.
        // Byte 1 (between 'e' and the combining mark) is a valid char
        // boundary but mid-grapheme. Backward snaps to 0; forward snaps to
        // the full grapheme length.
        let s = "e\u{0301}";
        assert_eq!(s.len(), 3); // 1 byte 'e' + 2 bytes combining mark
        assert_eq!(s.graphemes(true).count(), 1);
        assert_eq!(clamp_backward(s, 1), 0);
        assert_eq!(clamp_forward(s, 1), s.len());
    }

    #[test]
    fn clamp_backward_vs_forward_mid_byte() {
        // "é" (U+00E9) is 2 bytes. Byte 1 is not a char boundary.
        // Backward snaps to 0; forward snaps to 2 (end of the char).
        let s = "é";
        assert_eq!(s.len(), 2);
        assert_eq!(clamp_backward(s, 1), 0);
        assert_eq!(clamp_forward(s, 1), 2);
    }
}
