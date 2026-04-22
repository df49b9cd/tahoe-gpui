//! Content display components (HIG: Components > Content).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/components/content>
//!
//! HIG pages in this subcategory: Charts, Image views, Text views, Web views.
//! `Avatar` and `Badge` are crate extensions beyond the HIG inventory.

pub mod avatar;
pub mod badge;
pub mod chart;
pub mod image_view;
pub mod label;
pub mod selectable_text;
pub mod text_view;
pub mod web_view;

pub use avatar::{Avatar, AvatarSize, AvatarStatus};
pub use badge::{Badge, BadgeVariant};
pub use chart::{Chart, ChartDataSeries, ChartType};
pub use image_view::{ContentMode, ImageView};
pub use label::{Label, LabelVariant};
pub use selectable_text::{AnchorClickHandler, SelectableText, SelectionCoordinator};
pub use text_view::{TextRuns, TextView, TextViewSelection};
pub use web_view::WebView;
