//! Drag and drop pattern aligned with HIG.
//!
//! Drag and drop is the canonical way to transfer content between or
//! within apps. HIG: provide a clear drag preview, confirm drop targets
//! with a visible highlight, and support cancellation (drop outside any
//! target reverts to the origin).
//!
//! # See also
//!
//! - [`crate::code::file_tree::DraggedFilePath`] — payload carried when
//!   a file-tree row is dragged. Enable via
//!   [`crate::code::file_tree::FileTreeView::set_draggable`].
//! - [`crate::code::file_tree::DraggedFilePathView`] — default drag
//!   preview entity (small chip with icon + filename).
//! - `gpui::ExternalPaths` + `.on_drop(|paths, …|)` — receive external
//!   filesystem drags (macOS Finder, other apps). Used by
//!   [`crate::components::selection_and_input::image_well::ImageWell`].
//! - GPUI `.on_drag(payload, preview_fn)` and `.on_drag_move::<T>` —
//!   build your own drag source / drop target for custom payloads.
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/drag-and-drop>
