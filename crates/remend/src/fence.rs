//! Fenced-code-block and inline-code scanning primitives.
//!
//! Single source of truth for CommonMark §4.5 fence detection and the shared
//! state machine that powers `ranges::compute_code_ranges`,
//! `ranges::compute_complete_inline_code_ranges`, `utils::is_inside_code_block`,
//! and `utils::is_within_complete_inline_code`. Each of those callers was
//! previously a standalone re-implementation of the same loop; now they are
//! thin adapters around `scan_code_regions`.

use std::ops::ControlFlow;

/// Counts the length of a consecutive run of the given byte starting at `i`.
#[inline]
pub(crate) fn fence_run_length(bytes: &[u8], i: usize, ch: u8) -> usize {
    let mut len = 0;
    while i + len < bytes.len() && bytes[i + len] == ch {
        len += 1;
    }
    len
}

/// Result of a successful fence parse: the fence character, run length, and
/// byte offsets of the fence run (just past any leading spaces).
pub(crate) struct FenceHit {
    pub(crate) ch: u8,
    pub(crate) len: usize,
    /// Offset of the first fence char (backtick or tilde), past ≤3 leading spaces.
    pub(crate) run_start: usize,
    /// Offset just past the fence run.
    pub(crate) run_end: usize,
}

/// Attempts to parse a CommonMark §4.5 code fence at the current logical line
/// beginning at `line_start`. Skips up to 3 leading spaces (tab = 4, so any
/// leading tab disqualifies); then requires 3+ consecutive backticks or tildes.
///
/// Returns `None` if the line does not open (or close) a fence.
#[inline]
pub(crate) fn parse_fence_at_line_start(bytes: &[u8], line_start: usize) -> Option<FenceHit> {
    let mut i = line_start;
    let mut leading = 0usize;
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        leading += if bytes[i] == b'\t' { 4 } else { 1 };
        if leading > 3 {
            return None;
        }
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }
    let ch = bytes[i];
    if ch != b'`' && ch != b'~' {
        return None;
    }
    let run = fence_run_length(bytes, i, ch);
    if run < 3 {
        return None;
    }
    Some(FenceHit {
        ch,
        len: run,
        run_start: i,
        run_end: i + run,
    })
}

/// Tracks whether a byte-wise scan is currently inside a fenced code block
/// (`` ``` `` or `~~~` runs of 3+, per CommonMark §4.5). A single scanner
/// instance is threaded through a loop and consulted at each line start.
#[derive(Default)]
pub(crate) struct FenceScanner {
    in_code_block: bool,
    opening_fence_char: u8,
    opening_fence_len: usize,
}

impl FenceScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the most recent `consume_fence_at_line_start` call
    /// left the scanner inside a fenced code block.
    #[inline]
    pub fn in_code_block(&self) -> bool {
        self.in_code_block
    }

    /// If `line_start` begins a CommonMark fence line (≤3 leading spaces then
    /// 3+ backticks or tildes), update the fence state and return
    /// `Some(run_end)` — the position just past the fence run. Returns `None`
    /// when no fence starts at this line.
    ///
    /// A closing fence must use the same character and be at least as long as
    /// the opening fence: `` ```` `` is not closed by ``` ``` ``, and a
    /// backtick fence is not closed by a tilde run.
    #[inline]
    pub fn consume_fence_at_line_start(
        &mut self,
        bytes: &[u8],
        line_start: usize,
    ) -> Option<usize> {
        let hit = parse_fence_at_line_start(bytes, line_start)?;
        if !self.in_code_block {
            self.in_code_block = true;
            self.opening_fence_char = hit.ch;
            self.opening_fence_len = hit.len;
        } else if hit.ch == self.opening_fence_char && hit.len >= self.opening_fence_len {
            self.in_code_block = false;
            self.opening_fence_char = 0;
            self.opening_fence_len = 0;
        }
        Some(hit.run_end)
    }
}

/// Iterates every byte position in `bytes` that is outside a fenced code block,
/// calling `visitor(byte, i, at_line_start)` for each. The callback receives
/// the byte value, its index, and whether it sits at a line start (just after
/// a `\n` or at offset 0). `\n` bytes are themselves visited (with
/// `at_line_start = false` for the newline itself — the following byte, if any,
/// gets `at_line_start = true`).
pub(crate) fn for_each_byte_outside_fence(bytes: &[u8], mut visitor: impl FnMut(u8, usize, bool)) {
    let len = bytes.len();
    let mut scanner = FenceScanner::new();
    let mut i = 0;
    let mut line_start = 0usize;

    while i < len {
        if i == line_start
            && let Some(next) = scanner.consume_fence_at_line_start(bytes, line_start)
        {
            i = next;
            continue;
        }
        if scanner.in_code_block() {
            if bytes[i] == b'\n' {
                line_start = i + 1;
            }
            i += 1;
            continue;
        }
        let at_line_start = i == line_start;
        visitor(bytes[i], i, at_line_start);
        if bytes[i] == b'\n' {
            line_start = i + 1;
        }
        i += 1;
    }
}

/// A code region discovered by `scan_code_regions`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum CodeRegion {
    Fence(FenceRegion),
    Inline(InlineRegion),
}

/// A fenced code block (§4.5).
#[derive(Debug, Clone, Copy)]
pub(crate) struct FenceRegion {
    /// Position of the first byte of the opening fence run.
    pub open_run_start: usize,
    /// Position of the first byte of the closing fence run, or `text.len()` if
    /// the fence never closed.
    pub close_run_start: usize,
    /// `true` iff a matching closing fence was found.
    pub closed: bool,
}

/// An inline code span.
#[derive(Debug, Clone, Copy)]
pub(crate) struct InlineRegion {
    /// Position of the opening backtick.
    pub open_pos: usize,
    /// How the span ended.
    pub terminator: InlineTerminator,
}

/// Terminal state of an inline code span.
#[derive(Debug, Clone, Copy)]
pub(crate) enum InlineTerminator {
    /// Closed properly by a backtick at the given position.
    Closed(usize),
    /// Terminated by a `\n` at the given position (CommonMark: inline spans
    /// don't cross hard line breaks in this streaming approximation).
    Newline(usize),
    /// Unterminated at EOF — callers substitute `text.len()` for boundary math.
    Eof,
}

/// Scans `text` for fenced code blocks (§4.5) and inline code spans, invoking
/// `visitor(region)` for each completed region in order of opening position.
/// The visitor may return `ControlFlow::Break(())` to stop scanning early.
///
/// Rules, derived from the original four per-caller state machines:
/// - Opening and closing fences must start at a line with ≤3 leading spaces;
///   a closing fence must match the opening character and be at least as long.
/// - Mid-line 3+ backtick/tilde runs are inert literals (issue #50).
/// - Escaped backticks (`` \` ``) are literal.
/// - Inline backticks inside a fenced block do not open an inline span.
/// - A newline inside an unclosed inline span terminates it (emitted as
///   `InlineTerminator::Newline`).
/// - An unterminated fence or inline span at EOF is emitted as
///   `FenceRegion { closed: false, ... }` or `InlineTerminator::Eof(len)`.
pub(crate) fn scan_code_regions<F>(text: &str, mut visitor: F)
where
    F: FnMut(CodeRegion) -> ControlFlow<()>,
{
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut in_fence = false;
    let mut fence_open_run_start: usize = 0;
    let mut opening_fence_char: u8 = 0;
    let mut opening_fence_len: usize = 0;
    let mut in_inline = false;
    let mut inline_open_pos: usize = 0;
    let mut fence_on_line: Option<FenceHit> = parse_fence_at_line_start(bytes, 0);
    let mut i = 0;

    while i < len {
        // Fence transitions fire only at a fence line's `run_start` (past ≤3
        // leading spaces) and only while outside an inline span.
        if !in_inline
            && let Some(hit) = fence_on_line.as_ref()
            && i == hit.run_start
        {
            if !in_fence {
                in_fence = true;
                fence_open_run_start = hit.run_start;
                opening_fence_char = hit.ch;
                opening_fence_len = hit.len;
            } else if hit.ch == opening_fence_char && hit.len >= opening_fence_len {
                let region = CodeRegion::Fence(FenceRegion {
                    open_run_start: fence_open_run_start,
                    close_run_start: hit.run_start,
                    closed: true,
                });
                if visitor(region).is_break() {
                    return;
                }
                in_fence = false;
                opening_fence_char = 0;
                opening_fence_len = 0;
            }
            // Mismatched char or too-short closer: fall through and skip the run.
            i = hit.run_end;
            continue;
        }

        // Escaped backticks are literal.
        if bytes[i] == b'\\' && i + 1 < len && bytes[i + 1] == b'`' {
            i += 2;
            continue;
        }

        // Mid-line 3+ backtick/tilde runs are inert literals (issue #50
        // follow-up): not a fence (handled at line-start above), and not a
        // single-backtick closer for an open inline span.
        if bytes[i] == b'`' || bytes[i] == b'~' {
            let run = fence_run_length(bytes, i, bytes[i]);
            if run >= 3 {
                i += run;
                continue;
            }
        }

        // Inline code toggle — disabled inside a fenced block.
        if !in_fence && bytes[i] == b'`' {
            if in_inline {
                let region = CodeRegion::Inline(InlineRegion {
                    open_pos: inline_open_pos,
                    terminator: InlineTerminator::Closed(i),
                });
                if visitor(region).is_break() {
                    return;
                }
                in_inline = false;
            } else {
                in_inline = true;
                inline_open_pos = i;
            }
        }

        if bytes[i] == b'\n' {
            fence_on_line = parse_fence_at_line_start(bytes, i + 1);
            if in_inline {
                let region = CodeRegion::Inline(InlineRegion {
                    open_pos: inline_open_pos,
                    terminator: InlineTerminator::Newline(i),
                });
                if visitor(region).is_break() {
                    return;
                }
                in_inline = false;
            }
        }

        i += 1;
    }

    if in_fence {
        let _ = visitor(CodeRegion::Fence(FenceRegion {
            open_run_start: fence_open_run_start,
            close_run_start: len,
            closed: false,
        }));
    } else if in_inline {
        let _ = visitor(CodeRegion::Inline(InlineRegion {
            open_pos: inline_open_pos,
            terminator: InlineTerminator::Eof,
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CodeRegion, FenceScanner, InlineTerminator, for_each_byte_outside_fence, scan_code_regions,
    };
    use std::ops::ControlFlow;

    fn collect(text: &str) -> Vec<CodeRegion> {
        let mut out = Vec::new();
        scan_code_regions(text, |r| {
            out.push(r);
            ControlFlow::Continue(())
        });
        out
    }

    #[test]
    fn fence_scanner_consumes_backtick_fence() {
        let mut scanner = FenceScanner::new();
        let bytes = b"```rust\ncode\n```";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(3));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 13), Some(16));
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_consumes_tilde_fence() {
        let mut scanner = FenceScanner::new();
        let bytes = b"~~~\nx\n~~~";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(3));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 6), Some(9));
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_ignores_short_runs() {
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"`x`", 0), None);
        assert_eq!(scanner.consume_fence_at_line_start(b"``x``", 0), None);
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_ignores_non_fence_bytes() {
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"abc", 0), None);
        assert_eq!(scanner.consume_fence_at_line_start(b"", 0), None);
    }

    #[test]
    fn fence_scanner_rejects_prose_line_with_mid_line_run() {
        let mut scanner = FenceScanner::new();
        assert_eq!(
            scanner.consume_fence_at_line_start(b"hello ```\ncode", 0),
            None
        );
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_accepts_indented_fence() {
        let mut scanner = FenceScanner::new();
        assert_eq!(
            scanner.consume_fence_at_line_start(b"   ```\nx", 0),
            Some(6)
        );
        assert!(scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_rejects_four_space_indent() {
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"    ```\nx", 0), None);
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_rejects_leading_tab() {
        let mut scanner = FenceScanner::new();
        assert_eq!(scanner.consume_fence_at_line_start(b"\t```\nx", 0), None);
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_backtick_not_closed_by_tilde() {
        let mut scanner = FenceScanner::new();
        let bytes = b"```\ncode\n~~~\nmore";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(3));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 9), Some(12));
        assert!(scanner.in_code_block());
    }

    #[test]
    fn fence_scanner_short_closer_does_not_close_long_opener() {
        let mut scanner = FenceScanner::new();
        let bytes = b"````\nx\n```\ny\n````";
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 0), Some(4));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 7), Some(10));
        assert!(scanner.in_code_block());
        assert_eq!(scanner.consume_fence_at_line_start(bytes, 13), Some(17));
        assert!(!scanner.in_code_block());
    }

    #[test]
    fn for_each_byte_outside_fence_skips_fenced_content() {
        let bytes = b"abc\n```\nskip\n```\nxyz";
        let mut seen = Vec::new();
        for_each_byte_outside_fence(bytes, |b, i, _| seen.push((b, i)));
        let chars: String = seen.iter().map(|(b, _)| *b as char).collect();
        assert_eq!(chars, "abc\n\nxyz");
    }

    #[test]
    fn scan_emits_closed_backtick_fence() {
        let regions = collect("```\ncode\n```");
        assert_eq!(regions.len(), 1);
        match regions[0] {
            CodeRegion::Fence(f) => {
                assert_eq!(f.open_run_start, 0);
                assert_eq!(f.close_run_start, 9);
                assert!(f.closed);
            }
            _ => panic!("expected fence"),
        }
    }

    #[test]
    fn scan_emits_unterminated_fence_at_eof() {
        let text = "```\ncode";
        let regions = collect(text);
        assert_eq!(regions.len(), 1);
        match regions[0] {
            CodeRegion::Fence(f) => {
                assert_eq!(f.open_run_start, 0);
                assert_eq!(f.close_run_start, text.len());
                assert!(!f.closed);
            }
            _ => panic!("expected fence"),
        }
    }

    #[test]
    fn scan_emits_closed_inline_spans() {
        let regions = collect("a`b`c`d`e");
        assert_eq!(regions.len(), 2);
        let mut opens = Vec::new();
        let mut closes = Vec::new();
        for r in regions {
            match r {
                CodeRegion::Inline(s) => {
                    opens.push(s.open_pos);
                    if let InlineTerminator::Closed(pos) = s.terminator {
                        closes.push(pos);
                    }
                }
                _ => panic!("expected inline"),
            }
        }
        assert_eq!(opens, vec![1, 5]);
        assert_eq!(closes, vec![3, 7]);
    }

    #[test]
    fn scan_terminates_unclosed_inline_at_newline() {
        let regions = collect("`unclosed\nnext");
        assert_eq!(regions.len(), 1);
        match regions[0] {
            CodeRegion::Inline(s) => {
                assert_eq!(s.open_pos, 0);
                assert!(matches!(s.terminator, InlineTerminator::Newline(9)));
            }
            _ => panic!("expected inline"),
        }
    }

    #[test]
    fn scan_terminates_unclosed_inline_at_eof() {
        let regions = collect("`still open");
        assert_eq!(regions.len(), 1);
        match regions[0] {
            CodeRegion::Inline(s) => {
                assert_eq!(s.open_pos, 0);
                assert!(matches!(s.terminator, InlineTerminator::Eof));
            }
            _ => panic!("expected inline"),
        }
    }

    #[test]
    fn scan_skips_inline_inside_fence() {
        // Backticks inside a fenced block do not open inline spans.
        let regions = collect("```\na`b`c\n```");
        assert_eq!(regions.len(), 1);
        assert!(matches!(regions[0], CodeRegion::Fence(_)));
    }

    #[test]
    fn scan_treats_mid_line_run_as_inert() {
        // Issue #50: `hello ```\ncode` — the mid-line ``` does not open a fence.
        let regions = collect("hello ```\ncode");
        assert!(regions.is_empty());
    }

    #[test]
    fn scan_ignores_escaped_backtick() {
        let regions = collect("\\`not code`");
        // `\\\`` is escaped, leaving `not code\`` with an unterminated backtick
        // at the end — emitted as an Eof-terminated inline span.
        assert_eq!(regions.len(), 1);
        assert!(matches!(
            regions[0],
            CodeRegion::Inline(super::InlineRegion {
                terminator: InlineTerminator::Eof,
                ..
            })
        ));
    }

    #[test]
    fn scan_visitor_can_break_early() {
        let mut count = 0;
        scan_code_regions("`a` `b` `c`", |_| {
            count += 1;
            ControlFlow::Break(())
        });
        assert_eq!(count, 1);
    }
}
