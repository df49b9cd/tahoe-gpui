//! Git commit display component with compound sub-components.
//!
//! Provides a commit card with author info, hash, timestamp, message,
//! and collapsible file changes. Supports both a convenience builder API
//! and compound composition.
//!
//! # Convenience builder
//! ```ignore
//! Commit::new("abc1234", "Fix streaming bug")
//!     .author("Jane Doe")
//!     .timestamp(1712188800)
//!     .file_changes(vec![
//!         CommitFileData::new("src/lib.rs", FileStatus::Modified)
//!             .additions(10).deletions(3),
//!     ])
//!     .open(false)
//!     .on_toggle(|open, w, cx| { /* ... */ })
//! ```
//!
//! # Compound composition
//! ```ignore
//! Commit::from_parts("commit-1")
//!     .header(CommitHeader::new()
//!         .justify_between(false).padding(false).gap(true)
//!         .child(CommitAuthor::new("Jane Doe"))
//!         .child(CommitInfo::new()
//!             .child(CommitHash::new("abc1234"))
//!             .child(CommitSeparator::default())
//!             .child(CommitTimestamp::from_epoch(1712188800)))
//!         .child(CommitActions::new()
//!             .child(CommitCopyButton::new("abc1234"))))
//!     .message(CommitMessage::new("Fix streaming bug"))
//!     .content(CommitContent::new()
//!         .child(CommitFiles::new()
//!             .child(CommitFileRow::new("src/lib.rs", FileStatus::Modified)
//!                 .additions(10).deletions(3))))
//!     .open(false)
//!     .on_toggle(|open, w, cx| { /* ... */ })
//! ```

pub mod subcomponents;
pub mod types;

pub use subcomponents::*;
pub use types::*;

use std::rc::Rc;

use crate::callback_types::OnToggle;
use crate::components::menus_and_actions::copy_button::CopyButton;
use crate::foundations::icons::{Icon, IconName};
use crate::foundations::theme::{ActiveTheme, TextStyle, TextStyledExt};
use crate::ids::next_element_id;
use gpui::prelude::*;
use gpui::{
    App, ClickEvent, ElementId, Entity, FontWeight, KeyDownEvent, SharedString, Window, div, px,
};

use types::now_epoch_secs;

// =============================================================================
// Top-level Commit component
// =============================================================================

/// A git commit display card with collapsible file changes.
///
/// Supports two usage modes:
/// - **Convenience builder**: `Commit::new(sha, message).author(...)`
/// - **Compound composition**: `Commit::from_parts(id).header(...).content(...)`
#[derive(IntoElement)]
pub struct Commit {
    id: ElementId,
    // Convenience builder fields
    sha: Option<SharedString>,
    message_text: Option<SharedString>,
    author_name: Option<SharedString>,
    date: Option<SharedString>,
    timestamp: Option<i64>,
    file_changes: Vec<CommitFileData>,
    copy_button_entity: Option<Entity<CopyButton>>,
    // Compound composition fields
    header: Option<CommitHeader>,
    commit_message: Option<CommitMessage>,
    commit_content: Option<CommitContent>,
    // Shared fields
    is_open: bool,
    on_toggle: OnToggle,
}

impl Commit {
    /// Convenience constructor with SHA and message.
    pub fn new(sha: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            id: next_element_id("commit"),
            sha: Some(sha.into()),
            message_text: Some(message.into()),
            author_name: None,
            date: None,
            timestamp: None,
            file_changes: Vec::new(),
            copy_button_entity: None,
            header: None,
            commit_message: None,
            commit_content: None,
            is_open: false,
            on_toggle: None,
        }
    }

    /// Compound constructor. Use `.header()`, `.message()`, `.content()` to compose.
    pub fn from_parts(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            sha: None,
            message_text: None,
            author_name: None,
            date: None,
            timestamp: None,
            file_changes: Vec::new(),
            copy_button_entity: None,
            header: None,
            commit_message: None,
            commit_content: None,
            is_open: false,
            on_toggle: None,
        }
    }

    // -- Convenience builder methods ------------------------------------------

    pub fn author(mut self, author: impl Into<SharedString>) -> Self {
        self.author_name = Some(author.into());
        self
    }

    pub fn date(mut self, date: impl Into<SharedString>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Set an epoch timestamp for relative time display.
    /// When set, takes precedence over `date` for rendering.
    pub fn timestamp(mut self, epoch_secs: i64) -> Self {
        self.timestamp = Some(epoch_secs);
        self
    }

    /// Add detailed file changes with status and addition/deletion counts.
    pub fn file_changes(mut self, changes: Vec<CommitFileData>) -> Self {
        self.file_changes = changes;
        self
    }

    /// Provide a pre-created `CopyButton` entity for persistent copy state.
    pub fn copy_button(mut self, button: Entity<CopyButton>) -> Self {
        self.copy_button_entity = Some(button);
        self
    }

    // -- Compound composition methods -----------------------------------------

    /// Set the header sub-component (compound API).
    pub fn header(mut self, header: CommitHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// Set the message sub-component (compound API).
    pub fn message(mut self, message: CommitMessage) -> Self {
        self.commit_message = Some(message);
        self
    }

    /// Set the content sub-component (compound API).
    ///
    /// In compound mode, the built-in collapse/expand toggle UI is not rendered.
    /// Callers must provide their own toggle mechanism and manage `is_open` state.
    pub fn content(mut self, content: CommitContent) -> Self {
        self.commit_content = Some(content);
        self
    }

    // -- Shared methods -------------------------------------------------------

    /// Set initial open state for the collapsible file list (default: `false`).
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set a callback invoked when the file list toggle is clicked.
    /// Receives the new open state (toggled).
    pub fn on_toggle(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }

    fn render_date_display(&self) -> Option<SharedString> {
        if let Some(ts) = self.timestamp {
            Some(format_relative_time(ts, now_epoch_secs()).into())
        } else {
            self.date.clone()
        }
    }
}

impl RenderOnce for Commit {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        assert!(
            self.header.is_none() || self.sha.is_none(),
            "Commit: set either `header` (compound) or `sha` (convenience), not both"
        );
        assert!(
            self.commit_content.is_none() || self.file_changes.is_empty(),
            "Commit: set either `content` (compound) or `file_changes` (convenience), not both"
        );
        assert!(
            self.commit_message.is_none() || self.message_text.is_none(),
            "Commit: set either `message` (compound) or message text (convenience), not both"
        );

        let theme = cx.theme();

        let mut card = div()
            .flex()
            .flex_col()
            .gap(theme.spacing_sm)
            .px(theme.spacing_md)
            .py(theme.spacing_sm)
            .bg(theme.surface)
            .rounded(theme.radius_md)
            .border_1()
            .border_color(theme.border);

        // -- Header -----------------------------------------------------------
        if let Some(header) = self.header {
            card = card.child(header);
        } else if let Some(ref sha) = self.sha {
            let mut header = div().flex().items_center().gap(theme.spacing_sm);

            // Git commit icon
            header = header.child(
                Icon::new(IconName::GitCommit)
                    .size(theme.icon_size_inline)
                    .color(theme.text_muted),
            );

            // Author avatar + name
            if let Some(ref author) = self.author_name {
                header = header.child(CommitAuthor::new(author.clone()));
                header = header.child(CommitSeparator::default());
            }

            // SHA badge
            header = header.child(CommitHash::new(sha.clone()));

            // Date/timestamp
            if let Some(date_str) = self.render_date_display() {
                header = header
                    .child(CommitSeparator::default())
                    .child(CommitTimestamp::new(date_str));
            }

            // Spacer + copy button (right-aligned)
            header = header.child(div().flex_1());

            let copy_btn = CommitCopyButton::new(sha.clone());
            let copy_btn = if let Some(btn_entity) = self.copy_button_entity {
                copy_btn.copy_button(btn_entity)
            } else {
                copy_btn
            };
            header = header.child(copy_btn);

            card = card.child(header);
        }

        // -- Message ----------------------------------------------------------
        if let Some(msg) = self.commit_message {
            card = card.child(msg);
        } else if let Some(msg_text) = self.message_text {
            card = card.child(CommitMessage::new(msg_text));
        }

        // -- Collapsible file changes -----------------------------------------
        if let Some(content) = self.commit_content {
            if self.is_open {
                card = card.child(content);
            }
        } else if !self.file_changes.is_empty() {
            let total_additions: usize = self.file_changes.iter().map(|f| f.additions).sum();
            let total_deletions: usize = self.file_changes.iter().map(|f| f.deletions).sum();

            let chevron = if self.is_open {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            };

            let new_state = !self.is_open;

            let mut summary = div()
                .id(self.id.clone())
                .flex()
                .items_center()
                .gap(theme.spacing_sm)
                .text_style(TextStyle::Caption1, theme)
                .cursor_pointer()
                .child(Icon::new(chevron).size(px(12.0)).color(theme.text_muted))
                .child(
                    div()
                        .text_color(theme.text_muted)
                        .child(format!("{} files changed", self.file_changes.len())),
                )
                .when(total_additions > 0, |el| {
                    el.child(
                        div()
                            .text_color(theme.success)
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                            .child(format!("+{total_additions}")),
                    )
                })
                .when(total_deletions > 0, |el| {
                    el.child(
                        div()
                            .text_color(theme.error)
                            .font_weight(theme.effective_weight(FontWeight::MEDIUM))
                            .child(format!("-{total_deletions}")),
                    )
                });

            if let Some(handler) = self.on_toggle {
                let handler = Rc::new(handler);
                let click_handler = handler.clone();
                summary = summary
                    .on_click(move |_event: &ClickEvent, window, cx| {
                        click_handler(new_state, window, cx);
                    })
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        if crate::foundations::keyboard::is_activation_key(event) {
                            cx.stop_propagation();
                            handler(new_state, window, cx);
                        }
                    });
            }

            card = card.child(summary);

            if self.is_open {
                let file_rows = self.file_changes.iter().map(|file| {
                    CommitFileRow::new(file.path.clone(), file.status)
                        .additions(file.additions)
                        .deletions(file.deletions)
                });
                card = card.child(CommitFiles::new().children(file_rows));
            }
        }

        card
    }
}

#[cfg(test)]
mod tests;
