//! Menu and action components (HIG: Components > Menus and actions).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/components/menus-and-actions>

pub mod activity_view;
pub mod button;
pub mod button_group;
pub mod button_like;
pub mod context_menu;
pub mod copy_button;
pub mod dock_menu;
pub mod edit_menu;
pub mod home_screen_quick_action;
pub mod menu_bar;
pub mod ornament;
pub mod popup_button;
pub mod pulldown_button;
pub mod share_button;

pub use crate::foundations::keyboard_shortcuts::{MenuShortcut, ModifierKey};
pub use button::{Button, ButtonShape, ButtonSize, ButtonVariant};
pub use button_group::ButtonGroup;
pub use button_like::ButtonLike;
pub use context_menu::{ContextMenu, ContextMenuEntry, ContextMenuItem, ContextMenuItemStyle};
pub use copy_button::CopyButton;
pub use dock_menu::{DockMenu, DockMenuItem};
pub use edit_menu::{EditCommand, edit_menu_standard};
pub use menu_bar::{Menu, MenuBar, MenuBarController, MenuBarWarning};
pub use popup_button::{PopupButton, PopupItem};
pub use pulldown_button::{PulldownButton, PulldownItem, PulldownItemStyle};
pub use share_button::{ShareButton, ShareService};
