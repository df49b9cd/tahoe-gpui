//! Stateless subcomponents for the commit display.

use std::sync::Arc;
use std::time::Duration;

use crate::components::content::avatar::Avatar;
use crate::components::content::badge::Badge;
use crate::components::layout_and_organization::FlexHeader;
use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, HslaAlphaExt, TextStyle, TextStyledExt};
use gpui::prelude::*;
use gpui::{AnyElement, App, Entity, FontWeight, SharedString, Window, div, px};

use super::types::{
    FileStatus, author_initials, format_relative_time, now_epoch_secs, status_color,
};

// =============================================================================
// Leaf subcomponents
// =============================================================================

// -- CommitSeparator ----------------------------------------------------------

/// A separator dot between commit metadata items.
#[derive(IntoElement)]
pub struct CommitSeparator {
    pub text: SharedString,
}

impl CommitSeparator {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl Default for CommitSeparator {
    fn default() -> Self {
        Self {
            text: "\u{00b7}".into(),
        }
    }
}

impl RenderOnce for CommitSeparator {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .child(self.text)
    }
}

// -- CommitHash ---------------------------------------------------------------

/// Displays the commit SHA as a badge.
#[derive(IntoElement)]
pub struct CommitHash {
    pub(crate) hash: SharedString,
}

impl CommitHash {
    pub fn new(hash: impl Into<SharedString>) -> Self {
        Self { hash: hash.into() }
    }
}

impl RenderOnce for CommitHash {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        Badge::new(self.hash)
    }
}

// -- CommitTimestamp ----------------------------------------------------------

/// Displays a commit timestamp, either as a literal string or relative time.
#[derive(IntoElement)]
pub struct CommitTimestamp {
    pub(crate) display: SharedString,
}

impl CommitTimestamp {
    /// Create with a pre-formatted date string.
    pub fn new(date: impl Into<SharedString>) -> Self {
        Self {
            display: date.into(),
        }
    }

    /// Create from an epoch timestamp, displayed as relative time.
    pub fn from_epoch(epoch_secs: i64) -> Self {
        Self {
            display: format_relative_time(epoch_secs, now_epoch_secs()).into(),
        }
    }
}

impl RenderOnce for CommitTimestamp {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .text_style(TextStyle::Caption1, theme)
            .text_color(theme.text_muted)
            .child(self.display)
    }
}

// -- CommitAuthorAvatar -------------------------------------------------------

/// Displays an avatar with initials derived from the author name.
#[derive(IntoElement)]
pub struct CommitAuthorAvatar {
    pub(crate) initials: SharedString,
}

impl CommitAuthorAvatar {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            initials: author_initials(name.as_ref()).into(),
        }
    }

    /// Create with explicit initials (no derivation).
    pub fn from_initials(initials: impl Into<SharedString>) -> Self {
        Self {
            initials: initials.into(),
        }
    }
}

impl RenderOnce for CommitAuthorAvatar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        Avatar::new(self.initials).size(px(20.0))
    }
}

// -- CommitMessage ------------------------------------------------------------

/// Displays the commit message text.
#[derive(IntoElement)]
pub struct CommitMessage {
    pub(crate) text: SharedString,
}

impl CommitMessage {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for CommitMessage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .text_style(TextStyle::Subheadline, theme)
            .text_color(theme.text)
            .child(self.text)
    }
}

// -- CommitFileStatus ---------------------------------------------------------

/// Displays a colored file status label (A/M/D/R).
#[derive(IntoElement)]
pub struct CommitFileStatus {
    pub(crate) status: FileStatus,
}

impl CommitFileStatus {
    pub fn new(status: FileStatus) -> Self {
        Self { status }
    }
}

impl RenderOnce for CommitFileStatus {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .text_color(status_color(self.status, theme))
            .font_weight(theme.effective_weight(FontWeight::BOLD))
            .min_w(px(14.0))
            .child(self.status.label())
    }
}

// -- CommitFileIcon -----------------------------------------------------------

/// Displays a file icon, optionally varying by status.
#[derive(IntoElement)]
pub struct CommitFileIcon {
    pub(crate) icon: IconName,
}

impl CommitFileIcon {
    pub fn new(icon: IconName) -> Self {
        Self { icon }
    }
}

impl Default for CommitFileIcon {
    fn default() -> Self {
        Self {
            icon: IconName::File,
        }
    }
}

impl RenderOnce for CommitFileIcon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        Icon::new(self.icon).size(px(12.0)).color(theme.text_muted)
    }
}

// -- CommitFilePath -----------------------------------------------------------

/// Displays a file path in monospace.
#[derive(IntoElement)]
pub struct CommitFilePath {
    pub(crate) path: SharedString,
}

impl CommitFilePath {
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self { path: path.into() }
    }
}

impl RenderOnce for CommitFilePath {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div().flex_1().text_color(theme.text_muted).child(self.path)
    }
}

// -- CommitFileAdditions ------------------------------------------------------

/// Displays a green "+N" additions count.
#[derive(IntoElement)]
pub struct CommitFileAdditions {
    pub count: usize,
}

impl CommitFileAdditions {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl RenderOnce for CommitFileAdditions {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        // HIG §Color + WCAG 1.4.1: pair the semantic green foreground with
        // a low-opacity green fill so the gutter reads at a glance and
        // colour is not the sole carrier of meaning. Mirrors GitHub,
        // Xcode, and Zed's git_panel. Finding N3 in
        // the HIG Code-surface audit.
        div()
            .px(theme.spacing_xs)
            .rounded(theme.radius_sm)
            .bg(theme.success.opacity(0.12))
            .text_color(theme.success)
            .child(format!("+{}", self.count))
    }
}

// -- CommitFileDeletions ------------------------------------------------------

/// Displays a red "-N" deletions count.
#[derive(IntoElement)]
pub struct CommitFileDeletions {
    pub count: usize,
}

impl CommitFileDeletions {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl RenderOnce for CommitFileDeletions {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .px(theme.spacing_xs)
            .rounded(theme.radius_sm)
            .bg(theme.error.opacity(0.12))
            .text_color(theme.error)
            .child(format!("-{}", self.count))
    }
}

// =============================================================================
// Container subcomponents
// =============================================================================

// -- CommitFileChanges --------------------------------------------------------

/// Groups file additions and deletions counts.
#[derive(Default, IntoElement)]
pub struct CommitFileChanges {
    pub(crate) children: Vec<AnyElement>,
}

impl CommitFileChanges {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CommitFileChanges {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .children(self.children)
    }
}

// -- CommitFileInfo -----------------------------------------------------------

/// Groups file status, icon, and path.
#[derive(Default, IntoElement)]
pub struct CommitFileInfo {
    pub(crate) children: Vec<AnyElement>,
}

impl CommitFileInfo {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CommitFileInfo {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .flex_1()
            .items_center()
            .gap(theme.spacing_xs)
            .children(self.children)
    }
}

// -- CommitFileRow ------------------------------------------------------------

/// A single file row within the file list.
///
/// Supports both convenience (`new(path, status)`) and compound (`from_parts()`) APIs.
#[derive(IntoElement)]
pub struct CommitFileRow {
    // Convenience fields
    pub(crate) path: Option<SharedString>,
    pub(crate) status: Option<FileStatus>,
    pub(crate) additions: usize,
    pub(crate) deletions: usize,
    // Compound fields
    pub(crate) custom_children: Vec<AnyElement>,
}

impl CommitFileRow {
    /// Convenience constructor with path and status.
    pub fn new(path: impl Into<SharedString>, status: FileStatus) -> Self {
        Self {
            path: Some(path.into()),
            status: Some(status),
            additions: 0,
            deletions: 0,
            custom_children: Vec::new(),
        }
    }

    /// Compound constructor. Use `.child()` to compose content.
    pub fn from_parts() -> Self {
        Self {
            path: None,
            status: None,
            additions: 0,
            deletions: 0,
            custom_children: Vec::new(),
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

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.custom_children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CommitFileRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();

        let mut row = div().flex().items_center().gap(theme.spacing_xs);

        if !self.custom_children.is_empty() {
            row = row.children(self.custom_children);
        } else if let Some(status) = self.status {
            row = row.child(CommitFileStatus::new(status)).child(
                div()
                    .flex_1()
                    .text_color(theme.text_muted)
                    .child(self.path.unwrap_or_default()),
            );

            if self.additions > 0 {
                row = row.child(CommitFileAdditions::new(self.additions));
            }
            if self.deletions > 0 {
                row = row.child(CommitFileDeletions::new(self.deletions));
            }
        }

        row
    }
}

// -- CommitFiles --------------------------------------------------------------

/// A collapsible file list with summary header.
#[derive(Default, IntoElement)]
pub struct CommitFiles {
    pub(crate) children: Vec<AnyElement>,
}

impl CommitFiles {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for CommitFiles {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .flex_col()
            .gap(px(2.0))
            .font(theme.mono_font())
            .text_style(TextStyle::Caption1, theme)
            .children(self.children)
    }
}

// -- CommitAuthor -------------------------------------------------------------

/// Displays author avatar and name.
#[derive(IntoElement)]
pub struct CommitAuthor {
    pub(crate) name: SharedString,
}

impl CommitAuthor {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self { name: name.into() }
    }
}

impl RenderOnce for CommitAuthor {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_xs)
            .child(CommitAuthorAvatar::new(self.name.as_ref()))
            .child(
                div()
                    .text_style(TextStyle::Caption1, theme)
                    .text_color(theme.text_muted)
                    .child(self.name),
            )
    }
}

// -- CommitInfo ---------------------------------------------------------------

/// Groups hash, separators, and timestamp in a flex row.
#[derive(Default, IntoElement)]
pub struct CommitInfo {
    pub(crate) children: Vec<AnyElement>,
}

impl CommitInfo {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CommitInfo {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .children(self.children)
    }
}

// -- CommitMetadata -----------------------------------------------------------

/// Generic container for extra metadata in the header.
#[derive(Default, IntoElement)]
pub struct CommitMetadata {
    pub(crate) children: Vec<AnyElement>,
}

impl CommitMetadata {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }
}

impl RenderOnce for CommitMetadata {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .gap(theme.spacing_sm)
            .children(self.children)
    }
}

// -- CommitActions ------------------------------------------------------------

/// Container for action buttons (copy, etc.), rendered right-aligned.
///
/// Type alias for [`crate::components::layout_and_organization::FlexActions`]
/// — a horizontal flex row with gap spacing.
pub type CommitActions = crate::components::layout_and_organization::FlexActions;

// -- CommitCopyButton ---------------------------------------------------------

/// A copy-to-clipboard button for the commit hash.
///
/// Wraps `Entity<CopyButton>`. Pass a pre-created entity via `.copy_button()`
/// for persistent copy-feedback state (the "copied!" checkmark).
///
/// **Note:** Without a pre-created entity, a new `CopyButton` is allocated on
/// each render and copy-feedback state will not persist across re-renders.
#[derive(IntoElement)]
pub struct CommitCopyButton {
    pub(crate) hash: SharedString,
    pub(crate) copy_button: Option<Entity<CopyButton>>,
    pub(crate) on_copy: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    pub(crate) timeout: Option<Duration>,
}

impl CommitCopyButton {
    pub fn new(hash: impl Into<SharedString>) -> Self {
        Self {
            hash: hash.into(),
            copy_button: None,
            on_copy: None,
            timeout: None,
        }
    }

    /// Provide a pre-created `CopyButton` entity for persistent copy state.
    pub fn copy_button(mut self, button: Entity<CopyButton>) -> Self {
        self.copy_button = Some(button);
        self
    }

    pub fn on_copy(mut self, callback: Arc<dyn Fn() + Send + Sync + 'static>) -> Self {
        self.on_copy = Some(callback);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl RenderOnce for CommitCopyButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if let Some(btn) = self.copy_button {
            btn.update(cx, |btn, _cx| {
                btn.set_content(self.hash.to_string());
                if let Some(timeout) = self.timeout {
                    btn.set_timeout(timeout);
                }
                if let Some(on_copy) = self.on_copy {
                    btn.set_on_copy(on_copy);
                }
            });
            btn
        } else {
            let btn = CopyButton::new(self.hash.to_string(), cx);
            if self.timeout.is_some() || self.on_copy.is_some() {
                btn.update(cx, |btn, _cx| {
                    if let Some(timeout) = self.timeout {
                        btn.set_timeout(timeout);
                    }
                    if let Some(on_copy) = self.on_copy {
                        btn.set_on_copy(on_copy);
                    }
                });
            }
            btn
        }
    }
}

// -- CommitHeader -------------------------------------------------------------

/// Flex row container for commit header items.
///
/// Type alias for [`FlexHeader`] configured with gap, no justify-between, and no padding.
pub type CommitHeader = FlexHeader;

// -- CommitContent ------------------------------------------------------------

/// Container for the collapsible commit body content.
#[derive(Default, IntoElement)]
pub struct CommitContent {
    pub(crate) children: Vec<AnyElement>,
}

impl CommitContent {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        self.children
            .extend(children.into_iter().map(|c| c.into_any_element()));
        self
    }
}

impl RenderOnce for CommitContent {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .children(self.children)
    }
}

#[cfg(test)]
mod diff_tests {
    use super::{DiffLineKind, classify_diff_line, parse_unified_diff};
    use core::prelude::v1::test;

    #[test]
    fn classify_hunk_header() {
        assert_eq!(
            classify_diff_line("@@ -1,2 +1,3 @@"),
            DiffLineKind::HunkHeader
        );
    }

    #[test]
    fn classify_file_headers() {
        for line in [
            "diff --git a/foo b/foo",
            "index abcd..ef01 100644",
            "--- a/foo",
            "+++ b/foo",
            "new file mode 100644",
            "deleted file mode 100644",
            "rename from old",
            "rename to new",
        ] {
            assert_eq!(
                classify_diff_line(line),
                DiffLineKind::FileHeader,
                "line: {line}"
            );
        }
    }

    #[test]
    fn classify_added_removed_context() {
        assert_eq!(classify_diff_line("+added"), DiffLineKind::Added);
        assert_eq!(classify_diff_line("-removed"), DiffLineKind::Removed);
        assert_eq!(classify_diff_line(" context"), DiffLineKind::Context);
        // No-newline marker is a meta line, not a deletion.
        assert_eq!(
            classify_diff_line("\\ No newline at end of file"),
            DiffLineKind::Meta
        );
    }

    #[test]
    fn parse_unified_diff_roundtrip() {
        let diff = "@@ -1,2 +1,3 @@\n context\n-old\n+new a\n+new b\n";
        let lines = parse_unified_diff(diff);
        let kinds: Vec<_> = lines.iter().map(|l| l.kind).collect();
        assert_eq!(
            kinds,
            vec![
                DiffLineKind::HunkHeader,
                DiffLineKind::Context,
                DiffLineKind::Removed,
                DiffLineKind::Added,
                DiffLineKind::Added,
            ]
        );
    }

    #[test]
    fn parse_unified_diff_trims_trailing_empty_context() {
        // Ending the diff text with a newline introduces a trailing empty
        // "context" entry from `split('\n')`. The parser must drop it so
        // the rendered diff doesn't pick up a blank row.
        let lines = parse_unified_diff("@@ -1 +1 @@\n-a\n+b\n");
        assert_eq!(lines.len(), 3);
        assert!(matches!(lines.last().unwrap().kind, DiffLineKind::Added));
    }
}

// -- CommitDiff ---------------------------------------------------------------

/// Classification of a unified-diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineKind {
    /// File header (`diff --git …`, `--- a/…`, `+++ b/…`, `index …`).
    FileHeader,
    /// Hunk header (`@@ -l,s +l,s @@ …`). Rendered with a muted fill
    /// mirroring Zed's `git_panel` and GitHub's diff viewer.
    HunkHeader,
    /// Line starting with `+` (addition). Background-tinted with the
    /// success palette.
    Added,
    /// Line starting with `-` (deletion). Background-tinted with the
    /// error palette.
    Removed,
    /// Context line (leading space) rendered with no tint.
    Context,
    /// "\ No newline at end of file" marker — rendered muted.
    Meta,
}

/// One parsed line of a unified diff, ready for render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: SharedString,
}

/// Parse a unified-diff text blob into classified lines. Preserves the
/// original line ordering and content (no normalisation), so callers can
/// render the diff with per-line gutters identical to the source.
pub fn parse_unified_diff(diff: &str) -> Vec<DiffLine> {
    let mut out = Vec::with_capacity(diff.lines().count());
    for line in diff.split('\n') {
        let kind = classify_diff_line(line);
        out.push(DiffLine {
            kind,
            text: SharedString::from(line.to_string()),
        });
    }
    // Trim a trailing empty line that `split('\n')` produces when the
    // input ends with a newline — keeps the rendered output tight.
    if matches!(out.last(), Some(l) if l.text.is_empty() && matches!(l.kind, DiffLineKind::Context))
    {
        out.pop();
    }
    out
}

fn classify_diff_line(line: &str) -> DiffLineKind {
    if line.starts_with("@@") {
        DiffLineKind::HunkHeader
    } else if line.starts_with("diff --git")
        || line.starts_with("index ")
        || line.starts_with("--- ")
        || line.starts_with("+++ ")
        || line.starts_with("new file mode")
        || line.starts_with("deleted file mode")
        || line.starts_with("similarity index")
        || line.starts_with("rename from")
        || line.starts_with("rename to")
    {
        DiffLineKind::FileHeader
    } else if line.starts_with('\\') {
        DiffLineKind::Meta
    } else if line.starts_with('+') {
        DiffLineKind::Added
    } else if line.starts_with('-') {
        DiffLineKind::Removed
    } else {
        DiffLineKind::Context
    }
}

/// A line-level unified-diff view with per-line gutters and hunk headers.
///
/// Renders classified diff lines (`Added`, `Removed`, `HunkHeader`, …)
/// with background fills tuned to match GitHub, Xcode Source Control,
/// and Zed's `git_panel`. Pair with the summary `CommitFileRow` for a
/// two-tier view (file-level counts on top, line-level hunks below).
///
/// # Example
/// ```ignore
/// CommitDiff::new(
///     "@@ -1,2 +1,3 @@\n context\n-old\n+new one\n+new two\n",
/// )
/// ```
#[derive(IntoElement)]
pub struct CommitDiff {
    lines: Vec<DiffLine>,
}

impl CommitDiff {
    /// Build a diff view from raw unified-diff text.
    pub fn new(diff: impl AsRef<str>) -> Self {
        Self {
            lines: parse_unified_diff(diff.as_ref()),
        }
    }

    /// Build from pre-classified lines — useful when the caller already
    /// holds a parsed representation (e.g. from `libgit2`).
    pub fn from_lines(lines: Vec<DiffLine>) -> Self {
        Self { lines }
    }

    /// Read-only access to the parsed lines, primarily for tests.
    pub fn lines(&self) -> &[DiffLine] {
        &self.lines
    }
}

impl RenderOnce for CommitDiff {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let mut container = div()
            .flex()
            .flex_col()
            .font(theme.mono_font())
            .text_style(TextStyle::Caption1, theme)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border)
            .overflow_hidden();

        for line in self.lines {
            let text = if line.text.is_empty() {
                SharedString::from(" ")
            } else {
                line.text.clone()
            };
            let mut row = div()
                .flex()
                .items_start()
                .px(theme.spacing_sm)
                .py(px(1.0))
                .w_full();

            match line.kind {
                DiffLineKind::Added => {
                    row = row.bg(theme.success.opacity(0.12)).text_color(theme.text);
                }
                DiffLineKind::Removed => {
                    row = row.bg(theme.error.opacity(0.12)).text_color(theme.text);
                }
                DiffLineKind::HunkHeader => {
                    row = row
                        .bg(theme.surface)
                        .text_color(theme.text_muted)
                        .font_weight(theme.effective_weight(FontWeight::MEDIUM));
                }
                DiffLineKind::FileHeader => {
                    row = row
                        .bg(theme.surface)
                        .text_color(theme.text_muted)
                        .font_weight(theme.effective_weight(FontWeight::MEDIUM));
                }
                DiffLineKind::Meta => {
                    row = row.text_color(theme.text_muted);
                }
                DiffLineKind::Context => {
                    row = row.text_color(theme.text);
                }
            }

            row = row.child(div().whitespace_nowrap().child(text));
            container = container.child(row);
        }

        container
    }
}
