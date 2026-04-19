//! Presentation components (HIG: Components > Presentation).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/components/presentation>
//!
//! Includes `Panel` and `ScrollView` — both relocated from
//! `layout_and_organization/` to match HIG taxonomy.

pub mod action_sheet;
pub mod alert;
pub mod hover_card;
pub mod modal;
pub mod page_controls;
pub mod panel;
pub mod popover;
pub mod scroll_view;
pub mod sheet;
pub mod tooltip;
pub mod window;

pub use action_sheet::{ActionSheet, ActionSheetItem, ActionSheetPresentation, ActionSheetStyle};
pub use alert::{Alert, AlertAction, AlertActionRole};
pub use hover_card::{HOVER_CARD_DEFAULT_DELAY_MS, HoverCard, HoverCardPlacement};
pub use modal::{Modal, ModalLevel};
pub use page_controls::{PageControls, page_controls_supported_on};
pub use panel::{Panel, PanelPosition, PanelStyle};
pub use popover::{Popover, PopoverPlacement};
pub use scroll_view::ScrollView;
pub use sheet::{Sheet, SheetDetent, SheetPresentation};
pub use tooltip::Tooltip;
pub use window::WindowStyle;
