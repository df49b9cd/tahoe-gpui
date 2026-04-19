//! Data types for git commit display.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::foundations::theme::TahoeTheme;
use gpui::SharedString;

// =============================================================================
// Data types
// =============================================================================

/// Status of a file in a commit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl FileStatus {
    pub fn label(&self) -> &'static str {
        match self {
            FileStatus::Added => "A",
            FileStatus::Modified => "M",
            FileStatus::Deleted => "D",
            FileStatus::Renamed => "R",
        }
    }
}

/// A file changed in a commit, with optional addition/deletion counts.
#[derive(Clone)]
pub struct CommitFileData {
    pub path: SharedString,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
}

impl CommitFileData {
    pub fn new(path: impl Into<SharedString>, status: FileStatus) -> Self {
        Self {
            path: path.into(),
            status,
            additions: 0,
            deletions: 0,
        }
    }

    pub fn additions(mut self, n: usize) -> Self {
        self.additions = n;
        self
    }

    pub fn deletions(mut self, n: usize) -> Self {
        self.deletions = n;
        self
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Extract 1-2 uppercase initials from a name.
pub fn author_initials(name: &str) -> String {
    let mut initials = String::with_capacity(2);
    let mut count = 0usize;
    for word in name.split_whitespace() {
        if let Some(ch) = word.chars().next() {
            initials.extend(ch.to_uppercase());
            count += 1;
            if count >= 2 {
                break;
            }
        }
    }
    if initials.is_empty() {
        "?".to_string()
    } else {
        initials
    }
}

/// Format an epoch timestamp as a relative time string.
pub fn format_relative_time(epoch_secs: i64, now_secs: i64) -> String {
    let diff = now_secs - epoch_secs;
    if diff < 0 {
        return "in the future".to_string();
    }
    let diff = diff as u64;
    match diff {
        0..=59 => "just now".to_string(),
        60..=3599 => {
            let m = diff / 60;
            if m == 1 {
                "1 minute ago".to_string()
            } else {
                format!("{m} minutes ago")
            }
        }
        3600..=86399 => {
            let h = diff / 3600;
            if h == 1 {
                "1 hour ago".to_string()
            } else {
                format!("{h} hours ago")
            }
        }
        86400..=172799 => "yesterday".to_string(),
        _ => {
            let d = diff / 86400;
            format!("{d} days ago")
        }
    }
}

pub(crate) fn now_epoch_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub(crate) fn status_color(status: FileStatus, theme: &TahoeTheme) -> gpui::Hsla {
    match status {
        FileStatus::Added => theme.success,
        FileStatus::Modified => theme.warning,
        FileStatus::Deleted => theme.error,
        FileStatus::Renamed => theme.info,
    }
}
