//! File management pattern aligned with HIG.
//!
//! File management surfaces reveal the structure of the user's data and
//! let them move, rename, and preview files. HIG: use document-type
//! icons (not generic boxes), show a disclosure chevron on folders,
//! support Cmd/Shift multi-select, and participate in system
//! drag-and-drop.
//!
//! # See also
//!
//! - [`crate::code::file_tree::FileTreeView`] — virtualised file tree
//!   with expand/collapse, multi-select, keyboard navigation, and
//!   optional drag-source registration.
//! - [`crate::code::file_tree::icon_for_extension`] — map file
//!   extensions to SF-Symbol-backed [`crate::foundations::icons::IconName`]
//!   variants (LangRust, LangTypeScript, Image, Book, …).
//! - [`crate::code::file_tree::TreeNode`] — data model for recursive
//!   file/folder trees.
//! - [`crate::code::file_tree::DraggedFilePath`] — payload for drag
//!   sources; see [`crate::patterns::drag_and_drop`] for full integration.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/file-management>
