//! Tests for the commit display components.

use super::{
    Commit, CommitActions, CommitAuthor, CommitAuthorAvatar, CommitContent, CommitCopyButton,
    CommitFileAdditions, CommitFileChanges, CommitFileData, CommitFileDeletions, CommitFileIcon,
    CommitFileInfo, CommitFilePath, CommitFileRow, CommitFileStatus, CommitFiles, CommitHash,
    CommitHeader, CommitInfo, CommitMessage, CommitMetadata, CommitSeparator, CommitTimestamp,
    FileStatus, author_initials, format_relative_time,
};
use crate::foundations::icons::IconName;
use core::prelude::v1::test;
use gpui::div;
use std::sync::Arc;
use std::time::Duration;

// -- FileStatus -----------------------------------------------------------

#[test]
fn file_status_labels() {
    assert_eq!(FileStatus::Added.label(), "A");
    assert_eq!(FileStatus::Modified.label(), "M");
    assert_eq!(FileStatus::Deleted.label(), "D");
    assert_eq!(FileStatus::Renamed.label(), "R");
}

#[test]
fn file_status_equality() {
    assert_eq!(FileStatus::Added, FileStatus::Added);
    assert_ne!(FileStatus::Added, FileStatus::Deleted);
}

// -- CommitFileData -------------------------------------------------------

#[test]
fn commit_file_data_builder() {
    let f = CommitFileData::new("src/main.rs", FileStatus::Modified)
        .additions(10)
        .deletions(3);
    assert_eq!(f.additions, 10);
    assert_eq!(f.deletions, 3);
    assert_eq!(f.status, FileStatus::Modified);
}

// -- author_initials ------------------------------------------------------

#[test]
fn initials_two_words() {
    assert_eq!(author_initials("John Doe"), "JD");
}

#[test]
fn initials_single_word() {
    assert_eq!(author_initials("Claude"), "C");
}

#[test]
fn initials_three_words_takes_first_two() {
    assert_eq!(author_initials("Mary Jane Watson"), "MJ");
}

#[test]
fn initials_empty_returns_fallback() {
    assert_eq!(author_initials(""), "?");
}

#[test]
fn initials_whitespace_only_returns_fallback() {
    assert_eq!(author_initials("   "), "?");
}

#[test]
fn initials_email_style() {
    assert_eq!(author_initials("dev@example.com"), "D");
}

// -- format_relative_time -------------------------------------------------

#[test]
fn relative_time_just_now() {
    assert_eq!(format_relative_time(1000, 1030), "just now");
}

#[test]
fn relative_time_one_minute() {
    assert_eq!(format_relative_time(1000, 1060), "1 minute ago");
}

#[test]
fn relative_time_minutes() {
    assert_eq!(format_relative_time(1000, 1000 + 300), "5 minutes ago");
}

#[test]
fn relative_time_one_hour() {
    assert_eq!(format_relative_time(1000, 1000 + 3600), "1 hour ago");
}

#[test]
fn relative_time_hours() {
    assert_eq!(format_relative_time(1000, 1000 + 7200), "2 hours ago");
}

#[test]
fn relative_time_yesterday() {
    assert_eq!(format_relative_time(1000, 1000 + 86400), "yesterday");
}

#[test]
fn relative_time_days() {
    assert_eq!(format_relative_time(1000, 1000 + 86400 * 5), "5 days ago");
}

#[test]
fn relative_time_future() {
    assert_eq!(format_relative_time(2000, 1000), "in the future");
}

// -- CommitSeparator ------------------------------------------------------

#[test]
fn separator_default_text() {
    let sep = CommitSeparator::default();
    assert_eq!(sep.text.as_ref(), "\u{00b7}");
}

#[test]
fn separator_custom_text() {
    let sep = CommitSeparator::new("/");
    assert_eq!(sep.text.as_ref(), "/");
}

// -- CommitHash -----------------------------------------------------------

#[test]
fn commit_hash_stores_value() {
    let h = CommitHash::new("abc1234");
    assert_eq!(h.hash.as_ref(), "abc1234");
}

// -- CommitTimestamp ------------------------------------------------------

#[test]
fn timestamp_new_stores_display() {
    let ts = CommitTimestamp::new("2026-04-04");
    assert_eq!(ts.display.as_ref(), "2026-04-04");
}

#[test]
fn timestamp_from_epoch_formats() {
    // Just verify it produces a non-empty string
    let ts = CommitTimestamp::from_epoch(0);
    assert!(!ts.display.is_empty());
}

// -- CommitAuthorAvatar ---------------------------------------------------

#[test]
fn author_avatar_derives_initials() {
    let av = CommitAuthorAvatar::new("Jane Doe");
    assert_eq!(av.initials.as_ref(), "JD");
}

#[test]
fn author_avatar_explicit_initials() {
    let av = CommitAuthorAvatar::from_initials("XY");
    assert_eq!(av.initials.as_ref(), "XY");
}

// -- CommitMessage --------------------------------------------------------

#[test]
fn commit_message_stores_text() {
    let msg = CommitMessage::new("Fix bug");
    assert_eq!(msg.text.as_ref(), "Fix bug");
}

// -- CommitFileStatus -----------------------------------------------------

#[test]
fn file_status_component_stores_status() {
    let fs = CommitFileStatus::new(FileStatus::Added);
    assert_eq!(fs.status, FileStatus::Added);
}

// -- CommitFileIcon -------------------------------------------------------

#[test]
fn file_icon_default() {
    let icon = CommitFileIcon::default();
    assert_eq!(icon.icon, IconName::File);
}

#[test]
fn file_icon_custom() {
    let icon = CommitFileIcon::new(IconName::FileCode);
    assert_eq!(icon.icon, IconName::FileCode);
}

// -- CommitFilePath -------------------------------------------------------

#[test]
fn file_path_stores_value() {
    let p = CommitFilePath::new("src/lib.rs");
    assert_eq!(p.path.as_ref(), "src/lib.rs");
}

// -- CommitFileAdditions / CommitFileDeletions ----------------------------

#[test]
fn file_additions_stores_count() {
    let a = CommitFileAdditions::new(42);
    assert_eq!(a.count, 42);
}

#[test]
fn file_deletions_stores_count() {
    let d = CommitFileDeletions::new(7);
    assert_eq!(d.count, 7);
}

// -- CommitFileChanges ----------------------------------------------------

#[test]
fn file_changes_empty() {
    let c = CommitFileChanges::new();
    assert!(c.children.is_empty());
}

#[test]
fn file_changes_child_appends() {
    let c = CommitFileChanges::new().child(div()).child(div());
    assert_eq!(c.children.len(), 2);
}

// -- CommitFileInfo -------------------------------------------------------

#[test]
fn file_info_empty() {
    let i = CommitFileInfo::new();
    assert!(i.children.is_empty());
}

#[test]
fn file_info_child_appends() {
    let i = CommitFileInfo::new().child(div());
    assert_eq!(i.children.len(), 1);
}

// -- CommitFileRow --------------------------------------------------------

#[test]
fn file_row_convenience_defaults() {
    let row = CommitFileRow::new("src/lib.rs", FileStatus::Modified);
    assert_eq!(row.path.as_ref().map(|s| s.as_ref()), Some("src/lib.rs"));
    assert_eq!(row.status, Some(FileStatus::Modified));
    assert_eq!(row.additions, 0);
    assert_eq!(row.deletions, 0);
    assert!(row.custom_children.is_empty());
}

#[test]
fn file_row_builder_chain() {
    let row = CommitFileRow::new("main.rs", FileStatus::Added)
        .additions(10)
        .deletions(3);
    assert_eq!(row.additions, 10);
    assert_eq!(row.deletions, 3);
}

#[test]
fn file_row_from_parts() {
    let row = CommitFileRow::from_parts().child(div());
    assert!(row.path.is_none());
    assert!(row.status.is_none());
    assert_eq!(row.custom_children.len(), 1);
}

// -- CommitFiles ----------------------------------------------------------

#[test]
fn files_empty() {
    let f = CommitFiles::new();
    assert!(f.children.is_empty());
}

#[test]
fn files_child_appends() {
    let f = CommitFiles::new().child(div()).child(div());
    assert_eq!(f.children.len(), 2);
}

#[test]
fn files_children_bulk() {
    let f = CommitFiles::new().children(vec![div(), div(), div()]);
    assert_eq!(f.children.len(), 3);
}

// -- CommitAuthor ---------------------------------------------------------

#[test]
fn author_stores_name() {
    let a = CommitAuthor::new("Jane Doe");
    assert_eq!(a.name.as_ref(), "Jane Doe");
}

// -- CommitInfo ------------------------------------------------------------

#[test]
fn info_empty() {
    let i = CommitInfo::new();
    assert!(i.children.is_empty());
}

#[test]
fn info_child_appends() {
    let i = CommitInfo::new().child(div()).child(div());
    assert_eq!(i.children.len(), 2);
}

// -- CommitMetadata -------------------------------------------------------

#[test]
fn metadata_empty() {
    let m = CommitMetadata::new();
    assert!(m.children.is_empty());
}

#[test]
fn metadata_child_appends() {
    let m = CommitMetadata::new().child(div());
    assert_eq!(m.children.len(), 1);
}

// -- CommitActions --------------------------------------------------------

#[test]
fn actions_empty() {
    let a = CommitActions::new();
    assert!(a.children.is_empty());
}

#[test]
fn actions_child_appends() {
    let a = CommitActions::new().child(div());
    assert_eq!(a.children.len(), 1);
}

// -- CommitCopyButton -----------------------------------------------------

#[test]
fn copy_button_defaults() {
    let b = CommitCopyButton::new("abc1234");
    assert_eq!(b.hash.as_ref(), "abc1234");
    assert!(b.copy_button.is_none());
    assert!(b.on_copy.is_none());
    assert!(b.timeout.is_none());
}

#[test]
fn copy_button_timeout() {
    let b = CommitCopyButton::new("abc1234").timeout(Duration::from_secs(5));
    assert_eq!(b.timeout, Some(Duration::from_secs(5)));
}

#[test]
fn copy_button_on_copy() {
    let b = CommitCopyButton::new("abc1234").on_copy(Arc::new(|| {}));
    assert!(b.on_copy.is_some());
}

// -- CommitHeader ---------------------------------------------------------

#[test]
fn header_empty() {
    let h = CommitHeader::new();
    assert!(h.children.is_empty());
}

#[test]
fn header_child_appends() {
    let h = CommitHeader::new().child(div()).child(div());
    assert_eq!(h.children.len(), 2);
}

// -- CommitContent --------------------------------------------------------

#[test]
fn content_empty() {
    let c = CommitContent::new();
    assert!(c.children.is_empty());
}

#[test]
fn content_child_appends() {
    let c = CommitContent::new().child(div()).child(div());
    assert_eq!(c.children.len(), 2);
}

#[test]
fn content_children_bulk() {
    let c = CommitContent::new().children(vec![div(), div(), div()]);
    assert_eq!(c.children.len(), 3);
}

// -- Commit (top-level) ---------------------------------------------------

#[test]
fn commit_new_defaults() {
    let c = Commit::new("abc1234", "Fix bug");
    assert_eq!(c.sha.as_ref().map(|s| s.as_ref()), Some("abc1234"));
    assert_eq!(c.message_text.as_ref().map(|s| s.as_ref()), Some("Fix bug"));
    assert!(c.author_name.is_none());
    assert!(c.date.is_none());
    assert!(c.timestamp.is_none());
    assert!(c.file_changes.is_empty());
    assert!(!c.is_open);
    assert!(c.on_toggle.is_none());
    assert!(c.header.is_none());
    assert!(c.commit_message.is_none());
    assert!(c.commit_content.is_none());
}

#[test]
fn commit_convenience_builder() {
    let c = Commit::new("abc", "msg")
        .author("Jane")
        .date("2026-04-04")
        .open(false);
    assert_eq!(c.author_name.as_ref().map(|s| s.as_ref()), Some("Jane"));
    assert_eq!(c.date.as_ref().map(|s| s.as_ref()), Some("2026-04-04"));
    assert!(!c.is_open);
}

#[test]
fn commit_timestamp_overrides_date() {
    let c = Commit::new("abc", "msg")
        .date("2026-04-04")
        .timestamp(1712188800);
    assert!(c.timestamp.is_some());
    // render_date_display prefers timestamp
    let display = c.render_date_display().unwrap();
    assert!(!display.is_empty());
    assert_ne!(display.as_ref(), "2026-04-04");
}

#[test]
fn commit_file_changes_stored() {
    let c = Commit::new("abc", "msg").file_changes(vec![
        CommitFileData::new("a.rs", FileStatus::Added),
        CommitFileData::new("b.rs", FileStatus::Modified),
    ]);
    assert_eq!(c.file_changes.len(), 2);
}

#[test]
fn commit_on_toggle_sets_handler() {
    let c = Commit::new("abc", "msg").on_toggle(|_open, _w, _cx| {});
    assert!(c.on_toggle.is_some());
}

#[test]
fn commit_from_parts_defaults() {
    let c = Commit::from_parts("c1");
    assert!(c.sha.is_none());
    assert!(c.message_text.is_none());
    assert!(!c.is_open);
    assert!(c.header.is_none());
    assert!(c.commit_message.is_none());
    assert!(c.commit_content.is_none());
}

#[test]
fn commit_from_parts_with_header() {
    let c = Commit::from_parts("c1").header(CommitHeader::new().child(div()));
    assert!(c.header.is_some());
    assert_eq!(c.header.as_ref().unwrap().children.len(), 1);
}

#[test]
fn commit_from_parts_with_message() {
    let c = Commit::from_parts("c1").message(CommitMessage::new("Fix it"));
    assert!(c.commit_message.is_some());
}

#[test]
fn commit_from_parts_with_content() {
    let c = Commit::from_parts("c1").content(CommitContent::new().child(div()).child(div()));
    assert!(c.commit_content.is_some());
    assert_eq!(c.commit_content.as_ref().unwrap().children.len(), 2);
}

#[test]
fn commit_compound_full() {
    let c = Commit::from_parts("c1")
        .header(CommitHeader::new())
        .message(CommitMessage::new("msg"))
        .content(CommitContent::new().child(div()))
        .open(false);
    assert!(c.header.is_some());
    assert!(c.commit_message.is_some());
    assert!(c.commit_content.is_some());
    assert!(!c.is_open);
}

// -- render_date_display edge cases ----------------------------------------

#[test]
fn render_date_display_none_when_no_date_or_timestamp() {
    let c = Commit::new("abc", "msg");
    assert!(c.render_date_display().is_none());
}

#[test]
fn render_date_display_returns_date_when_no_timestamp() {
    let c = Commit::new("abc", "msg").date("2026-04-04");
    assert_eq!(c.render_date_display().unwrap().as_ref(), "2026-04-04");
}

// -- format_relative_time boundary values ----------------------------------

#[test]
fn relative_time_boundary_59s_is_just_now() {
    assert_eq!(format_relative_time(1000, 1059), "just now");
}

#[test]
fn relative_time_boundary_60s_is_one_minute() {
    assert_eq!(format_relative_time(1000, 1060), "1 minute ago");
}

#[test]
fn relative_time_boundary_yesterday_upper() {
    assert_eq!(format_relative_time(1000, 1000 + 172799), "yesterday");
}

#[test]
fn relative_time_boundary_two_days() {
    assert_eq!(format_relative_time(1000, 1000 + 172800), "2 days ago");
}

// -- author_initials Unicode -----------------------------------------------

#[test]
fn initials_multibyte_two_words() {
    assert_eq!(
        author_initials("\u{00dc}nsal \u{00d6}zdemir"),
        "\u{00dc}\u{00d6}"
    );
}

// -- CommitHeader children (plural) ----------------------------------------

#[test]
fn header_children_bulk() {
    let h = CommitHeader::new().children(vec![div(), div(), div()]);
    assert_eq!(h.children.len(), 3);
}
