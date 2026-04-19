//! Layout and organization components (HIG: Components > Layout and organization).
//!
//! # HIG Reference
//!
//! <https://developer.apple.com/design/human-interface-guidelines/components/layout-and-organization>
//!
//! Note: `Panel` and `ScrollView` live under `presentation/` to match the HIG
//! taxonomy.

pub mod box_view;
pub mod collection_view;
pub mod column_view;
pub mod disclosure;
pub mod disclosure_group;
pub mod flex_header;
pub mod list;
pub mod lockup;
pub mod outline_view;
pub mod separator;
pub mod split_view;
pub mod tab_view;
pub mod table;

pub use box_view::{BoxStyle, BoxView};
pub use collection_view::{CollectionLayout, CollectionSection, CollectionView};
pub use column_view::{Column, ColumnItem, ColumnView};
pub use disclosure::Disclosure;
pub use disclosure_group::DisclosureGroup;
pub use flex_header::{FlexActions, FlexAlign, FlexContent, FlexHeader};
pub use list::{List, ListRow, ListSection, ListStyle};
pub use outline_view::{
    FlatEntry as OutlineFlatEntry, OutlineNode, OutlineView,
    flatten_visible as outline_flatten_visible,
};
pub use separator::{Separator, SeparatorOrientation};
pub use split_view::{SplitOrientation, SplitView};
pub use tab_view::{Tab, TabView};
pub use table::{SelectionMode, SortDirection, Table, TableColumn, TableRow};
