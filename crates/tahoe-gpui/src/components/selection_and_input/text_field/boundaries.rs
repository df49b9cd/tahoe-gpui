//! Text boundary calculation utilities for cursor movement.

use super::TextField;

impl TextField {
    /// Find the previous grapheme cluster boundary.
    pub(super) fn previous_boundary(&self, offset: usize) -> usize {
        self.content[..offset]
            .char_indices()
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
                .chars()
                .next()
                .map_or(0, |c| c.len_utf8())
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
