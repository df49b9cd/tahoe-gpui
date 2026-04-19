//! Flat prelude ordered by HIG taxonomy.
//!
//! Importing this module (`use tahoe_gpui::prelude::*;`) pulls in the
//! common public surface across all 8 HIG component subcategories plus the
//! foundations types most host apps rely on. Organized by HIG taxonomy so
//! coverage gaps are easy to spot.
//!
//! Stub modules (Chart, WebView, TabView, ColumnView, Lockup,
//! VirtualKeyboard, Ornament, and the `system_experiences` surface) do not
//! re-export anything — they only carry HIG documentation URLs today.

// ── Foundations ────────────────────────────────────────────────────────────

pub use crate::foundations::blink_manager::{
    BlinkManager, BlinkPhaseChanged, CURSOR_BLINK_INTERVAL_MS,
};
pub use crate::foundations::color::HslaAlphaExt;
pub use crate::foundations::icons::{
    AnimatedIcon, EmbeddedIconAssets, Icon, IconAnimation, IconLayoutBehavior, IconName,
    IconRenderMode, IconScale, IconStyle,
};
pub use crate::foundations::layout::{
    DROPDOWN_SNAP_MARGIN, FlexExt, h_flex, snap_to_window_margin, v_flex,
};
pub use crate::foundations::right_to_left::{IconDirection, icon_direction};
pub use crate::foundations::theme::{
    ActiveTheme, GlassSize, SurfaceContext, TahoeTheme, TextStyle, TextStyledExt,
};

// ── Components > Content ───────────────────────────────────────────────────

pub use crate::components::content::{Avatar, Badge, BadgeVariant, Label, TextView};

// ── Components > Layout and organization ───────────────────────────────────

pub use crate::components::layout_and_organization::{
    BoxView, CollectionLayout, CollectionView, Disclosure, DisclosureGroup, FlexActions, FlexAlign,
    FlexContent, FlexHeader, OutlineNode, OutlineView, SelectionMode, Separator,
    SeparatorOrientation, SortDirection, SplitView, Table, TableColumn, TableRow,
};

// ── Components > Menus and actions ─────────────────────────────────────────

pub use crate::components::menus_and_actions::{
    Button, ButtonGroup, ButtonShape, ButtonSize, ButtonVariant, ContextMenu, ContextMenuEntry,
    ContextMenuItem, ContextMenuItemStyle, CopyButton, Menu, MenuBar, PopupButton, PopupItem,
    PulldownButton, PulldownItem, PulldownItemStyle,
};

// ── Components > Navigation and search ─────────────────────────────────────

pub use crate::components::navigation_and_search::{
    NavigationBarIOS, NavigationSplitView, PathControl, PathControlStyle, PathSegment, SearchBar,
    SearchField, Sidebar, SidebarItem, SidebarPosition, SidebarSection, TabBar, TabItem,
    TokenField, TokenItem, Toolbar,
};

// ── Components > Presentation ──────────────────────────────────────────────

pub use crate::components::presentation::{
    ActionSheet, ActionSheetItem, ActionSheetStyle, Alert, AlertAction, AlertActionRole, HoverCard,
    Modal, PageControls, Panel, PanelPosition, Popover, PopoverPlacement, ScrollView, Sheet,
    SheetDetent, Tooltip, WindowStyle,
};

// ── Components > Selection and input ───────────────────────────────────────

pub use crate::components::selection_and_input::{
    Checkbox, CheckboxState, ColorWell, ComboBox, DatePicker, DigitEntry, ImageWell, Picker,
    PickerItem, SegmentItem, SegmentedControl, SimpleDate, Slider, Stepper, TextField,
    TextFieldValidation, TimePicker, Toggle,
};

// ── Components > Status ────────────────────────────────────────────────────

/// `RatingIndicator` is re-exported only on macOS — HIG restricts the
/// control to that platform ("Not supported in iOS, iPadOS, tvOS,
/// visionOS, or watchOS").
#[cfg(target_os = "macos")]
pub use crate::components::status::RatingIndicator;
pub use crate::components::status::{
    ACTIVITY_RING_EXERCISE, ACTIVITY_RING_MOVE, ACTIVITY_RING_STAND, ActivityIndicator,
    ActivityIndicatorStyle, ActivityRing, ActivityRingSet, Gauge, GaugeDirection, GaugeStyle,
    ProgressIndicator, ProgressIndicatorSize, ProgressIndicatorValue, Shimmer, ShimmerEasing,
    SweepDirection, TextShimmer, TextShimmerState,
};

// ── Components > System experiences ────────────────────────────────────────
// Stub-only today: Widgets, Notifications, Live Activities, App Shortcuts,
// Controls, Status bars, Complications, Top Shelf, Watch faces.

// ── Markdown ───────────────────────────────────────────────────────────────

pub use crate::markdown::{
    IncrementalMarkdownParser, InlineContent, MarkdownBlock, StreamSettings, TableAlignment,
};

// ── Citations ──────────────────────────────────────────────────────────────

pub use crate::citation::{CitationSource, InlineCitation};

// ── Keybindings ────────────────────────────────────────────────────────────

pub use crate::{all_keybindings, text_keybindings, textfield_keybindings};
