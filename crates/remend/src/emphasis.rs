use std::borrow::Cow;

use super::ranges::CodeBlockRanges;
use super::utils::{
    FenceScanner, cow_append, fence_run_length, find_trailing_delimiter, is_empty_or_markers,
    is_escaped, is_horizontal_rule, is_list_marker_line, is_within_html_tag,
    is_within_link_or_image_url, is_within_math_block, is_word_char,
};

// ---------------------------------------------------------------------------
// Asterisk skip logic
// ---------------------------------------------------------------------------

fn should_skip_asterisk(text: &str, index: usize, prev: u8, next: u8, has_dollar: bool) -> bool {
    // Skip if escaped.
    if is_escaped(text.as_bytes(), index) {
        return true;
    }

    // Skip if within math block.
    if has_dollar && is_within_math_block(text, index) {
        return true;
    }

    let bytes = text.as_bytes();
    let next_next = if index + 2 < bytes.len() {
        bytes[index + 2]
    } else {
        0
    };

    // Special handling for *** sequences.
    if prev != b'*' && next == b'*' {
        if next_next == b'*' {
            // First * in a *** sequence — count it.
            return false;
        }
        // First * in ** — skip.
        return true;
    }

    // Skip if second or third * in a sequence.
    if prev == b'*' {
        return true;
    }

    // Skip if word-internal (use proper Unicode char lookup).
    if index > 0 && index + 1 < text.len() {
        let prev_char = text[..index].chars().next_back();
        let next_char = text[index + 1..].chars().next();
        if let (Some(pc), Some(nc)) = (prev_char, next_char)
            && is_word_char(pc)
            && is_word_char(nc)
        {
            return true;
        }
    }

    // Skip if flanked by whitespace on both sides.
    let prev_ws = prev == 0 || matches!(prev, b' ' | b'\t' | b'\n');
    let next_ws = next == 0 || matches!(next, b' ' | b'\t' | b'\n');
    if prev_ws && next_ws {
        return true;
    }

    false
}

/// Counts single asterisks that are not part of `**`/`***`, not escaped,
/// not list markers, not word-internal, and not inside fenced code blocks.
pub fn count_single_asterisks(text: &str) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let has_dollar = text.contains('$');
    let mut count = 0;
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
        if bytes[i] == b'*' {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = if i + 1 < len { bytes[i + 1] } else { 0 };
            if !should_skip_asterisk(text, i, prev, next, has_dollar) {
                count += 1;
            }
        }
        if bytes[i] == b'\n' {
            line_start = i + 1;
        }
        i += 1;
    }
    count
}

// ---------------------------------------------------------------------------
// Underscore skip logic
// ---------------------------------------------------------------------------

fn should_skip_underscore(text: &str, index: usize, prev: u8, next: u8, has_dollar: bool) -> bool {
    if is_escaped(text.as_bytes(), index) {
        return true;
    }
    if has_dollar && is_within_math_block(text, index) {
        return true;
    }
    if is_within_link_or_image_url(text, index) {
        return true;
    }
    if is_within_html_tag(text, index) {
        return true;
    }
    // Skip if part of __.
    if prev == b'_' || next == b'_' {
        return true;
    }
    // Skip if word-internal (use proper Unicode char lookup for multi-byte chars).
    if index > 0 && index + 1 < text.len() {
        let prev_char = text[..index].chars().next_back();
        let next_char = text[index + 1..].chars().next();
        if let (Some(pc), Some(nc)) = (prev_char, next_char)
            && is_word_char(pc)
            && is_word_char(nc)
        {
            return true;
        }
    }
    false
}

pub fn count_single_underscores(text: &str) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let has_dollar = text.contains('$');
    let mut count = 0;
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
        if bytes[i] == b'_' {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = if i + 1 < len { bytes[i + 1] } else { 0 };
            if !should_skip_underscore(text, i, prev, next, has_dollar) {
                count += 1;
            }
        }
        if bytes[i] == b'\n' {
            line_start = i + 1;
        }
        i += 1;
    }
    count
}

// ---------------------------------------------------------------------------
// Triple asterisks counting
// ---------------------------------------------------------------------------

pub fn count_triple_asterisks(text: &str) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut count = 0;
    let mut consecutive = 0usize;
    let mut scanner = FenceScanner::new();
    let mut i = 0;
    let mut line_start = 0usize;

    while i < len {
        if i == line_start
            && let Some(next) = scanner.consume_fence_at_line_start(bytes, line_start)
        {
            if consecutive >= 3 {
                count += consecutive / 3;
            }
            consecutive = 0;
            i = next;
            continue;
        }
        // Backtick/tilde runs that aren't line-start fences: skip past without
        // resetting `consecutive` so a streak of `***` split by a stray `` ` ``
        // (or a mid-line ```) still counts.
        if bytes[i] == b'`' || bytes[i] == b'~' {
            i += fence_run_length(bytes, i, bytes[i]);
            continue;
        }
        if scanner.in_code_block() {
            if bytes[i] == b'\n' {
                line_start = i + 1;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'*' {
            // Skip escaped asterisks.
            if is_escaped(bytes, i) {
                if consecutive >= 3 {
                    count += consecutive / 3;
                }
                consecutive = 0;
            } else {
                consecutive += 1;
            }
        } else {
            if consecutive >= 3 {
                count += consecutive / 3;
            }
            consecutive = 0;
        }
        if bytes[i] == b'\n' {
            line_start = i + 1;
        }
        i += 1;
    }
    if consecutive >= 3 {
        count += consecutive / 3;
    }
    count
}

// ---------------------------------------------------------------------------
// Double marker counting (outside code blocks)
// ---------------------------------------------------------------------------

fn count_double_markers_outside_code_blocks(text: &str, marker: u8) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut count = 0;
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
        if bytes[i] == marker && i + 1 < len && bytes[i + 1] == marker {
            count += 1;
            i += 2;
            continue;
        }
        if bytes[i] == b'\n' {
            line_start = i + 1;
        }
        i += 1;
    }
    count
}

fn count_double_asterisks(text: &str) -> usize {
    count_double_markers_outside_code_blocks(text, b'*')
}

fn count_double_underscores(text: &str) -> usize {
    count_double_markers_outside_code_blocks(text, b'_')
}

// ---------------------------------------------------------------------------
// Pattern matching helpers (replaces regex patterns)
// ---------------------------------------------------------------------------

/// Finds the last `**` matching the TS pattern `/(\*\*)([^*]*\*?)$/`.
/// Content after `**` must contain no `*` except optionally one at the very end.
fn find_trailing_bold(text: &str) -> Option<(usize, &str)> {
    let bytes = text.as_bytes();
    let mut i = bytes.len();
    while i >= 2 {
        i -= 1;
        if bytes[i] == b'*' && bytes[i - 1] == b'*' {
            let marker_start = i - 1;
            // Skip if this is actually `***`.
            if marker_start > 0 && bytes[marker_start - 1] == b'*' {
                continue;
            }
            let content = &text[i + 1..];
            // Match TS pattern [^*]*\*?$ — content must have no `*` except
            // optionally one at the very end.
            if content.is_empty() {
                return Some((marker_start, content));
            }
            let has_inner_star = if let Some(stripped) = content.strip_suffix('*') {
                // Allow trailing `*`, but check rest has no `*`.
                stripped.contains('*')
            } else {
                content.contains('*')
            };
            if has_inner_star {
                continue;
            }
            return Some((marker_start, content));
        }
    }
    None
}

/// Finds the last `__` followed by non-`_` content at the end of text.
/// Matches the TS pattern `/(__)([^_]*?)$/` — content must not contain `_`.
fn find_trailing_double_underscore(text: &str) -> Option<(usize, &str)> {
    find_trailing_delimiter(text, b"__")
}

/// Finds the last `***` followed by non-`*` content at the end of text.
fn find_trailing_bold_italic(text: &str) -> Option<(usize, &str)> {
    let bytes = text.as_bytes();
    let mut i = bytes.len();
    while i >= 3 {
        i -= 1;
        if bytes[i] == b'*' && i >= 2 && bytes[i - 1] == b'*' && bytes[i - 2] == b'*' {
            let marker_start = i - 2;
            // Skip if 4+ asterisks.
            if marker_start > 0 && bytes[marker_start - 1] == b'*' {
                continue;
            }
            let content = &text[i + 1..];
            if content.starts_with('*') {
                continue;
            }
            return Some((marker_start, content));
        }
    }
    None
}

/// Finds the last `~~` followed by non-`~` content at the end of text.
/// Matches the TS pattern `/(~~)([^~]*?)$/` — content must not contain `~`.
pub fn find_trailing_strikethrough(text: &str) -> Option<(usize, &str)> {
    find_trailing_delimiter(text, b"~~")
}

// ---------------------------------------------------------------------------
// Skip completion checks
// ---------------------------------------------------------------------------

fn should_skip_bold_completion(text: &str, content: &str, marker_index: usize) -> bool {
    if content.is_empty() || is_empty_or_markers(content) {
        return true;
    }

    // Check if in a list item with multiline content.
    let before = &text[..marker_index];
    let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_before = &text[line_start..marker_index];
    if is_list_marker_line(line_before) && content.contains('\n') {
        return true;
    }

    is_horizontal_rule(text, marker_index, b'*')
}

fn should_skip_italic_completion(text: &str, content: &str, marker_index: usize) -> bool {
    if content.is_empty() || is_empty_or_markers(content) {
        return true;
    }

    let before = &text[..marker_index];
    let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
    let line_before = &text[line_start..marker_index];
    if is_list_marker_line(line_before) && content.contains('\n') {
        return true;
    }

    is_horizontal_rule(text, marker_index, b'_')
}

// ---------------------------------------------------------------------------
// Find first single marker index
// ---------------------------------------------------------------------------

fn find_first_single_asterisk_index(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let has_dollar = text.contains('$');
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

        if bytes[i] == b'*' {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = if i + 1 < len { bytes[i + 1] } else { 0 };

            // Must be a single * (not part of ** or ***).
            if prev == b'*' || next == b'*' {
                i += 1;
                continue;
            }
            if prev == b'\\' {
                i += 1;
                continue;
            }
            if has_dollar && is_within_math_block(text, i) {
                i += 1;
                continue;
            }

            // Skip if flanked by whitespace on both sides.
            let prev_ws = prev == 0 || matches!(prev, b' ' | b'\t' | b'\n');
            let next_ws = next == 0 || matches!(next, b' ' | b'\t' | b'\n');
            if prev_ws && next_ws {
                i += 1;
                continue;
            }

            // Skip if word-internal (Unicode-aware).
            if i > 0 && i + 1 < text.len() {
                let pc = text[..i].chars().next_back();
                let nc = text[i + 1..].chars().next();
                if let (Some(pc), Some(nc)) = (pc, nc)
                    && is_word_char(pc)
                    && is_word_char(nc)
                {
                    i += 1;
                    continue;
                }
            }

            return Some(i);
        }
        if bytes[i] == b'\n' {
            line_start = i + 1;
        }
        i += 1;
    }
    None
}

fn find_first_single_underscore_index(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let has_dollar = text.contains('$');
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

        if bytes[i] == b'_' {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = if i + 1 < len { bytes[i + 1] } else { 0 };

            if prev == b'_' || next == b'_' {
                i += 1;
                continue;
            }
            if prev == b'\\' {
                i += 1;
                continue;
            }
            if has_dollar && is_within_math_block(text, i) {
                i += 1;
                continue;
            }
            if is_within_link_or_image_url(text, i) {
                i += 1;
                continue;
            }
            // Skip if word-internal (Unicode-aware).
            if i > 0 && i + 1 < text.len() {
                let pc = text[..i].chars().next_back();
                let nc = text[i + 1..].chars().next();
                if let (Some(pc), Some(nc)) = (pc, nc)
                    && is_word_char(pc)
                    && is_word_char(nc)
                {
                    i += 1;
                    continue;
                }
            }

            return Some(i);
        }
        if bytes[i] == b'\n' {
            line_start = i + 1;
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------
// Public handler functions
// ---------------------------------------------------------------------------

/// Completes incomplete bold formatting (`**`).
/// Test-only convenience wrapper that builds `CodeBlockRanges` on the fly.
#[cfg(test)]
fn handle_bold(text: &str) -> Cow<'_, str> {
    handle_bold_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete bold formatting, using pre-computed code block ranges.
pub(crate) fn handle_bold_with_ranges<'a>(text: &'a str, ranges: &CodeBlockRanges) -> Cow<'a, str> {
    let Some((marker_index, content)) = find_trailing_bold(text) else {
        return Cow::Borrowed(text);
    };

    if ranges.is_inside_code(marker_index) || ranges.is_within_complete_inline_code(marker_index) {
        return Cow::Borrowed(text);
    }

    if should_skip_bold_completion(text, content, marker_index) {
        return Cow::Borrowed(text);
    }

    let pairs = count_double_asterisks(text);
    if pairs % 2 == 1 {
        // Half-complete: **content* → **content**
        if content.ends_with('*') {
            return cow_append(text, "*");
        }
        return cow_append(text, "**");
    }

    Cow::Borrowed(text)
}

/// Completes incomplete double-underscore italic (`__`).
/// Test-only convenience wrapper that builds `CodeBlockRanges` on the fly.
#[cfg(test)]
fn handle_double_underscore(text: &str) -> Cow<'_, str> {
    handle_double_underscore_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete double-underscore italic, using pre-computed code block ranges.
pub(crate) fn handle_double_underscore_with_ranges<'a>(
    text: &'a str,
    ranges: &CodeBlockRanges,
) -> Cow<'a, str> {
    // First check for trailing `__content` pattern.
    if let Some((marker_index, content)) = find_trailing_double_underscore(text)
        && !ranges.is_inside_code(marker_index)
        && !ranges.is_within_complete_inline_code(marker_index)
        && !should_skip_italic_completion(text, content, marker_index)
    {
        let pairs = count_double_underscores(text);
        if pairs % 2 == 1 {
            return cow_append(text, "__");
        }
    }

    // Check for half-complete: __content_ → __content__
    if let Some(pos) = find_half_complete_underscore(text)
        && !ranges.is_inside_code(pos)
        && !ranges.is_within_complete_inline_code(pos)
    {
        let pairs = count_double_underscores(text);
        if pairs % 2 == 1 {
            return cow_append(text, "_");
        }
    }

    Cow::Borrowed(text)
}

/// Finds `__content_` pattern (half-complete closing).
/// Returns the marker index of the opening `__`.
fn find_half_complete_underscore(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    // Must end with single `_` but not `__`.
    if !text.ends_with('_') || text.ends_with("__") {
        return None;
    }
    let content_end = bytes.len() - 1; // index of trailing `_`
    // Look backward for opening `__`.
    if content_end < 3 {
        return None;
    }
    let mut i = content_end - 1; // start before the trailing `_`
    while i >= 1 {
        if bytes[i] == b'_' && bytes[i - 1] == b'_' {
            // Check that content between `__` and the trailing `_` doesn't have `_`.
            let between = &text[i + 1..content_end];
            if !between.contains('_') {
                return Some(i - 1);
            }
        }
        i -= 1;
    }
    None
}

/// Completes incomplete italic with single asterisk (`*`).
/// Test-only convenience wrapper that builds `CodeBlockRanges` on the fly.
#[cfg(test)]
fn handle_italic_asterisk(text: &str) -> Cow<'_, str> {
    handle_italic_asterisk_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete italic with single asterisk, using pre-computed code block ranges.
pub(crate) fn handle_italic_asterisk_with_ranges<'a>(
    text: &'a str,
    ranges: &CodeBlockRanges,
) -> Cow<'a, str> {
    // Check for trailing single asterisk pattern.
    let bytes = text.as_bytes();
    // Quick check: does text end with a non-`*` followed sometime by a `*` earlier?
    if !bytes.contains(&b'*') {
        return Cow::Borrowed(text);
    }

    let Some(first_idx) = find_first_single_asterisk_index(text) else {
        return Cow::Borrowed(text);
    };

    if ranges.is_inside_code(first_idx) || ranges.is_within_complete_inline_code(first_idx) {
        return Cow::Borrowed(text);
    }

    let content_after = &text[first_idx + 1..];
    if content_after.is_empty() || is_empty_or_markers(content_after) {
        return Cow::Borrowed(text);
    }

    let count = count_single_asterisks(text);
    if count % 2 == 1 {
        return cow_append(text, "*");
    }

    Cow::Borrowed(text)
}

/// Completes incomplete italic with single underscore (`_`).
/// Test-only convenience wrapper that builds `CodeBlockRanges` on the fly.
#[cfg(test)]
fn handle_italic_underscore(text: &str) -> Cow<'_, str> {
    handle_italic_underscore_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete italic with single underscore, using pre-computed code block ranges.
pub(crate) fn handle_italic_underscore_with_ranges<'a>(
    text: &'a str,
    ranges: &CodeBlockRanges,
) -> Cow<'a, str> {
    if !text.as_bytes().contains(&b'_') {
        return Cow::Borrowed(text);
    }

    let Some(first_idx) = find_first_single_underscore_index(text) else {
        return Cow::Borrowed(text);
    };

    let content_after = &text[first_idx + 1..];
    if content_after.is_empty() || is_empty_or_markers(content_after) {
        return Cow::Borrowed(text);
    }

    if ranges.is_inside_code(first_idx) || ranges.is_within_complete_inline_code(first_idx) {
        return Cow::Borrowed(text);
    }

    let count = count_single_underscores(text);
    if count % 2 == 1 {
        // Check if we need to insert `_` before trailing `**` for proper nesting.
        if let Some(result) = handle_trailing_asterisks_for_underscore(text) {
            return Cow::Owned(result);
        }
        return insert_closing_underscore(text);
    }

    Cow::Borrowed(text)
}

/// If text ends with `**` that was added to close an unclosed bold,
/// and there's an unclosed `_` before it, insert `_` before the `**`.
fn handle_trailing_asterisks_for_underscore(text: &str) -> Option<String> {
    if !text.ends_with("**") {
        return None;
    }

    let without = &text[..text.len() - 2];
    let pairs = count_double_markers_outside_code_blocks(without, b'*');
    if pairs % 2 != 1 {
        return None;
    }

    let first_double = without.find("**")?;
    let underscore_idx = find_first_single_underscore_index(without)?;

    if first_double < underscore_idx {
        let mut result = String::with_capacity(text.len() + 1);
        result.push_str(without);
        result.push('_');
        result.push_str("**");
        return Some(result);
    }

    None
}

/// Inserts closing `_`, placing it before any trailing newlines.
fn insert_closing_underscore(text: &str) -> Cow<'_, str> {
    let bytes = text.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b'\n' {
        end -= 1;
    }
    if end < bytes.len() {
        let mut result = String::with_capacity(text.len() + 1);
        result.push_str(&text[..end]);
        result.push('_');
        result.push_str(&text[end..]);
        Cow::Owned(result)
    } else {
        cow_append(text, "_")
    }
}

/// Completes incomplete bold-italic formatting (`***`).
/// Test-only convenience wrapper that builds `CodeBlockRanges` on the fly.
#[cfg(test)]
fn handle_bold_italic(text: &str) -> Cow<'_, str> {
    handle_bold_italic_with_ranges(text, &CodeBlockRanges::new(text))
}

/// Completes incomplete bold-italic formatting, using pre-computed code block ranges.
pub(crate) fn handle_bold_italic_with_ranges<'a>(
    text: &'a str,
    ranges: &CodeBlockRanges,
) -> Cow<'a, str> {
    // Don't process if text is only 4+ asterisks.
    if text.len() >= 4 && text.bytes().all(|b| b == b'*') {
        return Cow::Borrowed(text);
    }

    let Some((marker_index, content)) = find_trailing_bold_italic(text) else {
        return Cow::Borrowed(text);
    };

    if content.is_empty() || is_empty_or_markers(content) {
        return Cow::Borrowed(text);
    }

    if ranges.is_inside_code(marker_index) || ranges.is_within_complete_inline_code(marker_index) {
        return Cow::Borrowed(text);
    }

    if is_horizontal_rule(text, marker_index, b'*') {
        return Cow::Borrowed(text);
    }

    let triples = count_triple_asterisks(text);
    if triples % 2 == 1 {
        // If both ** and * are already balanced, don't add ***.
        let double_pairs = count_double_asterisks(text);
        let single_count = count_single_asterisks(text);
        if double_pairs.is_multiple_of(2) && single_count.is_multiple_of(2) {
            return Cow::Borrowed(text);
        }
        return cow_append(text, "***");
    }

    Cow::Borrowed(text)
}

#[cfg(test)]
mod tests {
    use super::{
        count_double_asterisks, count_double_underscores, count_single_asterisks,
        count_single_underscores, count_triple_asterisks, find_first_single_asterisk_index,
        find_first_single_underscore_index, handle_bold, handle_bold_italic,
        handle_double_underscore, handle_italic_asterisk, handle_italic_underscore,
    };
    use std::borrow::Cow;

    // Direct counter coverage (issue #50 follow-up): verify the six fence-aware
    // emphasis helpers all treat mid-line 3+ runs as prose and honor line-start
    // fences identically.

    #[test]
    fn single_asterisks_counted_outside_mid_line_run() {
        // `*italic` sits after a mid-line ``` which must NOT open a fence.
        assert_eq!(count_single_asterisks("hello ```\n*italic"), 1);
    }

    #[test]
    fn single_asterisks_ignored_inside_fenced_block() {
        assert_eq!(count_single_asterisks("```\n*italic\n```"), 0);
    }

    #[test]
    fn single_underscores_counted_outside_mid_line_run() {
        assert_eq!(count_single_underscores("hello ```\n_italic"), 1);
    }

    #[test]
    fn single_underscores_ignored_inside_fenced_block() {
        assert_eq!(count_single_underscores("```\n_italic\n```"), 0);
    }

    #[test]
    fn triple_asterisks_counted_across_mid_line_run() {
        // Mid-line ``` splits a `***` streak; run is inert, streak continues.
        assert_eq!(count_triple_asterisks("**```*"), 1);
    }

    #[test]
    fn triple_asterisks_ignored_inside_fenced_block() {
        assert_eq!(count_triple_asterisks("```\n***\n```"), 0);
    }

    #[test]
    fn double_markers_counted_outside_mid_line_run() {
        // Mid-line ``` must not open a fence, so **bold is countable.
        assert_eq!(count_double_asterisks("x ```\n**bold"), 1);
        assert_eq!(count_double_underscores("x ```\n__bold"), 1);
    }

    #[test]
    fn double_markers_ignored_inside_fenced_block() {
        assert_eq!(count_double_asterisks("```\n**bold\n```"), 0);
        assert_eq!(count_double_underscores("```\n__bold\n```"), 0);
    }

    #[test]
    fn first_single_asterisk_index_skips_fenced_block() {
        // Asterisk inside the fence is ignored; the one on the "after" line wins.
        assert_eq!(
            find_first_single_asterisk_index("```\n*inside\n```\n*after"),
            Some(16),
        );
    }

    #[test]
    fn first_single_underscore_index_skips_fenced_block() {
        assert_eq!(
            find_first_single_underscore_index("```\n_inside\n```\n_after"),
            Some(16),
        );
    }

    // Bold tests
    #[test]
    fn bold_completes_incomplete() {
        assert_eq!(handle_bold("**bold text").as_ref(), "**bold text**");
    }

    #[test]
    fn bold_leaves_complete() {
        assert!(matches!(handle_bold("**bold**"), Cow::Borrowed(_)));
    }

    #[test]
    fn bold_half_complete() {
        assert_eq!(handle_bold("**bold*").as_ref(), "**bold**");
    }

    #[test]
    fn bold_inside_code_block() {
        assert!(matches!(handle_bold("```\n**bold\n```"), Cow::Borrowed(_)));
    }

    #[test]
    fn bold_empty_content() {
        assert!(matches!(handle_bold("**"), Cow::Borrowed(_)));
    }

    // Italic asterisk tests
    #[test]
    fn italic_asterisk_completes() {
        assert_eq!(
            handle_italic_asterisk("*italic text").as_ref(),
            "*italic text*"
        );
    }

    #[test]
    fn italic_asterisk_leaves_complete() {
        assert!(matches!(
            handle_italic_asterisk("*italic*"),
            Cow::Borrowed(_)
        ));
    }

    // Italic underscore tests
    #[test]
    fn italic_underscore_completes() {
        assert_eq!(
            handle_italic_underscore("_italic text").as_ref(),
            "_italic text_"
        );
    }

    #[test]
    fn italic_underscore_word_internal() {
        // user_name should not be treated as italic.
        assert!(matches!(
            handle_italic_underscore("user_name"),
            Cow::Borrowed(_)
        ));
    }

    // Double underscore tests
    #[test]
    fn double_underscore_completes() {
        assert_eq!(
            handle_double_underscore("__italic text").as_ref(),
            "__italic text__"
        );
    }

    #[test]
    fn double_underscore_half_complete() {
        assert_eq!(handle_double_underscore("__italic_").as_ref(), "__italic__");
    }

    // Bold-italic tests
    #[test]
    fn bold_italic_completes() {
        assert_eq!(
            handle_bold_italic("***bold italic text").as_ref(),
            "***bold italic text***"
        );
    }

    #[test]
    fn bold_italic_four_asterisks() {
        assert!(matches!(handle_bold_italic("****"), Cow::Borrowed(_)));
    }

    // Tilde fence tests
    #[test]
    fn bold_inside_tilde_fence() {
        assert!(matches!(handle_bold("~~~\n**bold\n~~~"), Cow::Borrowed(_)));
    }

    #[test]
    fn italic_asterisk_inside_tilde_fence() {
        assert!(matches!(
            handle_italic_asterisk("~~~\n*italic\n~~~"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn italic_underscore_inside_tilde_fence() {
        assert!(matches!(
            handle_italic_underscore("~~~\n_italic\n~~~"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn double_underscore_inside_tilde_fence() {
        assert!(matches!(
            handle_double_underscore("~~~\n__italic\n~~~"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn bold_italic_inside_tilde_fence() {
        assert!(matches!(
            handle_bold_italic("~~~\n***bold italic\n~~~"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn four_tilde_fence_not_closed_by_three() {
        assert!(matches!(handle_bold("~~~~\n~~~\n**bold"), Cow::Borrowed(_)));
    }
}
